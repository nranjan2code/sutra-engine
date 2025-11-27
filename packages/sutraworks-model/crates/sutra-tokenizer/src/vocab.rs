use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Vocabulary mapping between tokens and IDs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vocab {
    /// Token to ID mapping
    token_to_id: HashMap<String, u32>,

    /// ID to token mapping
    id_to_token: HashMap<u32, String>,

    /// Special tokens
    special_tokens: HashMap<String, u32>,
}

impl Vocab {
    /// Create a new empty vocabulary
    pub fn new() -> Self {
        Self {
            token_to_id: HashMap::new(),
            id_to_token: HashMap::new(),
            special_tokens: HashMap::new(),
        }
    }

    /// Add a token to the vocabulary
    pub fn add_token(&mut self, token: impl Into<String>) -> u32 {
        let token = token.into();
        if let Some(&id) = self.token_to_id.get(&token) {
            return id;
        }

        let id = self.token_to_id.len() as u32;
        self.token_to_id.insert(token.clone(), id);
        self.id_to_token.insert(id, token);
        id
    }

    /// Add a special token
    pub fn add_special_token(&mut self, token: impl Into<String>) -> u32 {
        let token = token.into();
        let id = self.add_token(&token);
        self.special_tokens.insert(token, id);
        id
    }

    /// Get token ID
    pub fn get_id(&self, token: &str) -> Option<u32> {
        self.token_to_id.get(token).copied()
    }

    /// Get token from ID
    pub fn get_token(&self, id: u32) -> Option<&str> {
        self.id_to_token.get(&id).map(|s| s.as_str())
    }

    /// Get vocabulary size
    pub fn size(&self) -> usize {
        self.token_to_id.len()
    }

    /// Check if token is special
    pub fn is_special(&self, token: &str) -> bool {
        self.special_tokens.contains_key(token)
    }

    /// Get all special tokens
    pub fn special_tokens(&self) -> Vec<&str> {
        self.special_tokens.keys().map(|s| s.as_str()).collect()
    }

    /// Load vocabulary from JSON file
    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let token_to_id: HashMap<String, u32> = serde_json::from_str(&content)?;

        let mut id_to_token = HashMap::new();
        for (token, id) in &token_to_id {
            id_to_token.insert(*id, token.clone());
        }

        Ok(Self {
            token_to_id,
            id_to_token,
            special_tokens: HashMap::new(),
        })
    }

    /// Save vocabulary to JSON file
    pub fn save(&self, path: &str) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.token_to_id)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}

impl Default for Vocab {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating vocabularies
pub struct VocabBuilder {
    vocab: Vocab,
}

impl VocabBuilder {
    /// Create a new vocabulary builder
    pub fn new() -> Self {
        Self {
            vocab: Vocab::new(),
        }
    }

    /// Add standard special tokens
    pub fn with_standard_special_tokens(mut self) -> Self {
        self.vocab.add_special_token("<pad>");
        self.vocab.add_special_token("<unk>");
        self.vocab.add_special_token("<s>");
        self.vocab.add_special_token("</s>");
        self
    }

    /// Add custom special tokens
    pub fn with_special_tokens(mut self, tokens: &[&str]) -> Self {
        for token in tokens {
            self.vocab.add_special_token(*token);
        }
        self
    }

    /// Add tokens from a list
    pub fn with_tokens(mut self, tokens: &[&str]) -> Self {
        for token in tokens {
            self.vocab.add_token(*token);
        }
        self
    }

    /// Build the vocabulary
    pub fn build(self) -> Vocab {
        self.vocab
    }
}

impl Default for VocabBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vocab_basic() {
        let mut vocab = Vocab::new();

        let id1 = vocab.add_token("hello");
        let id2 = vocab.add_token("world");
        let id3 = vocab.add_token("hello"); // Duplicate

        assert_eq!(id1, id3);
        assert_ne!(id1, id2);
        assert_eq!(vocab.size(), 2);

        assert_eq!(vocab.get_token(id1), Some("hello"));
        assert_eq!(vocab.get_id("world"), Some(id2));
    }

    #[test]
    fn test_special_tokens() {
        let mut vocab = Vocab::new();
        vocab.add_special_token("<pad>");
        vocab.add_token("hello");

        assert!(vocab.is_special("<pad>"));
        assert!(!vocab.is_special("hello"));
        assert_eq!(vocab.special_tokens().len(), 1);
    }

    #[test]
    fn test_vocab_builder() {
        let vocab = VocabBuilder::new()
            .with_standard_special_tokens()
            .with_tokens(&["hello", "world"])
            .build();

        assert_eq!(vocab.size(), 6); // 4 special + 2 regular
        assert!(vocab.is_special("<pad>"));
        assert!(vocab.get_id("hello").is_some());
    }
}
