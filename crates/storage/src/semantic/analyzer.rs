/// Production-Grade Semantic Analyzer
/// 
/// Deterministic pattern-based semantic classification.
/// No ML models, no fallbacks - pure rule-based system.
use super::types::*;
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;

/// Compiled regex patterns for semantic classification
struct SemanticPatterns {
    // Temporal patterns
    temporal_after: Regex,
    temporal_before: Regex,
    temporal_during: Regex,
    temporal_between: Regex,
    #[allow(dead_code)]
    temporal_at: Regex,
    
    // Rule patterns
    rule_modal: Regex,
    rule_conditional: Regex,
    rule_imperative: Regex,
    
    // Negation patterns
    negation_explicit: Regex,
    negation_exception: Regex,
    
    // Causal patterns
    causal_direct: Regex,
    causal_enabling: Regex,
    causal_preventing: Regex,
    
    // Condition patterns
    condition_if: Regex,
    condition_when: Regex,
    condition_unless: Regex,
    
    // Quantitative patterns
    quantitative_number: Regex,
    quantitative_percentage: Regex,
    quantitative_measurement: Regex,
    
    // Definitional patterns
    definitional_is_a: Regex,
    definitional_defined_as: Regex,
    
    // Event patterns
    event_past: Regex,
    event_future: Regex,
    event_ongoing: Regex,
    
    // Domain patterns
    domain_medical: Regex,
    domain_legal: Regex,
    domain_financial: Regex,
    domain_technical: Regex,
    domain_scientific: Regex,
    domain_business: Regex,
}

static PATTERNS: Lazy<SemanticPatterns> = Lazy::new(|| {
    SemanticPatterns {
        // Temporal patterns
        temporal_after: Regex::new(r"(?i)\b(after|following|subsequent to|later than|post)\b").unwrap(),
        temporal_before: Regex::new(r"(?i)\b(before|prior to|preceding|earlier than|pre)\b").unwrap(),
        temporal_during: Regex::new(r"(?i)\b(during|throughout|while|in the course of)\b").unwrap(),
        temporal_between: Regex::new(r"(?i)\b(between)\b").unwrap(),
        temporal_at: Regex::new(r"(?i)\b(at|on|in)\s+\d{4}|\b(january|february|march|april|may|june|july|august|september|october|november|december)\b").unwrap(),
        
        // Rule patterns
        rule_modal: Regex::new(r"(?i)\b(must|shall|should|ought to|required|mandatory|obligatory)\b").unwrap(),
        rule_conditional: Regex::new(r"(?i)\b(if\s+\w+\s+then|when\s+\w+\s+must)\b").unwrap(),
        rule_imperative: Regex::new(r"(?i)^(do not|never|always|ensure|verify|confirm|check)\s+\w+").unwrap(),
        
        // Negation patterns
        negation_explicit: Regex::new(r"(?i)\b(not|no|never|none|nothing|neither|nor)\b").unwrap(),
        negation_exception: Regex::new(r"(?i)\b(except|unless|excluding|other than|but not|save for)\b").unwrap(),
        
        // Causal patterns
        causal_direct: Regex::new(r"(?i)\b(causes?|leads? to|results? in|triggers?|produces?|brings about)\b").unwrap(),
        causal_enabling: Regex::new(r"(?i)\b(enables?|allows?|permits?|facilitates?|makes? possible)\b").unwrap(),
        causal_preventing: Regex::new(r"(?i)\b(prevents?|stops?|blocks?|inhibits?|prohibits?)\b").unwrap(),
        
        // Condition patterns
        condition_if: Regex::new(r"(?i)\b(if|provided that|given that|assuming)\b").unwrap(),
        condition_when: Regex::new(r"(?i)\b(when|whenever|once|as soon as)\b").unwrap(),
        condition_unless: Regex::new(r"(?i)\b(unless|except if|only if)\b").unwrap(),
        
        // Quantitative patterns
        quantitative_number: Regex::new(r"\b\d+(\.\d+)?\s*(million|billion|thousand|hundred|dozen)?\b").unwrap(),
        quantitative_percentage: Regex::new(r"\b\d+(\.\d+)?%|\bpercent\b").unwrap(),
        quantitative_measurement: Regex::new(r"\b\d+(\.\d+)?\s*(kg|lb|meter|mile|liter|gallon|USD|EUR|GBP)\b").unwrap(),
        
        // Definitional patterns
        definitional_is_a: Regex::new(r"(?i)\b(is a|are|represents?|means?|refers? to)\b").unwrap(),
        definitional_defined_as: Regex::new(r"(?i)\b(defined as|definition of|classified as|categorized as)\b").unwrap(),
        
        // Event patterns
        event_past: Regex::new(r"(?i)\b(occurred|happened|took place|was|were)\b").unwrap(),
        event_future: Regex::new(r"(?i)\b(will occur|will happen|scheduled|planned)\b").unwrap(),
        event_ongoing: Regex::new(r"(?i)\b(is occurring|is happening|ongoing|in progress)\b").unwrap(),
        
        // Domain patterns
        domain_medical: Regex::new(r"(?i)\b(patient|diagnosis|treatment|symptom|disease|medical|clinical|hospital|doctor|nurse|therapy|medication|surgical)\b").unwrap(),
        domain_legal: Regex::new(r"(?i)\b(law|legal|court|statute|regulation|compliance|contract|liability|plaintiff|defendant|attorney|judge)\b").unwrap(),
        domain_financial: Regex::new(r"(?i)\b(financial|investment|revenue|profit|cost|budget|portfolio|asset|liability|equity|dividend|interest rate)\b").unwrap(),
        domain_technical: Regex::new(r"(?i)\b(system|software|hardware|algorithm|API|database|server|network|protocol|architecture|deployment)\b").unwrap(),
        domain_scientific: Regex::new(r"(?i)\b(experiment|hypothesis|research|study|analysis|data|measurement|observation|theory|methodology)\b").unwrap(),
        domain_business: Regex::new(r"(?i)\b(business|company|organization|management|strategy|operations|marketing|sales|customer|stakeholder)\b").unwrap(),
    }
});

/// Production semantic analyzer
pub struct SemanticAnalyzer {
    /// Custom domain patterns (user-provided)
    custom_patterns: HashMap<DomainContext, Vec<Regex>>,
}

impl SemanticAnalyzer {
    /// Create new semantic analyzer
    pub fn new() -> Self {
        Self {
            custom_patterns: HashMap::new(),
        }
    }
    
    /// Add custom domain pattern
    pub fn add_domain_pattern(&mut self, domain: DomainContext, pattern: Regex) {
        self.custom_patterns.entry(domain).or_default().push(pattern);
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
    
    /// Classify primary semantic type
    fn classify_type(&self, text: &str) -> SemanticType {
        let mut scores: HashMap<SemanticType, f32> = HashMap::new();
        
        // Rule patterns (highest priority for policy docs)
        if PATTERNS.rule_modal.is_match(text) {
            *scores.entry(SemanticType::Rule).or_insert(0.0) += 3.0;
        }
        if PATTERNS.rule_conditional.is_match(text) {
            *scores.entry(SemanticType::Rule).or_insert(0.0) += 2.5;
        }
        if PATTERNS.rule_imperative.is_match(text) {
            *scores.entry(SemanticType::Rule).or_insert(0.0) += 2.0;
        }
        
        // Temporal patterns
        if PATTERNS.temporal_after.is_match(text) || PATTERNS.temporal_before.is_match(text) {
            *scores.entry(SemanticType::Temporal).or_insert(0.0) += 2.0;
        }
        if PATTERNS.temporal_during.is_match(text) || PATTERNS.temporal_between.is_match(text) {
            *scores.entry(SemanticType::Temporal).or_insert(0.0) += 1.5;
        }
        
        // Negation patterns
        if PATTERNS.negation_explicit.is_match(text) {
            *scores.entry(SemanticType::Negation).or_insert(0.0) += 2.0;
        }
        if PATTERNS.negation_exception.is_match(text) {
            *scores.entry(SemanticType::Negation).or_insert(0.0) += 2.5;
        }
        
        // Causal patterns
        if PATTERNS.causal_direct.is_match(text) {
            *scores.entry(SemanticType::Causal).or_insert(0.0) += 2.5;
        }
        if PATTERNS.causal_enabling.is_match(text) || PATTERNS.causal_preventing.is_match(text) {
            *scores.entry(SemanticType::Causal).or_insert(0.0) += 2.0;
        }
        
        // Condition patterns
        if PATTERNS.condition_if.is_match(text) {
            *scores.entry(SemanticType::Condition).or_insert(0.0) += 2.0;
        }
        if PATTERNS.condition_when.is_match(text) || PATTERNS.condition_unless.is_match(text) {
            *scores.entry(SemanticType::Condition).or_insert(0.0) += 1.5;
        }
        
        // Quantitative patterns
        if PATTERNS.quantitative_number.is_match(text) || PATTERNS.quantitative_percentage.is_match(text) {
            *scores.entry(SemanticType::Quantitative).or_insert(0.0) += 1.0;
        }
        if PATTERNS.quantitative_measurement.is_match(text) {
            *scores.entry(SemanticType::Quantitative).or_insert(0.0) += 1.5;
        }
        
        // Definitional patterns
        if PATTERNS.definitional_is_a.is_match(text) {
            *scores.entry(SemanticType::Definitional).or_insert(0.0) += 1.5;
        }
        if PATTERNS.definitional_defined_as.is_match(text) {
            *scores.entry(SemanticType::Definitional).or_insert(0.0) += 2.0;
        }
        
        // Event patterns
        if PATTERNS.event_past.is_match(text) || PATTERNS.event_future.is_match(text) {
            *scores.entry(SemanticType::Event).or_insert(0.0) += 1.5;
        }
        if PATTERNS.event_ongoing.is_match(text) {
            *scores.entry(SemanticType::Event).or_insert(0.0) += 1.0;
        }
        
        // Return highest scoring type, default to Entity
        scores.into_iter()
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
                let relation = if PATTERNS.temporal_after.is_match(text) {
                    TemporalRelation::After
                } else if PATTERNS.temporal_before.is_match(text) {
                    TemporalRelation::Before
                } else if PATTERNS.temporal_during.is_match(text) {
                    TemporalRelation::During
                } else if PATTERNS.temporal_between.is_match(text) {
                    TemporalRelation::Between
                } else {
                    TemporalRelation::At
                };
                
                return Some(TemporalBounds::new(
                    Some(timestamp),
                    None,
                    relation
                ));
            }
        }
        
        None
    }
    
    /// Extract causal relations from text
    fn extract_causal(&self, text: &str) -> Vec<CausalRelation> {
        let mut relations = Vec::new();
        
        if PATTERNS.causal_direct.is_match(text) {
            relations.push(CausalRelation {
                confidence: 0.8,
                relation_type: CausalType::Direct,
                strength: 0.7,
            });
        }
        
        if PATTERNS.causal_enabling.is_match(text) {
            relations.push(CausalRelation {
                confidence: 0.7,
                relation_type: CausalType::Enabling,
                strength: 0.5,
            });
        }
        
        if PATTERNS.causal_preventing.is_match(text) {
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
        
        // Check built-in patterns
        if PATTERNS.domain_medical.is_match(text) {
            *scores.entry(DomainContext::Medical).or_insert(0) += PATTERNS.domain_medical.find_iter(text).count() as u32;
        }
        if PATTERNS.domain_legal.is_match(text) {
            *scores.entry(DomainContext::Legal).or_insert(0) += PATTERNS.domain_legal.find_iter(text).count() as u32;
        }
        if PATTERNS.domain_financial.is_match(text) {
            *scores.entry(DomainContext::Financial).or_insert(0) += PATTERNS.domain_financial.find_iter(text).count() as u32;
        }
        if PATTERNS.domain_technical.is_match(text) {
            *scores.entry(DomainContext::Technical).or_insert(0) += PATTERNS.domain_technical.find_iter(text).count() as u32;
        }
        if PATTERNS.domain_scientific.is_match(text) {
            *scores.entry(DomainContext::Scientific).or_insert(0) += PATTERNS.domain_scientific.find_iter(text).count() as u32;
        }
        if PATTERNS.domain_business.is_match(text) {
            *scores.entry(DomainContext::Business).or_insert(0) += PATTERNS.domain_business.find_iter(text).count() as u32;
        }
        
        // Check custom patterns
        for (domain, patterns) in &self.custom_patterns {
            for pattern in patterns {
                if pattern.is_match(text) {
                    *scores.entry(*domain).or_insert(0) += pattern.find_iter(text).count() as u32;
                }
            }
        }
        
        // Return highest scoring domain, default to General
        scores.into_iter()
            .max_by_key(|&(_, count)| count)
            .map(|(domain, _)| domain)
            .unwrap_or(DomainContext::General)
    }
    
    /// Extract negation scope
    fn extract_negation(&self, text: &str) -> Option<NegationScope> {
        let negation_type = if PATTERNS.negation_explicit.is_match(text) {
            NegationType::Explicit
        } else if PATTERNS.negation_exception.is_match(text) {
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
                PATTERNS.rule_modal.find_iter(text).count() +
                PATTERNS.rule_conditional.find_iter(text).count() +
                PATTERNS.rule_imperative.find_iter(text).count()
            },
            SemanticType::Temporal => {
                PATTERNS.temporal_after.find_iter(text).count() +
                PATTERNS.temporal_before.find_iter(text).count() +
                PATTERNS.temporal_during.find_iter(text).count()
            },
            SemanticType::Negation => {
                PATTERNS.negation_explicit.find_iter(text).count() +
                PATTERNS.negation_exception.find_iter(text).count()
            },
            SemanticType::Causal => {
                PATTERNS.causal_direct.find_iter(text).count() +
                PATTERNS.causal_enabling.find_iter(text).count() +
                PATTERNS.causal_preventing.find_iter(text).count()
            },
            _ => 1,
        };
        
        let pattern_factor = (pattern_matches as f32 / 3.0).min(1.0);
        
        // Combine factors: base 0.5 + length boost + pattern boost
        0.5 + (length_factor * 0.25) + (pattern_factor * 0.25)
    }
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_classify_rule() {
        let analyzer = SemanticAnalyzer::new();
        let metadata = analyzer.analyze("Patients must complete the consent form before treatment.");
        assert_eq!(metadata.semantic_type, SemanticType::Rule);
        assert_eq!(metadata.domain_context, DomainContext::Medical);
    }
    
    #[test]
    fn test_classify_temporal() {
        let analyzer = SemanticAnalyzer::new();
        let metadata = analyzer.analyze("This policy became effective after 2024.");
        assert_eq!(metadata.semantic_type, SemanticType::Temporal);
        assert!(metadata.temporal_bounds.is_some());
    }
    
    #[test]
    fn test_classify_negation() {
        let analyzer = SemanticAnalyzer::new();
        let metadata = analyzer.analyze("Do not proceed unless authorized.");
        assert_eq!(metadata.semantic_type, SemanticType::Negation);
    }
    
    #[test]
    fn test_classify_causal() {
        let analyzer = SemanticAnalyzer::new();
        let metadata = analyzer.analyze("High blood pressure causes cardiovascular disease.");
        assert_eq!(metadata.semantic_type, SemanticType::Causal);
        assert!(!metadata.causal_relations.is_empty());
    }
    
    #[test]
    fn test_domain_detection() {
        let analyzer = SemanticAnalyzer::new();
        
        let medical = analyzer.analyze("The patient requires immediate surgical treatment.");
        assert_eq!(medical.domain_context, DomainContext::Medical);
        
        let legal = analyzer.analyze("The contract specifies liability in case of breach.");
        assert_eq!(legal.domain_context, DomainContext::Legal);
        
        let financial = analyzer.analyze("Portfolio returns exceeded budget expectations.");
        assert_eq!(financial.domain_context, DomainContext::Financial);
    }
}
