mod index;
mod manifest;
mod quantization;
mod segment;
/// Sutra Storage - Next-Generation Knowledge Graph Storage
///
/// A custom storage engine designed specifically for temporal,
/// continuously-learning knowledge graphs. Not a database.
///
/// Key Features:
/// - Log-structured append-only storage
/// - Memory-mapped zero-copy access  
/// - Lock-free concurrent reads
/// - Temporal decay and evolution
/// - Native vector storage with quantization
mod types;
mod vectors;
mod wal;

// 🔥 NEW: Semantic understanding module
pub mod semantic;

// Unified learning pipeline modules
pub mod embedding_client;
pub mod embedding_provider;
pub mod inference; // 🔥 NEW: Local inference module
pub mod learning_pipeline;
pub mod nl_parser; // 🔥 NEW: NL Command Parser
pub mod semantic_extractor;

// New concurrent memory modules
mod adaptive_reconciler; // AI-native adaptive reconciliation
mod concurrent_memory;
mod mmap_store;
mod parallel_paths;
mod read_view;
mod write_log;

// Scalability modules
mod hnsw_container;
mod namespace_manager;
mod sharded_storage;
mod storage_trait;
mod transaction; // 🔥 NEW: 2PC transaction coordinator for cross-shard atomicity

// Autonomy engine
pub mod autonomy;

// Security and authentication
pub mod auth;
mod rate_limiter;
pub mod tls;

// Re-export rate limiter for auth module (internal use)
pub use rate_limiter::{RateLimitError, RateLimiter, RateLimiterConfig, RateLimiterStats};

// TCP server for distributed architecture
pub mod secure_tcp_server;
pub mod tcp_server;

pub use types::{
    AssociationId, AssociationRecord, AssociationType, ConceptId, ConceptRecord, GraphPath,
    SegmentHeader,
};

pub use index::{ConceptLocation, GraphIndex, IndexStats};
pub use manifest::{Manifest, SegmentMetadata};
pub use quantization::ProductQuantizer;
pub use segment::{ConceptIterator, Segment, SegmentStats};
pub use vectors::{VectorConfig, VectorMetadata, VectorStats, VectorStore};
pub use wal::{LogEntry, Operation, WriteAheadLog};

// New concurrent memory exports
pub use adaptive_reconciler::{
    AdaptiveReconciler, AdaptiveReconcilerConfig, AdaptiveReconcilerStats,
};
pub use concurrent_memory::{
    ConcurrentConfig, ConcurrentMemory, ConcurrentStats, HnswStats, SnapshotInfo,
};
pub use mmap_store::{MmapStats, MmapStore};
pub use parallel_paths::{ParallelPathFinder, PathResult};
pub use read_view::{ConceptNode, GraphSnapshot, ReadView};
pub use write_log::{WriteEntry, WriteLog, WriteLogError, WriteLogStats};

// Scalability exports
pub use hnsw_container::{HnswConfig, HnswContainer, HnswContainerStats};
pub use namespace_manager::NamespaceManager;
pub use sharded_storage::{AggregatedStats, ShardConfig, ShardMap, ShardStats, ShardedStorage};
pub use storage_trait::LearningStorage;
pub use transaction::{
    Transaction, TransactionCoordinator, TxnCoordinatorStats, TxnError, TxnOperation, TxnState,
}; // 🔥 NEW

// Autonomy exports
pub use autonomy::{AutonomyConfig, AutonomyManager};

/// Version of the storage format
pub const STORAGE_VERSION: u32 = 3;

/// Magic bytes for segment files
pub const MAGIC_BYTES: &[u8; 8] = b"SUTRASEG";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(STORAGE_VERSION, 3);
    }
}
