//! Text normalizers for preprocessing

use unicode_normalization::UnicodeNormalization;

/// Normalize text before tokenization
pub trait Normalizer {
    fn normalize(&self, text: &str) -> String;
}

/// Lowercase normalizer
pub struct LowercaseNormalizer;

impl Normalizer for LowercaseNormalizer {
    fn normalize(&self, text: &str) -> String {
        text.to_lowercase()
    }
}

/// Unicode NFD normalizer
pub struct NfdNormalizer;

impl Normalizer for NfdNormalizer {
    fn normalize(&self, text: &str) -> String {
        text.nfd().collect()
    }
}

/// Strip accents normalizer
pub struct StripAccentsNormalizer;

impl Normalizer for StripAccentsNormalizer {
    fn normalize(&self, text: &str) -> String {
        text.nfd().filter(|c| !c.is_mark()).collect()
    }
}

/// Identity normalizer (no-op)
pub struct IdentityNormalizer;

impl Normalizer for IdentityNormalizer {
    fn normalize(&self, text: &str) -> String {
        text.to_string()
    }
}

trait CharExt {
    fn is_mark(&self) -> bool;
}

impl CharExt for char {
    fn is_mark(&self) -> bool {
        matches!(self, '\u{0300}'..='\u{036F}')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lowercase() {
        let normalizer = LowercaseNormalizer;
        assert_eq!(normalizer.normalize("Hello World"), "hello world");
    }

    #[test]
    fn test_identity() {
        let normalizer = IdentityNormalizer;
        assert_eq!(normalizer.normalize("Hello World"), "Hello World");
    }
}
