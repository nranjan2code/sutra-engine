/// Core graph storage implementation
use crate::types::*;
use dashmap::DashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Legacy graph storage engine (simple in-memory implementation)
///
/// Note: Production code should use `ConcurrentMemory` instead, which includes:
/// - Memory-mapped segments via `MmapStore`
/// - Adaptive reconciliation (replaces LSM-tree compaction)
/// - Lock-free concurrent access
/// - Write-Ahead Log for durability
pub struct GraphStore {
    #[allow(dead_code)]
    path: PathBuf,
    concepts: Arc<DashMap<ConceptId, ConceptRecord>>,
    associations: Arc<DashMap<(ConceptId, ConceptId), AssociationRecord>>,
}

impl GraphStore {
    pub fn new<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let path = path.as_ref().to_path_buf();
        std::fs::create_dir_all(&path)?;
        
        Ok(Self {
            path,
            concepts: Arc::new(DashMap::new()),
            associations: Arc::new(DashMap::new()),
        })
    }
    
    pub fn write_concept(&self, record: ConceptRecord) -> anyhow::Result<ConceptId> {
        let id = record.concept_id;
        self.concepts.insert(id, record);
        Ok(id)
    }
    
    pub fn read_concept(&self, id: ConceptId) -> Option<ConceptRecord> {
        self.concepts.get(&id).map(|r| *r.value())
    }
    
    pub fn write_association(&self, record: AssociationRecord) -> anyhow::Result<()> {
        let key = (record.source_id, record.target_id);
        self.associations.insert(key, record);
        Ok(())
    }
    
    pub fn get_neighbors(&self, id: ConceptId) -> Vec<ConceptId> {
        self.associations
            .iter()
            .filter_map(|entry| {
                let (source, target) = entry.key();
                if *source == id {
                    Some(*target)
                } else if *target == id {
                    Some(*source)
                } else {
                    None
                }
            })
            .collect()
    }
}
