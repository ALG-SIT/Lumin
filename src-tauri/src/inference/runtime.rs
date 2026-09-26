//! One ONNX Runtime, with platform execution providers. No model-specific runtime.
use anyhow::{anyhow, bail, Result};
use std::{
    path::PathBuf,
    sync::{Mutex, OnceLock},
};

static RESOURCE_DIR: OnceLock<PathBuf> = OnceLock::new();
static INITIALIZED: OnceLock<std::result::Result<(), String>> = OnceLock::new();
static WEBGPU_REGISTERED: OnceLock<std::result::Result<(), String>> = OnceLock::new();

// Record the provider only after both model graphs have loaded successfully.
static ACTIVE_PROVIDER: Mutex<Option<Provider>> = Mutex::new(None);

pub fn active_provider_label() -> Option<&'static str> {
    ACTIVE_PROVIDER
        .lock()
        .ok()
        .and_then(|provider| provider.map(Provider::label))
}

pub fn record_active_provider(provider: Provider) {
    if let Ok(mut active) = ACTIVE_PROVIDER.lock() {
        *active = Some(provider);
    }
}

pub fn set_resource_dir(path: PathBuf) {
    let _ = RESOURCE_DIR.set(path);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    WebGpu,
    Cuda,
    TensorRt,
    DirectMl,
    CoreMl,
    Nnapi,
    Xnnpack,
    Cpu,
}

impl Provider {
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "webgpu" => Ok(Self::WebGpu),
            "cuda" => Ok(Self::Cuda),
            "tensorrt" => Ok(Self::TensorRt),
            "directml" => Ok(Self::DirectMl),
            "coreml" => Ok(Self::CoreMl),
            "nnapi" => Ok(Self::Nnapi),
            "xnnpack" => Ok(Self::Xnnpack),
            "cpu" => Ok(Self::Cpu),
            _ => bail!("不明な実行プロバイダ: {value}"),
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::WebGpu => {
                if cfg!(target_os = "macos") {
                    "ONNX Runtime / WebGPU (Metal)"
                } else if cfg!(target_os = "windows") {
                    "ONNX Runtime / WebGPU (Direct3D 12)"
                } else {
                    "ONNX Runtime / WebGPU (Vulkan)"
                }
            }
            Self::Cuda => "ONNX Runtime / CUDA",
            Self::TensorRt => "ONNX Runtime / TensorRT + CUDA",
            Self::DirectMl => "ONNX Runtime / DirectML",
            Self::CoreMl => "ONNX Runtime / CoreML",
            Self::Nnapi => "ONNX Runtime / NNAPI",
            Self::Xnnpack => "ONNX Runtime / XNNPACK",
            Self::Cpu => "ONNX Runtime / CPU",
        }
    }
}

pub fn selected() -> Result<Provider> {
    if let Ok(value) = std::env::var("LUMIN_EXECUTION_PROVIDER") {
        if value == "auto" {
            return Ok(default_provider());
        }
        return Provider::parse(&value);
    }
    Ok(default_provider())
}

fn default_provider() -> Provider {
    // An accelerator build feature is an explicit deployment preference.
    if cfg!(feature = "tensorrt") {
        Provider::TensorRt
    } else if cfg!(feature = "cuda") {
        Provider::Cuda
    } else if cfg!(feature = "directml") {
        Provider::DirectMl
    } else if cfg!(feature = "coreml") {
        Provider::CoreMl
    } else if cfg!(feature = "nnapi") {
        Provider::Nnapi
    } else if cfg!(feature = "xnnpack") {
        Provider::Xnnpack
    } else if cfg!(target_os = "ios") {
        // WebGPU EP は別の dylib を実行時に登録する必要があり、iOS では同梱できない。
        // 静的リンクした ONNX Runtime に入っている CoreML が唯一の GPU 経路。
        Provider::CoreMl
    } else {
        Provider::WebGpu
    }
}

/// Candidates for an unpinned desktop session. Linux prefers CUDA then WebGPU;
/// Windows falls back from WebGPU to CPU when D3D12 initialization fails.
pub fn automatic_candidates() -> Vec<Provider> {
    if cfg!(target_os = "linux") {
        vec![Provider::Cuda, Provider::WebGpu, Provider::Cpu]
    } else if cfg!(target_os = "windows") {
        vec![Provider::WebGpu, Provider::Cpu]
    } else {
        vec![default_provider()]
    }
}

/// Gemma 4's exported attention/cache graph cannot run in CoreML.
/// iOS ships CPU kernels in the same static ORT; choose them deliberately
/// for this family rather than attempting to load a desktop WebGPU dylib.
pub fn selected_for_variant(variant: &super::catalog::Variant) -> Result<Provider> {
    if cfg!(target_os = "ios")
        && variant.chat_format == super::tokenizer::ChatFormat::Gemma4Turn
        && std::env::var_os("LUMIN_EXECUTION_PROVIDER").is_none()
    {
        return Ok(Provider::Cpu);
    }
    selected()
}

/// Preserve explicit overrides and the iOS Gemma 4 CPU rule while allowing
/// Linux builds to test CUDA and WebGPU before falling back to the CPU.
pub fn candidates_for_variant(variant: &super::catalog::Variant) -> Result<Vec<Provider>> {
    if cfg!(target_os = "ios")
        && variant.chat_format == super::tokenizer::ChatFormat::Gemma4Turn
        && std::env::var_os("LUMIN_EXECUTION_PROVIDER").is_none()
    {
        return Ok(vec![Provider::Cpu]);
    }
    if let Ok(value) = std::env::var("LUMIN_EXECUTION_PROVIDER") {
        if value != "auto" {
            return Ok(vec![Provider::parse(&value)?]);
        }
    }
    Ok(automatic_candidates())
}

pub fn library_path(plugin: bool) -> Result<PathBuf> {
    let variable = if plugin {
        "LUMIN_WEBGPU_LIBRARY"
    } else {
        "ORT_DYLIB_PATH"
    };
    if let Some(path) = std::env::var_os(variable) {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Ok(path);
        }
        bail!("{variable}のライブラリが見つかりません: {}", path.display());
    }
    let name = match (
        cfg!(target_os = "windows"),
        cfg!(target_os = "macos"),
        plugin,
    ) {
        (true, _, false) => "onnxruntime.dll",
        (true, _, true) => "onnxruntime_providers_webgpu.dll",
        (_, true, false) => "libonnxruntime.dylib",
        (_, true, true) => "libonnxruntime_providers_webgpu.dylib",
        (_, _, false) => "libonnxruntime.so",
        (_, _, true) => "libonnxruntime_providers_webgpu.so",
    };
    let mut roots = Vec::new();
    if let Some(resource) = RESOURCE_DIR.get() {
        roots.push(resource.join("runtime"));
    }
    if cfg!(any(debug_assertions, test)) {
        roots.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("runtime"));
    }
    for root in roots {
        let path = root.join(name);
        if path.is_file() {
            return Ok(path);
        }
    }
    bail!("ONNX Runtimeライブラリが同梱されていません。開発時は bun run prepare:runtime を実行してください ({name})")
}

pub fn initialize() -> Result<()> {
    INITIALIZED
        .get_or_init(|| {
            // iOS は静的リンク、それ以外は同梱ライブラリの実行時読み込み。
            #[cfg(not(target_os = "ios"))]
            {
                ort::init_from(library_path(false).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?
                    .commit();
            }
            #[cfg(target_os = "ios")]
            {
                ort::init().commit();
            }
            Ok(())
        })
        .as_ref()
        .map_err(|e| anyhow!(e.clone()))
        .copied()
}

pub fn register_webgpu() -> Result<std::sync::Arc<ort::environment::Environment>> {
    initialize()?;
    let env = ort::environment::Environment::current()?;
    WEBGPU_REGISTERED
        .get_or_init(|| {
            let library = library_path(true).map_err(|e| e.to_string())?;
            #[cfg(target_os = "windows")]
            {
                let directory = library.parent().ok_or_else(|| {
                    format!(
                        "WebGPU EP DLL has no parent directory: {}",
                        library.display()
                    )
                })?;
                // Dawn's D3D12 backend loads these helper DLLs by basename.
                // Preload the official wheel copies by absolute path so they
                // work when the app's runtime directory is not on PATH.
                for name in ["dxil.dll", "dxcompiler.dll"] {
                    let dependency = directory.join(name);
                    if !dependency.is_file() {
                        return Err(format!(
                            "Dawn D3D12 dependency is missing: {}",
                            dependency.display()
                        ));
                    }
                    ort::util::preload_dylib(&dependency).map_err(|e| e.to_string())?;
                }
            }
            env.register_ep_library("lumin_webgpu", library)
                .map(|_| ())
                .map_err(|e| e.to_string())
        })
        .as_ref()
        .map_err(|e| anyhow!("WebGPU EPを登録できません: {e}"))?;
    Ok(env)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unknown_provider_cannot_silently_select_cpu() {
        assert!(Provider::parse("cdua").is_err());
        assert_eq!(Provider::parse("cpu").unwrap(), Provider::Cpu);
        assert_eq!(Provider::parse("webgpu").unwrap(), Provider::WebGpu);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_auto_prefers_cuda_then_webgpu_then_cpu() {
        assert_eq!(
            automatic_candidates(),
            vec![Provider::Cuda, Provider::WebGpu, Provider::Cpu]
        );
    }
}
