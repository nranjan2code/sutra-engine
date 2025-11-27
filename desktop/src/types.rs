//! Core types for Sutra Desktop
//!
//! Shared data structures used across all UI components.
//! These types wrap storage engine types for UI presentation.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use chrono::{DateTime, Local};
use sutra_storage::ConceptId;

// ============================================================================
// Application Messages (Async)
// ============================================================================

#[derive(Debug)]
pub enum AppMessage {
    LearnResult {
        content: String,
        concept_id: Option<ConceptId>,
        error: Option<String>,
        duration_ms: u64,
    },
    SearchResults {
        query: String,
        results: Vec<(ConceptId, String, f32)>, // id, content, confidence
        duration_ms: u64,
        generated_response: Option<String>,
    },
    KnowledgeLoaded {
        concepts: Vec<ConceptInfo>,
    },
    QuickLearnCompleted {
        content: String,
        success: bool,
        error: Option<String>,
        elapsed_ms: u64,
    },
    BatchLearnCompleted {
        total: usize,
        successes: usize,
        failures: Vec<(String, String)>, // (content, error)
        elapsed_ms: u64,
    },
    GraphLoaded {
        nodes: HashMap<ConceptId, GraphNode>,
        edges: Vec<GraphEdge>,
    },
    ReasoningPathsFound {
        paths: Vec<ReasoningPath>,
        consensus: Option<ConsensusResult>,
        error: Option<String>,
    },
    CausalAnalysisComplete {
        chains: Vec<CausalChain>,
        root_causes: Vec<CausalNode>,
        error: Option<String>,
    },
    QueryBuilderResults {
        results: Vec<ConceptInfo>,
        duration_ms: u64,
    },
    BatchImportProgress {
        completed: usize,
        total: usize,
        errors: usize,
    },
    BatchImportComplete {
        completed: usize,
        errors: usize,
        error: Option<String>,
    },
    TemporalEventsLoaded {
        events: Vec<TimelineEvent>,
    },
    ConceptDeleted {
        concept_id: String,
        success: bool,
        error: Option<String>,
    },
}

/// Concept info for UI display
#[derive(Debug, Clone)]
pub struct ConceptInfo {
    pub id: String,
    pub content: String,
    pub strength: f32,
    pub confidence: f32,
    pub neighbors: Vec<String>,
}

// ============================================================================
// Graph Visualization Types
// ============================================================================

/// Node in the graph visualization
#[derive(Debug, Clone)]
pub struct GraphNode {
    pub id: ConceptId,
    pub label: String,
    pub content: String,
    pub confidence: f32,
    pub strength: f32,
    pub neighbor_count: usize,
    
    // Layout position (updated by force-directed algorithm)
    pub x: f32,
    pub y: f32,
    pub vx: f32,  // velocity x
    pub vy: f32,  // velocity y
}

impl GraphNode {
    pub fn new(id: ConceptId, content: String, confidence: f32, strength: f32) -> Self {
        // Generate initial random position
        let hash = id.to_hex();
        let x = hash_to_float(&hash[0..8]) * 800.0 - 400.0;
        let y = hash_to_float(&hash[8..16]) * 600.0 - 300.0;
        
        let label = if content.len() > 40 {
            format!("{}...", &content[..40])
        } else {
            content.clone()
        };
        
        Self {
            id,
            label,
            content,
            confidence,
            strength,
            neighbor_count: 0,
            x,
            y,
            vx: 0.0,
            vy: 0.0,
        }
    }
}

/// Edge in the graph visualization
#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub from: ConceptId,
    pub to: ConceptId,
    pub strength: f32,
    pub edge_type: EdgeType,
}

/// Types of edges in the knowledge graph
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeType {
    Semantic,    // General semantic relationship
    Causal,      // "causes", "leads to"
    Temporal,    // "before", "after"
    Hierarchical,// "is a", "part of"
    Similar,     // Similar content
}

impl EdgeType {
    pub fn color(&self) -> (u8, u8, u8) {
        match self {
            EdgeType::Semantic => (167, 139, 250),    // Purple (PRIMARY)
            EdgeType::Causal => (248, 113, 113),      // Red (ERROR)
            EdgeType::Temporal => (96, 165, 250),     // Blue (SECONDARY)
            EdgeType::Hierarchical => (52, 211, 153), // Green (SUCCESS)
            EdgeType::Similar => (251, 191, 36),      // Amber (ACCENT)
        }
    }
    
    pub fn name(&self) -> &'static str {
        match self {
            EdgeType::Semantic => "Semantic",
            EdgeType::Causal => "Causal",
            EdgeType::Temporal => "Temporal",
            EdgeType::Hierarchical => "Hierarchical",
            EdgeType::Similar => "Similar",
        }
    }
}

// ============================================================================
// Reasoning Path Types (MPPA)
// ============================================================================

/// A single reasoning path from source to target
#[derive(Debug, Clone)]
pub struct ReasoningPath {
    pub path: Vec<PathStep>,
    pub confidence: f32,
    pub depth: usize,
}

impl ReasoningPath {
    /// Calculate overall confidence with decay
    pub fn calculate_confidence(steps: &[PathStep], decay: f32) -> f32 {
        if steps.is_empty() {
            return 1.0;
        }
        
        let mut conf = 1.0;
        for (i, step) in steps.iter().enumerate() {
            conf *= step.confidence * decay.powi(i as i32);
        }
        conf
    }
}

/// A step in a reasoning path
#[derive(Debug, Clone)]
pub struct PathStep {
    pub concept_id: ConceptId,
    pub content: String,
    pub confidence: f32,
    pub relation: String,  // Relation from previous step
}

/// Result of MPPA consensus analysis
#[derive(Debug, Clone)]
pub struct ConsensusResult {
    pub primary_cluster: PathCluster,
    pub alternatives: Vec<PathCluster>,
    pub total_paths: usize,
    pub explanation: String,
}

/// Cluster of paths leading to the same conclusion
#[derive(Debug, Clone)]
pub struct PathCluster {
    pub destination: ConceptId,
    pub destination_content: String,
    pub paths: Vec<ReasoningPath>,
    pub avg_confidence: f32,
    pub consensus_weight: f32,
    pub support_ratio: f32,  // paths in cluster / total paths
}

// ============================================================================
// Temporal Analysis Types
// ============================================================================

/// Event in a temporal timeline
#[derive(Debug, Clone)]
pub struct TimelineEvent {
    pub concept_id: String,
    pub label: String,
    pub description: String,
    pub timestamp: String,
    pub relative_time: i32,  // Relative position (-3, -2, -1, 0, 1, 2, 3)
    pub confidence: f32,
}

/// Time range filter for temporal views
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimeRange {
    #[default]
    AllTime,
    Today,
    Week,
    Month,
    Year,
}

/// Temporal relation between events
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemporalRelation {
    Before,
    After,
    During,
    Concurrent,
}

/// Causal relationship between events
#[derive(Debug, Clone)]
pub struct CausalRelation {
    pub from: ConceptId,
    pub to: ConceptId,
    pub relation_type: String,  // "causes", "leads to", "triggers"
    pub confidence: f32,
}

// ============================================================================
// Causal Analysis Types
// ============================================================================

/// A chain of causal relationships
#[derive(Debug, Clone)]
pub struct CausalChain {
    pub nodes: Vec<CausalNode>,
    pub confidence: f32,
    pub depth: usize,
}

/// A node in a causal chain
#[derive(Debug, Clone)]
pub struct CausalNode {
    pub id: String,
    pub label: String,
    pub content: String,
    pub confidence: f32,
    pub is_root_cause: bool,
    pub relation_type: CausalRelationType,
}

/// Type of causal relationship
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CausalRelationType {
    #[default]
    DirectCause,
    IndirectCause,
    Contributing,
    Correlation,
}

// ============================================================================
// Analytics Types
// ============================================================================

/// Metrics for the analytics dashboard
#[derive(Debug, Clone, Default)]
pub struct AnalyticsMetrics {
    pub total_concepts: usize,
    pub total_edges: usize,
    pub storage_size_bytes: u64,
    pub hnsw_indexed: usize,
    pub hnsw_coverage: f32,
    
    // Query performance
    pub queries_today: usize,
    pub avg_query_latency_ms: f32,
    pub p95_query_latency_ms: f32,
    pub p99_query_latency_ms: f32,
    
    // Learning activity
    pub concepts_today: usize,
    pub concepts_this_hour: usize,
}

/// Historical data point for charts
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub timestamp: Instant,
    pub concept_count: usize,
    pub query_latency_ms: f32,
}

/// Log entry for recent activity
#[derive(Debug, Clone)]
pub struct ActivityEntry {
    pub timestamp: DateTime<Local>,
    pub activity_type: ActivityType,
    pub description: String,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityType {
    Learn,
    Query,
    Search,
    Export,
    Import,
}

/// Query log for analytics
#[derive(Debug, Clone)]
pub struct QueryLogEntry {
    pub timestamp: Instant,
    pub query: String,
    pub duration_ms: u64,
    pub results_count: usize,
}

// ============================================================================
// Query Builder Types
// ============================================================================

/// Query type selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QueryType {
    #[default]
    TextSearch,
    SemanticSearch,
    PathFinding,
}

/// Query filters configuration
#[derive(Debug, Clone, Default)]
pub struct QueryFilters {
    pub min_confidence: f32,
    pub max_results: usize,
    pub ef_search: usize,  // For HNSW
    pub has_causal: bool,
    pub has_temporal: bool,
    pub min_neighbors: usize,
    pub created_after: Option<DateTime<Local>>,
}

impl QueryFilters {
    pub fn new() -> Self {
        Self {
            min_confidence: 0.0,
            max_results: 10,
            ef_search: 50,
            has_causal: false,
            has_temporal: false,
            min_neighbors: 0,
            created_after: None,
        }
    }
}

/// Saved query preset
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SavedQuery {
    pub name: String,
    pub query_text: String,
    pub query_type: String,
    pub min_confidence: f32,
    pub max_results: usize,
}

// ============================================================================
// Export/Import Types
// ============================================================================

/// Export format selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExportFormat {
    #[default]
    Json,
    Csv,
    GraphML,
    Cypher,
}

impl ExportFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Json => "json",
            ExportFormat::Csv => "csv",
            ExportFormat::GraphML => "graphml",
            ExportFormat::Cypher => "cypher",
        }
    }
    
    pub fn name(&self) -> &'static str {
        match self {
            ExportFormat::Json => "JSON",
            ExportFormat::Csv => "CSV",
            ExportFormat::GraphML => "GraphML",
            ExportFormat::Cypher => "Cypher",
        }
    }
}

/// Export options configuration
#[derive(Debug, Clone, Default)]
pub struct ExportOptions {
    pub include_vectors: bool,
    pub include_metadata: bool,
    pub min_confidence: f32,
    pub filter_query: String,
}

/// Import mode selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ImportMode {
    #[default]
    Merge,
    Overwrite,
    NewWorkspace,
}

/// Import preview data
#[derive(Debug, Clone, Default)]
pub struct ImportPreview {
    pub concepts_count: usize,
    pub edges_count: usize,
    pub duplicates_count: usize,
    pub new_count: usize,
    pub sample_concepts: Vec<String>,
}

/// Progress state for batch operations
#[derive(Debug, Clone, Default)]
pub struct BatchProgress {
    pub total: usize,
    pub completed: usize,
    pub errors: usize,
    pub is_running: bool,
}

impl BatchProgress {
    pub fn percent(&self) -> f32 {
        if self.total == 0 {
            0.0
        } else {
            (self.completed as f32 / self.total as f32) * 100.0
        }
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Convert a hex string to a float between 0 and 1
fn hash_to_float(hex: &str) -> f32 {
    let val = u32::from_str_radix(hex, 16).unwrap_or(0);
    (val as f32) / (u32::MAX as f32)
}

/// Calculate percentile from a sorted list
pub fn calculate_percentile(values: &[f32], p: f32) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    let idx = ((values.len() as f32) * p) as usize;
    values.get(idx.min(values.len() - 1)).copied().unwrap_or(0.0)
}
