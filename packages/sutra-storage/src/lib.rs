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
/// - Python bindings via PyO3
mod types;
mod segment;
mod manifest;
mod lsm;
mod store;
mod index;
mod wal;
mod quantization;
mod vectors;
// mod python;
// mod python_concurrent;
mod reasoning_store;

// 🔥 NEW: Semantic understanding module
pub mod semantic;

// Unified learning pipeline modules
pub mod embedding_client;
pub mod semantic_extractor;
pub mod learning_pipeline;

// New concurrent memory modules
mod write_log;
mod read_view;
mod reconciler;
mod adaptive_reconciler; // 🔥 NEW: AI-native adaptive reconciliation
mod concurrent_memory;
mod mmap_store;
mod parallel_paths;

// Scalability modules
// mod hnsw_persistence;  // DEPRECATED - replaced by hnsw_container with USearch
mod hnsw_container;
mod sharded_storage;
mod storage_trait;
mod transaction; // 🔥 NEW: 2PC transaction coordinator for cross-shard atomicity

// Self-monitoring module (eating our own dogfood)
mod event_emitter;

// Security and authentication
pub mod auth;
pub mod tls;

// Python bindings (OPTIONAL - conditional compilation)
#[cfg(feature = "python-bindings")]
mod python_concurrent;

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
pub use reconciler::{Reconciler, ReconcilerConfig, ReconcilerStats};
pub use adaptive_reconciler::{AdaptiveReconciler, AdaptiveReconcilerConfig, AdaptiveReconcilerStats}; // 🔥 NEW
pub use mmap_store::{MmapStore, MmapStats};
pub use parallel_paths::{ParallelPathFinder, PathResult};

// Scalability exports
// pub use hnsw_persistence::{HnswPersistence, HnswConfig, HnswStats as HnswPersistenceStats, DistanceMetric};  // DEPRECATED
pub use hnsw_container::{HnswContainer, HnswConfig, HnswContainerStats};
pub use sharded_storage::{ShardedStorage, ShardConfig, ShardMap, ShardStats, AggregatedStats};
pub use storage_trait::LearningStorage;
pub use transaction::{TransactionCoordinator, TxnOperation, TxnState, TxnError, Transaction, TxnCoordinatorStats}; // 🔥 NEW

// Self-monitoring exports
pub use event_emitter::{StorageEventEmitter, StorageEvent};

// Re-export Python bindings (conditional)
#[cfg(feature = "python-bindings")]
pub use python_concurrent::*;

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
