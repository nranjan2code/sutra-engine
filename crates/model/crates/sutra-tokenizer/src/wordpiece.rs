use crate::error::Result;
use crate::vocab::Vocab;
use serde::{Deserialize, Serialize};

/// WordPiece tokenizer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordPieceConfig {
    pub vocab: Vocab,
    pub unk_token: String,
    pub max_input_chars_per_word: usize,
    pub continuing_subword_prefix: String,
}

impl Default for WordPieceConfig {
    fn default() -> Self {
        Self {
            vocab: Vocab::new(),
            unk_token: "[UNK]".to_string(),
            max_input_chars_per_word: 100,
            continuing_subword_prefix: "##".to_string(),
        }
    }
}

/// WordPiece tokenizer (used by BERT)
pub struct WordPieceTokenizer {
    pub config: WordPieceConfig,
}

impl WordPieceTokenizer {
    pub fn new(config: WordPieceConfig) -> Self {
        Self { config }
    }

    pub fn encode(&self, text: &str) -> Result<Vec<u32>> {
        let words = text.split_whitespace();
        let mut tokens = Vec::new();

        for word in words {
            if word.len() > self.config.max_input_chars_per_word {
                if let Some(unk_id) = self.config.vocab.get_id(&self.config.unk_token) {
                    tokens.push(unk_id);
                }
                continue;
            }

            let word_tokens = self.tokenize_word(word);
            tokens.extend(word_tokens);
        }

        Ok(tokens)
    }

    fn tokenize_word(&self, word: &str) -> Vec<u32> {
        let mut tokens = Vec::new();
        let mut start = 0;

        while start < word.len() {
            let mut end = word.len();
            let mut found = false;

            // Greedily match longest subword
            while start < end {
                let substr = &word[start..end];
                let token = if start > 0 {
                    format!("{}{}", self.config.continuing_subword_prefix, substr)
                } else {
                    substr.to_string()
                };

                if let Some(id) = self.config.vocab.get_id(&token) {
                    tokens.push(id);
                    start = end;
                    found = true;
                    break;
                }

                end -= 1;
            }

            if !found {
                // Unknown token
                if let Some(unk_id) = self.config.vocab.get_id(&self.config.unk_token) {
                    tokens.push(unk_id);
                }
                break;
            }
        }

        tokens
    }

    pub fn decode(&self, tokens: &[u32]) -> Result<String> {
        let mut text = String::new();
        let prefix = &self.config.continuing_subword_prefix;

        for &token_id in tokens {
            if let Some(token) = self.config.vocab.get_token(token_id) {
                if token.starts_with(prefix) {
                    text.push_str(&token[prefix.len()..]);
                } else {
                    if !text.is_empty() {
                        text.push(' ');
                    }
                    text.push_str(token);
                }
            }
        }

        Ok(text)
    }

    pub fn vocab_size(&self) -> usize {
        self.config.vocab.size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vocab::VocabBuilder;

    #[test]
    fn test_wordpiece_basic() {
        let vocab = VocabBuilder::new()
            .with_special_tokens(&["[UNK]", "[PAD]", "[CLS]", "[SEP]"])
            .with_tokens(&["hello", "##world", "test"])
            .build();

        let config = WordPieceConfig {
            vocab,
            ..Default::default()
        };

        let tokenizer = WordPieceTokenizer::new(config);
        assert_eq!(tokenizer.vocab_size(), 7);
    }
}
