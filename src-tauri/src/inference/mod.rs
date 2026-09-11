pub mod bench;
pub mod download;
pub mod generate;
pub mod model_download;
pub mod session;
pub mod tokenizer;

use crate::inference::session::AppState;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use sysinfo::System;
use tauri::State;

const SHA_1B_INT4_ONNX: &str = "69686023e5892376e38fcbcdd0c77af432c55b3bcd03aee6d561bd1f04507da0";
const APPROX_1B_INT4_BYTES: u64 = 1_200_000_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    pub id: String,
    pub name: String,
    pub size_bytes: u64,
    pub variant: String,
    pub status: String,
    pub progress: Option<f64>,
    pub recommended: bool,
}

fn physical_memory_gb() -> u64 {
    let system = System::new_all();
    system.total_memory() / (1024 * 1024 * 1024)
}

fn variant_file_paths(model_dir: &Path) -> Vec<PathBuf> {
    vec![
        model_dir.join("gemma-3-1b-it-int4.onnx"),
        model_dir.join("model_q4.onnx_data"),
        model_dir.join("tokenizer.json"),
    ]
}

fn installed_variant_size(model_dir: &Path) -> u64 {
    variant_file_paths(model_dir)
        .iter()
        .filter_map(|path| std::fs::metadata(path).ok().map(|m| m.len()))
        .sum()
}

fn is_variant_installed(model_dir: &Path) -> bool {
    variant_file_paths(model_dir)
        .iter()
        .all(|path| path.exists())
}

#[tauri::command]
pub async fn list_models(state: State<'_, AppState>) -> Result<Vec<ModelEntry>, String> {
    let model_dir = &state.model_dir;
    let installed = is_variant_installed(model_dir);
    let size_bytes = if installed {
        installed_variant_size(model_dir)
    } else {
        APPROX_1B_INT4_BYTES
    };

    let memory_gb = physical_memory_gb();
    let recommended = memory_gb >= 8;

    Ok(vec![ModelEntry {
        id: "gemma3-1b-int4".to_string(),
        name: "Gemma 3 1B INT4".to_string(),
        size_bytes,
        variant: "1b-int4".to_string(),
        status: if installed {
            "installed".to_string()
        } else {
            "available".to_string()
        },
        progress: None,
        recommended,
    }])
}

#[tauri::command]
pub async fn import_model(
    onnx_path: String,
    tokenizer_path: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let model_dir = &state.model_dir;
    let onnx_src = PathBuf::from(onnx_path);
    let tokenizer_src = PathBuf::from(tokenizer_path);

    if !onnx_src.exists() {
        return Err("ONNX file not found".to_string());
    }
    if !tokenizer_src.exists() {
        return Err("tokenizer.json not found".to_string());
    }

    let onnx_dest = model_dir.join("gemma-3-1b-it-int4.onnx");
    let tokenizer_dest = model_dir.join("tokenizer.json");

    tokio::fs::copy(&onnx_src, &onnx_dest)
        .await
        .map_err(|e| format!("failed to copy ONNX file: {e}"))?;
    tokio::fs::copy(&tokenizer_src, &tokenizer_dest)
        .await
        .map_err(|e| format!("failed to copy tokenizer: {e}"))?;

    download::verify_sha256(&onnx_dest, SHA_1B_INT4_ONNX)
        .await
        .map_err(|e| format!("ONNX SHA256 verification failed: {e}"))?;
    download::verify_sha256(&tokenizer_dest, download::SHA_1B_TOKENIZER)
        .await
        .map_err(|e| format!("tokenizer SHA256 verification failed: {e}"))?;

    Ok(())
}

#[tauri::command]
pub fn cancel_download(state: State<'_, AppState>) {
    state.download_cancelled.store(true, Ordering::SeqCst);
}
