/// Semantic Understanding Module
/// 
/// Production-grade semantic analysis built into storage layer.
/// Provides domain-aware type classification, temporal reasoning,
/// causal analysis, and negation detection.
pub mod types;
pub mod analyzer;
pub mod query;
pub mod pathfinding;

pub use types::{
    SemanticType, SemanticMetadata, TemporalBounds, TemporalRelation,
    CausalRelation, CausalType, DomainContext, NegationScope, NegationType,
};
pub use analyzer::SemanticAnalyzer;
pub use query::{
    SemanticFilter, SemanticQuery, TemporalConstraint, CausalFilter, SortOrder,
    queries,
};
pub use pathfinding::{SemanticPathFinder, SemanticPath};
