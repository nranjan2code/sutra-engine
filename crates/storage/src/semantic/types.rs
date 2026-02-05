/// Production-Grade Semantic Type System
///
/// Domain-aware semantic understanding built into storage layer.
/// No fallbacks, no compromises - deterministic semantic classification.
use serde::{Deserialize, Serialize};
use std::fmt;

/// Core semantic types for domain understanding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum SemanticType {
    /// Named entities (people, places, organizations, concepts)
    Entity = 0,

    /// Events that occur at a point in time
    Event = 1,

    /// Rules, policies, regulations (if-then, must, shall)
    Rule = 2,

    /// Temporal expressions (after, before, during, between)
    Temporal = 3,

    /// Negations and exceptions (not, except, unless)
    Negation = 4,

    /// Conditions and constraints (when, if, only if)
    Condition = 5,

    /// Causal relationships (because, causes, leads to)
    Causal = 6,

    /// Quantities and measurements (amount, count, percentage)
    Quantitative = 7,

    /// Definitions and classifications (is a, type of, defined as)
    Definitional = 8,

    /// Goals and objectives (track, achieve, monitor)
    Goal = 9,
}

impl SemanticType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Entity),
            1 => Some(Self::Event),
            2 => Some(Self::Rule),
            3 => Some(Self::Temporal),
            4 => Some(Self::Negation),
            5 => Some(Self::Condition),
            6 => Some(Self::Causal),
            7 => Some(Self::Quantitative),
            8 => Some(Self::Definitional),
            9 => Some(Self::Goal),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Entity => "entity",
            Self::Event => "event",
            Self::Rule => "rule",
            Self::Temporal => "temporal",
            Self::Negation => "negation",
            Self::Condition => "condition",
            Self::Causal => "causal",
            Self::Quantitative => "quantitative",
            Self::Definitional => "definitional",
            Self::Goal => "goal",
        }
    }
}

impl fmt::Display for SemanticType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Temporal bounds for concepts
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TemporalBounds {
    /// Start timestamp (Unix epoch seconds), None = unbounded start
    pub start: Option<i64>,

    /// End timestamp (Unix epoch seconds), None = unbounded end
    pub end: Option<i64>,

    /// Temporal relation type
    pub relation: TemporalRelation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum TemporalRelation {
    /// Occurs at a specific point
    At = 0,

    /// Occurs after a point
    After = 1,

    /// Occurs before a point
    Before = 2,

    /// Occurs during a range
    During = 3,

    /// Occurs between two points
    Between = 4,
}

impl TemporalBounds {
    pub fn new(start: Option<i64>, end: Option<i64>, relation: TemporalRelation) -> Self {
        Self {
            start,
            end,
            relation,
        }
    }

    /// Check if this temporal bound contains a given timestamp
    pub fn contains(&self, timestamp: i64) -> bool {
        match (self.start, self.end) {
            (Some(start), Some(end)) => timestamp >= start && timestamp <= end,
            (Some(start), None) => timestamp >= start,
            (None, Some(end)) => timestamp <= end,
            (None, None) => true, // Unbounded
        }
    }

    /// Check if this temporal bound overlaps with another
    pub fn overlaps(&self, other: &TemporalBounds) -> bool {
        match (self.start, self.end, other.start, other.end) {
            (Some(s1), Some(e1), Some(s2), Some(e2)) => {
                // Two bounded ranges overlap if start1 <= end2 && start2 <= end1
                s1 <= e2 && s2 <= e1
            }
            _ => true, // If either is unbounded, consider them overlapping
        }
    }
}

/// Causal relationship metadata
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CausalRelation {
    /// Confidence in causal relationship (0.0 - 1.0)
    pub confidence: f32,

    /// Type of causal relationship
    pub relation_type: CausalType,

    /// Strength of causal effect (0.0 - 1.0)
    pub strength: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum CausalType {
    /// Direct causation (A causes B)
    Direct = 0,

    /// Indirect causation (A causes B via intermediate)
    Indirect = 1,

    /// Enabling condition (A enables B but doesn't cause it)
    Enabling = 2,

    /// Preventing condition (A prevents B)
    Preventing = 3,

    /// Correlation (A and B co-occur but causation unclear)
    Correlation = 4,
}

/// Domain context for concepts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum DomainContext {
    /// Medical/healthcare domain
    Medical = 0,

    /// Legal domain
    Legal = 1,

    /// Financial domain
    Financial = 2,

    /// Technical/engineering domain
    Technical = 3,

    /// Scientific domain
    Scientific = 4,

    /// Business/organizational domain
    Business = 5,

    /// General/unspecified domain
    General = 6,
}

impl DomainContext {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Medical),
            1 => Some(Self::Legal),
            2 => Some(Self::Financial),
            3 => Some(Self::Technical),
            4 => Some(Self::Scientific),
            5 => Some(Self::Business),
            6 => Some(Self::General),
            _ => None,
        }
    }
}

/// Negation scope - tracks what concepts this negates
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NegationScope {
    /// Concepts that are negated by this concept
    pub negated_concept_ids: Vec<[u8; 16]>, // ConceptId bytes

    /// Confidence in negation (0.0 - 1.0)
    pub confidence: f32,

    /// Type of negation
    pub negation_type: NegationType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum NegationType {
    /// Explicit negation (not, no, never)
    Explicit = 0,

    /// Exception (except, unless, excluding)
    Exception = 1,

    /// Contradiction (conflicts with)
    Contradiction = 2,
}

/// Complete semantic metadata for a concept
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticMetadata {
    /// Primary semantic type
    pub semantic_type: SemanticType,

    /// Temporal bounds (if applicable)
    pub temporal_bounds: Option<TemporalBounds>,

    /// Causal relationships (if applicable)
    pub causal_relations: Vec<CausalRelation>,

    /// Domain context
    pub domain_context: DomainContext,

    /// Negation scope (if this is a negation)
    pub negation_scope: Option<NegationScope>,

    /// Confidence in semantic classification (0.0 - 1.0)
    pub classification_confidence: f32,
}

impl SemanticMetadata {
    /// Create new semantic metadata with defaults
    pub fn new(semantic_type: SemanticType) -> Self {
        Self {
            semantic_type,
            temporal_bounds: None,
            causal_relations: Vec::new(),
            domain_context: DomainContext::General,
            negation_scope: None,
            classification_confidence: 1.0,
        }
    }

    /// Check if this concept is valid at a given timestamp
    pub fn is_valid_at(&self, timestamp: i64) -> bool {
        self.temporal_bounds
            .as_ref()
            .is_none_or(|bounds| bounds.contains(timestamp))
    }

    /// Check if this concept conflicts with another
    pub fn conflicts_with(&self, other: &SemanticMetadata) -> bool {
        // Check negation scope
        if let Some(ref negation) = self.negation_scope {
            if negation.negation_type == NegationType::Contradiction {
                return true;
            }
        }

        // Check temporal overlap for rules in same domain
        if self.semantic_type == SemanticType::Rule
            && other.semantic_type == SemanticType::Rule
            && self.domain_context == other.domain_context
        {
            if let (Some(ref t1), Some(ref t2)) = (&self.temporal_bounds, &other.temporal_bounds) {
                return t1.overlaps(t2);
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temporal_bounds_contains() {
        let bounds = TemporalBounds::new(Some(1000), Some(2000), TemporalRelation::During);

        assert!(bounds.contains(1500));
        assert!(!bounds.contains(500));
        assert!(!bounds.contains(2500));
    }

    #[test]
    fn test_temporal_bounds_overlaps() {
        let bounds1 = TemporalBounds::new(Some(1000), Some(2000), TemporalRelation::During);
        let bounds2 = TemporalBounds::new(Some(1500), Some(2500), TemporalRelation::During);
        let bounds3 = TemporalBounds::new(Some(3000), Some(4000), TemporalRelation::During);

        assert!(bounds1.overlaps(&bounds2));
        assert!(!bounds1.overlaps(&bounds3));
    }

    #[test]
    fn test_semantic_metadata_valid_at() {
        let mut metadata = SemanticMetadata::new(SemanticType::Rule);
        metadata.temporal_bounds = Some(TemporalBounds::new(
            Some(1000),
            Some(2000),
            TemporalRelation::During,
        ));

        assert!(metadata.is_valid_at(1500));
        assert!(!metadata.is_valid_at(500));
    }
}
