pub mod analyzer;
pub mod pathfinding;
pub mod query;
/// Semantic Understanding Module
///
/// Production-grade semantic analysis built into storage layer.
/// Provides domain-aware type classification, temporal reasoning,
/// causal analysis, and negation detection.
pub mod types;

pub use analyzer::SemanticAnalyzer;
pub use pathfinding::{SemanticPath, SemanticPathFinder};
pub use query::{
    queries, CausalFilter, SemanticFilter, SemanticQuery, SortOrder, TemporalConstraint,
};
pub use types::{
    CausalRelation, CausalType, DomainContext, NegationScope, NegationType, SemanticMetadata,
    SemanticType, TemporalBounds, TemporalRelation,
};
