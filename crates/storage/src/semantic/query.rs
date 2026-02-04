/// Production-Grade Semantic Query System
///
/// Filters and query operations for semantic-aware graph traversal.
/// Zero overhead - all filters are inline predicates.
use super::types::*;
use crate::types::ConceptId;
use std::collections::HashSet;

/// Semantic query filter for graph traversal
#[derive(Debug, Clone)]
pub struct SemanticFilter {
    /// Required semantic type (None = any type)
    pub semantic_type: Option<SemanticType>,

    /// Required domain context (None = any domain)
    pub domain_context: Option<DomainContext>,

    /// Temporal constraint
    pub temporal_constraint: Option<TemporalConstraint>,

    /// Causal relationship filter
    pub causal_filter: Option<CausalFilter>,

    /// Negation type filter
    pub negation_type: Option<NegationType>,

    /// Minimum classification confidence (0.0 - 1.0)
    pub min_confidence: f32,

    /// Text content must contain these terms (case-insensitive)
    pub required_terms: Vec<String>,

    /// Exclude concepts with these IDs
    pub excluded_ids: HashSet<ConceptId>,
}

impl SemanticFilter {
    /// Create a new empty filter (matches everything)
    pub fn new() -> Self {
        Self {
            semantic_type: None,
            domain_context: None,
            temporal_constraint: None,
            causal_filter: None,
            negation_type: None,
            min_confidence: 0.0,
            required_terms: Vec::new(),
            excluded_ids: HashSet::new(),
        }
    }

    /// Builder: Filter by semantic type
    pub fn with_type(mut self, semantic_type: SemanticType) -> Self {
        self.semantic_type = Some(semantic_type);
        self
    }

    /// Builder: Filter by domain context
    pub fn with_domain(mut self, domain: DomainContext) -> Self {
        self.domain_context = Some(domain);
        self
    }

    /// Builder: Filter by temporal constraint
    pub fn with_temporal(mut self, constraint: TemporalConstraint) -> Self {
        self.temporal_constraint = Some(constraint);
        self
    }

    /// Builder: Filter by causal relationship
    pub fn with_causal(mut self, filter: CausalFilter) -> Self {
        self.causal_filter = Some(filter);
        self
    }

    /// Builder: Filter by negation type
    pub fn with_negation_type(mut self, negation_type: NegationType) -> Self {
        self.negation_type = Some(negation_type);
        self
    }

    /// Builder: Set minimum confidence
    pub fn with_min_confidence(mut self, confidence: f32) -> Self {
        self.min_confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Builder: Add required text term
    pub fn with_term(mut self, term: String) -> Self {
        self.required_terms.push(term.to_lowercase());
        self
    }

    /// Builder: Exclude concept by ID
    pub fn excluding(mut self, id: ConceptId) -> Self {
        self.excluded_ids.insert(id);
        self
    }

    /// Check if semantic metadata matches this filter
    pub fn matches(
        &self,
        metadata: &SemanticMetadata,
        content: &str,
        concept_id: &ConceptId,
    ) -> bool {
        // Check excluded IDs first (fast path)
        if self.excluded_ids.contains(concept_id) {
            return false;
        }

        // Check semantic type
        if let Some(required_type) = self.semantic_type {
            if metadata.semantic_type != required_type {
                return false;
            }
        }

        // Check domain context
        if let Some(required_domain) = self.domain_context {
            if metadata.domain_context != required_domain {
                return false;
            }
        }

        // Check temporal constraint
        if let Some(ref constraint) = self.temporal_constraint {
            if !constraint.matches(&metadata.temporal_bounds) {
                return false;
            }
        }

        // Check causal filter
        if let Some(ref causal_filter) = self.causal_filter {
            if !causal_filter.matches(&metadata.causal_relations) {
                return false;
            }
        }

        // Check negation type
        if let Some(required_negation) = self.negation_type {
            match &metadata.negation_scope {
                Some(scope) if scope.negation_type == required_negation => {}
                _ => return false,
            }
        }

        // Check confidence threshold
        if metadata.classification_confidence < self.min_confidence {
            return false;
        }

        // Check required terms
        if !self.required_terms.is_empty() {
            let content_lower = content.to_lowercase();
            for term in &self.required_terms {
                if !content_lower.contains(term) {
                    return false;
                }
            }
        }

        true
    }
}

impl Default for SemanticFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// Temporal constraint for filtering concepts
#[derive(Debug, Clone)]
pub enum TemporalConstraint {
    /// Concept must be valid at this timestamp
    ValidAt(i64),

    /// Concept must start after this timestamp
    After(i64),

    /// Concept must start before this timestamp
    Before(i64),

    /// Concept must overlap with this time range
    During { start: i64, end: i64 },

    /// Concept must have any temporal bounds
    HasTemporalBounds,

    /// Concept must NOT have temporal bounds
    NoTemporalBounds,
}

impl TemporalConstraint {
    pub fn matches(&self, bounds: &Option<TemporalBounds>) -> bool {
        match self {
            Self::ValidAt(timestamp) => bounds.as_ref().is_none_or(|b| b.contains(*timestamp)),
            Self::After(timestamp) => bounds
                .as_ref()
                .is_some_and(|b| b.start.is_some_and(|s| s > *timestamp)),
            Self::Before(timestamp) => bounds
                .as_ref()
                .is_some_and(|b| b.start.is_some_and(|s| s < *timestamp)),
            Self::During { start, end } => bounds.as_ref().is_some_and(|b| {
                b.overlaps(&TemporalBounds::new(
                    Some(*start),
                    Some(*end),
                    TemporalRelation::During,
                ))
            }),
            Self::HasTemporalBounds => bounds.is_some(),
            Self::NoTemporalBounds => bounds.is_none(),
        }
    }
}

/// Causal relationship filter
#[derive(Debug, Clone)]
pub enum CausalFilter {
    /// Must have any causal relationship
    HasCausalRelation,

    /// Must have specific causal relation type
    HasRelationType(CausalType),

    /// Must have causal relation with minimum confidence
    MinCausalConfidence(f32),

    /// Must have causal relation with minimum strength
    MinCausalStrength(f32),
}

impl CausalFilter {
    pub fn matches(&self, relations: &[CausalRelation]) -> bool {
        match self {
            Self::HasCausalRelation => !relations.is_empty(),
            Self::HasRelationType(causal_type) => {
                relations.iter().any(|r| r.relation_type == *causal_type)
            }
            Self::MinCausalConfidence(min_conf) => {
                relations.iter().any(|r| r.confidence >= *min_conf)
            }
            Self::MinCausalStrength(min_strength) => {
                relations.iter().any(|r| r.strength >= *min_strength)
            }
        }
    }
}

/// Semantic query builder for complex queries
#[derive(Debug, Clone)]
pub struct SemanticQuery {
    /// Primary filter for concepts
    pub filter: SemanticFilter,

    /// Limit number of results
    pub limit: Option<usize>,

    /// Sort order
    pub sort_by: SortOrder,
}

#[derive(Debug, Clone, Copy)]
pub enum SortOrder {
    /// Sort by classification confidence (descending)
    ByConfidence,

    /// Sort by temporal order (ascending)
    ByTemporalStart,

    /// Sort by creation time (newest first)
    ByCreationTime,

    /// No sorting (graph traversal order)
    None,
}

impl SemanticQuery {
    pub fn new() -> Self {
        Self {
            filter: SemanticFilter::new(),
            limit: None,
            sort_by: SortOrder::None,
        }
    }

    pub fn with_filter(mut self, filter: SemanticFilter) -> Self {
        self.filter = filter;
        self
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn with_sort(mut self, sort_by: SortOrder) -> Self {
        self.sort_by = sort_by;
        self
    }
}

impl Default for SemanticQuery {
    fn default() -> Self {
        Self::new()
    }
}

/// Quick semantic query builders for common patterns
pub mod queries {
    use super::*;

    /// Find all rules in a domain
    pub fn rules_in_domain(domain: DomainContext) -> SemanticFilter {
        SemanticFilter::new()
            .with_type(SemanticType::Rule)
            .with_domain(domain)
    }

    /// Find concepts added after a timestamp
    pub fn added_after(timestamp: i64) -> SemanticFilter {
        SemanticFilter::new().with_temporal(TemporalConstraint::After(timestamp))
    }

    /// Find causal relationships
    pub fn causal_relations() -> SemanticFilter {
        SemanticFilter::new()
            .with_type(SemanticType::Causal)
            .with_causal(CausalFilter::HasCausalRelation)
    }

    /// Find negations/exceptions
    pub fn negations() -> SemanticFilter {
        SemanticFilter::new().with_type(SemanticType::Negation)
    }

    /// Find temporal concepts
    pub fn temporal_concepts() -> SemanticFilter {
        SemanticFilter::new()
            .with_type(SemanticType::Temporal)
            .with_temporal(TemporalConstraint::HasTemporalBounds)
    }

    /// Find high-confidence rules
    pub fn high_confidence_rules(domain: DomainContext) -> SemanticFilter {
        SemanticFilter::new()
            .with_type(SemanticType::Rule)
            .with_domain(domain)
            .with_min_confidence(0.8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_filter_type_matching() {
        let filter = SemanticFilter::new().with_type(SemanticType::Rule);

        let mut metadata = SemanticMetadata::new(SemanticType::Rule);
        let content = "Test content";
        let id = ConceptId::from_bytes([0u8; 16]);

        assert!(filter.matches(&metadata, content, &id));

        metadata.semantic_type = SemanticType::Entity;
        assert!(!filter.matches(&metadata, content, &id));
    }

    #[test]
    fn test_temporal_constraint_after() {
        let constraint = TemporalConstraint::After(1000);

        let bounds_after = Some(TemporalBounds::new(
            Some(2000),
            None,
            TemporalRelation::After,
        ));
        let bounds_before = Some(TemporalBounds::new(
            Some(500),
            None,
            TemporalRelation::After,
        ));

        assert!(constraint.matches(&bounds_after));
        assert!(!constraint.matches(&bounds_before));
    }

    #[test]
    fn test_causal_filter() {
        let filter = CausalFilter::HasRelationType(CausalType::Direct);

        let relations = vec![CausalRelation {
            confidence: 0.8,
            relation_type: CausalType::Direct,
            strength: 0.7,
        }];

        assert!(filter.matches(&relations));

        let relations_indirect = vec![CausalRelation {
            confidence: 0.8,
            relation_type: CausalType::Indirect,
            strength: 0.7,
        }];

        assert!(!filter.matches(&relations_indirect));
    }

    #[test]
    fn test_quick_query_builders() {
        let filter = queries::rules_in_domain(DomainContext::Medical);
        assert!(filter.semantic_type == Some(SemanticType::Rule));
        assert!(filter.domain_context == Some(DomainContext::Medical));

        let filter = queries::causal_relations();
        assert!(filter.semantic_type == Some(SemanticType::Causal));
    }
}
