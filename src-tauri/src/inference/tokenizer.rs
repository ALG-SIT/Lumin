use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
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

    /// Encode a fully rendered prompt for generation.
    ///
    /// Special tokens are deliberately off: the prompt already opens with
    /// [`ChatFormat::BOS`], and Gemma 3's tokenizer would prepend a second one.
    pub fn encode_prompt(&self, prompt: &str) -> Result<Vec<i64>> {
        self.encode(prompt, false)
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

/// Who spoke one turn of a conversation.
///
/// Serialized as the names the frontend uses; the on-the-wire spelling of the
/// assistant role differs from Gemma's (`model`), which [`ChatFormat`] applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    User,
    Assistant,
}

impl ChatRole {
    /// The role name Gemma's chat template expects.
    fn marker(&self) -> &'static str {
        match self {
            ChatRole::User => "user",
            ChatRole::Assistant => "model",
        }
    }
}

/// One turn of a conversation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatTurn {
    pub role: ChatRole,
    pub content: String,
}

impl ChatTurn {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::User,
            content: content.into(),
        }
    }

    /// Assistant turns reach the backend by deserialization; this exists for
    /// tests that build a conversation by hand.
    #[cfg(test)]
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::Assistant,
            content: content.into(),
        }
    }
}

impl ChatFormat {
    /// Opening token of every Gemma prompt.
    ///
    /// It is spelled out in the prompt string rather than left to the
    /// tokenizer because the two families disagree: Gemma 3's `tokenizer.json`
    /// has a `TemplateProcessing` post-processor that prepends `<bos>`, while
    /// Gemma 4's does not. Encoding therefore always runs with
    /// `add_special_tokens` off (see [`super::generate`]) and this is the one
    /// place the token comes from - otherwise Gemma 3 starts every prompt with
    /// a duplicated `<bos>` and Gemma 4 with none.
    pub const BOS: &'static str = "<bos>";

    /// Turn markers of this family, as `(start, end)`.
    fn turn_markers(&self) -> (&'static str, &'static str) {
        match self {
            ChatFormat::GemmaTurn => ("<start_of_turn>", "<end_of_turn>"),
            ChatFormat::Gemma4Turn => ("<|turn>", "<turn|>"),
        }
    }

    /// Wrap a user prompt into a single-turn conversation that ends ready for
    /// the model to speak.
    pub fn apply(&self, prompt: &str) -> String {
        self.apply_conversation(std::slice::from_ref(&ChatTurn::user(prompt)))
    }

    /// Render a whole conversation, one marked turn per message, ending ready
    /// for the model to speak.
    ///
    /// Each turn gets its own boundary so the model can tell earlier turns
    /// from the question being asked now. Flattening the history into the
    /// text of a single user turn loses that distinction, and the model then
    /// answers the oldest question or repeats its previous reply.
    /// Empty turns are dropped rather than emitted as an empty boundary.
    pub fn apply_conversation(&self, turns: &[ChatTurn]) -> String {
        let (start, end) = self.turn_markers();
        let mut rendered = String::from(Self::BOS);
        for turn in turns.iter().filter(|t| !t.content.trim().is_empty()) {
            rendered.push_str(start);
            rendered.push_str(turn.role.marker());
            rendered.push('\n');
            rendered.push_str(turn.content.trim());
            rendered.push_str(end);
            rendered.push('\n');
        }
        rendered.push_str(start);
        rendered.push_str("model\n");
        rendered
    }

    /// A prompt sent without a chat template. It still has to open with
    /// `<bos>`, which the tokenizer no longer supplies.
    pub fn apply_raw(prompt: &str) -> String {
        format!("{}{prompt}", Self::BOS)
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
    use super::{ChatFormat, ChatTurn};

    #[test]
    fn every_turn_of_a_conversation_gets_its_own_boundary() {
        let turns = [
            ChatTurn::user("一つ目"),
            ChatTurn::assistant("一つ目の答え"),
            ChatTurn::user("二つ目"),
        ];
        let rendered = ChatFormat::GemmaTurn.apply_conversation(&turns);
        assert_eq!(
            rendered,
            "<bos><start_of_turn>user\n一つ目<end_of_turn>\n\
             <start_of_turn>model\n一つ目の答え<end_of_turn>\n\
             <start_of_turn>user\n二つ目<end_of_turn>\n\
             <start_of_turn>model\n"
        );
        // The newest question is the last thing the model reads, so it cannot
        // be mistaken for an earlier one.
        let last_user = rendered.rfind("<start_of_turn>user").unwrap();
        assert!(rendered[last_user..].contains("二つ目"));
        assert!(!rendered[last_user..].contains("一つ目"));
    }

    #[test]
    fn gemma4_history_uses_its_own_markers_and_drops_empty_turns() {
        let rendered = ChatFormat::Gemma4Turn.apply_conversation(&[
            ChatTurn::user("質問"),
            ChatTurn::assistant("   "),
            ChatTurn::user("次の質問"),
        ]);
        assert!(!rendered.contains("<start_of_turn>"));
        assert_eq!(rendered.matches("<|turn>").count(), 3);
        assert!(!rendered.contains("model\n<turn|>"));
    }

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
    fn every_prompt_opens_with_exactly_one_bos() {
        // Gemma 3's tokenizer prepends <bos> on its own, Gemma 4's does not,
        // so the token lives in the prompt string and encoding runs with
        // add_special_tokens off.
        for rendered in [
            ChatFormat::GemmaTurn.apply("やあ"),
            ChatFormat::Gemma4Turn.apply("やあ"),
            ChatFormat::apply_raw("やあ"),
        ] {
            assert!(rendered.starts_with(ChatFormat::BOS), "{rendered}");
            assert_eq!(rendered.matches(ChatFormat::BOS).count(), 1, "{rendered}");
        }
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
