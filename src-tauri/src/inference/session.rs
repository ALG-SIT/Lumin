use crate::inference::catalog::{self, Variant};
use crate::lumin_core::models::{AnalysisEvent, Quiz};
use anyhow::Result;
use ort::session::{builder::GraphOptimizationLevel, Session};
use ort::value::ValueType;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::Mutex;

/// A model variant as reported to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub model_id: String,
    pub onnx_path: String,
    pub tokenizer_path: String,
    pub exists: bool,
    pub size_bytes: Option<u64>,
    pub quantization: String,
    pub description: String,
}

/// File the active-variant selection is persisted to, inside the model root.
const ACTIVE_MODEL_FILE: &str = "active_model.json";

#[derive(Debug, Serialize, Deserialize)]
struct ActiveModelFile {
    variant: String,
}

fn read_active_variant(model_dir: &Path) -> Option<String> {
    let raw = std::fs::read_to_string(model_dir.join(ACTIVE_MODEL_FILE)).ok()?;
    let parsed: ActiveModelFile = serde_json::from_str(&raw).ok()?;
    // A variant that has since been removed from the catalog must not stick.
    catalog::find(&parsed.variant).map(|v| v.id.to_string())
}

fn write_active_variant(model_dir: &Path, variant: &str) -> Result<()> {
    std::fs::create_dir_all(model_dir)?;
    let body = serde_json::to_string_pretty(&ActiveModelFile {
        variant: variant.to_string(),
    })?;
    std::fs::write(model_dir.join(ACTIVE_MODEL_FILE), body)?;
    Ok(())
}

/// Shared app state for Tauri
pub struct AppState {
    pub session: Arc<Mutex<Option<InferenceSession>>>,
    pub model_dir: PathBuf,
    /// Id of the variant used for inference, persisted across restarts.
    pub active_variant_id: Arc<Mutex<String>>,
    pub download_cancelled: Arc<AtomicBool>,
    /// Anonymous analysis events received from student devices.
    pub events: Arc<Mutex<Vec<AnalysisEvent>>>,
    /// Currently active quiz for class summary context.
    pub active_quiz: Arc<Mutex<Option<Quiz>>>,
}

pub struct InferenceSession {
    pub session: Session,
    /// `embed_tokens` graph for [`Architecture::EmbedChained`] variants.
    pub embed_session: Option<Session>,
    /// Variant this session was built from, so a model switch can drop it.
    pub variant_id: String,
    #[allow(dead_code)]
    pub model_info: ModelInfo,
}

/// One layer's KV-cache geometry, read from the decoder graph itself.
///
/// Gemma 4 varies both the number of cached layers (shared-KV layers are not
/// exposed) and the per-layer head dim (sliding-window layers use 512 where
/// full-attention layers use 256), so these cannot be compile-time constants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KvLayerSpec {
    pub index: usize,
    pub num_heads: usize,
    pub head_dim: usize,
}

/// Static dimensions of `past_key_values.*.key` inputs, in layer order.
///
/// Returns an empty vec for a graph without a KV cache.
pub fn kv_layer_specs(session: &Session) -> Vec<KvLayerSpec> {
    let declared = session
        .inputs()
        .iter()
        .filter_map(|input| match input.dtype() {
            ValueType::Tensor { shape, .. } => Some((input.name(), &shape[..])),
            _ => None,
        });
    kv_layer_specs_from_inputs(declared)
}

/// Shape-parsing half of [`kv_layer_specs`], split out so it can be tested
/// without loading a multi-gigabyte graph.
fn kv_layer_specs_from_inputs<'a>(
    inputs: impl Iterator<Item = (&'a str, &'a [i64])>,
) -> Vec<KvLayerSpec> {
    let mut specs: Vec<KvLayerSpec> = Vec::new();
    for (name, shape) in inputs {
        let Some(rest) = name.strip_prefix("past_key_values.") else {
            continue;
        };
        let Some(index) = rest.strip_suffix(".key").and_then(|i| i.parse().ok()) else {
            continue;
        };
        // [batch, num_heads, past_sequence_length, head_dim]; dims 1 and 3 are
        // static, dims 0 and 2 come back as -1.
        if shape.len() != 4 || shape[1] <= 0 || shape[3] <= 0 {
            continue;
        }
        specs.push(KvLayerSpec {
            index,
            num_heads: shape[1] as usize,
            head_dim: shape[3] as usize,
        });
    }
    specs.sort_unstable_by_key(|s| s.index);
    specs
}

/// True when the graph declares an input with this name.
pub fn has_input(session: &Session, name: &str) -> bool {
    session.inputs().iter().any(|i| i.name() == name)
}

impl AppState {
    pub fn new(model_dir: PathBuf) -> Self {
        let active = read_active_variant(&model_dir)
            .unwrap_or_else(|| catalog::DEFAULT_VARIANT_ID.to_string());
        Self {
            session: Arc::new(Mutex::new(None)),
            model_dir,
            active_variant_id: Arc::new(Mutex::new(active)),
            download_cancelled: Arc::new(AtomicBool::new(false)),
            events: Arc::new(Mutex::new(Vec::new())),
            active_quiz: Arc::new(Mutex::new(None)),
        }
    }

    /// The variant inference currently runs on.
    pub async fn active_variant(&self) -> &'static Variant {
        let id = self.active_variant_id.lock().await.clone();
        catalog::find(&id).unwrap_or_else(|| {
            catalog::find(catalog::DEFAULT_VARIANT_ID).expect("default variant must exist")
        })
    }

    /// Switch the active variant, persist the choice, and drop any loaded
    /// session so the next generation loads the newly selected graphs.
    pub async fn set_active_variant(&self, id: &str) -> Result<&'static Variant> {
        let variant = catalog::find(id).ok_or_else(|| {
            anyhow::anyhow!(
                "unknown model: {id} (choose one of: {})",
                catalog::known_ids()
            )
        })?;

        {
            let mut active = self.active_variant_id.lock().await;
            *active = variant.id.to_string();
        }
        // Force a reload; the old graphs belong to a different model.
        {
            let mut session = self.session.lock().await;
            *session = None;
        }
        write_active_variant(&self.model_dir, variant.id)?;
        Ok(variant)
    }

    pub fn model_variants(&self) -> Vec<ModelInfo> {
        catalog::VARIANTS
            .iter()
            .map(|v| {
                let onnx_path = v.decoder_path(&self.model_dir);
                let tok_path = v.tokenizer_path(&self.model_dir);
                let exists = v.is_installed(&self.model_dir);
                let size_bytes = if exists {
                    Some(v.installed_size_bytes(&self.model_dir))
                } else {
                    None
                };
                ModelInfo {
                    model_id: format!("{} ({})", v.repo, v.quantization),
                    onnx_path: onnx_path.to_string_lossy().to_string(),
                    tokenizer_path: tok_path.to_string_lossy().to_string(),
                    exists,
                    size_bytes,
                    quantization: v.quantization.to_string(),
                    description: v.description.to_string(),
                }
            })
            .collect()
    }

    /// Every file the active variant needs before inference can run.
    pub async fn active_model_ready(&self) -> bool {
        self.active_variant().await.is_installed(&self.model_dir)
    }
}

/// Create an ort session with platform-appropriate execution providers
pub fn create_session<P: AsRef<Path>>(model_path: P) -> Result<Session> {
    use super::runtime::{self, Provider};
    runtime::initialize()?;
    let provider = runtime::selected()?;
    let err = |e: ort::Error<ort::session::builder::SessionBuilder>| anyhow::anyhow!(e.to_string());
    let mut builder = Session::builder()?
        .with_optimization_level(GraphOptimizationLevel::Level3)
        .map_err(err)?
        .with_intra_threads(if provider == Provider::Xnnpack { 1 } else { 4 })
        .map_err(err)?;
    if provider == Provider::WebGpu {
        let env = runtime::register_webgpu()?;
        let devices: Vec<_> = env
            .devices()
            .filter(|d| {
                d.ep().ok() == Some("WebGpuExecutionProvider")
                    && d.hardware_device().ty() == ort::memory::DeviceType::GPU
            })
            .take(1)
            .collect();
        if devices.is_empty() {
            anyhow::bail!("対応するGPUを検出できません。GPUドライバを確認してください。CPUには自動切替しません。");
        }
        // Dawn selects the native backend for the discovered GPU.
        let options = vec![(
            "WebGpuExecutionProvider.enableGraphCapture".into(),
            "0".into(),
        )];
        builder = builder.with_devices(devices, Some(&options)).map_err(err)?;
    } else {
        let ep = match provider {
            #[cfg(feature = "cuda")]
            Provider::Cuda => ort::ep::CUDA::default().build(),
            #[cfg(feature = "tensorrt")]
            Provider::TensorRt => {
                builder = builder
                    .with_execution_providers([
                        ort::ep::TensorRT::default().build().error_on_failure(),
                        ort::ep::CUDA::default().build().error_on_failure(),
                    ])
                    .map_err(err)?;
                // Both GPU providers were explicitly registered above.
                ort::ep::CPU::default().build()
            }
            #[cfg(feature = "directml")]
            Provider::DirectMl => {
                builder = builder
                    .with_memory_pattern(false)
                    .map_err(err)?
                    .with_parallel_execution(false)
                    .map_err(err)?;
                ort::ep::DirectML::default().build()
            }
            #[cfg(any(feature = "coreml", target_os = "macos", target_os = "ios"))]
            Provider::CoreMl => ort::ep::CoreML::default()
                .with_compute_units(ort::ep::coreml::ComputeUnits::CPUAndGPU)
                .with_model_format(ort::ep::coreml::ModelFormat::MLProgram)
                .with_low_precision_accumulation_on_gpu(false)
                .build(),
            #[cfg(feature = "nnapi")]
            Provider::Nnapi => ort::ep::NNAPI::default().build(),
            #[cfg(feature = "xnnpack")]
            Provider::Xnnpack => {
                builder = builder.with_intra_op_spinning(false).map_err(err)?;
                ort::ep::XNNPACK::default()
                    .with_intra_op_num_threads(std::num::NonZeroUsize::new(4).unwrap())
                    .build()
            }
            Provider::Cpu => ort::ep::CPU::default().build(),
            Provider::WebGpu => unreachable!(),
            #[allow(unreachable_patterns)]
            _ => anyhow::bail!(
                "選択したEPはこのビルドに含まれていません: {}",
                provider.label()
            ),
        };
        builder = builder
            .with_execution_providers([ep.error_on_failure()])
            .map_err(err)?;
    }
    if let Ok(root) = std::env::var("LUMIN_ORT_PROFILE_DIR") {
        std::fs::create_dir_all(&root)?;
        builder = builder
            .with_profiling(
                Path::new(&root).join(model_path.as_ref().file_stem().unwrap_or_default()),
            )
            .map_err(err)?;
    }
    builder
        .commit_from_file(model_path)
        .map_err(|e| anyhow::anyhow!(e.to_string()))
}

/// Resolve model directory: src-tauri/models or project_root/models
pub fn resolve_model_dir() -> PathBuf {
    // Try multiple locations for dev vs bundled
    let candidates = [
        PathBuf::from("models"),
        PathBuf::from("../models"),
        PathBuf::from("src-tauri/models"),
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.join("models")))
            .unwrap_or(PathBuf::from("models")),
    ];

    for c in candidates {
        if c.exists() {
            return c;
        }
    }
    // Default to project models dir (will be created on demand)
    PathBuf::from("models")
}

#[cfg(test)]
mod tests {
    /// Gemma 4 exposes fewer cache entries than it has layers (shared-KV
    /// layers are folded away) and mixes head dims between sliding-window and
    /// full-attention layers, so the geometry has to come from the graph.
    #[test]
    fn kv_specs_follow_the_graph_not_a_constant() {
        // Shape of Gemma 4 E2B's decoder: 15 cache entries, head dim 512 on
        // layers 4, 9 and 14.
        let mut declared: Vec<(String, Vec<i64>)> = Vec::new();
        for layer in 0..15 {
            let head_dim = if layer % 5 == 4 { 512 } else { 256 };
            for part in ["key", "value"] {
                declared.push((
                    format!("past_key_values.{layer}.{part}"),
                    vec![-1, 1, -1, head_dim],
                ));
            }
        }

        let specs =
            super::kv_layer_specs_from_inputs(declared.iter().map(|(n, s)| (n.as_str(), &s[..])));

        assert_eq!(specs.len(), 15, "one spec per cached layer, keys only");
        assert_eq!(specs[0].head_dim, 256);
        assert_eq!(specs[4].head_dim, 512);
        assert_eq!(specs[9].head_dim, 512);
        assert!(specs.iter().all(|s| s.num_heads == 1));
        // Layer order must be numeric, not the lexicographic order the graph
        // may list them in ("10" would otherwise sort before "2").
        let indices: Vec<usize> = specs.iter().map(|s| s.index).collect();
        assert_eq!(indices, (0..15).collect::<Vec<_>>());
    }

    #[test]
    fn kv_specs_sort_numerically() {
        let declared = [
            (
                "past_key_values.10.key".to_string(),
                vec![-1i64, 2, -1, 256],
            ),
            ("past_key_values.2.key".to_string(), vec![-1i64, 2, -1, 512]),
        ];
        let specs =
            super::kv_layer_specs_from_inputs(declared.iter().map(|(n, s)| (n.as_str(), &s[..])));
        assert_eq!(
            specs.iter().map(|s| s.index).collect::<Vec<_>>(),
            vec![2, 10]
        );
        assert_eq!(specs[0].head_dim, 512);
    }

    #[test]
    fn non_cache_inputs_are_ignored() {
        let declared = [
            ("inputs_embeds".to_string(), vec![-1i64, -1, 1536]),
            ("attention_mask".to_string(), vec![-1i64, -1]),
            ("num_logits_to_keep".to_string(), vec![]),
            ("past_key_values.0.key".to_string(), vec![-1i64, 1, -1, 256]),
        ];
        let specs =
            super::kv_layer_specs_from_inputs(declared.iter().map(|(n, s)| (n.as_str(), &s[..])));
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].index, 0);
    }

    #[test]
    fn a_graph_without_a_cache_yields_no_specs() {
        let declared: [(String, Vec<i64>); 0] = [];
        let specs =
            super::kv_layer_specs_from_inputs(declared.iter().map(|(n, s)| (n.as_str(), &s[..])));
        assert!(specs.is_empty());
    }
}
