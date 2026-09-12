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

#[allow(dead_code)]
const APPLE_SILICON_COREML: bool = cfg!(all(target_os = "macos", target_arch = "aarch64"));

/// Execution provider selected first for this build. Unsupported CoreML nodes
/// continue on ONNX Runtime's CPU provider.
#[allow(dead_code)]
pub fn preferred_execution_provider() -> &'static str {
    if APPLE_SILICON_COREML || cfg!(feature = "coreml") {
        "CoreML (GPU + CPU fallback)"
    } else if cfg!(feature = "tensorrt") {
        "TensorRT"
    } else if cfg!(feature = "cuda") {
        "CUDA"
    } else if cfg!(feature = "directml") {
        "DirectML"
    } else if cfg!(feature = "nnapi") {
        "NNAPI"
    } else {
        "CPU"
    }
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
    let _ = ort::init().commit();

    let mut builder = Session::builder().map_err(|e| anyhow::anyhow!("{}", e))?;
    builder = builder
        .with_optimization_level(GraphOptimizationLevel::Level3)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    // XNNPACK uses its own thread pool; ORT intra threads should be 1 to avoid contention
    #[cfg(feature = "xnnpack")]
    {
        let xnn_threads = std::num::NonZeroUsize::new(
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4)
                .clamp(1, 4),
        )
        .unwrap();
        builder = builder
            .with_intra_threads(1)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        // Disable ORT spinning when XNNPACK is active (recommended)
        if let Ok(b) = builder.with_intra_op_spinning(false) {
            builder = b;
        }
        // XNNPACK provider will be configured below with xnn_threads
        let _ = xnn_threads;
    }
    #[cfg(not(feature = "xnnpack"))]
    {
        let intra_threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
            .clamp(1, 4);
        builder = builder
            .with_intra_threads(intra_threads)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
    }

    #[cfg(any(feature = "coreml", all(target_os = "macos", target_arch = "aarch64")))]
    let coreml_cache_dir = model_path
        .as_ref()
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(".coreml-cache");
    #[cfg(any(feature = "coreml", all(target_os = "macos", target_arch = "aarch64")))]
    std::fs::create_dir_all(&coreml_cache_dir)?;

    // Execution providers are ordered by priority and unsupported nodes fall
    // back to CPU. Apple Silicon desktop builds enable CoreML automatically.
    #[cfg(feature = "xnnpack")]
    let xnn_threads = std::num::NonZeroUsize::new(
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
            .clamp(1, 4),
    )
    .unwrap();
    let mut builder = builder
        .with_execution_providers([
            #[cfg(feature = "tensorrt")]
            ort::ep::TensorRT::default().build(),
            #[cfg(feature = "cuda")]
            ort::ep::CUDA::default().build(),
            #[cfg(feature = "directml")]
            ort::ep::DirectML::default().build(),
            #[cfg(any(feature = "coreml", all(target_os = "macos", target_arch = "aarch64")))]
            {
                let profile_compute_plan =
                    std::env::var("LUMIN_COREML_PROFILE").as_deref() == Ok("1");
                ort::ep::CoreML::default()
                    .with_compute_units(ort::ep::coreml::ComputeUnits::CPUAndGPU)
                    .with_model_format(ort::ep::coreml::ModelFormat::MLProgram)
                    .with_low_precision_accumulation_on_gpu(true)
                    .with_model_cache_dir(coreml_cache_dir.to_string_lossy())
                    .with_profile_compute_plan(profile_compute_plan)
                    .build()
            },
            #[cfg(feature = "nnapi")]
            ort::ep::NNAPI::default().build(),
            #[cfg(feature = "xnnpack")]
            ort::ep::XNNPACK::default()
                .with_intra_op_num_threads(xnn_threads)
                .build(),
        ])
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let session = builder
        .commit_from_file(model_path)
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    Ok(session)
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
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    #[test]
    fn apple_silicon_build_includes_coreml() {
        use super::preferred_execution_provider;
        use ort::ep::ExecutionProvider;

        assert_eq!(
            preferred_execution_provider(),
            "CoreML (GPU + CPU fallback)"
        );
        assert!(
            ort::ep::CoreML::default().is_available().unwrap(),
            "the linked ONNX Runtime binary does not include CoreML"
        );
    }

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
