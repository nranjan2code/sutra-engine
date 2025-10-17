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
mod python;
mod python_concurrent;
mod reasoning_store;

// New concurrent memory modules
mod write_log;
mod read_view;
mod reconciler;
mod concurrent_memory;
mod mmap_store;

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
pub use mmap_store::{MmapStore, MmapStats};

// Re-export Python bindings
// TODO: Uncomment when ready
// pub use python::*;

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
