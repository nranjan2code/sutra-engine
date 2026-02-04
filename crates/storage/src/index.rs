/// Advanced indexing structures for fast lookups
/// 
/// Provides multiple index types:
/// - Concept Index: ConceptId → (SegmentId, Offset) for O(1) lookups
/// - Adjacency Index: ConceptId → Vec<ConceptId> for fast neighbor queries
/// - Inverted Index: Word → Set<ConceptId> for text search
/// - Temporal Index: Timestamp → Vec<ConceptId> for time-travel queries
use crate::types::ConceptId;
use dashmap::DashMap;
use parking_lot::RwLock;
use smallvec::SmallVec;
use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;

/// Location of a concept in storage
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConceptLocation {
    /// Segment ID where concept is stored
    pub segment_id: u32,
    /// Byte offset within segment
    pub offset: u64,
}

impl ConceptLocation {
    pub fn new(segment_id: u32, offset: u64) -> Self {
        Self { segment_id, offset }
    }
}

/// Complete indexing system
pub struct GraphIndex {
    /// Concept ID → Storage Location
    /// Fast O(1) lookups for any concept
    concept_index: DashMap<ConceptId, ConceptLocation>,
    
    /// Concept ID → Neighbors
    /// SmallVec optimized for small number of connections (8 or fewer on stack)
    adjacency_index: DashMap<ConceptId, SmallVec<[ConceptId; 8]>>,
    
    /// Word → Set of Concept IDs
    /// For text search queries
    inverted_index: DashMap<String, HashSet<ConceptId>>,
    
    /// Timestamp (ms) → Concept IDs created at that time
    /// BTreeMap for range queries and time-travel
    temporal_index: Arc<RwLock<BTreeMap<u64, Vec<ConceptId>>>>,
    
    /// Statistics
    total_concepts: Arc<RwLock<u64>>,
    total_edges: Arc<RwLock<u64>>,
}

impl GraphIndex {
    /// Create a new empty index
    pub fn new() -> Self {
        Self {
            concept_index: DashMap::new(),
            adjacency_index: DashMap::new(),
            inverted_index: DashMap::new(),
            temporal_index: Arc::new(RwLock::new(BTreeMap::new())),
            total_concepts: Arc::new(RwLock::new(0)),
            total_edges: Arc::new(RwLock::new(0)),
        }
    }
    
    /// Insert a concept into the index
    pub fn insert_concept(
        &self,
        concept_id: ConceptId,
        location: ConceptLocation,
        created_at: u64,
    ) {
        // Add to concept index
        self.concept_index.insert(concept_id, location);
        
        // Add to temporal index
        let mut temporal = self.temporal_index.write();
        temporal
            .entry(created_at)
            .or_default()
            .push(concept_id);
        drop(temporal);
        
        // Update stats
        *self.total_concepts.write() += 1;
    }
    
    /// Look up a concept's location
    pub fn lookup_concept(&self, concept_id: ConceptId) -> Option<ConceptLocation> {
        self.concept_index.get(&concept_id).map(|v| *v.value())
    }
    
    /// Remove a concept from all indexes
    pub fn remove_concept(&self, concept_id: ConceptId) {
        if self.concept_index.remove(&concept_id).is_some() {
            *self.total_concepts.write() -= 1;
        }
        
        // Remove from adjacency (and update edge count)
        if let Some(neighbors) = self.adjacency_index.remove(&concept_id) {
            *self.total_edges.write() -= neighbors.1.len() as u64;
        }
        
        // Note: We don't remove from temporal index (for historical queries)
        // Note: We don't remove from inverted index (expensive, low benefit)
    }
    
    /// Add an edge to the adjacency index
    pub fn add_edge(&self, source: ConceptId, target: ConceptId) {
        // Add forward edge
        self.adjacency_index
            .entry(source)
            .or_default()
            .push(target);
        
        // Add backward edge (for bidirectional queries)
        self.adjacency_index
            .entry(target)
            .or_default()
            .push(source);
        
        *self.total_edges.write() += 1;
    }
    
    /// Get all neighbors of a concept
    pub fn get_neighbors(&self, concept_id: ConceptId) -> Vec<ConceptId> {
        self.adjacency_index
            .get(&concept_id)
            .map(|v| v.value().to_vec())
            .unwrap_or_default()
    }
    
    /// Add words to inverted index for a concept
    pub fn index_words(&self, concept_id: ConceptId, words: &[String]) {
        for word in words {
            let normalized = word.to_lowercase();
            self.inverted_index
                .entry(normalized)
                .or_default()
                .insert(concept_id);
        }
    }
    
    /// Search for concepts by word
    pub fn search_by_word(&self, word: &str) -> Vec<ConceptId> {
        let normalized = word.to_lowercase();
        self.inverted_index
            .get(&normalized)
            .map(|set| set.value().iter().copied().collect())
            .unwrap_or_default()
    }
    
    /// Search for concepts by multiple words (intersection)
    pub fn search_by_words(&self, words: &[String]) -> Vec<ConceptId> {
        if words.is_empty() {
            return Vec::new();
        }
        
        // Get concept sets for each word
        let mut sets: Vec<HashSet<ConceptId>> = words
            .iter()
            .filter_map(|word| {
                let normalized = word.to_lowercase();
                self.inverted_index
                    .get(&normalized)
                    .map(|set| set.value().clone())
            })
            .collect();
        
        if sets.is_empty() {
            return Vec::new();
        }
        
        // Intersection of all sets
        let mut result = sets.remove(0);
        for set in sets {
            result.retain(|id| set.contains(id));
        }
        
        result.into_iter().collect()
    }
    
    /// Get concepts created in a time range
    pub fn query_time_range(&self, start_ms: u64, end_ms: u64) -> Vec<ConceptId> {
        let temporal = self.temporal_index.read();
        temporal
            .range(start_ms..=end_ms)
            .flat_map(|(_, concepts)| concepts.iter().copied())
            .collect()
    }
    
    /// Get concepts created at a specific timestamp
    pub fn query_at_time(&self, timestamp_ms: u64) -> Vec<ConceptId> {
        let temporal = self.temporal_index.read();
        temporal
            .get(&timestamp_ms).cloned()
            .unwrap_or_default()
    }
    
    /// Get all concepts created before a timestamp
    pub fn query_before(&self, timestamp_ms: u64) -> Vec<ConceptId> {
        let temporal = self.temporal_index.read();
        temporal
            .range(..timestamp_ms)
            .flat_map(|(_, concepts)| concepts.iter().copied())
            .collect()
    }
    
    /// Get statistics
    pub fn stats(&self) -> IndexStats {
        let temporal = self.temporal_index.read();
        IndexStats {
            total_concepts: *self.total_concepts.read(),
            total_edges: *self.total_edges.read(),
            total_words: self.inverted_index.len() as u64,
            total_timestamps: temporal.len() as u64,
        }
    }
    
    /// Clear all indexes
    pub fn clear(&self) {
        self.concept_index.clear();
        self.adjacency_index.clear();
        self.inverted_index.clear();
        self.temporal_index.write().clear();
        *self.total_concepts.write() = 0;
        *self.total_edges.write() = 0;
    }
    
    /// Rebuild indexes from scratch (used after compaction)
    pub fn rebuild<F>(&self, build_fn: F)
    where
        F: Fn(&Self),
    {
        self.clear();
        build_fn(self);
    }
}

impl Default for GraphIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// Index statistics
#[derive(Debug, Clone, Copy)]
pub struct IndexStats {
    pub total_concepts: u64,
    pub total_edges: u64,
    pub total_words: u64,
    pub total_timestamps: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn make_concept_id(byte: u8) -> ConceptId {
        ConceptId::from_bytes([byte; 16])
    }
    
    #[test]
    fn test_concept_index() {
        let index = GraphIndex::new();
        
        let id1 = make_concept_id(1);
        let loc1 = ConceptLocation::new(0, 100);
        
        index.insert_concept(id1, loc1, 1000);
        
        assert_eq!(index.lookup_concept(id1), Some(loc1));
        assert_eq!(index.stats().total_concepts, 1);
    }
    
    #[test]
    fn test_adjacency_index() {
        let index = GraphIndex::new();
        
        let id1 = make_concept_id(1);
        let id2 = make_concept_id(2);
        let id3 = make_concept_id(3);
        
        index.add_edge(id1, id2);
        index.add_edge(id1, id3);
        
        let neighbors = index.get_neighbors(id1);
        assert_eq!(neighbors.len(), 2);
        assert!(neighbors.contains(&id2));
        assert!(neighbors.contains(&id3));
        
        assert_eq!(index.stats().total_edges, 2);
    }
    
    #[test]
    fn test_inverted_index() {
        let index = GraphIndex::new();
        
        let id1 = make_concept_id(1);
        let id2 = make_concept_id(2);
        
        index.index_words(id1, &["rust".to_string(), "programming".to_string()]);
        index.index_words(id2, &["rust".to_string(), "language".to_string()]);
        
        // Search single word
        let results = index.search_by_word("rust");
        assert_eq!(results.len(), 2);
        
        // Search multiple words (intersection)
        let results = index.search_by_words(&["rust".to_string(), "programming".to_string()]);
        assert_eq!(results.len(), 1);
        assert!(results.contains(&id1));
        
        assert_eq!(index.stats().total_words, 3);
    }
    
    #[test]
    fn test_temporal_index() {
        let index = GraphIndex::new();
        
        let id1 = make_concept_id(1);
        let id2 = make_concept_id(2);
        let id3 = make_concept_id(3);
        
        index.insert_concept(id1, ConceptLocation::new(0, 0), 1000);
        index.insert_concept(id2, ConceptLocation::new(0, 128), 2000);
        index.insert_concept(id3, ConceptLocation::new(0, 256), 3000);
        
        // Query at specific time
        let at_2000 = index.query_at_time(2000);
        assert_eq!(at_2000.len(), 1);
        assert!(at_2000.contains(&id2));
        
        // Query range
        let range = index.query_time_range(1000, 2000);
        assert_eq!(range.len(), 2);
        assert!(range.contains(&id1));
        assert!(range.contains(&id2));
        
        // Query before
        let before = index.query_before(2500);
        assert_eq!(before.len(), 2);
    }
    
    #[test]
    fn test_remove_concept() {
        let index = GraphIndex::new();
        
        let id1 = make_concept_id(1);
        let id2 = make_concept_id(2);
        
        index.insert_concept(id1, ConceptLocation::new(0, 0), 1000);
        index.add_edge(id1, id2);
        
        assert_eq!(index.stats().total_concepts, 1);
        assert_eq!(index.stats().total_edges, 1);
        
        index.remove_concept(id1);
        
        assert_eq!(index.lookup_concept(id1), None);
        assert_eq!(index.stats().total_concepts, 0);
    }
    
    #[test]
    fn test_clear() {
        let index = GraphIndex::new();
        
        let id1 = make_concept_id(1);
        index.insert_concept(id1, ConceptLocation::new(0, 0), 1000);
        index.add_edge(id1, make_concept_id(2));
        index.index_words(id1, &["test".to_string()]);
        
        assert_eq!(index.stats().total_concepts, 1);
        
        index.clear();
        
        let stats = index.stats();
        assert_eq!(stats.total_concepts, 0);
        assert_eq!(stats.total_edges, 0);
        assert_eq!(stats.total_words, 0);
    }
    
    #[test]
    fn test_case_insensitive_search() {
        let index = GraphIndex::new();
        
        let id1 = make_concept_id(1);
        index.index_words(id1, &["Rust".to_string(), "Programming".to_string()]);
        
        // Should find regardless of case
        let results1 = index.search_by_word("rust");
        let results2 = index.search_by_word("RUST");
        let results3 = index.search_by_word("Rust");
        
        assert_eq!(results1.len(), 1);
        assert_eq!(results2.len(), 1);
        assert_eq!(results3.len(), 1);
    }
}
