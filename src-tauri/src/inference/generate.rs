use anyhow::Result;
use ort::session::{Session, SessionInputValue};
use ort::value::Tensor;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

use super::catalog::{Architecture, Variant};
use super::session::{
    has_input, kv_layer_specs, AppState, InferenceSession, KvLayerSpec, ModelLoadProgress,
};
use super::tokenizer::{load_tokenizer, ChatFormat, ChatTurn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateOptions {
    pub prompt: String,
    pub max_tokens: Option<usize>,
    /// Reserved for sampling support; current decoding is deterministic argmax.
    pub temperature: Option<f32>,
    pub use_chat_template: Option<bool>,
}

/// What a generation is doing right now.
///
/// Reading the prompt into the KV cache happens before any token exists, and
/// on a long prompt it is most of the wait. Reporting it separately is the
/// difference between a spinner and a caller that can say how far along the
/// wait is.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "stage")]
pub enum GenerationEvent {
    /// Prompt tokens read into the cache so far, out of the whole prompt.
    Prefill { done: usize, total: usize },
    /// One decoded chunk, with its position in the token budget.
    Token {
        text: String,
        generated: usize,
        max: usize,
    },
}

/// Absolute ceiling on a single generation, whatever the caller asks for.
///
/// Not a memory limit - [`token_budget`] derives that from the machine - but a
/// stop on a model that never emits its end-of-turn token.
const MAX_TOKENS_CEILING: usize = 4096;

/// Live copies of the KV cache at the peak of one decoding step.
///
/// The cache is read out of the graph into owned buffers, cloned again into
/// the next step's input tensors, and the previous one is still alive while
/// that happens.
const KV_CACHE_LIVE_COPIES: usize = 3;

/// Bytes of KV cache one more token costs, over every cached layer.
///
/// Read from the graph rather than assumed: the number of cached layers and
/// the per-layer head dim differ between families and even between layers.
fn kv_bytes_per_token(specs: &[KvLayerSpec]) -> usize {
    specs
        .iter()
        .map(|s| s.num_heads * s.head_dim * 2 * std::mem::size_of::<f32>())
        .sum()
}

/// Share of the machine's memory the KV cache may grow into.
///
/// An eighth: the graphs themselves are already resident and are the larger
/// tenant, and the catalog's `min_memory_gb` is what keeps those within the
/// device in the first place.
const KV_CACHE_MEMORY_SHARE: u64 = 8;

/// How many tokens of context this machine can hold, given what one costs.
///
/// Measured against TOTAL memory rather than free memory. `available_memory`
/// looks like the right input and is not: on macOS it reports 0 once the
/// multi-gigabyte graphs are mapped in (measured here with 16 GB installed and
/// the machine running perfectly well), which would have the app refuse to
/// answer at all. Total memory is stable, and what the model itself occupies
/// is already bounded by the variant's `min_memory_gb`.
///
/// Clamped at both ends so a small machine still answers and a large one still
/// stops a model that never ends its turn.
fn token_budget(bytes_per_token: usize, total_bytes: u64) -> usize {
    let per_token = (bytes_per_token * KV_CACHE_LIVE_COPIES).max(1) as u64;
    let budget = (total_bytes / KV_CACHE_MEMORY_SHARE) / per_token;
    (budget as usize).clamp(256, MAX_TOKENS_CEILING)
}

/// The same, measured against this machine.
fn machine_token_budget(specs: &[KvLayerSpec]) -> usize {
    let mut system = sysinfo::System::new();
    system.refresh_memory();
    token_budget(kv_bytes_per_token(specs), system.total_memory())
}

/// Sink for [`GenerationEvent`]s produced while generating.
pub type EventSink<'a> = &'a (dyn Fn(GenerationEvent) -> Result<()> + Send + Sync);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateResult {
    pub text: String,
    pub prompt_tokens: usize,
    pub generated_tokens: usize,
    pub total_tokens: usize,
    pub latency_ms: u64,
    pub tokens_per_sec: f64,
    pub is_mock: bool,
    #[serde(default)]
    pub truncated: bool,
    pub model_id: String,
}

/// Core generation - if model not present, returns mock response for pipeline validation
pub async fn generate_text(state: &AppState, opts: GenerateOptions) -> Result<GenerateResult> {
    let max_tokens = opts.max_tokens.unwrap_or(128).min(MAX_TOKENS_CEILING);
    let variant = state.active_variant().await;
    let use_template = opts.use_chat_template.unwrap_or(true);
    let prompt = if use_template {
        variant.chat_format.apply(&opts.prompt)
    } else {
        ChatFormat::apply_raw(&opts.prompt)
    };

    // Mock path when the selected model is not fully installed - allows UI
    // validation without a multi-gigabyte download.
    if !state.active_model_ready().await {
        return Ok(mock_generate(
            &opts.prompt,
            max_tokens,
            variant.display_name,
        ));
    }

    // Once model files exist, inference errors must be visible to the caller;
    // silently returning mock output makes a broken real setup look healthy.
    try_real_inference(state, &prompt, max_tokens, None).await
}

/// Continue a conversation, rendering every turn with its own boundary.
///
/// The last turn is what the model is being asked now; earlier turns are
/// context. Use this instead of folding a transcript into one prompt, which
/// leaves the model unable to tell the two apart.
///
/// `max_tokens` of `None` means "as long as this machine can hold", which is
/// what an open-ended answer wants: the ceiling then comes from the KV cache
/// budget rather than from a number picked in advance.
pub async fn generate_conversation(
    state: &AppState,
    turns: &[ChatTurn],
    max_tokens: Option<usize>,
    emit: Option<EventSink<'_>>,
) -> Result<GenerateResult> {
    let max_tokens = max_tokens.unwrap_or(MAX_TOKENS_CEILING).min(MAX_TOKENS_CEILING);
    let variant = state.active_variant().await;

    if !state.active_model_ready().await {
        let latest = turns
            .iter()
            .rev()
            .find(|t| t.role == super::tokenizer::ChatRole::User)
            .map(|t| t.content.as_str())
            .unwrap_or_default();
        return Ok(mock_generate(latest, max_tokens, variant.display_name));
    }

    let prompt = variant.chat_format.apply_conversation(turns);
    try_real_inference(state, &prompt, max_tokens, emit).await
}

/// Load the active variant into memory if it is not already loaded.
///
/// Returns false when the selected model is not installed, so generation
/// would run in mock mode and there is nothing to load.
pub async fn ensure_active_session(state: &AppState) -> Result<bool> {
    if !state.active_model_ready().await {
        return Ok(false);
    }
    let variant = state.active_variant().await;
    let mut guard = state.session.lock().await;
    ensure_session_loaded(state, variant, &mut guard).await?;
    Ok(true)
}

/// Put the variant's graphs and tokenizer in memory, reporting progress.
///
/// A no-op when the requested variant is already loaded. Loading several
/// gigabytes of graph takes tens of seconds even from a warm local file, so
/// it runs on a blocking thread and reports each stage rather than leaving
/// the caller (and the UI) with nothing to show.
async fn ensure_session_loaded(
    state: &AppState,
    variant: &'static Variant,
    guard: &mut Option<InferenceSession>,
) -> Result<()> {
    if guard.as_ref().is_some_and(|s| s.variant_id == variant.id) {
        return Ok(());
    }
    // Anything still loaded belongs to a different model; free it before
    // pulling the next one in, so both are never resident at once.
    *guard = None;

    match load_variant_session(state, variant).await {
        Ok(session) => {
            *guard = Some(session);
            state.emit_load_progress(&ModelLoadProgress::new(
                variant,
                "ready",
                "モデルの準備ができました",
                100.0,
            ));
            Ok(())
        }
        Err(e) => {
            state.emit_load_progress(
                &ModelLoadProgress::new(
                    variant,
                    "error",
                    "モデルの読み込みに失敗しました",
                    0.0,
                )
                .with_error(e.to_string()),
            );
            Err(e)
        }
    }
}

async fn load_variant_session(
    state: &AppState,
    variant: &'static Variant,
) -> Result<InferenceSession> {
    let (decoder_bytes, embed_bytes) = variant.graph_bytes(&state.model_dir);
    let total = (decoder_bytes + embed_bytes).max(1) as f64;
    let decoder_share = decoder_bytes as f64 / total * 100.0;

    let tok_path = variant.tokenizer_path(&state.model_dir);
    let model_path = variant.decoder_path(&state.model_dir);

    state.emit_load_progress(&ModelLoadProgress::new(
        variant,
        "tokenizer",
        "トークナイザーを読み込み中…",
        0.0,
    ));
    let tokenizer_path = tok_path.clone();
    let tokenizer = blocking_with(move || load_tokenizer(tokenizer_path)).await?;

    state.emit_load_progress(&ModelLoadProgress::new(
        variant,
        "decoder",
        "モデル本体をメモリに読み込み中…",
        0.0,
    ));
    let embed_path = match variant.architecture {
        Architecture::EmbedChained => {
            let path = variant
                .embed_path(&state.model_dir)
                .ok_or_else(|| anyhow::anyhow!("{} declares no embed_tokens graph", variant.id))?;
            if !path.exists() {
                anyhow::bail!("embed_tokens graph missing for {}: {:?}", variant.id, path);
            }
            state.emit_load_progress(&ModelLoadProgress::new(
                variant,
                "embed",
                "埋め込みグラフをメモリに読み込み中…",
                decoder_share,
            ));
            Some(path)
        }
        Architecture::DecoderOnly => None,
    };

    // An EP must load both Gemma graphs. A decoder-only success followed by an
    // embed failure would otherwise look like GPU inference while generation
    // still cannot start.
    let mut failures = Vec::new();
    let mut loaded = None;
    for provider in super::runtime::candidates_for_variant(variant)? {
        let decoder_path = model_path.clone();
        let embed_path = embed_path.clone();
        match blocking_with(move || {
            let decoder = super::session::create_session_with_provider(&decoder_path, provider)?;
            let embed = embed_path
                .as_ref()
                .map(|path| super::session::create_session_with_provider(path, provider))
                .transpose()?;
            Ok((decoder, embed))
        })
        .await
        {
            Ok((session, embed_session)) => {
                super::runtime::record_provider(
                    provider,
                    (!failures.is_empty()).then(|| failures.join(" / ")),
                );
                loaded = Some((session, embed_session));
                break;
            }
            Err(error) => failures.push(format!("{}: {error}", provider.label())),
        }
    }
    let (session, embed_session) = loaded.ok_or_else(|| {
        anyhow::anyhow!("ONNX Runtime セッションを作成できません: {}", failures.join(" / "))
    })?;

    Ok(InferenceSession {
        session,
        embed_session,
        tokenizer,
        variant_id: variant.id.to_string(),
        model_info: super::session::ModelInfo {
            model_id: variant.display_name.to_string(),
            onnx_path: model_path.to_string_lossy().to_string(),
            tokenizer_path: tok_path.to_string_lossy().to_string(),
            exists: true,
            size_bytes: None,
            quantization: variant.quantization.to_string(),
            description: variant.description.to_string(),
        },
    })
}

/// Run a blocking load off the async runtime, so progress events and the rest
/// of the app keep flowing while ONNX Runtime reads gigabytes from disk.
async fn blocking_with<T: Send + 'static>(
    work: impl FnOnce() -> Result<T> + Send + 'static,
) -> Result<T> {
    tokio::task::spawn_blocking(work)
        .await
        .map_err(|e| anyhow::anyhow!("model load task failed: {e}"))?
}


fn mock_generate(prompt: &str, max_tokens: usize, model_name: &str) -> GenerateResult {
    let start = Instant::now();
    // Simulate small latency for bench consistency
    std::thread::sleep(Duration::from_millis(30));
    let generated = format!(
        "[MOCK] Lumin response for: \"{}\" ({} tokens max)\n\n\
        これはモック推論です。選択中のモデル ({}) のファイルが揃うと ort による実推論が有効になります。\n\
        モデル管理画面からダウンロードしてください。",
        prompt.chars().take(80).collect::<String>(),
        max_tokens,
        model_name
    );
    let latency = start.elapsed().as_millis() as u64;
    let tokens = 32;
    GenerateResult {
        text: generated,
        prompt_tokens: prompt.split_whitespace().count(),
        generated_tokens: tokens,
        total_tokens: prompt.split_whitespace().count() + tokens,
        latency_ms: latency,
        tokens_per_sec: tokens as f64 / (latency as f64 / 1000.0).max(0.001),
        is_mock: true,
        truncated: false,
        model_id: format!("mock/{model_name}"),
    }
}

/// Token embeddings produced by an `embed_tokens` graph.
struct Embeddings {
    /// `[1, seq_len, hidden_size]`, flattened.
    inputs_embeds: Vec<f32>,
    hidden_size: usize,
    /// `[1, seq_len, num_layers, per_layer_dim]`, flattened. Gemma 3n/4 only.
    per_layer_inputs: Option<(Vec<f32>, usize, usize)>,
}

/// Run the `embed_tokens` graph to turn token ids into decoder inputs.
fn embed_tokens(embed: &mut Session, ids: &[i64]) -> Result<Embeddings> {
    let seq_len = ids.len();
    let input_ids = Tensor::from_array(([1, seq_len], ids.to_vec()))
        .map_err(|e| anyhow::anyhow!("embed input tensor error: {e}"))?;

    let outputs = embed
        .run(vec![(
            "input_ids".to_string(),
            SessionInputValue::from(input_ids),
        )])
        .map_err(|e| anyhow::anyhow!("embed_tokens run error: {e}"))?;

    let (embeds_shape, embeds) = outputs["inputs_embeds"]
        .try_extract_tensor::<f32>()
        .map_err(|e| anyhow::anyhow!("extract inputs_embeds error: {e}"))?;
    if embeds_shape.len() != 3 {
        anyhow::bail!("unexpected inputs_embeds shape: {embeds_shape:?}");
    }
    let hidden_size = embeds_shape[2] as usize;

    // Gemma 3n / Gemma 4 also emit per-layer inputs for their PLE blocks.
    let per_layer_inputs = match outputs.get("per_layer_inputs") {
        Some(value) => {
            let (shape, data) = value
                .try_extract_tensor::<f32>()
                .map_err(|e| anyhow::anyhow!("extract per_layer_inputs error: {e}"))?;
            if shape.len() != 4 {
                anyhow::bail!("unexpected per_layer_inputs shape: {shape:?}");
            }
            Some((data.to_vec(), shape[2] as usize, shape[3] as usize))
        }
        None => None,
    };

    Ok(Embeddings {
        inputs_embeds: embeds.to_vec(),
        hidden_size,
        per_layer_inputs,
    })
}

/// Zero-filled cache positions prepended to the KV cache and masked out for
/// the whole generation.
///
/// Legacy Gemma 3 / 3n convention for CoreML's zero-element restriction.
/// Gemma 4 must use a genuinely empty cache instead; masking a synthetic
/// prefix is not equivalent for its exported GQA graph.
const CACHE_PAD: usize = 1;

/// Build the decoder's inputs for one step, supplying only what the graph
/// declares. Gemma 3 takes `input_ids`; Gemma 3n/4 take `inputs_embeds` plus
/// `per_layer_inputs`, `position_ids` and `num_logits_to_keep`.
///
/// `cache_len` counts the padded positions; `position` is the index of the
/// first real token in the sequence and so excludes them.
fn build_decoder_inputs(
    decoder: &Session,
    kv_specs: &[KvLayerSpec],
    token_ids: &[i64],
    embeddings: Option<&Embeddings>,
    cache: &Option<Vec<(Vec<f32>, Vec<f32>)>>,
    cache_len: usize,
    position: usize,
) -> Result<Vec<(String, SessionInputValue<'static>)>> {
    let seq_len = token_ids.len();
    let mut inputs: Vec<(String, SessionInputValue)> = Vec::new();

    if has_input(decoder, "input_ids") {
        let t = Tensor::from_array(([1, seq_len], token_ids.to_vec()))
            .map_err(|e| anyhow::anyhow!("input_ids tensor error: {e}"))?;
        inputs.push(("input_ids".to_string(), t.into()));
    }

    if has_input(decoder, "inputs_embeds") {
        let emb = embeddings.ok_or_else(|| {
            anyhow::anyhow!("decoder needs inputs_embeds but no embed_tokens graph is loaded")
        })?;
        let t = Tensor::from_array(([1, seq_len, emb.hidden_size], emb.inputs_embeds.clone()))
            .map_err(|e| anyhow::anyhow!("inputs_embeds tensor error: {e}"))?;
        inputs.push(("inputs_embeds".to_string(), t.into()));
    }

    if has_input(decoder, "per_layer_inputs") {
        let (data, layers, dim) = embeddings
            .and_then(|e| e.per_layer_inputs.as_ref())
            .map(|(d, l, dim)| (d.clone(), *l, *dim))
            .ok_or_else(|| {
                anyhow::anyhow!("decoder needs per_layer_inputs but embed_tokens did not emit them")
            })?;
        let t = Tensor::from_array(([1, seq_len, layers, dim], data))
            .map_err(|e| anyhow::anyhow!("per_layer_inputs tensor error: {e}"))?;
        inputs.push(("per_layer_inputs".to_string(), t.into()));
    }

    if has_input(decoder, "attention_mask") {
        let attention_len = cache_len + seq_len;
        // The leading padded positions stay masked for the whole generation.
        let mut mask = vec![1i64; attention_len];
        for slot in mask
            .iter_mut()
            .take(cache_len.saturating_sub(position).min(attention_len))
        {
            *slot = 0;
        }
        let t = Tensor::from_array(([1, attention_len], mask))
            .map_err(|e| anyhow::anyhow!("attention_mask tensor error: {e}"))?;
        inputs.push(("attention_mask".to_string(), t.into()));
    }

    if has_input(decoder, "position_ids") {
        let positions: Vec<i64> = (position..position + seq_len).map(|p| p as i64).collect();
        let t = Tensor::from_array(([1, seq_len], positions))
            .map_err(|e| anyhow::anyhow!("position_ids tensor error: {e}"))?;
        inputs.push(("position_ids".to_string(), t.into()));
    }

    if has_input(decoder, "num_logits_to_keep") {
        // Rank-0 scalar: only the last position's logits are needed.
        let t = Tensor::from_array(((), vec![1i64]))
            .map_err(|e| anyhow::anyhow!("num_logits_to_keep tensor error: {e}"))?;
        inputs.push(("num_logits_to_keep".to_string(), t.into()));
    }

    for (slot, spec) in kv_specs.iter().enumerate() {
        let (key, value) = match cache {
            Some(layers) => layers[slot].clone(),
            // First step: the cache is nothing but the masked padding.
            None => {
                let zeros = vec![0f32; spec.num_heads * cache_len * spec.head_dim];
                (zeros.clone(), zeros)
            }
        };
        let shape = [1, spec.num_heads, cache_len, spec.head_dim];
        let key_tensor = Tensor::<f32>::from_array((shape, key))
            .map_err(|e| anyhow::anyhow!("past key tensor error: {e}"))?;
        let value_tensor = Tensor::<f32>::from_array((shape, value))
            .map_err(|e| anyhow::anyhow!("past value tensor error: {e}"))?;
        inputs.push((
            format!("past_key_values.{}.key", spec.index),
            key_tensor.into(),
        ));
        inputs.push((
            format!("past_key_values.{}.value", spec.index),
            value_tensor.into(),
        ));
    }

    Ok(inputs)
}

async fn try_real_inference(
    state: &AppState,
    prompt: &str,
    max_tokens: usize,
    emit: Option<EventSink<'_>>,
) -> Result<GenerateResult> {
    let start = Instant::now();
    let variant = state.active_variant().await;
    if variant.chat_format == ChatFormat::Gemma4Turn
        && super::runtime::selected_for_variant(variant)? == super::runtime::Provider::CoreMl
    {
        anyhow::bail!("このGemma 4 ONNXはCoreML非対応演算と空キャッシュを使います。ONNX Runtime WebGPU EP（標準設定）を使用してください。");
    }
    // Load or reuse the session; a model switch clears it, so a stale variant
    // here means the state was replaced without going through
    // set_active_variant.
    let mut guard = state.session.lock().await;
    ensure_session_loaded(state, variant, &mut guard).await?;

    // Split borrow: the decoder, the embed graph and the tokenizer are used
    // together throughout the loop below.
    let InferenceSession {
        session: decoder,
        embed_session,
        tokenizer,
        ..
    } = guard.as_mut().expect("loaded above");

    let input_ids = tokenizer.encode_prompt(prompt)?;
    let prompt_tokens = input_ids.len();
    let kv_specs = kv_layer_specs(decoder);
    let eos_token_ids = variant.chat_format.eos_token_ids();

    // What the caller asked for, less what this machine can actually hold:
    // the prompt is already in the cache and every generated token adds to it.
    let context_budget = machine_token_budget(&kv_specs);
    if prompt_tokens >= context_budget {
        anyhow::bail!(
            "プロンプトが長すぎます（{prompt_tokens}トークン）。この端末の空きメモリで扱えるのは約{context_budget}トークンです。"
        );
    }
    let max_tokens = max_tokens.min(context_budget - prompt_tokens);

    let mut generated_ids: Vec<i64> = Vec::new();
    let mut ended = false;
    // Bound prefill matrix sizes while retaining every prompt token in KV order.
    const PREFILL_CHUNK: usize = 128;
    let mut current_ids = input_ids[..input_ids.len().min(PREFILL_CHUNK)].to_vec();
    let mut cache: Option<Vec<(Vec<f32>, Vec<f32>)>> = None;
    // Length of the KV cache including the masked padding, and the position of
    // the next real token (which the padding must not shift).
    // Gemma 4 uses an empty cache with zero-based positions. A synthetic
    // masked prefix changes the exported GQA attention behavior. WebGPU and
    // CUDA accept an empty cache; other families retain their padding convention.
    let mut cache_len = if variant.chat_format == ChatFormat::Gemma4Turn {
        0
    } else {
        CACHE_PAD
    };
    let mut position = 0usize;
    let mut decode_stream = emit.map(|_| tokenizer.inner().decode_stream(true));

    if let Some(emit) = emit {
        emit(GenerationEvent::Prefill {
            done: 0,
            total: prompt_tokens,
        })?;
    }

    while generated_ids.len() < max_tokens {
        let embeddings = match embed_session.as_mut() {
            Some(embed) => Some(embed_tokens(embed, &current_ids)?),
            None => None,
        };

        let inputs = build_decoder_inputs(
            decoder,
            &kv_specs,
            &current_ids,
            embeddings.as_ref(),
            &cache,
            cache_len,
            position,
        )?;

        let outputs = decoder
            .run(inputs)
            .map_err(|e| anyhow::anyhow!("ort run error: {e}"))?;

        let logits = outputs["logits"]
            .try_extract_tensor::<f32>()
            .map_err(|e| anyhow::anyhow!("extract error: {e}"))?;
        let (shape, data) = logits;
        // shape is &[i64] for ort 2.0; cast to usize
        if shape.len() != 3 {
            anyhow::bail!("unexpected logits shape: {:?}", shape);
        }
        let vocab = shape[2] as usize;
        let seq = shape[1] as usize;
        // last token logits
        let last_offset = (seq - 1) * vocab;
        let last_logits = &data[last_offset..last_offset + vocab];
        let next_id = argmax(last_logits) as i64;

        cache_len += current_ids.len();
        position += current_ids.len();

        let mut next_cache = Vec::with_capacity(kv_specs.len());
        for spec in &kv_specs {
            let (_, key) = outputs[format!("present.{}.key", spec.index)]
                .try_extract_tensor::<f32>()
                .map_err(|e| anyhow::anyhow!("extract present key error: {e}"))?;
            let (_, value) = outputs[format!("present.{}.value", spec.index)]
                .try_extract_tensor::<f32>()
                .map_err(|e| anyhow::anyhow!("extract present value error: {e}"))?;
            next_cache.push((key.to_vec(), value.to_vec()));
        }
        cache = Some(next_cache);

        if position < input_ids.len() {
            current_ids =
                input_ids[position..(position + PREFILL_CHUNK).min(input_ids.len())].to_vec();
            if let Some(emit) = emit {
                emit(GenerationEvent::Prefill {
                    done: position,
                    total: prompt_tokens,
                })?;
            }
            continue;
        }

        if eos_token_ids.contains(&next_id) {
            ended = true;
            break;
        }

        generated_ids.push(next_id);
        current_ids = vec![next_id];

        if let (Some(emit), Some(decoder)) = (emit, decode_stream.as_mut()) {
            // The prompt is fully cached by now, so report the prefill done
            // before the first token rather than leaving it at the last chunk.
            if generated_ids.len() == 1 {
                emit(GenerationEvent::Prefill {
                    done: prompt_tokens,
                    total: prompt_tokens,
                })?;
            }
            // A token does not always close a character, so a step can yield
            // nothing; the count still advances and the caller still hears it.
            let text = decoder
                .step(next_id as u32)
                .map_err(|e| anyhow::anyhow!("stream decode error: {e}"))?
                .unwrap_or_default();
            emit(GenerationEvent::Token {
                text,
                generated: generated_ids.len(),
                max: max_tokens,
            })?;
        }

        if generated_ids.len() >= max_tokens {
            break;
        }
    }

    // Empty generations and tokenizer errors are not mock responses. Keep
    // them observable so feature-level guards can reject them.
    let text = tokenizer.decode(&generated_ids, true)?;

    let latency_ms = start.elapsed().as_millis() as u64;
    let tokens_per_sec = generated_ids.len() as f64 / (latency_ms as f64 / 1000.0).max(0.001);

    Ok(GenerateResult {
        text,
        prompt_tokens,
        generated_tokens: generated_ids.len(),
        total_tokens: prompt_tokens + generated_ids.len(),
        latency_ms,
        tokens_per_sec,
        is_mock: false,
        truncated: !ended,
        model_id: variant.display_name.to_string(),
    })
}

fn argmax(slice: &[f32]) -> usize {
    let mut max_idx = 0;
    let mut max_val = slice[0];
    for (i, &v) in slice.iter().enumerate().skip(1) {
        if v > max_val {
            max_val = v;
            max_idx = i;
        }
    }
    max_idx
}

/// Streaming generation - emits tokens via tauri event
pub async fn generate_stream(
    state: &AppState,
    opts: GenerateOptions,
    emit: impl Fn(GenerationEvent) -> Result<()> + Send + Sync,
) -> Result<GenerateResult> {
    let max_tokens = opts.max_tokens.unwrap_or(32).min(MAX_TOKENS_CEILING);

    if !state.active_model_ready().await {
        let variant = state.active_variant().await;
        let res = mock_generate(&opts.prompt, max_tokens, variant.display_name);
        // Simulate token-by-token emit
        let words: Vec<&str> = res.text.split_whitespace().collect();
        for (i, tok) in words.iter().enumerate() {
            emit(GenerationEvent::Token {
                text: format!("{tok} "),
                generated: i + 1,
                max: max_tokens,
            })?;
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        return Ok(res);
    }

    let prompt = if opts.use_chat_template.unwrap_or(true) {
        state.active_variant().await.chat_format.apply(&opts.prompt)
    } else {
        ChatFormat::apply_raw(&opts.prompt)
    };
    try_real_inference(state, &prompt, max_tokens, Some(&emit)).await
}

#[cfg(test)]
mod budget_tests {
    use super::*;

    fn specs(layers: usize, head_dim: usize) -> Vec<KvLayerSpec> {
        (0..layers)
            .map(|index| KvLayerSpec {
                index,
                num_heads: 1,
                head_dim,
            })
            .collect()
    }

    #[test]
    fn per_token_cost_comes_from_the_graph_not_a_constant() {
        // Key and value, f32, over every cached layer.
        assert_eq!(kv_bytes_per_token(&specs(15, 256)), 15 * 256 * 2 * 4);
        // A family with more cached layers costs proportionally more, which is
        // why the budget cannot be one number shared by every model.
        assert_eq!(
            kv_bytes_per_token(&specs(24, 256)),
            kv_bytes_per_token(&specs(12, 256)) * 2
        );
        assert_eq!(kv_bytes_per_token(&[]), 0);
    }

    #[test]
    fn the_budget_follows_the_machine_and_the_model() {
        let small_machine = 512 * 1024 * 1024;
        let big_machine = 8 * 1024 * 1024 * 1024;
        let light = kv_bytes_per_token(&specs(15, 256));
        let heavy = kv_bytes_per_token(&specs(24, 512));

        assert!(token_budget(light, small_machine) < token_budget(light, big_machine));
        // A model with a costlier cache gets fewer tokens on the same machine.
        assert!(token_budget(heavy, small_machine) < token_budget(light, small_machine));
    }

    #[test]
    fn a_crowded_machine_still_answers_and_a_roomy_one_still_stops() {
        let per_token = kv_bytes_per_token(&specs(24, 512));
        // Almost nothing free: still enough budget to reply at all.
        assert_eq!(token_budget(per_token, 1024), 256);
        // Plenty free: bounded anyway, so a model that never ends its turn
        // cannot run forever.
        assert_eq!(
            token_budget(per_token, 1024 * 1024 * 1024 * 1024),
            MAX_TOKENS_CEILING
        );
    }

    #[test]
    fn a_graph_without_a_cache_is_not_a_division_by_zero() {
        assert_eq!(token_budget(0, 8 * 1024 * 1024 * 1024), MAX_TOKENS_CEILING);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    /// Every smoke test pins its variant: the active model is a persisted
    /// setting, so relying on the default would make the result depend on
    /// whatever ran last.
    #[tokio::test]
    #[ignore]
    async fn real_inference_smoke() {
        let state = AppState::new(std::path::PathBuf::from("../models"));
        state.set_active_variant("1b-int4").await.unwrap();
        let opts = GenerateOptions {
            prompt: "こんにちは".to_string(),
            max_tokens: Some(8),
            temperature: None,
            use_chat_template: Some(true),
        };

        let result = generate_text(&state, opts).await.expect("real inference");
        println!(
            "[gemma3-1b] {} tok in {} ms\n{}",
            result.generated_tokens, result.latency_ms, result.text
        );
        assert!(!result.is_mock);
        assert!(!result.text.is_empty());
    }

    #[tokio::test]
    #[ignore]
    async fn real_streaming_inference_smoke() {
        streaming_smoke("1b-int4").await;
    }

    /// The streaming path re-decodes token by token, so it depends on the
    /// tokenizer rather than the decoder graph; Gemma 4 ships a different one
    /// and needs its own run.
    #[tokio::test]
    #[ignore]
    async fn gemma4_e4b_streaming_smoke() {
        streaming_smoke("4-e4b-int4").await;
    }

    async fn streaming_smoke(variant: &str) {
        let state = AppState::new(std::path::PathBuf::from("../models"));
        state.set_active_variant(variant).await.unwrap();
        assert!(
            state.active_model_ready().await,
            "{variant} is not installed"
        );
        let opts = GenerateOptions {
            prompt: "こんにちは".to_string(),
            max_tokens: Some(8),
            temperature: None,
            use_chat_template: Some(true),
        };
        let emitted = Arc::new(Mutex::new(Vec::<String>::new()));
        let emitted_for_callback = Arc::clone(&emitted);

        let result = generate_stream(&state, opts, move |event| {
            if let GenerationEvent::Token { text, .. } = event {
                emitted_for_callback.lock().unwrap().push(text);
            }
            Ok(())
        })
        .await
        .expect("real streaming inference");

        println!("[{variant} stream] {}", result.text);
        assert!(!result.is_mock);
        assert!(!result.text.is_empty());
        let streamed = emitted.lock().unwrap().concat();
        assert!(!streamed.is_empty());
        assert_eq!(streamed, result.text);
    }

    /// A follow-up turn must be answered using the turn before it.
    ///
    /// The question "その答えに5を足すと" is unanswerable on its own, so a
    /// reply of 12 can only come from the model having seen the earlier turns
    /// as turns. This is the regression that motivated per-turn rendering: with
    /// the transcript folded into a single prompt, follow-ups came back
    /// answering the first question instead.
    ///
    /// Requires the models under ../models. ONNX sessions must not be built
    /// concurrently, so run it serially:
    ///   cargo test multi_turn_conversation_uses_the_previous_turns \
    ///     -- --ignored --nocapture --test-threads=1
    #[tokio::test]
    #[ignore]
    async fn multi_turn_conversation_uses_the_previous_turns() {
        let mut checked = 0;
        for variant in ["1b-int4", "4-e2b-int4", "4-e4b-int4"] {
            let state = AppState::new(std::path::PathBuf::from("../models"));
            state.set_active_variant(variant).await.unwrap();
            if !state.active_model_ready().await {
                println!("[{variant}] not installed, skipped");
                continue;
            }

            let first = ChatTurn::user("3+4はいくつですか。数字だけ答えてください。");
            let answer = generate_conversation(&state, &[first.clone()], Some(24), None)
                .await
                .expect("first turn");
            let answer = answer.text.trim().to_string();
            println!("[{variant} turn1] {answer}");
            assert!(answer.contains('7'), "[{variant}] turn 1 went wrong: {answer}");

            let follow_up =
                ChatTurn::user("その答えに5を足すといくつですか。数字だけ答えてください。");
            let with_history = generate_conversation(
                &state,
                &[first, ChatTurn::assistant(&answer), follow_up.clone()],
                Some(24),
                None,
            )
            .await
            .expect("second turn");
            let with_history = with_history.text.trim().to_string();
            println!("[{variant} turn2 with history] {with_history}");

            let alone = generate_conversation(&state, &[follow_up], Some(24), None)
                .await
                .expect("second turn alone");
            let alone = alone.text.trim().to_string();
            println!("[{variant} turn2 without history] {alone}");

            assert!(
                with_history.contains("12"),
                "[{variant}] the follow-up was not answered from the history: {with_history}"
            );
            // The control: without the earlier turns there is nothing to add 5
            // to, so 12 cannot be reached. If this ever also said 12, the test
            // above would prove nothing.
            assert!(
                !alone.contains("12"),
                "[{variant}] the question is answerable without history, so the \
                 check above is not evidence: {alone}"
            );
            checked += 1;
        }
        assert!(checked > 0, "no variant installed under ../models");
    }

    /// The real cache geometry, what a token of context costs on it, and the
    /// budget that follows. Run it when tuning the budget or adding a model.
    #[tokio::test]
    #[ignore]
    async fn kv_geometry_probe() {
        for variant in ["1b-int4", "4-e2b-int4", "4-e4b-int4"] {
            let state = AppState::new(std::path::PathBuf::from("../models"));
            state.set_active_variant(variant).await.unwrap();
            if !state.active_model_ready().await {
                continue;
            }
            ensure_active_session(&state).await.unwrap();
            let guard = state.session.lock().await;
            let specs = kv_layer_specs(&guard.as_ref().unwrap().session);
            let per_token = kv_bytes_per_token(&specs);
            let mut system = sysinfo::System::new();
            system.refresh_memory();
            println!(
                "[{variant}] layers={} heads={:?} dims={:?} per_token={} KB total={} MB available={} MB budget={} tok",
                specs.len(),
                specs.iter().map(|s| s.num_heads).collect::<std::collections::BTreeSet<_>>(),
                specs.iter().map(|s| s.head_dim).collect::<std::collections::BTreeSet<_>>(),
                per_token / 1024,
                system.total_memory() / 1024 / 1024,
                system.available_memory() / 1024 / 1024,
                token_budget(per_token, system.total_memory()),
            );
        }
    }

    /// The wait before the first token is the prompt being read into the
    /// cache. It has to be reported, in order, and it has to finish before any
    /// text arrives - otherwise the UI has nothing truthful to show for the
    /// part of the wait the teacher actually notices.
    ///
    /// Requires the models under ../models; run with --test-threads=1.
    #[tokio::test]
    #[ignore]
    async fn prompt_processing_is_reported_before_the_first_token() {
        for variant in ["1b-int4", "4-e2b-int4", "4-e4b-int4"] {
            let state = AppState::new(std::path::PathBuf::from("../models"));
            state.set_active_variant(variant).await.unwrap();
            if !state.active_model_ready().await {
                println!("[{variant}] not installed, skipped");
                continue;
            }

            let events = Arc::new(Mutex::new(Vec::<GenerationEvent>::new()));
            let sink = Arc::clone(&events);
            // Long enough to need several prefill chunks of 128 tokens.
            let question = "次の授業の導入を考えています。".repeat(40);
            let result = generate_conversation(
                &state,
                &[ChatTurn::user(question)],
                Some(16),
                Some(&move |event: GenerationEvent| {
                    sink.lock().unwrap().push(event);
                    Ok(())
                }),
            )
            .await
            .expect("generation");

            let events = events.lock().unwrap().clone();
            let prefill: Vec<(usize, usize)> = events
                .iter()
                .filter_map(|e| match e {
                    GenerationEvent::Prefill { done, total } => Some((*done, *total)),
                    _ => None,
                })
                .collect();
            let first_token = events
                .iter()
                .position(|e| matches!(e, GenerationEvent::Token { .. }))
                .expect("no token was reported");
            let last_prefill = events
                .iter()
                .rposition(|e| matches!(e, GenerationEvent::Prefill { .. }))
                .expect("no prefill was reported");

            println!(
                "[{variant}] prompt {} tok, {} prefill steps, first token at event {first_token}",
                result.prompt_tokens,
                prefill.len()
            );

            assert!(
                prefill.len() > 2,
                "[{variant}] a multi-chunk prompt reported {} prefill steps",
                prefill.len()
            );
            assert!(
                last_prefill < first_token,
                "[{variant}] prefill was still being reported after the first token"
            );
            // The reported total is the real prompt length, and progress runs
            // from nothing to all of it without going backwards.
            assert!(prefill.iter().all(|(_, total)| *total == result.prompt_tokens));
            assert_eq!(prefill.first().unwrap().0, 0);
            assert_eq!(prefill.last().unwrap(), &(result.prompt_tokens, result.prompt_tokens));
            assert!(
                prefill.windows(2).all(|w| w[0].0 <= w[1].0),
                "[{variant}] prefill progress went backwards: {prefill:?}"
            );

            // Token counts advance one at a time, against the real budget.
            let counts: Vec<usize> = events
                .iter()
                .filter_map(|e| match e {
                    GenerationEvent::Token { generated, max, .. } => {
                        assert_eq!(*max, 16, "[{variant}] wrong budget reported");
                        Some(*generated)
                    }
                    _ => None,
                })
                .collect();
            assert_eq!(counts, (1..=counts.len()).collect::<Vec<_>>());
        }
    }

    /// The chat UI renders chunks as they arrive and then settles on the
    /// returned text, so the two must agree exactly - otherwise the reply
    /// visibly rewrites itself when generation finishes.
    ///
    /// Requires the models under ../models; run with --test-threads=1.
    #[tokio::test]
    #[ignore]
    async fn conversation_streaming_matches_the_returned_text() {
        for variant in ["1b-int4", "4-e2b-int4", "4-e4b-int4"] {
            let state = AppState::new(std::path::PathBuf::from("../models"));
            state.set_active_variant(variant).await.unwrap();
            if !state.active_model_ready().await {
                println!("[{variant}] not installed, skipped");
                continue;
            }

            let chunks = Arc::new(Mutex::new(Vec::<String>::new()));
            let sink = Arc::clone(&chunks);
            let result = generate_conversation(
                &state,
                &[
                    ChatTurn::user("分数の通分とは何ですか。"),
                    ChatTurn::assistant("分母をそろえることです。"),
                    ChatTurn::user("一文で言い換えてください。"),
                ],
                Some(48),
                Some(&move |event: GenerationEvent| {
                    if let GenerationEvent::Token { text, .. } = event {
                        sink.lock().unwrap().push(text);
                    }
                    Ok(())
                }),
            )
            .await
            .expect("streaming conversation");

            let streamed = chunks.lock().unwrap().concat();
            println!("[{variant} streamed in {} chunks] {streamed}", chunks.lock().unwrap().len());
            assert!(!streamed.is_empty(), "[{variant}] nothing was streamed");
            assert_eq!(
                streamed, result.text,
                "[{variant}] streamed text and returned text disagree"
            );
            assert!(chunks.lock().unwrap().len() > 1, "[{variant}] arrived in one piece");
        }
    }

    /// End-to-end check of the embed-chained path. Requires the Gemma 4 E2B
    /// files under ../models; run with:
    ///   cargo test gemma4_e2b_inference_smoke -- --ignored --nocapture
    #[tokio::test]
    #[ignore]
    async fn gemma4_e2b_inference_smoke() {
        gemma4_smoke("4-e2b-int4").await;
    }

    /// Same, for the larger E4B option (two external-data shards per graph).
    #[tokio::test]
    #[ignore]
    async fn gemma4_e4b_inference_smoke() {
        gemma4_smoke("4-e4b-int4").await;
    }

    /// Longer run over the real classroom prompt shape. Eight tokens barely
    /// touch the KV cache; a cache that is mis-shaped or mis-ordered produces
    /// text that only degenerates after several steps, so this generates
    /// enough tokens for that to show up.
    #[tokio::test]
    #[ignore]
    async fn gemma4_e2b_long_generation_smoke() {
        gemma4_long_generation_smoke("4-e2b-int4").await;
    }

    /// Same for E4B: its KV geometry differs from E2B's (24 cached layers
    /// against 15), so the cache path has to be exercised on both.
    #[tokio::test]
    #[ignore]
    async fn gemma4_e4b_long_generation_smoke() {
        gemma4_long_generation_smoke("4-e4b-int4").await;
    }

    async fn gemma4_long_generation_smoke(variant: &str) {
        let state = AppState::new(std::path::PathBuf::from("../models"));
        state.set_active_variant(variant).await.unwrap();
        assert!(
            state.active_model_ready().await,
            "{variant} is not installed"
        );

        let result = generate_text(
            &state,
            GenerateOptions {
                prompt: "中学生に「一次関数の傾き」を、答えを言わずに気づかせるヒントを1つ考えてください。"
                    .to_string(),
                max_tokens: Some(64),
                temperature: None,
                use_chat_template: Some(true),
            },
        )
        .await
        .expect("real inference");

        println!(
            "[{variant} long] {} tok in {} ms\n{}",
            result.generated_tokens, result.latency_ms, result.text
        );

        assert!(!result.is_mock);
        assert!(result.generated_tokens >= 16, "stopped far too early");

        // A broken cache typically collapses into one repeated token.
        let chars: Vec<char> = result.text.chars().collect();
        let distinct = chars.iter().collect::<std::collections::HashSet<_>>().len();
        assert!(
            distinct > chars.len() / 4,
            "output looks degenerate ({distinct} distinct of {} chars): {}",
            chars.len(),
            result.text
        );
    }

    /// The `<bos>` duplication this guards against is invisible in the
    /// rendered prompt - it only appears once the family's tokenizer has run,
    /// so it needs the real tokenizer files.
    #[tokio::test]
    #[ignore]
    async fn installed_tokenizers_emit_exactly_one_bos() {
        let model_root = std::path::PathBuf::from("../models");
        let mut checked = 0;
        for variant in super::super::catalog::VARIANTS {
            if !variant.is_installed(&model_root) {
                continue;
            }
            let tokenizer = load_tokenizer(variant.tokenizer_path(&model_root)).unwrap();
            let ids = tokenizer
                .encode_prompt(&variant.chat_format.apply("こんにちは"))
                .unwrap();
            let bos = tokenizer.encode(ChatFormat::BOS, false).unwrap();
            assert_eq!(bos.len(), 1, "{} tokenizes <bos> oddly", variant.id);
            assert_eq!(ids.first(), bos.first(), "{} lost its <bos>", variant.id);
            assert_ne!(
                ids[1], bos[0],
                "{} starts with a duplicated <bos>",
                variant.id
            );
            checked += 1;
        }
        assert!(checked > 0, "no variant installed under ../models");
    }

    async fn gemma4_smoke(variant: &str) {
        let state = AppState::new(std::path::PathBuf::from("../models"));
        state
            .set_active_variant(variant)
            .await
            .expect("variant must be in the catalog");
        assert!(
            state.active_model_ready().await,
            "{variant} is not installed under ../models"
        );

        let result = generate_text(
            &state,
            GenerateOptions {
                prompt: "こんにちは".to_string(),
                max_tokens: Some(8),
                temperature: None,
                use_chat_template: Some(true),
            },
        )
        .await
        .expect("real inference");

        println!(
            "[{variant}] {} tok in {} ms ({:.1} tok/s)\n{}",
            result.generated_tokens, result.latency_ms, result.tokens_per_sec, result.text
        );

        assert!(!result.is_mock);
        assert!(!result.text.is_empty());
        assert!(result.generated_tokens > 0);
    }
}

#[cfg(test)]
mod mock_tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_mock_generate_when_model_absent() {
        let state = AppState::new(PathBuf::from("/nonexistent"));
        let opts = GenerateOptions {
            prompt: "test prompt".to_string(),
            max_tokens: Some(10),
            temperature: None,
            use_chat_template: Some(false),
        };
        let result = generate_text(&state, opts).await.unwrap();
        assert!(result.is_mock);
        assert!(result.text.contains("[MOCK]"));
    }

    /// Selecting Gemma 4 without downloading it must fall back to the mock and
    /// name the model the user actually picked.
    #[tokio::test]
    async fn mock_names_the_selected_gemma4_variant() {
        let dir = tempfile::tempdir().unwrap();
        let state = AppState::new(dir.path().to_path_buf());
        state.set_active_variant("4-e4b-int4").await.unwrap();

        let result = generate_text(
            &state,
            GenerateOptions {
                prompt: "test".to_string(),
                max_tokens: Some(10),
                temperature: None,
                use_chat_template: Some(false),
            },
        )
        .await
        .unwrap();

        assert!(result.is_mock);
        assert!(
            result.model_id.contains("Gemma 4 E4B"),
            "mock must report the selected model, got {}",
            result.model_id
        );
        assert!(result.text.contains("Gemma 4 E4B"));
    }
}
