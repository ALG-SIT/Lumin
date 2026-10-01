#![allow(dead_code)]

//! DNS-SD integration for Lumin classroom discovery.
//!
//! Wraps the `tauri-plugin-dns-sd` (`@momics/dns-sd-tauri`) plugin to provide
//! higher-level helpers for the teacher (advertise) and student (browse) flows.
//!
//! # Service layout
//!
//! - **Service type:** `_lumin-class._tcp`
//! - **TXT record keys:** `code_required` (bool flag), `version` (string), `transport` (string)
//! - **Port:** OS-assigned via the HTTP server (Todo 18); passed in at advertise time.

use std::collections::HashMap;

use tauri::ipc::InvokeResponseBody;
use tauri::AppHandle;
use tauri_plugin_dns_sd::{
    AdvertiseHandle as PluginAdvertiseHandle, AdvertiseOptions, AdvertiseServiceSpec,
    BrowseOptions, BrowseServiceSpec, DnsSdExt, TransportProtocol, TxtRecordValue,
};

// ── Constants ────────────────────────────────────────────────────────────────

/// The mDNS service type used for Lumin classrooms (without leading underscore).
const SERVICE_TYPE: &str = "lumin-class";

/// Transport protocol for the classroom service.
const PROTOCOL: TransportProtocol = TransportProtocol::Tcp;

/// Current service protocol version — embedded in TXT records so students can
/// detect mismatched versions early.
const PROTOCOL_VERSION: &str = "2";

// ── Discovery result contract ────────────────────────────────────────────────

/// One classroom discoverable by students.
///
/// `host` は常に入力可能なIP文字列(LAN優先、無ければloopback、最後にSRVホスト名)。
/// NOTE: フロントエンド `DiscoveredTeacher.session_uuid` が snake_case のため
/// camelCase リネームを行わない。
#[derive(Debug, Clone, serde::Serialize)]
pub struct TeacherDiscovered {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub session_uuid: Option<String>,
    pub compatible: bool,
}

// ── Internal browse message types (mirrors the plugin's unexported enum) ─────

/// Mirrors the plugin's `BrowseChannelMessage` for deserialization from IPC.
/// The plugin sends `{ browseId, service: { ... } }` or `{ browseId, reason }`.
#[derive(serde::Deserialize)]
#[serde(untagged)]
enum BrowseChannelMessage {
    Service {
        #[serde(rename = "browseId")]
        browse_id: u64,
        service: BrowseServiceRecord,
    },
    Stopped {
        #[serde(rename = "browseId")]
        browse_id: u64,
        reason: String,
    },
}

/// Simplified service record shape — only the fields we need for events.
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BrowseServiceRecord {
    name: String,
    host: Option<String>,
    port: Option<u16>,
    #[serde(default)]
    addresses: Vec<String>,
    #[serde(default)]
    is_active: bool,
    txt: Option<HashMap<String, serde_json::Value>>,
}

// ── Public API ───────────────────────────────────────────────────────────────

/// Advertise a Lumin teacher service on the local network.
///
/// Returns an opaque handle that can later be passed to [`stop_advertise`].
///
/// # Arguments
///
/// * `app`       – Tauri application handle (provides the DNS-SD plugin state).
/// * `port` – TCP port the classroom HTTP server will listen on.
///   Must be the real bound port (from Todo 18's axum server).
/// * `instance_name` – Human-readable name shown to browsing students,
///   e.g. "Ms. Smith's Math Class".
/// * `session_uuid`  – Unique session identifier embedded in TXT records.
/// * `join_code_required` – Whether students must enter a join code to connect.
pub async fn advertise_teacher(
    app: &AppHandle,
    port: u16,
    instance_name: &str,
    session_uuid: &str,
    join_code_required: bool,
) -> Result<PluginAdvertiseHandle, String> {
    let mut txt = HashMap::new();
    txt.insert(
        "code_required".to_string(),
        TxtRecordValue::BooleanFlag(join_code_required),
    );
    txt.insert(
        "session_uuid".to_string(),
        TxtRecordValue::BinaryData(session_uuid.as_bytes().to_vec()),
    );
    txt.insert(
        "version".to_string(),
        TxtRecordValue::BinaryData(PROTOCOL_VERSION.as_bytes().to_vec()),
    );
    txt.insert(
        "transport".to_string(),
        TxtRecordValue::BinaryData(b"noise".to_vec()),
    );

    let options = AdvertiseOptions {
        service: AdvertiseServiceSpec {
            name: instance_name.to_string(),
            type_name: SERVICE_TYPE.to_string(),
            protocol: PROTOCOL,
            port,
            host: None, // let OS resolve
            domain: None,
            subtypes: Vec::new(),
            txt,
        },
    };

    app.dns_sd()
        .advertise_start(app.clone(), options)
        .await
        .map_err(|e| format!("dns-sd advertise failed: {e}"))
}

/// Stop a previously started advertisement.
pub async fn stop_advertise(app: &AppHandle, advertise_id: u64) -> Result<(), String> {
    app.dns_sd()
        .advertise_stop(app.clone(), advertise_id)
        .await
        .map_err(|e| format!("dns-sd advertise_stop failed: {e}"))
}

/// Browse for Lumin teacher services on the local network.
///
/// Discovered teachers are emitted as Tauri events so the frontend can react:
///
/// | Event name          | Payload                 |
/// |---------------------|-------------------------|
/// | `"teacher-found"`   | [`TeacherFoundPayload`] |
/// | `"teacher-removed"` | [`TeacherRemovedPayload`]|
///
/// Returns a browse handle that can later be passed to [`stop_browse`].
/// Wait window for mDNS discovery (bounded so the invoke resolves).
const DISCOVERY_WINDOW_MS: u64 = 2500;

/// Resolve the address a student should connect to.
///
/// 優先順位: LAN IPv4 → loopback IPv4(同一PC) → その他リテラル → SRVホスト名。
pub(crate) fn pick_connect_address(host: &Option<String>, addresses: &[String]) -> String {
    let parsed = |a: &String| a.parse::<std::net::IpAddr>().ok();
    if let Some(lan) = addresses
        .iter()
        .find(|a| matches!(parsed(a), Some(ip) if ip.is_ipv4() && !ip.is_loopback()))
    {
        return lan.clone();
    }
    if let Some(lo) = addresses
        .iter()
        .find(|a| matches!(parsed(a), Some(ip) if ip.is_ipv4() && ip.is_loopback()))
    {
        return lo.clone();
    }
    addresses
        .first()
        .cloned()
        .unwrap_or_else(|| host.clone().unwrap_or_default())
}

/// `classroom_base_url("::1", 8765)` のようにIPv6リテラルを角括弧で包む。
pub(crate) fn classroom_base_url(host: &str, port: u16) -> String {
    let h = if host.contains(':') && !host.starts_with('[') {
        format!("[{host}]")
    } else {
        host.to_string()
    };
    format!("http://{h}:{port}")
}

/// プラグインwire(TxtWireValue untagged JSON)をプレーン文字列へ正規化する。
///
/// バイト配列(UUID等)はUTF-8文字列に復元する。
pub(crate) fn decode_txt_value(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::Bool(true) => "true".to_string(),
        serde_json::Value::Null => String::new(),
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(bytes) => {
            let raw: Vec<u8> = bytes
                .iter()
                .map(|b| u8::try_from(b.as_u64().unwrap_or(0)).unwrap_or(0))
                .collect();
            String::from_utf8_lossy(&raw).into_owned()
        }
        other => other.to_string(),
    }
}

/// 同一サービス名の複数回告知(IFごと/再告知)を最新レコードへ統合し、初出順を保つ。
pub(crate) fn merge_records(records: Vec<BrowseServiceRecord>) -> Vec<BrowseServiceRecord> {
    let mut order: Vec<String> = Vec::new();
    let mut latest: std::collections::HashMap<String, BrowseServiceRecord> =
        std::collections::HashMap::new();
    for r in records {
        if !order.contains(&r.name) {
            order.push(r.name.clone());
        }
        latest.insert(r.name.clone(), r);
    }
    order.drain(..).filter_map(|n| latest.remove(&n)).collect()
}

fn to_discovered(r: BrowseServiceRecord) -> Option<TeacherDiscovered> {
    let txt = r.txt.as_ref()?;
    let session_uuid = txt.get("session_uuid").map(decode_txt_value);
    let compatible = txt
        .get("version")
        .is_some_and(|v| decode_txt_value(v) == "2")
        && txt
            .get("transport")
            .is_some_and(|v| decode_txt_value(v) == "noise");
    Some(TeacherDiscovered {
        name: r.name,
        host: pick_connect_address(&r.host, &r.addresses),
        port: r.port?,
        session_uuid,
        compatible,
    })
}

/// Discover classrooms advertising `_lumin-class._tcp` on the local network.
///
/// フロントエンド契約(StudentJoin.tsx)は同期配列返却のため、固定ウィンドウで
/// 収集してからバウンドした配列を返す。
#[tauri::command]
pub async fn browse_teachers(app: tauri::AppHandle) -> Result<Vec<TeacherDiscovered>, String> {
    let options = BrowseOptions {
        service: BrowseServiceSpec {
            type_name: SERVICE_TYPE.to_string(),
            protocol: PROTOCOL,
            domain: None,
            subtypes: Vec::new(),
        },
        timeout_ms: Some(DISCOVERY_WINDOW_MS),
    };

    let records: std::sync::Arc<std::sync::Mutex<Vec<BrowseServiceRecord>>> =
        std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let sink = std::sync::Arc::clone(&records);

    let browse_channel = tauri::ipc::Channel::new(move |msg: InvokeResponseBody| {
        // The plugin sends BrowseChannelMessage as serialized JSON.
        let InvokeResponseBody::Json(json_str) = msg else {
            return Ok(());
        };
        let Ok(parsed) = serde_json::from_str::<BrowseChannelMessage>(&json_str) else {
            return Ok(());
        };
        match parsed {
            BrowseChannelMessage::Service { service, .. } => {
                if !service.is_active {
                    return Ok(()); // leave notification — keep only active services
                }
                sink.lock()
                    .expect("browse collector poisoned")
                    .push(service);
            }
            BrowseChannelMessage::Stopped { browse_id, reason } => {
                eprintln!("[dns_sd] browse {browse_id} stopped: {reason}");
            }
        }
        Ok(())
    });

    let handle = app
        .dns_sd()
        .browse_start(app.clone(), options, browse_channel)
        .await
        .map_err(|e| format!("dns-sd browse failed: {e}"))?;

    tokio::time::sleep(std::time::Duration::from_millis(DISCOVERY_WINDOW_MS + 150)).await;
    let _ = stop_browse(&app, handle.browse_id).await; // best-effort stop

    let mut collected: Vec<TeacherDiscovered> = Vec::new();
    for r in merge_records(
        records
            .lock()
            .expect("browse collector poisoned")
            .drain(..)
            .collect(),
    ) {
        match to_discovered(r) {
            Some(d) => collected.push(d),
            None => continue,
        }
    }
    Ok(collected)
}

/// Stop a previously started browse session.
pub async fn stop_browse(app: &AppHandle, browse_id: u64) -> Result<(), String> {
    app.dns_sd()
        .browse_stop(app.clone(), browse_id)
        .await
        .map_err(|e| format!("dns-sd browse_stop failed: {e}"))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod discovery_tests {
    use super::*;
    use serde_json::json;

    fn utf8_value(s: &str) -> serde_json::Value {
        json!(s.as_bytes().to_vec())
    }

    #[test]
    fn pick_connect_address_prefers_non_loopback_ipv4() {
        let got = pick_connect_address(
            &Some("Lumin---1234.local.".into()),
            &["127.0.0.1".to_string(), "192.168.10.5".to_string()],
        );
        assert_eq!(got, "192.168.10.5");
    }

    #[test]
    fn pick_connect_address_falls_back_to_loopback_for_same_host() {
        let got = pick_connect_address(&None, &["127.0.0.1".to_string()]);
        assert_eq!(got, "127.0.0.1");
    }

    #[test]
    fn pick_connect_address_falls_back_to_srv_hostname_when_no_addresses() {
        let got = pick_connect_address(&Some("lumin.local.".into()), &[]);
        assert_eq!(got, "lumin.local.");
    }

    #[test]
    fn classroom_base_url_wraps_ipv6_in_brackets() {
        assert_eq!(
            classroom_base_url("192.168.1.5", 8765),
            "http://192.168.1.5:8765"
        );
        assert_eq!(classroom_base_url("::1", 8765), "http://[::1]:8765");
    }

    #[test]
    fn decode_txt_value_decodes_byte_array_as_utf8() {
        // session_uuid はバイト配列として来る(実機計測値に準拠)
        assert_eq!(
            decode_txt_value(&utf8_value("session-uuid-1234")),
            "session-uuid-1234"
        );
    }

    #[test]
    fn decode_txt_value_handles_flag_null_and_string() {
        assert_eq!(decode_txt_value(&json!(true)), "true");
        assert_eq!(decode_txt_value(&serde_json::Value::Null), "");
        assert_eq!(decode_txt_value(&json!("plain")), "plain");
    }
    #[test]
    fn merge_records_dedupes_by_name_keeping_latest() {
        let mk = |name: &str, addr: &str| BrowseServiceRecord {
            name: name.into(),
            host: None,
            port: Some(8765),
            addresses: vec![addr.into()],
            is_active: true,
            txt: None,
        };
        let merged = merge_records(vec![
            mk("Lumin教室 A", "10.0.0.1"),
            mk("Lumin教室 A", "10.0.0.9"),
            mk("Lumin教室 B", "127.0.0.1"),
        ]);
        assert_eq!(merged.len(), 2);
        // 同名は最後(最新)のレコードを保持
        assert_eq!(
            merged
                .iter()
                .find(|r| r.name == "Lumin教室 A")
                .unwrap()
                .addresses,
            ["10.0.0.9"]
        );
    }
}
