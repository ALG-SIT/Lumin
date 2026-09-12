//! One ONNX Runtime, with platform execution providers. No model-specific runtime.
use anyhow::{anyhow, bail, Result};
use std::{path::PathBuf, sync::OnceLock};

static RESOURCE_DIR: OnceLock<PathBuf> = OnceLock::new();
static INITIALIZED: OnceLock<std::result::Result<(), String>> = OnceLock::new();
static WEBGPU_REGISTERED: OnceLock<std::result::Result<(), String>> = OnceLock::new();

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
            Self::Cpu => "ONNX Runtime / CPU (explicit)",
        }
    }
}

pub fn selected() -> Result<Provider> {
    if let Ok(value) = std::env::var("LUMIN_EXECUTION_PROVIDER") {
        return Provider::parse(&value);
    }
    // An accelerator build feature is an explicit deployment preference.
    if cfg!(feature = "tensorrt") {
        Ok(Provider::TensorRt)
    } else if cfg!(feature = "cuda") {
        Ok(Provider::Cuda)
    } else if cfg!(feature = "directml") {
        Ok(Provider::DirectMl)
    } else if cfg!(feature = "coreml") {
        Ok(Provider::CoreMl)
    } else if cfg!(feature = "nnapi") {
        Ok(Provider::Nnapi)
    } else if cfg!(feature = "xnnpack") {
        Ok(Provider::Xnnpack)
    } else if cfg!(target_os = "ios") {
        // WebGPU EP は別の dylib を実行時に登録する必要があり、iOS では同梱できない。
        // 静的リンクした ONNX Runtime に入っている CoreML が唯一の GPU 経路。
        Ok(Provider::CoreMl)
    } else {
        Ok(Provider::WebGpu)
    }
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
            env.register_ep_library(
                "lumin_webgpu",
                library_path(true).map_err(|e| e.to_string())?,
            )
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
}
