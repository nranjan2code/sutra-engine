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
mod store;
mod index;
mod python;

pub use types::{
    ConceptId, AssociationType, ConceptRecord, AssociationRecord,
    SegmentHeader, GraphPath,
};

pub use store::GraphStore;
pub use index::GraphIndex;

// Re-export Python bindings
pub use python::*;

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
