/// Production-Grade Semantic Analyzer
///
/// Deterministic pattern-based semantic classification.
/// No ML models, no fallbacks - pure rule-based system.
use super::config::SemanticConfig;
use super::types::*;
use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info, warn};

/// Compiled regex patterns for semantic classification
#[derive(Clone)]
struct SemanticPatterns {
    // Temporal patterns
    temporal_after: Vec<Regex>,
    temporal_before: Vec<Regex>,
    temporal_during: Vec<Regex>,
    temporal_between: Vec<Regex>,
    _temporal_at: Vec<Regex>,

    // Rule patterns
    rule_modal: Vec<Regex>,
    rule_conditional: Vec<Regex>,
    rule_imperative: Vec<Regex>,

    // Negation patterns
    negation_explicit: Vec<Regex>,
    negation_exception: Vec<Regex>,

    // Causal patterns
    causal_direct: Vec<Regex>,
    causal_enabling: Vec<Regex>,
    causal_preventing: Vec<Regex>,

    // Condition patterns
    condition_if: Vec<Regex>,
    condition_when: Vec<Regex>,
    condition_unless: Vec<Regex>,

    // Quantitative patterns
    quantitative_number: Vec<Regex>,
    quantitative_percentage: Vec<Regex>,
    quantitative_measurement: Vec<Regex>,

    // Definitional patterns
    definitional_is_a: Vec<Regex>,
    definitional_defined_as: Vec<Regex>,

    // Event patterns
    event_past: Vec<Regex>,
    event_future: Vec<Regex>,
    event_ongoing: Vec<Regex>,

    // Domain patterns
    domains: HashMap<String, Vec<Regex>>,
}

/// Production semantic analyzer
#[derive(Clone)]
pub struct SemanticAnalyzer {
    patterns: Arc<SemanticPatterns>,
}

impl SemanticAnalyzer {
    /// Create new semantic analyzer
    pub fn new() -> Self {
        info!("Initializing SemanticAnalyzer...");

        // Try to load from "semantics.toml" in current directory
        let config_path = std::path::Path::new("semantics.toml");
        let config = if config_path.exists() {
            info!("Loading semantic rules from {:?}", config_path);
            match std::fs::read_to_string(config_path) {
                Ok(content) => match toml::from_str(&content) {
                    Ok(c) => c,
                    Err(e) => {
                        error!("Failed to parse semantics.toml: {}", e);
                        SemanticConfig::default()
                    }
                },
                Err(e) => {
                    error!("Failed to read semantics.toml: {}", e);
                    SemanticConfig::default()
                }
            }
        } else {
            warn!("semantics.toml not found. Using default rules.");
            SemanticConfig::default()
        };

        let patterns = Self::compile_patterns(&config);

        Self {
            patterns: Arc::new(patterns),
        }
    }

    /// Compile string patterns into Regex
    fn compile_patterns(config: &SemanticConfig) -> SemanticPatterns {
        let compile = |patterns: &[String]| -> Vec<Regex> {
            patterns
                .iter()
                .filter_map(|p| match Regex::new(&format!("(?i){}", p)) {
                    Ok(r) => Some(r),
                    Err(e) => {
                        warn!("Invalid regex '{}': {}", p, e);
                        None
                    }
                })
                .collect()
        };

        SemanticPatterns {
            temporal_after: compile(&config.temporal.after),
            temporal_before: compile(&config.temporal.before),
            temporal_during: compile(&config.temporal.during),
            temporal_between: compile(&config.temporal.between),
            _temporal_at: compile(&config.temporal.at),
            rule_modal: compile(&config.rules.modal),
            rule_conditional: compile(&config.rules.conditional),
            rule_imperative: compile(&config.rules.imperative),
            negation_explicit: compile(&config.negation.explicit),
            negation_exception: compile(&config.negation.exception),
            causal_direct: compile(&config.causal.direct),
            causal_enabling: compile(&config.causal.enabling),
            causal_preventing: compile(&config.causal.preventing),
            condition_if: compile(&config.conditions.if_clause),
            condition_when: compile(&config.conditions.when_clause),
            condition_unless: compile(&config.conditions.unless_clause),
            quantitative_number: compile(&config.quantitative.number),
            quantitative_percentage: compile(&config.quantitative.percentage),
            quantitative_measurement: compile(&config.quantitative.measurement),
            definitional_is_a: compile(&config.definitional.is_a),
            definitional_defined_as: compile(&config.definitional.defined_as),
            event_past: compile(&config.events.past),
            event_future: compile(&config.events.future),
            event_ongoing: compile(&config.events.ongoing),
            domains: config
                .domains
                .iter()
                .map(|(k, v)| (k.clone(), compile(v)))
                .collect(),
        }
    }

    /// Analyze text and extract complete semantic metadata
    pub fn analyze(&self, text: &str) -> SemanticMetadata {
        // Primary semantic type classification
        let semantic_type = self.classify_type(text);

        // Extract temporal bounds
        let temporal_bounds = self.extract_temporal(text);

        // Extract causal relations
        let causal_relations = self.extract_causal(text);

        // Detect domain context
        let domain_context = self.detect_domain(text);

        // Extract negation scope
        let negation_scope = if semantic_type == SemanticType::Negation {
            self.extract_negation(text)
        } else {
            None
        };

        // Classification confidence based on pattern strength
        let classification_confidence = self.calculate_confidence(text, semantic_type);

        SemanticMetadata {
            semantic_type,
            temporal_bounds,
            causal_relations,
            domain_context,
            negation_scope,
            classification_confidence,
        }
    }

    /// Helper to check if any regex in a list matches
    fn any_match(patterns: &[Regex], text: &str) -> bool {
        patterns.iter().any(|p| p.is_match(text))
    }

    /// Helper to count matches
    fn count_matches(patterns: &[Regex], text: &str) -> usize {
        patterns.iter().map(|p| p.find_iter(text).count()).sum()
    }

    /// Classify primary semantic type
    fn classify_type(&self, text: &str) -> SemanticType {
        let mut scores: HashMap<SemanticType, f32> = HashMap::new();

        // Rule patterns
        if Self::any_match(&self.patterns.rule_modal, text) {
            *scores.entry(SemanticType::Rule).or_insert(0.0) += 3.0;
        }
        if Self::any_match(&self.patterns.rule_conditional, text) {
            *scores.entry(SemanticType::Rule).or_insert(0.0) += 2.5;
        }
        if Self::any_match(&self.patterns.rule_imperative, text) {
            *scores.entry(SemanticType::Rule).or_insert(0.0) += 2.0;
        }

        // Temporal patterns
        if Self::any_match(&self.patterns.temporal_after, text)
            || Self::any_match(&self.patterns.temporal_before, text)
        {
            *scores.entry(SemanticType::Temporal).or_insert(0.0) += 2.0;
        }
        if Self::any_match(&self.patterns.temporal_during, text)
            || Self::any_match(&self.patterns.temporal_between, text)
        {
            *scores.entry(SemanticType::Temporal).or_insert(0.0) += 1.5;
        }

        // Negation patterns
        if Self::any_match(&self.patterns.negation_explicit, text) {
            *scores.entry(SemanticType::Negation).or_insert(0.0) += 2.0;
        }
        if Self::any_match(&self.patterns.negation_exception, text) {
            *scores.entry(SemanticType::Negation).or_insert(0.0) += 2.5;
        }

        // Causal patterns
        if Self::any_match(&self.patterns.causal_direct, text) {
            *scores.entry(SemanticType::Causal).or_insert(0.0) += 2.5;
        }
        if Self::any_match(&self.patterns.causal_enabling, text)
            || Self::any_match(&self.patterns.causal_preventing, text)
        {
            *scores.entry(SemanticType::Causal).or_insert(0.0) += 2.0;
        }

        // Condition patterns
        if Self::any_match(&self.patterns.condition_if, text) {
            *scores.entry(SemanticType::Condition).or_insert(0.0) += 2.0;
        }
        if Self::any_match(&self.patterns.condition_when, text)
            || Self::any_match(&self.patterns.condition_unless, text)
        {
            *scores.entry(SemanticType::Condition).or_insert(0.0) += 1.5;
        }

        // Quantitative patterns
        if Self::any_match(&self.patterns.quantitative_number, text)
            || Self::any_match(&self.patterns.quantitative_percentage, text)
        {
            *scores.entry(SemanticType::Quantitative).or_insert(0.0) += 1.0;
        }
        if Self::any_match(&self.patterns.quantitative_measurement, text) {
            *scores.entry(SemanticType::Quantitative).or_insert(0.0) += 1.5;
        }

        // Definitional patterns
        if Self::any_match(&self.patterns.definitional_is_a, text) {
            *scores.entry(SemanticType::Definitional).or_insert(0.0) += 1.5;
        }
        if Self::any_match(&self.patterns.definitional_defined_as, text) {
            *scores.entry(SemanticType::Definitional).or_insert(0.0) += 2.0;
        }

        // Event patterns
        if Self::any_match(&self.patterns.event_past, text)
            || Self::any_match(&self.patterns.event_future, text)
        {
            *scores.entry(SemanticType::Event).or_insert(0.0) += 1.5;
        }
        if Self::any_match(&self.patterns.event_ongoing, text) {
            *scores.entry(SemanticType::Event).or_insert(0.0) += 1.0;
        }

        // Return highest scoring type, default to Entity
        scores
            .into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(t, _)| t)
            .unwrap_or(SemanticType::Entity)
    }

    /// Extract temporal bounds from text
    fn extract_temporal(&self, text: &str) -> Option<TemporalBounds> {
        // Try to extract year/date
        let year_pattern = Regex::new(r"\b(19|20)\d{2}\b").unwrap();

        if let Some(year_match) = year_pattern.find(text) {
            if let Ok(year) = year_match.as_str().parse::<i64>() {
                let timestamp = (year - 1970) * 365 * 24 * 3600; // Approximate Unix timestamp

                // Determine temporal relation
                let relation = if Self::any_match(&self.patterns.temporal_after, text) {
                    TemporalRelation::After
                } else if Self::any_match(&self.patterns.temporal_before, text) {
                    TemporalRelation::Before
                } else if Self::any_match(&self.patterns.temporal_during, text) {
                    TemporalRelation::During
                } else if Self::any_match(&self.patterns.temporal_between, text) {
                    TemporalRelation::Between
                } else {
                    // Use _temporal_at checks here if needed, or just default
                    TemporalRelation::At
                };

                return Some(TemporalBounds::new(Some(timestamp), None, relation));
            }
        }

        None
    }

    /// Extract causal relations from text
    fn extract_causal(&self, text: &str) -> Vec<CausalRelation> {
        let mut relations = Vec::new();

        if Self::any_match(&self.patterns.causal_direct, text) {
            relations.push(CausalRelation {
                confidence: 0.8,
                relation_type: CausalType::Direct,
                strength: 0.7,
            });
        }

        if Self::any_match(&self.patterns.causal_enabling, text) {
            relations.push(CausalRelation {
                confidence: 0.7,
                relation_type: CausalType::Enabling,
                strength: 0.5,
            });
        }

        if Self::any_match(&self.patterns.causal_preventing, text) {
            relations.push(CausalRelation {
                confidence: 0.75,
                relation_type: CausalType::Preventing,
                strength: 0.6,
            });
        }

        relations
    }

    /// Detect domain context
    fn detect_domain(&self, text: &str) -> DomainContext {
        let mut scores: HashMap<DomainContext, u32> = HashMap::new();

        // Check configured patterns
        for (domain_name, patterns) in &self.patterns.domains {
            let count = Self::count_matches(patterns, text);
            if count > 0 {
                let context = match domain_name.to_lowercase().as_str() {
                    "medical" => DomainContext::Medical,
                    "legal" => DomainContext::Legal,
                    "financial" => DomainContext::Financial,
                    "technical" => DomainContext::Technical,
                    "scientific" => DomainContext::Scientific,
                    "business" => DomainContext::Business,
                    _ => DomainContext::General,
                };
                *scores.entry(context).or_insert(0) += count as u32;
            }
        }

        // Return highest scoring domain, default to General
        scores
            .into_iter()
            .max_by_key(|&(_, count)| count)
            .map(|(domain, _)| domain)
            .unwrap_or(DomainContext::General)
    }

    /// Extract negation scope
    fn extract_negation(&self, text: &str) -> Option<NegationScope> {
        let negation_type = if Self::any_match(&self.patterns.negation_explicit, text) {
            NegationType::Explicit
        } else if Self::any_match(&self.patterns.negation_exception, text) {
            NegationType::Exception
        } else {
            return None;
        };

        Some(NegationScope {
            negated_concept_ids: Vec::new(), // Populated later during graph construction
            confidence: 0.8,
            negation_type,
        })
    }

    /// Calculate classification confidence
    fn calculate_confidence(&self, text: &str, semantic_type: SemanticType) -> f32 {
        let word_count = text.split_whitespace().count();

        // Base confidence on text length (more text = more context = higher confidence)
        let length_factor = (word_count as f32 / 50.0).min(1.0);

        // Boost confidence based on multiple pattern matches
        let pattern_matches = match semantic_type {
            SemanticType::Rule => {
                Self::count_matches(&self.patterns.rule_modal, text)
                    + Self::count_matches(&self.patterns.rule_conditional, text)
                    + Self::count_matches(&self.patterns.rule_imperative, text)
            }
            SemanticType::Temporal => {
                Self::count_matches(&self.patterns.temporal_after, text)
                    + Self::count_matches(&self.patterns.temporal_before, text)
                    + Self::count_matches(&self.patterns.temporal_during, text)
            }
            SemanticType::Negation => {
                Self::count_matches(&self.patterns.negation_explicit, text)
                    + Self::count_matches(&self.patterns.negation_exception, text)
            }
            SemanticType::Causal => {
                Self::count_matches(&self.patterns.causal_direct, text)
                    + Self::count_matches(&self.patterns.causal_enabling, text)
                    + Self::count_matches(&self.patterns.causal_preventing, text)
            }
            _ => 1,
        } as f32;

        let pattern_factor = (pattern_matches / 3.0).min(1.0);

        // Combine factors: base 0.5 + length boost + pattern boost
        0.5 + (length_factor * 0.25) + (pattern_factor * 0.25)
    }
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
