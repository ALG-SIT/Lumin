#![allow(dead_code)]

//! DNS-SD integration for Lumin classroom discovery.
//!
//! Wraps the `tauri-plugin-dns-sd` (`@momics/dns-sd-tauri`) plugin to provide
//! higher-level helpers for the teacher (advertise) and student (browse) flows.
//!
//! # Service layout
//!
//! - **Service type:** `_lumin-class._tcp`
//! - **TXT record keys:** `code_required` (bool flag), `version` (string)
//! - **Port:** OS-assigned via the HTTP server (Todo 18); passed in at advertise time.

use std::collections::HashMap;

use tauri::ipc::InvokeResponseBody;
use tauri::{AppHandle, Emitter};
use tauri_plugin_dns_sd::{
    AdvertiseHandle as PluginAdvertiseHandle, AdvertiseOptions, AdvertiseServiceSpec,
    BrowseHandle as PluginBrowseHandle, BrowseOptions, BrowseServiceSpec, DnsSdExt,
    TransportProtocol, TxtRecordValue,
};

// ── Constants ────────────────────────────────────────────────────────────────

/// The mDNS service type used for Lumin classrooms (without leading underscore).
const SERVICE_TYPE: &str = "lumin-class";

/// Transport protocol for the classroom service.
const PROTOCOL: TransportProtocol = TransportProtocol::Tcp;

/// Current service protocol version — embedded in TXT records so students can
/// detect mismatched versions early.
const PROTOCOL_VERSION: &str = "1.0";

// ── Tauri events emitted during browse ───────────────────────────────────────

/// Event payload emitted when a teacher service is discovered or updated.
#[derive(Clone, serde::Serialize)]
pub struct TeacherFoundPayload {
    pub name: String,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub txt: HashMap<String, String>,
}

/// Event payload emitted when a teacher service is removed.
#[derive(Clone, serde::Serialize)]
pub struct TeacherRemovedPayload {
    pub name: String,
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
struct BrowseServiceRecord {
    name: String,
    host: Option<String>,
    port: Option<u16>,
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
pub async fn browse_teachers(app: &AppHandle) -> Result<PluginBrowseHandle, String> {
    let options = BrowseOptions {
        service: BrowseServiceSpec {
            type_name: SERVICE_TYPE.to_string(),
            protocol: PROTOCOL,
            domain: None,
            subtypes: Vec::new(),
        },
        timeout_ms: Some(0), // 0 = no timeout, browse until stopped
    };

    let app_handle = app.clone();
    let browse_channel = tauri::ipc::Channel::new(move |msg: InvokeResponseBody| {
        // The plugin sends BrowseChannelMessage as serialized JSON.
        // Deserialize and re-emit as typed Tauri events for the frontend.
        let InvokeResponseBody::Json(json_str) = msg else {
            return Ok(());
        };
        let Ok(parsed) = serde_json::from_str::<BrowseChannelMessage>(&json_str) else {
            return Ok(());
        };

        match parsed {
            BrowseChannelMessage::Service { service, .. } => {
                let txt = service
                    .txt
                    .unwrap_or_default()
                    .into_iter()
                    .map(|(k, v)| {
                        let val = match v {
                            serde_json::Value::Bool(true) => "true".to_string(),
                            serde_json::Value::Null => String::new(),
                            serde_json::Value::String(s) => s,
                            other => other.to_string(),
                        };
                        (k, val)
                    })
                    .collect();

                let _ = app_handle.emit(
                    "teacher-found",
                    TeacherFoundPayload {
                        name: service.name,
                        host: service.host,
                        port: service.port,
                        txt,
                    },
                );
            }
            BrowseChannelMessage::Stopped { browse_id, reason } => {
                eprintln!("[dns_sd] browse {browse_id} stopped: {reason}");
            }
        }
        Ok(())
    });

    app.dns_sd()
        .browse_start(app.clone(), options, browse_channel)
        .await
        .map_err(|e| format!("dns-sd browse failed: {e}"))
}

/// Stop a previously started browse session.
pub async fn stop_browse(app: &AppHandle, browse_id: u64) -> Result<(), String> {
    app.dns_sd()
        .browse_stop(app.clone(), browse_id)
        .await
        .map_err(|e| format!("dns-sd browse_stop failed: {e}"))
}
