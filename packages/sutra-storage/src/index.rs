/// Graph indexing structures
use crate::types::ConceptId;
use dashmap::DashMap;

pub struct GraphIndex {
    // TODO: Add HNSW index for semantic search
    // TODO: Add inverted index for word lookup
    // TODO: Add temporal index
    adjacency: DashMap<ConceptId, Vec<ConceptId>>,
}

impl GraphIndex {
    pub fn new() -> Self {
        Self {
            adjacency: DashMap::new(),
        }
    }
}
