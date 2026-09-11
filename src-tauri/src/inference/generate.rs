use anyhow::Result;
use ort::session::{Session, SessionInputValue};
use ort::value::Tensor;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

use super::catalog::Architecture;
use super::session::{has_input, kv_layer_specs, AppState, KvLayerSpec};
use super::tokenizer::{apply_gemma_chat_template, load_tokenizer, mock_detokenize};

/// Token ids that end a turn across the Gemma families Lumin supports:
/// `<eos>` (1), `<end_of_turn>` (106), and Gemma 4's additional stop id 50.
const EOS_TOKEN_IDS: &[i64] = &[1, 106, 50];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateOptions {
    pub prompt: String,
    pub max_tokens: Option<usize>,
    pub temperature: Option<f32>,
    pub use_chat_template: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateResult {
    pub text: String,
    pub prompt_tokens: usize,
    pub generated_tokens: usize,
    pub total_tokens: usize,
    pub latency_ms: u64,
    pub tokens_per_sec: f64,
    pub is_mock: bool,
    pub model_id: String,
}

/// Core generation - if model not present, returns mock response for pipeline validation
pub async fn generate_text(state: &AppState, opts: GenerateOptions) -> Result<GenerateResult> {
    let max_tokens = opts.max_tokens.unwrap_or(128).min(512);
    let use_template = opts.use_chat_template.unwrap_or(true);
    let prompt = if use_template {
        apply_gemma_chat_template(&opts.prompt)
    } else {
        opts.prompt.clone()
    };

    // Mock path when the selected model is not fully installed - allows UI
    // validation without a multi-gigabyte download.
    if !state.active_model_ready().await {
        let variant = state.active_variant().await;
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

/// Build the decoder's inputs for one step, supplying only what the graph
/// declares. Gemma 3 takes `input_ids`; Gemma 3n/4 take `inputs_embeds` plus
/// `per_layer_inputs`, `position_ids` and `num_logits_to_keep`.
fn build_decoder_inputs(
    decoder: &Session,
    kv_specs: &[KvLayerSpec],
    token_ids: &[i64],
    embeddings: Option<&Embeddings>,
    cache: &Option<Vec<(Vec<f32>, Vec<f32>)>>,
    past_len: usize,
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
        let attention_len = past_len + seq_len;
        let t = Tensor::from_array(([1, attention_len], vec![1i64; attention_len]))
            .map_err(|e| anyhow::anyhow!("attention_mask tensor error: {e}"))?;
        inputs.push(("attention_mask".to_string(), t.into()));
    }

    if has_input(decoder, "position_ids") {
        let positions: Vec<i64> = (past_len..past_len + seq_len).map(|p| p as i64).collect();
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
            None => (Vec::new(), Vec::new()),
        };
        let shape = [1, spec.num_heads, past_len, spec.head_dim];
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
    emit: Option<&(dyn Fn(String) -> Result<()> + Send + Sync)>,
) -> Result<GenerateResult> {
    let start = Instant::now();
    let variant = state.active_variant().await;
    let tok_path = variant.tokenizer_path(&state.model_dir);
    let model_path = variant.decoder_path(&state.model_dir);

    let tokenizer = load_tokenizer(&tok_path)?;
    let input_ids = tokenizer.encode(prompt, true)?;
    let prompt_tokens = input_ids.len();

    // Load or reuse session; a model switch clears it, so a stale variant here
    // means the state was replaced without going through set_active_variant.
    let mut guard = state.session.lock().await;
    if guard.as_ref().map_or(true, |s| s.variant_id != variant.id) {
        let session = super::session::create_session(&model_path)?;
        // Gemma 3n / Gemma 4 decoders take inputs_embeds, so their embed graph
        // is part of the install set; a missing one is a broken install, not a
        // reason to fall back to mock output.
        let embed_session = match variant.architecture {
            Architecture::EmbedChained => {
                let path = variant.embed_path(&state.model_dir).ok_or_else(|| {
                    anyhow::anyhow!("{} declares no embed_tokens graph", variant.id)
                })?;
                if !path.exists() {
                    anyhow::bail!("embed_tokens graph missing for {}: {:?}", variant.id, path);
                }
                Some(super::session::create_session(&path)?)
            }
            Architecture::DecoderOnly => None,
        };
        *guard = Some(super::session::InferenceSession {
            session,
            embed_session,
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
        });
    }

    let session = guard.as_mut().unwrap();
    let kv_specs = kv_layer_specs(&session.session);

    let mut generated_ids: Vec<i64> = Vec::new();
    let mut current_ids = input_ids.clone();
    let mut cache: Option<Vec<(Vec<f32>, Vec<f32>)>> = None;
    let mut past_len = 0usize;
    let mut decode_stream = emit.map(|_| tokenizer.inner().decode_stream(true));

    for _ in 0..max_tokens {
        let embeddings = match session.embed_session.as_mut() {
            Some(embed) => Some(embed_tokens(embed, &current_ids)?),
            None => None,
        };

        let inputs = build_decoder_inputs(
            &session.session,
            &kv_specs,
            &current_ids,
            embeddings.as_ref(),
            &cache,
            past_len,
        )?;

        let outputs = session
            .session
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

        past_len += current_ids.len();

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

        if EOS_TOKEN_IDS.contains(&next_id) {
            break;
        }

        generated_ids.push(next_id);
        current_ids = vec![next_id];

        if let (Some(emit), Some(decoder)) = (emit, decode_stream.as_mut()) {
            if let Some(text) = decoder
                .step(next_id as u32)
                .map_err(|e| anyhow::anyhow!("stream decode error: {e}"))?
            {
                emit(text)?;
            }
        }

        if generated_ids.len() >= max_tokens {
            break;
        }
    }

    let text = if generated_ids.is_empty() {
        mock_detokenize(&current_ids)
    } else {
        tokenizer
            .decode(&generated_ids, true)
            .unwrap_or_else(|_| mock_detokenize(&generated_ids))
    };

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
    emit: impl Fn(String) -> Result<()> + Send + Sync,
) -> Result<GenerateResult> {
    let max_tokens = opts.max_tokens.unwrap_or(32).min(512);

    if !state.active_model_ready().await {
        let variant = state.active_variant().await;
        let res = mock_generate(&opts.prompt, max_tokens, variant.display_name);
        // Simulate token-by-token emit
        for tok in res.text.split_whitespace() {
            emit(format!("{tok} "))?;
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        return Ok(res);
    }

    let prompt = if opts.use_chat_template.unwrap_or(true) {
        apply_gemma_chat_template(&opts.prompt)
    } else {
        opts.prompt.clone()
    };
    try_real_inference(state, &prompt, max_tokens, Some(&emit)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[tokio::test]
    #[ignore]
    async fn real_inference_smoke() {
        let state = AppState::new(std::path::PathBuf::from("../models"));
        let opts = GenerateOptions {
            prompt: "こんにちは".to_string(),
            max_tokens: Some(8),
            temperature: None,
            use_chat_template: Some(true),
        };

        let result = generate_text(&state, opts).await.expect("real inference");
        assert!(!result.is_mock);
        assert!(!result.text.is_empty());
    }

    #[tokio::test]
    #[ignore]
    async fn real_streaming_inference_smoke() {
        let state = AppState::new(std::path::PathBuf::from("../models"));
        let opts = GenerateOptions {
            prompt: "こんにちは".to_string(),
            max_tokens: Some(8),
            temperature: None,
            use_chat_template: Some(true),
        };
        let emitted = Arc::new(Mutex::new(Vec::<String>::new()));
        let emitted_for_callback = Arc::clone(&emitted);

        let result = generate_stream(&state, opts, move |token| {
            emitted_for_callback.lock().unwrap().push(token);
            Ok(())
        })
        .await
        .expect("real streaming inference");

        assert!(!result.is_mock);
        assert!(!result.text.is_empty());
        let streamed = emitted.lock().unwrap().concat();
        assert!(!streamed.is_empty());
        assert_eq!(streamed, result.text);
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
