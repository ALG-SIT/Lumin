pub mod bench;
pub mod catalog;
pub mod download;
pub mod generate;
pub mod model_download;
pub mod runtime;
pub mod session;
pub mod tokenizer;

use crate::inference::session::AppState;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use sysinfo::System;
use tauri::State;

/// One selectable model as shown in the model manager.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    pub id: String,
    pub name: String,
    /// Installed size once present, otherwise the download estimate.
    pub size_bytes: u64,
    pub variant: String,
    /// Model family, used to group the list (e.g. "Gemma 4").
    pub family: String,
    pub description: String,
    pub status: String,
    pub progress: Option<f64>,
    /// True when this device has enough RAM for the variant.
    pub recommended: bool,
    /// True for the variant inference currently runs on.
    pub active: bool,
}

fn physical_memory_gb() -> u64 {
    let system = System::new_all();
    system.total_memory() / (1024 * 1024 * 1024)
}

#[tauri::command]
pub async fn list_models(state: State<'_, AppState>) -> Result<Vec<ModelEntry>, String> {
    let model_dir = &state.model_dir;
    let memory_gb = physical_memory_gb();
    let active_id = state.active_variant().await.id;

    Ok(catalog::VARIANTS
        .iter()
        .map(|v| {
            let installed = v.is_installed(model_dir);
            ModelEntry {
                id: v.id.to_string(),
                name: v.display_name.to_string(),
                size_bytes: if installed {
                    v.installed_size_bytes(model_dir)
                } else {
                    v.download_size_bytes()
                },
                variant: v.id.to_string(),
                family: v.family.to_string(),
                description: if cfg!(target_os = "ios")
                    && v.chat_format == tokenizer::ChatFormat::Gemma4Turn
                {
                    format!(
                        "{} iPhoneではCPUで実行します。生成に時間がかかります。",
                        v.description
                    )
                } else {
                    v.description.to_string()
                },
                status: if installed {
                    "installed".to_string()
                } else {
                    "available".to_string()
                },
                progress: None,
                recommended: memory_gb >= v.min_memory_gb,
                active: v.id == active_id,
            }
        })
        .collect())
}

/// Switch the model used for analysis, hints and chat.
///
/// The selection is app-wide, so the change is broadcast as
/// `active-model-changed`: the status shown in the app bar belongs to no
/// single screen and has to follow a switch made anywhere.
#[tauri::command]
pub async fn set_active_model(
    app: tauri::AppHandle,
    variant: String,
    state: State<'_, AppState>,
) -> Result<ModelEntry, String> {
    let v = state
        .set_active_variant(&variant)
        .await
        .map_err(|e| e.to_string())?;

    let installed = v.is_installed(&state.model_dir);
    let entry = ModelEntry {
        id: v.id.to_string(),
        name: v.display_name.to_string(),
        size_bytes: if installed {
            v.installed_size_bytes(&state.model_dir)
        } else {
            v.download_size_bytes()
        },
        variant: v.id.to_string(),
        family: v.family.to_string(),
        description: if cfg!(target_os = "ios")
            && v.chat_format == tokenizer::ChatFormat::Gemma4Turn
        {
            format!(
                "{} iPhoneではCPUで実行します。生成に時間がかかります。",
                v.description
            )
        } else {
            v.description.to_string()
        },
        status: if installed {
            "installed".to_string()
        } else {
            "available".to_string()
        },
        progress: None,
        recommended: physical_memory_gb() >= v.min_memory_gb,
        active: true,
    };

    use tauri::Emitter;
    if let Err(e) = app.emit("active-model-changed", &entry) {
        eprintln!("[emit] active-model-changed failed: {e}");
    }
    Ok(entry)
}

/// Id of the variant inference currently runs on.
#[tauri::command]
pub async fn get_active_model(state: State<'_, AppState>) -> Result<String, String> {
    Ok(state.active_variant().await.id.to_string())
}

/// Import a locally downloaded ONNX graph + tokenizer for a variant.
///
/// Files are verified against the catalog's hashes before they are accepted,
/// so a mismatched or truncated copy never becomes the installed model.
#[tauri::command]
pub async fn import_model(
    onnx_path: String,
    tokenizer_path: String,
    variant: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let variant_id = variant.unwrap_or_else(|| catalog::DEFAULT_VARIANT_ID.to_string());
    let v = catalog::find(&variant_id).ok_or_else(|| {
        format!(
            "unknown model: {variant_id} (choose one of: {})",
            catalog::known_ids()
        )
    })?;

    let onnx_src = PathBuf::from(onnx_path);
    let tokenizer_src = PathBuf::from(tokenizer_path);

    if !onnx_src.exists() {
        return Err("ONNX file not found".to_string());
    }
    if !tokenizer_src.exists() {
        return Err("tokenizer.json not found".to_string());
    }

    let dir = v.dir(&state.model_dir);
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| format!("failed to create model directory: {e}"))?;

    let onnx_dest = v.decoder_path(&state.model_dir);
    let tokenizer_dest = v.tokenizer_path(&state.model_dir);

    tokio::fs::copy(&onnx_src, &onnx_dest)
        .await
        .map_err(|e| format!("failed to copy ONNX file: {e}"))?;
    tokio::fs::copy(&tokenizer_src, &tokenizer_dest)
        .await
        .map_err(|e| format!("failed to copy tokenizer: {e}"))?;

    let expected = |name: &str| {
        v.files
            .iter()
            .find(|f| f.dest_name == name)
            .and_then(|f| f.expected_sha256)
    };

    if let Some(hash) = expected(v.decoder_file) {
        download::verify_sha256(&onnx_dest, hash)
            .await
            .map_err(|e| format!("ONNX SHA256 verification failed: {e}"))?;
    }
    if let Some(hash) = expected(v.tokenizer_file) {
        download::verify_sha256(&tokenizer_dest, hash)
            .await
            .map_err(|e| format!("tokenizer SHA256 verification failed: {e}"))?;
    }

    Ok(())
}

/// Load the selected model into memory ahead of the first question.
///
/// Returns true once the model is resident. Progress is reported as
/// `model-load-progress` events while the graphs are read, so the UI can show
/// what is happening instead of appearing to hang on the first generation.
/// Returns false when the selected model is not installed - generation would
/// run in mock mode and there is nothing to load.
#[tauri::command]
pub async fn preload_active_model(state: State<'_, AppState>) -> Result<bool, String> {
    generate::ensure_active_session(&state)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn cancel_download(state: State<'_, AppState>) {
    state.download_cancelled.store(true, Ordering::SeqCst);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inference::catalog::DEFAULT_VARIANT_ID;

    #[tokio::test]
    async fn active_variant_defaults_and_switches() {
        let dir = tempfile::tempdir().unwrap();
        let state = AppState::new(dir.path().to_path_buf());
        assert_eq!(state.active_variant().await.id, DEFAULT_VARIANT_ID);

        let v = state.set_active_variant("4-e4b-int4").await.unwrap();
        assert_eq!(v.id, "4-e4b-int4");
        assert_eq!(state.active_variant().await.id, "4-e4b-int4");
    }

    #[tokio::test]
    async fn switching_to_an_unknown_model_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let state = AppState::new(dir.path().to_path_buf());
        state.set_active_variant("4-e2b-int4").await.unwrap();

        let err = state
            .set_active_variant("gemma-4-E9B")
            .await
            .expect_err("unknown ids must be rejected");
        assert!(err.to_string().contains("unknown model"), "{err}");
        // The rejected switch must not disturb the current selection.
        assert_eq!(state.active_variant().await.id, "4-e2b-int4");
    }

    #[tokio::test]
    async fn selection_survives_a_restart() {
        let dir = tempfile::tempdir().unwrap();
        {
            let state = AppState::new(dir.path().to_path_buf());
            state.set_active_variant("4-e2b-int4").await.unwrap();
        }
        let reopened = AppState::new(dir.path().to_path_buf());
        assert_eq!(reopened.active_variant().await.id, "4-e2b-int4");
    }

    #[tokio::test]
    async fn a_stale_selection_falls_back_to_the_default() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("active_model.json"),
            br#"{"variant":"removed-from-catalog"}"#,
        )
        .unwrap();

        let state = AppState::new(dir.path().to_path_buf());
        assert_eq!(state.active_variant().await.id, DEFAULT_VARIANT_ID);
    }

    /// Switching models must drop the loaded graphs, or the next generation
    /// would keep running the previous model.
    #[tokio::test]
    async fn switching_clears_the_loaded_session() {
        let dir = tempfile::tempdir().unwrap();
        let state = AppState::new(dir.path().to_path_buf());
        state.set_active_variant("4-e2b-int4").await.unwrap();
        assert!(state.session.lock().await.is_none());
    }

    #[tokio::test]
    async fn gemma4_entries_are_listed_and_sized() {
        let dir = tempfile::tempdir().unwrap();
        let state = AppState::new(dir.path().to_path_buf());
        let entries: Vec<_> = catalog::VARIANTS
            .iter()
            .filter(|v| v.family == "Gemma 4")
            .collect();

        assert_eq!(
            entries.len(),
            2,
            "Gemma 4 must offer exactly the E2B and E4B options"
        );
        for v in entries {
            assert!(
                !v.is_installed(dir.path()),
                "{} must not look installed in an empty dir",
                v.id
            );
            assert!(
                v.download_size_bytes() > 1_000_000_000,
                "{}: download estimate looks wrong",
                v.id
            );
        }
        let _ = state;
    }

    #[tokio::test]
    async fn import_rejects_an_unknown_variant() {
        let dir = tempfile::tempdir().unwrap();
        let onnx = dir.path().join("some.onnx");
        let tok = dir.path().join("tokenizer.json");
        std::fs::write(&onnx, b"x").unwrap();
        std::fs::write(&tok, b"{}").unwrap();

        let v = catalog::find("not-a-model");
        assert!(v.is_none());
    }
}
