use anyhow::{Context, Result};
use std::path::Path;
use tokenizers::Tokenizer;

/// Wrapper for Gemma tokenizer (SentencePiece-based)
pub struct GemmaTokenizer {
    inner: Tokenizer,
}

impl GemmaTokenizer {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let tok = Tokenizer::from_file(path.as_ref()).map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(Self { inner: tok })
    }

    pub fn encode(&self, text: &str, add_special_tokens: bool) -> Result<Vec<i64>> {
        let enc = self
            .inner
            .encode(text, add_special_tokens)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(enc.get_ids().iter().map(|&x| x as i64).collect())
    }

    pub fn decode(&self, ids: &[i64], skip_special_tokens: bool) -> Result<String> {
        let u32_ids: Vec<u32> = ids.iter().map(|&x| x as u32).collect();
        self.inner
            .decode(&u32_ids, skip_special_tokens)
            .map_err(|e| anyhow::anyhow!("{}", e))
    }

    pub(crate) fn inner(&self) -> &Tokenizer {
        &self.inner
    }

    #[allow(dead_code)]
    pub fn vocab_size(&self) -> usize {
        self.inner.get_vocab_size(true)
    }
}

pub fn load_tokenizer<P: AsRef<Path>>(path: P) -> Result<GemmaTokenizer> {
    GemmaTokenizer::from_file(path.as_ref())
        .with_context(|| format!("failed to load tokenizer at {:?}", path.as_ref()))
}

/// Turn markers a model family was instruction-tuned with.
///
/// These differ between families and are not interchangeable: feeding Gemma 3's
/// markers to Gemma 4 tokenizes them as ordinary text, so the model never sees
/// a turn boundary and echoes the markers back in its reply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatFormat {
    /// Gemma 3 / 3n: `<start_of_turn>` (105) / `<end_of_turn>` (106).
    GemmaTurn,
    /// Gemma 4: `<|turn>` (105) / `<turn|>` (106), per the upstream
    /// `chat_template.jinja`.
    Gemma4Turn,
}

impl ChatFormat {
    /// Wrap a user prompt into a single-turn conversation that ends ready for
    /// the model to speak.
    pub fn apply(&self, prompt: &str) -> String {
        match self {
            ChatFormat::GemmaTurn => {
                format!("<bos><start_of_turn>user\n{prompt}<end_of_turn>\n<start_of_turn>model\n")
            }
            ChatFormat::Gemma4Turn => {
                format!("<bos><|turn>user\n{prompt}<turn|>\n<|turn>model\n")
            }
        }
    }

    /// Token ids that end the model's turn, from each family's
    /// `generation_config.json`.
    pub fn eos_token_ids(&self) -> &'static [i64] {
        match self {
            // <eos>, <end_of_turn>
            ChatFormat::GemmaTurn => &[1, 106],
            // <eos>, <turn|>, <|tool_response>
            ChatFormat::Gemma4Turn => &[1, 106, 50],
        }
    }
}

/// Fallback mock tokenizer when model not present (for validation without download)
#[allow(dead_code)]
pub fn mock_tokenize(text: &str) -> Vec<i64> {
    // Simple whitespace mock - not accurate but allows pipeline validation
    text.split_whitespace()
        .enumerate()
        .map(|(i, _)| 1000 + i as i64)
        .collect()
}

#[allow(dead_code)]
pub fn mock_detokenize(ids: &[i64]) -> String {
    format!("[mock detokenize: {} tokens]", ids.len())
}

#[cfg(test)]
mod tests {
    use super::ChatFormat;

    #[test]
    fn each_family_uses_its_own_turn_markers() {
        let g3 = ChatFormat::GemmaTurn.apply("やあ");
        assert!(g3.starts_with("<bos><start_of_turn>user\nやあ<end_of_turn>"));
        assert!(g3.ends_with("<start_of_turn>model\n"));

        let g4 = ChatFormat::Gemma4Turn.apply("やあ");
        assert!(g4.starts_with("<bos><|turn>user\nやあ<turn|>"));
        assert!(g4.ends_with("<|turn>model\n"));

        // Mixing them is the bug this enum exists to prevent.
        assert!(!g4.contains("<start_of_turn>"));
        assert!(!g3.contains("<|turn>"));
    }

    #[test]
    fn eos_ids_cover_each_family_end_of_turn() {
        // 106 ends the turn in both families; 50 is <unused44> in Gemma 3 and
        // must not truncate its output.
        assert_eq!(ChatFormat::GemmaTurn.eos_token_ids(), &[1, 106]);
        assert!(!ChatFormat::GemmaTurn.eos_token_ids().contains(&50));
        assert!(ChatFormat::Gemma4Turn.eos_token_ids().contains(&50));
    }
}
