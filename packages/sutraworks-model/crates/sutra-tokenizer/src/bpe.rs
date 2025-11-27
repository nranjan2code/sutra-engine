use crate::error::Result;
use crate::vocab::Vocab;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// BPE (Byte Pair Encoding) tokenizer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BpeConfig {
    /// Vocabulary
    pub vocab: Vocab,

    /// Merge rules (pair -> merged token)
    pub merges: Vec<(String, String)>,

    /// Unknown token
    pub unk_token: String,

    /// Byte-level encoding
    pub byte_level: bool,
}

impl Default for BpeConfig {
    fn default() -> Self {
        Self {
            vocab: Vocab::new(),
            merges: Vec::new(),
            unk_token: "<unk>".to_string(),
            byte_level: true,
        }
    }
}

/// Byte Pair Encoding tokenizer
///
/// BPE iteratively merges the most frequent pair of bytes/characters.
/// Used by GPT-2, GPT-3, and many modern LLMs.
///
/// # Example
/// ```no_run
/// use sutra_tokenizer::BpeTokenizer;
///
/// let tokenizer = BpeTokenizer::from_file("vocab.json", "merges.txt")?;
/// let tokens = tokenizer.encode("Hello, world!")?;
/// let text = tokenizer.decode(&tokens)?;
/// # Ok::<(), sutra_tokenizer::TokenizerError>(())
/// ```
pub struct BpeTokenizer {
    pub config: BpeConfig,
    merge_ranks: HashMap<(String, String), usize>,
    byte_encoder: Option<HashMap<u8, char>>,
    byte_decoder: Option<HashMap<char, u8>>,
}

impl BpeTokenizer {
    /// Create a new BPE tokenizer from configuration
    pub fn new(config: BpeConfig) -> Self {
        let merge_ranks: HashMap<(String, String), usize> = config
            .merges
            .iter()
            .enumerate()
            .map(|(i, (a, b))| ((a.clone(), b.clone()), i))
            .collect();

        let (byte_encoder, byte_decoder) = if config.byte_level {
            let encoder = Self::bytes_to_unicode();
            let decoder = encoder.iter().map(|(&k, &v)| (v, k)).collect();
            (Some(encoder), Some(decoder))
        } else {
            (None, None)
        };

        Self {
            config,
            merge_ranks,
            byte_encoder,
            byte_decoder,
        }
    }

    /// Load BPE tokenizer from vocabulary and merges files
    pub fn from_file(vocab_path: &str, merges_path: &str) -> Result<Self> {
        let vocab = Vocab::from_file(vocab_path)?;

        // Load merges
        let merges_content = std::fs::read_to_string(merges_path)?;
        let merges: Vec<(String, String)> = merges_content
            .lines()
            .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    Some((parts[0].to_string(), parts[1].to_string()))
                } else {
                    None
                }
            })
            .collect();

        let config = BpeConfig {
            vocab,
            merges,
            unk_token: "<unk>".to_string(),
            byte_level: true,
        };

        Ok(Self::new(config))
    }

    /// Encode text to token IDs
    pub fn encode(&self, text: &str) -> Result<Vec<u32>> {
        let words = self.pre_tokenize(text);
        let mut tokens = Vec::new();

        for word in words {
            let word_tokens = self.bpe(&word);
            for token in word_tokens {
                if let Some(id) = self.config.vocab.get_id(&token) {
                    tokens.push(id);
                } else if let Some(unk_id) = self.config.vocab.get_id(&self.config.unk_token) {
                    tokens.push(unk_id);
                }
            }
        }

        Ok(tokens)
    }

    /// Decode token IDs to text
    pub fn decode(&self, tokens: &[u32]) -> Result<String> {
        let mut text = String::new();

        for &token_id in tokens {
            if let Some(token) = self.config.vocab.get_token(token_id) {
                if self.config.byte_level {
                    // Decode byte-level encoding
                    if let Some(decoder) = &self.byte_decoder {
                        let bytes: Vec<u8> = token
                            .chars()
                            .filter_map(|c| decoder.get(&c).copied())
                            .collect();
                        if let Ok(s) = String::from_utf8(bytes) {
                            text.push_str(&s);
                        }
                    }
                } else {
                    text.push_str(token);
                }
            }
        }

        Ok(text)
    }

    /// Pre-tokenize text (split into words)
    fn pre_tokenize(&self, text: &str) -> Vec<String> {
        if self.config.byte_level {
            // Byte-level: encode each word
            text.split_whitespace()
                .map(|word| {
                    if let Some(encoder) = &self.byte_encoder {
                        word.bytes()
                            .map(|b| encoder.get(&b).copied().unwrap_or('�'))
                            .collect()
                    } else {
                        word.to_string()
                    }
                })
                .collect()
        } else {
            text.split_whitespace().map(|s| s.to_string()).collect()
        }
    }

    /// Apply BPE algorithm to a word
    fn bpe(&self, word: &str) -> Vec<String> {
        if word.is_empty() {
            return vec![];
        }

        let mut word_chars: Vec<String> = word.chars().map(|c| c.to_string()).collect();

        loop {
            // Find the pair with minimum rank
            let mut min_pair = None;
            let mut min_rank = usize::MAX;

            for i in 0..word_chars.len().saturating_sub(1) {
                let pair = (word_chars[i].clone(), word_chars[i + 1].clone());
                if let Some(&rank) = self.merge_ranks.get(&pair) {
                    if rank < min_rank {
                        min_rank = rank;
                        min_pair = Some((i, pair));
                    }
                }
            }

            // If no more merges possible, break
            if min_pair.is_none() {
                break;
            }

            // Perform the merge
            if let Some((pos, (first, second))) = min_pair {
                let merged = format!("{}{}", first, second);
                word_chars[pos] = merged;
                word_chars.remove(pos + 1);
            }
        }

        word_chars
    }

    /// Create byte-to-unicode mapping for byte-level BPE
    fn bytes_to_unicode() -> HashMap<u8, char> {
        let mut byte_encoder = HashMap::new();
        let mut n = 0;

        // Printable ASCII
        for b in 33..=126 {
            byte_encoder.insert(b, char::from_u32(b as u32).unwrap());
        }
        for b in 161..=172 {
            byte_encoder.insert(b, char::from_u32(b as u32).unwrap());
        }
        for b in 174..=255 {
            byte_encoder.insert(b, char::from_u32(b as u32).unwrap());
        }

        // Fill in remaining bytes with shifted unicode
        for b in 0..=255u8 {
            if let std::collections::hash_map::Entry::Vacant(e) = byte_encoder.entry(b) {
                e.insert(char::from_u32(256 + n).unwrap());
                n += 1;
            }
        }

        byte_encoder
    }

    /// Get vocabulary size
    pub fn vocab_size(&self) -> usize {
        self.config.vocab.size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vocab::VocabBuilder;

    #[test]
    fn test_bpe_basic() {
        // Create a simple vocabulary
        let vocab = VocabBuilder::new()
            .with_special_tokens(&["<unk>", "<pad>"])
            .with_tokens(&["h", "e", "l", "o", "he", "llo"])
            .build();

        let merges = vec![
            ("h".to_string(), "e".to_string()),
            ("l".to_string(), "l".to_string()),
        ];

        let config = BpeConfig {
            vocab,
            merges,
            unk_token: "<unk>".to_string(),
            byte_level: false,
        };

        let tokenizer = BpeTokenizer::new(config);
        assert_eq!(tokenizer.vocab_size(), 8);
    }

    #[test]
    fn test_byte_encoder() {
        let encoder = BpeTokenizer::bytes_to_unicode();
        assert_eq!(encoder.len(), 256);
        assert!(encoder.contains_key(&65)); // 'A'
    }
}
