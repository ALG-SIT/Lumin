//! One ONNX Runtime, with platform execution providers. No model-specific runtime.
use anyhow::{anyhow, bail, Result};
use std::{
    path::PathBuf,
    sync::{Mutex, OnceLock},
};

static RESOURCE_DIR: OnceLock<PathBuf> = OnceLock::new();
static INITIALIZED: OnceLock<std::result::Result<(), String>> = OnceLock::new();
static WEBGPU_REGISTERED: OnceLock<std::result::Result<(), String>> = OnceLock::new();
static PROVIDER_STATUS: OnceLock<Mutex<ProviderStatus>> = OnceLock::new();

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderStatus {
    pub requested: String,
    pub selected: String,
    pub is_gpu: bool,
    pub fallback_reason: Option<String>,
}

impl Default for ProviderStatus {
    fn default() -> Self {
        Self {
            requested: std::env::var("LUMIN_EXECUTION_PROVIDER").unwrap_or_else(|_| "auto".into()),
            selected: "未初期化".into(),
            is_gpu: false,
            fallback_reason: None,
        }
    }
}

pub fn provider_status() -> ProviderStatus {
    PROVIDER_STATUS
        .get_or_init(|| Mutex::new(ProviderStatus::default()))
        .lock()
        .map(|status| status.clone())
        .unwrap_or_default()
}

pub fn record_provider(provider: Provider, fallback_reason: Option<String>) {
    let mut status = PROVIDER_STATUS
        .get_or_init(|| Mutex::new(ProviderStatus::default()))
        .lock()
        .expect("provider status mutex poisoned");
    status.selected = provider.label().into();
    status.is_gpu = !matches!(provider, Provider::Cpu | Provider::Xnnpack);
    status.fallback_reason = fallback_reason;
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

/// Candidates for an unpinned desktop session. WSL/NVIDIA benefits from CUDA
/// first; WebGPU keeps other Linux GPU vendors on the portable Vulkan path.
pub fn automatic_candidates() -> Vec<Provider> {
    if cfg!(target_os = "linux") {
        vec![Provider::Cuda, Provider::WebGpu, Provider::Cpu]
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

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_auto_prefers_cuda_then_webgpu_then_cpu() {
        assert_eq!(
            automatic_candidates(),
            vec![Provider::Cuda, Provider::WebGpu, Provider::Cpu]
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[ignore = "requires a Linux Vulkan driver and the bundled WebGPU EP"]
    fn linux_webgpu_device_is_available() {
        let env = register_webgpu().expect("WebGPU EP registration");
        assert!(env.devices().any(|device| {
            device.ep().ok() == Some("WebGpuExecutionProvider")
                && device.hardware_device().ty() == ort::memory::DeviceType::GPU
        }));
    }

    #[cfg(not(target_os = "ios"))]
    #[test]
    #[ignore = "requires the bundled CUDA EP and host CUDA runtime libraries"]
    fn cuda_ep_can_be_registered() {
        initialize().expect("ONNX Runtime initialization");
        let builder = ort::session::Session::builder().expect("session builder");
        builder
            .with_execution_providers([ort::ep::CUDA::default().build().error_on_failure()])
            .expect("CUDA EP registration");
    }
}
