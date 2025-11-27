use crate::error::Result;
use crate::vocab::Vocab;
use serde::{Deserialize, Serialize};

/// Unigram tokenizer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnigramConfig {
    pub vocab: Vocab,
    pub scores: Vec<f32>,
    pub unk_token: String,
}

impl Default for UnigramConfig {
    fn default() -> Self {
        Self {
            vocab: Vocab::new(),
            scores: Vec::new(),
            unk_token: "<unk>".to_string(),
        }
    }
}

/// Unigram tokenizer (used by SentencePiece)
pub struct UnigramTokenizer {
    pub config: UnigramConfig,
}

impl UnigramTokenizer {
    pub fn new(config: UnigramConfig) -> Self {
        Self { config }
    }

    pub fn encode(&self, text: &str) -> Result<Vec<u32>> {
        // Simplified unigram encoding
        let mut tokens = Vec::new();

        for word in text.split_whitespace() {
            let word_tokens = self.tokenize_word(word);
            tokens.extend(word_tokens);
        }

        Ok(tokens)
    }

    fn tokenize_word(&self, word: &str) -> Vec<u32> {
        // Viterbi algorithm for best segmentation
        let n = word.len();
        let mut best_score = vec![f32::NEG_INFINITY; n + 1];
        let mut best_prev = vec![0; n + 1];
        best_score[0] = 0.0;

        for i in 0..n {
            for j in i + 1..=n {
                let substr = &word[i..j];
                if let Some(id) = self.config.vocab.get_id(substr) {
                    let score = self.config.scores.get(id as usize).copied().unwrap_or(0.0);
                    let new_score = best_score[i] + score;

                    if new_score > best_score[j] {
                        best_score[j] = new_score;
                        best_prev[j] = i;
                    }
                }
            }
        }

        // Backtrack to find tokens
        let mut tokens = Vec::new();
        let mut pos = n;

        while pos > 0 {
            let prev = best_prev[pos];
            let substr = &word[prev..pos];
            if let Some(id) = self.config.vocab.get_id(substr) {
                tokens.push(id);
            }
            pos = prev;
        }

        tokens.reverse();
        tokens
    }

    pub fn decode(&self, tokens: &[u32]) -> Result<String> {
        let mut text = String::new();

        for &token_id in tokens {
            if let Some(token) = self.config.vocab.get_token(token_id) {
                text.push_str(token);
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

    #[test]
    fn test_unigram_basic() {
        let config = UnigramConfig::default();
        let tokenizer = UnigramTokenizer::new(config);
        assert_eq!(tokenizer.vocab_size(), 0);
    }
}
