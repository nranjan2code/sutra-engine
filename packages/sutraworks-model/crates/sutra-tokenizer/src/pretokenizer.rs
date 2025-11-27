//! Pre-tokenizers split text before applying main tokenization

use once_cell::sync::Lazy;
use regex::Regex;

/// Pre-tokenize text into chunks
pub trait PreTokenizer {
    fn pre_tokenize(&self, text: &str) -> Vec<String>;
}

/// Whitespace pre-tokenizer
pub struct WhitespacePreTokenizer;

impl PreTokenizer for WhitespacePreTokenizer {
    fn pre_tokenize(&self, text: &str) -> Vec<String> {
        text.split_whitespace().map(|s| s.to_string()).collect()
    }
}

/// ByteLevel pre-tokenizer (GPT-2 style)
pub struct ByteLevelPreTokenizer;

static BYTE_LEVEL_PATTERN: Lazy<Regex> = Lazy::new(|| {
    // Simplified regex without lookahead which isn't supported in the regex crate
    Regex::new(r"'s|'t|'re|'ve|'m|'ll|'d| ?\pL+| ?\pN+| ?[^\s\pL\pN]+|\s+").unwrap()
});

impl PreTokenizer for ByteLevelPreTokenizer {
    fn pre_tokenize(&self, text: &str) -> Vec<String> {
        BYTE_LEVEL_PATTERN
            .find_iter(text)
            .map(|m| m.as_str().to_string())
            .collect()
    }
}

/// Punctuation pre-tokenizer
pub struct PunctuationPreTokenizer;

static PUNCTUATION_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"\w+|[^\w\s]").unwrap());

impl PreTokenizer for PunctuationPreTokenizer {
    fn pre_tokenize(&self, text: &str) -> Vec<String> {
        PUNCTUATION_PATTERN
            .find_iter(text)
            .map(|m| m.as_str().to_string())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whitespace() {
        let pre_tok = WhitespacePreTokenizer;
        let result = pre_tok.pre_tokenize("Hello world!");
        assert_eq!(result, vec!["Hello", "world!"]);
    }

    #[test]
    fn test_byte_level() {
        let pre_tok = ByteLevelPreTokenizer;
        let result = pre_tok.pre_tokenize("Hello world");
        assert!(result.len() >= 2);
    }
}
