//! Simple pattern-based association extractor (V1)
//!
//! Extracts basic associations from text using configurable regex patterns.
//! This is a production-safe baseline that can be extended with NLP later.

use anyhow::Result;
use once_cell::sync::Lazy;
use regex::Regex;
use tracing::{debug};

#[derive(Debug, Clone, Copy)]
pub enum AssocKind {
    Semantic = 0,
    Causal = 1,
    Temporal = 2,
    Hierarchical = 3,
    Compositional = 4,
}

#[derive(Debug, Clone)]
pub struct ExtractedAssoc {
    pub target_term: String,
    pub confidence: f32,
    pub kind: AssocKind,
}

#[derive(Debug, Clone)]
pub struct AssocPattern {
    pub regex: Regex,
    pub kind: AssocKind,
    pub confidence: f32,
}

static DEFAULT_PATTERNS: Lazy<Vec<AssocPattern>> = Lazy::new(|| {
    vec![
        // Causal
        AssocPattern { regex: Regex::new(r"(?i)([A-Za-z][\w\- ]+)\s+causes\s+([A-Za-z][\w\- ]+)").unwrap(), kind: AssocKind::Causal, confidence: 0.8 },
        AssocPattern { regex: Regex::new(r"(?i)([A-Za-z][\w\- ]+)\s+leads to\s+([A-Za-z][\w\- ]+)").unwrap(), kind: AssocKind::Causal, confidence: 0.75 },
        // Hierarchical
        AssocPattern { regex: Regex::new(r"(?i)([A-Za-z][\w\- ]+)\s+is a[n]?\s+([A-Za-z][\w\- ]+)").unwrap(), kind: AssocKind::Hierarchical, confidence: 0.85 },
        AssocPattern { regex: Regex::new(r"(?i)([A-Za-z][\w\- ]+)\s+type of\s+([A-Za-z][\w\- ]+)").unwrap(), kind: AssocKind::Hierarchical, confidence: 0.8 },
        // Temporal
        AssocPattern { regex: Regex::new(r"(?i)([A-Za-z][\w\- ]+)\s+before\s+([A-Za-z][\w\- ]+)").unwrap(), kind: AssocKind::Temporal, confidence: 0.7 },
        AssocPattern { regex: Regex::new(r"(?i)([A-Za-z][\w\- ]+)\s+after\s+([A-Za-z][\w\- ]+)").unwrap(), kind: AssocKind::Temporal, confidence: 0.7 },
        // Compositional
        AssocPattern { regex: Regex::new(r"(?i)([A-Za-z][\w\- ]+)\s+part of\s+([A-Za-z][\w\- ]+)").unwrap(), kind: AssocKind::Compositional, confidence: 0.8 },
        AssocPattern { regex: Regex::new(r"(?i)([A-Za-z][\w\- ]+)\s+contains\s+([A-Za-z][\w\- ]+)").unwrap(), kind: AssocKind::Compositional, confidence: 0.75 },
    ]
});

#[derive(Debug, Clone)]
pub struct AssociationExtractorConfig {
    pub min_confidence: f32,
    pub max_associations_per_concept: usize,
}

impl Default for AssociationExtractorConfig {
    fn default() -> Self {
        Self {
            min_confidence: std::env::var("SUTRA_MIN_ASSOCIATION_CONFIDENCE").ok().and_then(|s| s.parse().ok()).unwrap_or(0.5),
            max_associations_per_concept: std::env::var("SUTRA_MAX_ASSOCIATIONS_PER_CONCEPT").ok().and_then(|s| s.parse().ok()).unwrap_or(10),
        }
    }
}

pub struct AssociationExtractor {
    patterns: &'static [AssocPattern],
    config: AssociationExtractorConfig,
}

impl AssociationExtractor {
    pub fn new(config: AssociationExtractorConfig) -> Result<Self> {
        Ok(Self { patterns: &DEFAULT_PATTERNS, config })
    }

    pub fn extract(&self, content: &str) -> Result<Vec<ExtractedAssoc>> {
        let mut results = Vec::new();
        let text = content.trim();

        for pat in self.patterns.iter() {
            if pat.confidence < self.config.min_confidence { continue; }

            for caps in pat.regex.captures_iter(text) {
                if let Some(target) = caps.get(2) {
                    let target_term = target.as_str().trim().to_string();
                    results.push(ExtractedAssoc { target_term, confidence: pat.confidence, kind: pat.kind });
                }
            }
        }

        // De-duplicate by target_term + kind
        results.sort_by(|a,b| a.target_term.cmp(&b.target_term));
        results.dedup_by(|a,b| a.target_term == b.target_term && (a.kind as u8) == (b.kind as u8));

        // Limit
        if results.len() > self.config.max_associations_per_concept {
            results.truncate(self.config.max_associations_per_concept);
        }

        debug!("AssociationExtractor: extracted {} associations", results.len());
        Ok(results)
    }
}
