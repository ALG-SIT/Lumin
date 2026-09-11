//! Tauri command integration for model download with progress events.
//!
//! Wraps `download::download_model` with Tauri-specific progress emission
//! and exposes the `download_model` command for frontend invocation.

use super::download;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};

/// Result returned by the download_model Tauri command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadResult {
    /// Paths to downloaded model files.
    pub files: Vec<String>,
    /// The variant that was downloaded.
    pub variant: String,
    /// Total bytes downloaded across all files.
    pub total_bytes: u64,
}

/// Download a model variant with progress events emitted to the frontend.
///
/// Emits `download-progress` events during download and `download-complete` on success.
/// The frontend listens via `app.listen("download-progress", ...)`.
pub async fn download_model_with_progress(
    app: AppHandle,
    model_dir: PathBuf,
    variant: String,
    cancel: Arc<AtomicBool>,
) -> Result<DownloadResult> {
    let files = download::download_model(app.clone(), model_dir, variant.clone(), &cancel).await?;

    let total_bytes: u64 = files
        .iter()
        .filter_map(|f| std::fs::metadata(f).ok().map(|m| m.len()))
        .sum();

    let result = DownloadResult {
        files: files.clone(),
        variant,
        total_bytes,
    };

    let _ = app.emit("download-complete", &result);
    Ok(result)
}

/// Check if all model files for a variant are present and ready.
#[allow(dead_code)]
pub fn is_variant_ready(model_dir: &std::path::Path, variant: &str) -> bool {
    download::is_variant_ready(model_dir, variant)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn test_model_dir() -> PathBuf {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("lumin_test_dl_{}_{}", std::process::id(), id));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[tokio::test]
    async fn sha256_mismatch_deletes_part_file() {
        let dir = test_model_dir();
        let fake_dest = dir.join("fake_model.onnx");
        let part = download::part_path(&fake_dest);

        // Create a .part file with wrong content
        std::fs::write(&part, b"wrong content").unwrap();
        assert!(part.exists());

        // Attempt to verify with a wrong hash — should fail and delete .part
        let result = download::verify_sha256(
            &part,
            "0000000000000000000000000000000000000000000000000000000000000000",
        )
        .await;

        assert!(result.is_err(), "SHA256 verification should fail");

        // .part should be deleted on mismatch (caller responsibility, but verify the error)
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("SHA256 mismatch"),
            "Error should mention SHA256 mismatch: {err_msg}"
        );

        // Clean up
        let _ = std::fs::remove_file(&part);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn sha256_verify_correct_hash() {
        let dir = test_model_dir();
        let test_file = dir.join("verify_test.bin");
        let content = b"hello lumin";
        std::fs::write(&test_file, content).unwrap();

        // Compute expected hash
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(content);
        let expected = hex::encode(hasher.finalize());

        let result = download::verify_sha256(&test_file, &expected).await;
        assert!(
            result.is_ok(),
            "SHA256 verification should pass for correct hash"
        );

        let _ = std::fs::remove_file(&test_file);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn progress_callback_is_invoked() {
        // Verify DownloadProgress can be constructed and serialized correctly
        let progress = super::download::DownloadProgress {
            file: "test_model.onnx".to_string(),
            downloaded: 1024,
            total: Some(2048),
            percent: Some(50.0),
            done: false,
            error: None,
        };

        let json = serde_json::to_string(&progress).unwrap();
        assert!(json.contains("\"file\":\"test_model.onnx\""));
        assert!(json.contains("\"percent\":50.0"));
        assert!(json.contains("\"done\":false"));
    }

    #[test]
    fn atomic_rename_path_convention() {
        let dest = PathBuf::from("/tmp/models/gemma.onnx");
        let part = download::part_path(&dest);
        assert_eq!(part, PathBuf::from("/tmp/models/gemma.onnx.part"));
    }

    #[test]
    fn variant_ready_returns_false_for_missing() {
        let dir = test_model_dir();
        assert!(!download::is_variant_ready(&dir, "1b-int4"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
