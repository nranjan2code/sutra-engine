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
mod segment;
mod manifest;
mod lsm;
mod store;
mod index;
mod wal;
mod quantization;
mod vectors;
mod reasoning_store;

// 🔥 NEW: Schema definitions for conversation-first UI
pub mod schema;

// 🔥 NEW: Semantic understanding module
pub mod semantic;

// Unified learning pipeline modules
pub mod embedding_provider;
pub mod embedding_client;
pub mod semantic_extractor;
pub mod learning_pipeline;

// New concurrent memory modules
mod write_log;
mod read_view;
mod adaptive_reconciler; // AI-native adaptive reconciliation
mod concurrent_memory;
mod mmap_store;
mod parallel_paths;

// Scalability modules
mod hnsw_container;
mod sharded_storage;
mod storage_trait;
mod transaction; // 🔥 NEW: 2PC transaction coordinator for cross-shard atomicity

// Security and authentication
pub mod auth;
pub mod tls;
mod rate_limiter;

// Re-export rate limiter for auth module (internal use)
pub use rate_limiter::{RateLimiter, RateLimiterConfig, RateLimiterStats, RateLimitError};

// TCP server for distributed architecture
pub mod tcp_server;
pub mod secure_tcp_server;

pub use types::{
    ConceptId, AssociationId, AssociationType, ConceptRecord, AssociationRecord,
    SegmentHeader, GraphPath,
};

pub use segment::{Segment, SegmentStats, ConceptIterator};
pub use manifest::{Manifest, SegmentMetadata};
pub use lsm::{LSMTree, LSMStats, CompactionConfig};
pub use store::GraphStore;
pub use index::{GraphIndex, IndexStats, ConceptLocation};
pub use wal::{WriteAheadLog, LogEntry, Operation};
pub use quantization::ProductQuantizer;
pub use vectors::{VectorStore, VectorConfig, VectorMetadata, VectorStats};
pub use reasoning_store::{ReasoningStore, ConceptData, AssociationData, ReasoningContext};

// New concurrent memory exports
pub use concurrent_memory::{ConcurrentMemory, ConcurrentConfig, ConcurrentStats, SnapshotInfo, HnswStats};
pub use write_log::{WriteLog, WriteEntry, WriteLogStats, WriteLogError};
pub use read_view::{ReadView, GraphSnapshot, ConceptNode};
pub use adaptive_reconciler::{AdaptiveReconciler, AdaptiveReconcilerConfig, AdaptiveReconcilerStats};
pub use mmap_store::{MmapStore, MmapStats};
pub use parallel_paths::{ParallelPathFinder, PathResult};

// Scalability exports
pub use hnsw_container::{HnswContainer, HnswConfig, HnswContainerStats};
pub use sharded_storage::{ShardedStorage, ShardConfig, ShardMap, ShardStats, AggregatedStats};
pub use storage_trait::LearningStorage;
pub use transaction::{TransactionCoordinator, TxnOperation, TxnState, TxnError, Transaction, TxnCoordinatorStats}; // 🔥 NEW

/// Version of the storage format
pub const STORAGE_VERSION: u32 = 1;

/// Magic bytes for segment files
pub const MAGIC_BYTES: &[u8; 8] = b"SUTRASEG";

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version() {
        assert_eq!(STORAGE_VERSION, 1);
    }
}
