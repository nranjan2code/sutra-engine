//! High-level tokenizer API

use crate::bpe::{BpeConfig, BpeTokenizer};
use crate::error::Result;
use crate::unigram::{UnigramConfig, UnigramTokenizer};
use crate::wordpiece::{WordPieceConfig, WordPieceTokenizer};
use serde::{Deserialize, Serialize};

/// Encoding result with token IDs and offsets
#[derive(Debug, Clone)]
pub struct Encoding {
    /// Token IDs
    pub ids: Vec<u32>,

    /// Token strings
    pub tokens: Vec<String>,

    /// Character offsets for each token
    pub offsets: Vec<(usize, usize)>,

    /// Attention mask
    pub attention_mask: Vec<u32>,
}

impl Encoding {
    pub fn new(ids: Vec<u32>, tokens: Vec<String>) -> Self {
        let attention_mask = vec![1; ids.len()];
        Self {
            ids,
            tokens,
            offsets: Vec::new(),
            attention_mask,
        }
    }

    pub fn with_offsets(mut self, offsets: Vec<(usize, usize)>) -> Self {
        self.offsets = offsets;
        self
    }

    pub fn len(&self) -> usize {
        self.ids.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }
}

/// Tokenizer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenizerConfig {
    /// Tokenizer type
    pub tokenizer_type: String,

    /// Add special tokens
    pub add_special_tokens: bool,

    /// Padding side
    pub padding_side: String,

    /// Truncation length
    pub max_length: Option<usize>,
}

impl Default for TokenizerConfig {
    fn default() -> Self {
        Self {
            tokenizer_type: "bpe".to_string(),
            add_special_tokens: true,
            padding_side: "right".to_string(),
            max_length: Some(512),
        }
    }
}

/// Unified tokenizer interface
pub enum Tokenizer {
    Bpe(BpeTokenizer),
    WordPiece(WordPieceTokenizer),
    Unigram(UnigramTokenizer),
}

impl Tokenizer {
    /// Create BPE tokenizer
    pub fn bpe(config: BpeConfig) -> Self {
        Tokenizer::Bpe(BpeTokenizer::new(config))
    }

    /// Create WordPiece tokenizer
    pub fn wordpiece(config: WordPieceConfig) -> Self {
        Tokenizer::WordPiece(WordPieceTokenizer::new(config))
    }

    /// Create Unigram tokenizer
    pub fn unigram(config: UnigramConfig) -> Self {
        Tokenizer::Unigram(UnigramTokenizer::new(config))
    }

    /// Load from files (auto-detect type)
    pub fn from_file(vocab_path: &str) -> Result<Self> {
        // Try to load as BPE (default)
        let merges_path = vocab_path.replace("vocab.json", "merges.txt");

        if std::path::Path::new(&merges_path).exists() {
            Ok(Tokenizer::Bpe(BpeTokenizer::from_file(
                vocab_path,
                &merges_path,
            )?))
        } else {
            // Default to WordPiece
            let vocab = crate::vocab::Vocab::from_file(vocab_path)?;
            let config = WordPieceConfig {
                vocab,
                ..Default::default()
            };
            Ok(Tokenizer::WordPiece(WordPieceTokenizer::new(config)))
        }
    }

    /// Encode text to token IDs
    pub fn encode(&self, text: &str) -> Result<Encoding> {
        let ids = match self {
            Tokenizer::Bpe(t) => t.encode(text)?,
            Tokenizer::WordPiece(t) => t.encode(text)?,
            Tokenizer::Unigram(t) => t.encode(text)?,
        };

        let tokens = self.convert_ids_to_tokens(&ids)?;
        Ok(Encoding::new(ids, tokens))
    }

    /// Decode token IDs to text
    pub fn decode(&self, token_ids: &[u32]) -> Result<String> {
        match self {
            Tokenizer::Bpe(t) => t.decode(token_ids),
            Tokenizer::WordPiece(t) => t.decode(token_ids),
            Tokenizer::Unigram(t) => t.decode(token_ids),
        }
    }

    /// Convert token IDs to token strings
    pub fn convert_ids_to_tokens(&self, ids: &[u32]) -> Result<Vec<String>> {
        let vocab = match self {
            Tokenizer::Bpe(t) => &t.config.vocab,
            Tokenizer::WordPiece(t) => &t.config.vocab,
            Tokenizer::Unigram(t) => &t.config.vocab,
        };

        Ok(ids
            .iter()
            .filter_map(|&id| vocab.get_token(id).map(|s| s.to_string()))
            .collect())
    }

    /// Get vocabulary size
    pub fn vocab_size(&self) -> usize {
        match self {
            Tokenizer::Bpe(t) => t.vocab_size(),
            Tokenizer::WordPiece(t) => t.vocab_size(),
            Tokenizer::Unigram(t) => t.vocab_size(),
        }
    }

    /// Encode batch of texts
    pub fn encode_batch(&self, texts: &[&str]) -> Result<Vec<Encoding>> {
        texts.iter().map(|text| self.encode(text)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vocab::VocabBuilder;

    #[test]
    fn test_encoding() {
        let ids = vec![1, 2, 3];
        let tokens = vec!["hello".to_string(), "world".to_string(), "!".to_string()];
        let encoding = Encoding::new(ids.clone(), tokens);

        assert_eq!(encoding.len(), 3);
        assert_eq!(encoding.ids, ids);
    }

    #[test]
    fn test_tokenizer_wordpiece() {
        let vocab = VocabBuilder::new()
            .with_standard_special_tokens()
            .with_tokens(&["hello", "world"])
            .build();

        let config = WordPieceConfig {
            vocab,
            ..Default::default()
        };

        let tokenizer = Tokenizer::wordpiece(config);
        assert!(tokenizer.vocab_size() > 0);
    }
}
