use anyhow::{Context, Result};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub file: String,
    pub downloaded: u64,
    pub total: Option<u64>,
    pub percent: Option<f64>,
    pub done: bool,
    pub error: Option<String>,
}

use super::catalog;

fn hf_url(repo: &str, path: &str) -> String {
    format!("https://huggingface.co/{}/resolve/main/{}", repo, path)
}

pub fn part_path(dest: &Path) -> PathBuf {
    PathBuf::from(format!("{}.part", dest.display()))
}

async fn compute_sha256(path: &Path) -> Result<String> {
    let mut file = tokio::fs::File::open(path)
        .await
        .with_context(|| format!("open for sha256 {:?}", path))?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = file.read(&mut buf).await.context("read chunk for sha256")?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

pub async fn verify_sha256(path: &Path, expected: &str) -> Result<()> {
    let actual = compute_sha256(path).await?;
    if !actual.eq_ignore_ascii_case(expected) {
        anyhow::bail!(
            "SHA256 mismatch for {:?}: expected {}, got {}",
            path,
            expected,
            actual
        );
    }
    Ok(())
}

fn hf_token() -> Option<String> {
    std::env::var("HF_TOKEN")
        .or_else(|_| std::env::var("HUGGING_FACE_HUB_TOKEN"))
        .ok()
        .filter(|s| !s.trim().is_empty())
}

/// Remote file size via HEAD; None when unknown (network error, non-2xx, no header)
async fn remote_content_length(client: &reqwest::Client, url: &str) -> Option<u64> {
    let mut req = client.head(url);
    if let Some(token) = hf_token() {
        req = req.bearer_auth(token);
    }
    let resp = req.send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    resp.content_length().filter(|&n| n > 0)
}

fn build_client() -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .user_agent("lumin/1.0")
        .read_timeout(Duration::from_secs(60))
        .connect_timeout(Duration::from_secs(30))
        .build()
        .context("build reqwest client")
}

async fn download_one(
    app: &AppHandle,
    url: String,
    dest: PathBuf,
    file_label: String,
    expected_sha256: Option<&'static str>,
    cancel: &AtomicBool,
) -> Result<()> {
    if let Some(parent) = dest.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .with_context(|| format!("create dir {:?}", parent))?;
    }

    let part = part_path(&dest);
    let client = build_client()?;

    // If final dest already exists, verify and skip if valid
    if dest.exists() {
        if let Some(expected) = expected_sha256 {
            match verify_sha256(&dest, expected).await {
                Ok(_) => {
                    let meta = tokio::fs::metadata(&dest).await.ok();
                    let total = meta.map(|m| m.len());
                    let _ = app.emit(
                        "download-progress",
                        DownloadProgress {
                            file: file_label.clone(),
                            downloaded: total.unwrap_or(0),
                            total,
                            percent: Some(100.0),
                            done: true,
                            error: None,
                        },
                    );
                    return Ok(());
                }
                Err(e) => {
                    eprintln!(
                        "[download] SHA256 mismatch for existing {:?}: {e:?} — re-downloading",
                        dest
                    );
                    let _ = tokio::fs::remove_file(&dest).await;
                }
            }
        } else {
            // No expected hash: only trust the file when its size matches the remote
            let local_len = tokio::fs::metadata(&dest)
                .await
                .map(|m| m.len())
                .unwrap_or(0);
            match remote_content_length(&client, &url).await {
                Some(remote_len) if remote_len == local_len => {
                    let _ = app.emit(
                        "download-progress",
                        DownloadProgress {
                            file: file_label.clone(),
                            downloaded: local_len,
                            total: Some(remote_len),
                            percent: Some(100.0),
                            done: true,
                            error: None,
                        },
                    );
                    return Ok(());
                }
                remote_len => {
                    eprintln!(
                        "[download] existing {:?} size {local_len} != remote {remote_len:?} — re-downloading",
                        dest
                    );
                    let _ = tokio::fs::remove_file(&dest).await;
                }
            }
        }
    }

    // Remove stale .part from previous interrupted download
    if part.exists() {
        let _ = tokio::fs::remove_file(&part).await;
    }

    let mut attempt = 0;
    let max_attempts = 3;
    let mut last_err: Option<anyhow::Error> = None;

    while attempt < max_attempts {
        attempt += 1;
        let send_res = async {
            let mut req = client.get(&url);
            if let Some(token) = hf_token() {
                req = req.bearer_auth(token);
            }
            let resp = req.send().await.with_context(|| format!("GET {url}"))?;
            if !resp.status().is_success() {
                anyhow::bail!("GET {url} failed: {}", resp.status());
            }
            let total = resp.content_length();
            let mut stream = resp.bytes_stream();
            let mut file = tokio::fs::File::create(&part)
                .await
                .with_context(|| format!("create file {:?}", part))?;

            let mut downloaded: u64 = 0;
            let mut last_emit = std::time::Instant::now();

            while let Some(chunk) = stream.next().await {
                if cancel.load(Ordering::Acquire) {
                    let _ = tokio::fs::remove_file(&part).await;
                    return Err(anyhow::anyhow!("cancelled"));
                }

                let chunk = chunk.context("stream chunk")?;
                file.write_all(&chunk).await.context("write chunk")?;
                downloaded += chunk.len() as u64;

                if last_emit.elapsed().as_millis() > 100 {
                    let percent = total.map(|t| (downloaded as f64 / t as f64) * 100.0);
                    let _ = app.emit(
                        "download-progress",
                        DownloadProgress {
                            file: file_label.clone(),
                            downloaded,
                            total,
                            percent,
                            done: false,
                            error: None,
                        },
                    );
                    last_emit = std::time::Instant::now();
                }
            }

            file.flush().await.context("flush")?;
            drop(file);

            if let Some(expected) = expected_sha256 {
                verify_sha256(&part, expected).await?;
            }

            tokio::fs::rename(&part, &dest)
                .await
                .with_context(|| format!("rename {:?} -> {:?}", part, dest))?;

            let _ = app.emit(
                "download-progress",
                DownloadProgress {
                    file: file_label.clone(),
                    downloaded,
                    total,
                    percent: Some(100.0),
                    done: true,
                    error: None,
                },
            );
            Ok::<(), anyhow::Error>(())
        }
        .await;

        match send_res {
            Ok(_) => return Ok(()),
            Err(e) => {
                let _ = tokio::fs::remove_file(&part).await;
                let err_str = e.to_string();
                let is_not_found_or_auth =
                    err_str.contains("401") || err_str.contains("404") || err_str.contains("403");
                let is_cancelled = err_str == "cancelled";
                let has_attempt_left =
                    attempt < max_attempts && !is_not_found_or_auth && !is_cancelled;
                eprintln!(
                    "[download] attempt {}/{} for {} failed: {e:?} (retry={})",
                    attempt, max_attempts, file_label, has_attempt_left
                );
                last_err = Some(e);
                if !has_attempt_left {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(500 * attempt as u64)).await;
            }
        }
    }

    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("download failed for {file_label}")))
}

/// Download the ONNX graphs + tokenizer for the selected variant.
///
/// Files land in the variant's own directory (see [`catalog::Variant::dir`]),
/// so installing one variant never overwrites another's external data.
/// Emits `download-progress` events and `download-complete` at the end.
pub async fn download_model(
    app: AppHandle,
    model_dir: PathBuf,
    variant: String,
    cancel: &AtomicBool,
) -> Result<Vec<String>> {
    let spec_variant = catalog::find(&variant).ok_or_else(|| {
        anyhow::anyhow!(
            "unknown variant: {variant} (choose one of: {})",
            catalog::known_ids()
        )
    })?;
    let dest_dir = spec_variant.dir(&model_dir);

    let mut downloaded = Vec::new();
    for spec in spec_variant.files {
        let url = hf_url(spec_variant.repo, spec.url_path);
        let dest = dest_dir.join(spec.dest_name);
        let label = spec.dest_name.to_string();

        let _ = app.emit(
            "download-progress",
            DownloadProgress {
                file: label.clone(),
                downloaded: 0,
                total: None,
                percent: Some(0.0),
                done: false,
                error: None,
            },
        );

        match download_one(
            &app,
            url.clone(),
            dest.clone(),
            label.clone(),
            spec.expected_sha256,
            cancel,
        )
        .await
        {
            Ok(_) => {
                downloaded.push(dest.to_string_lossy().to_string());
            }
            Err(e) => {
                let _ = app.emit(
                    "download-progress",
                    DownloadProgress {
                        file: label.clone(),
                        downloaded: 0,
                        total: None,
                        percent: None,
                        done: true,
                        error: Some(e.to_string()),
                    },
                );
                let _ = tokio::fs::remove_file(part_path(&dest)).await;
                let _ = tokio::fs::remove_file(&dest).await;
                return Err(e.context(format!("failed to download {label} from {url}")));
            }
        }
    }

    let _ = app.emit("download-complete", &downloaded);
    Ok(downloaded)
}

/// Check if every file of the variant is present on disk.
pub fn is_variant_ready(model_dir: &Path, variant: &str) -> bool {
    catalog::find(variant).is_some_and(|v| v.is_installed(model_dir))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_catalog_variant_resolves_to_a_repo_url() {
        for v in catalog::VARIANTS {
            for f in v.files {
                let url = hf_url(v.repo, f.url_path);
                assert!(
                    url.starts_with("https://huggingface.co/"),
                    "{}: {url}",
                    v.id
                );
                assert!(url.contains(v.repo), "{}: {url}", v.id);
            }
        }
    }

    #[test]
    fn gemma4_downloads_come_from_the_gemma4_repos() {
        let e2b = catalog::find("4-e2b-int4").unwrap();
        let e4b = catalog::find("4-e4b-int4").unwrap();
        assert_eq!(e2b.repo, "onnx-community/gemma-4-E2B-it-ONNX");
        assert_eq!(e4b.repo, "onnx-community/gemma-4-E4B-it-ONNX");

        // Every Gemma 4 file must be hash-pinned: these are multi-GB weights
        // fetched over the network into a classroom device.
        for v in [e2b, e4b] {
            for f in v.files {
                assert!(
                    f.expected_sha256.is_some_and(|h| h.len() == 64),
                    "{}: {} is missing a SHA256 pin",
                    v.id,
                    f.dest_name
                );
            }
        }
    }

    #[test]
    fn unknown_variant_is_not_ready() {
        assert!(!is_variant_ready(Path::new("/nonexistent"), "gemma-9000"));
    }

    #[test]
    fn variant_ready_requires_all_files() {
        let dir = tempfile::tempdir().unwrap();
        let v = catalog::find("4-e2b-int4").unwrap();
        assert!(!is_variant_ready(dir.path(), v.id));

        let vdir = v.dir(dir.path());
        std::fs::create_dir_all(&vdir).unwrap();
        for (i, f) in v.files.iter().enumerate() {
            assert!(
                !is_variant_ready(dir.path(), v.id),
                "must not be ready with only {i} of {} files",
                v.files.len()
            );
            std::fs::write(vdir.join(f.dest_name), b"x").unwrap();
        }
        assert!(is_variant_ready(dir.path(), v.id));
    }
}
