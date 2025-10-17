/// Immutable read view for zero-contention graph traversal
/// 
/// Design:
/// - Immutable snapshot (readers never block)
/// - Atomic pointer swap (arc-swap for lock-free updates)
/// - Graph-optimized layout (edges co-located with concepts)
/// - Zero-copy traversal via indexing

use crate::types::{AssociationRecord, ConceptId};
use arc_swap::ArcSwap;
use dashmap::DashMap;
use std::sync::Arc;

/// In-memory concept with co-located edges
#[derive(Debug, Clone)]
pub struct ConceptNode {
    pub id: ConceptId,
    pub content: Arc<[u8]>,
    pub vector: Option<Arc<[f32]>>,
    pub strength: f32,
    pub confidence: f32,
    pub created: u64,
    pub last_accessed: u64,
    pub access_count: u32,
    
    /// Co-located edges for cache-friendly traversal
    pub neighbors: Vec<ConceptId>,
    pub associations: Vec<AssociationRecord>,
}

impl ConceptNode {
    pub fn new(
        id: ConceptId,
        content: Vec<u8>,
        vector: Option<Vec<f32>>,
        strength: f32,
        confidence: f32,
        timestamp: u64,
    ) -> Self {
        Self {
            id,
            content: Arc::from(content),
            vector: vector.map(Arc::from),
            strength,
            confidence,
            created: timestamp,
            last_accessed: timestamp,
            access_count: 0,
            neighbors: Vec::new(),
            associations: Vec::new(),
        }
    }
    
    /// Add an edge to another concept
    pub fn add_edge(&mut self, target: ConceptId, record: AssociationRecord) {
        if !self.neighbors.contains(&target) {
            self.neighbors.push(target);
        }
        self.associations.push(record);
    }
    
    /// Get neighbors sorted by association strength
    pub fn neighbors_by_strength(&self) -> Vec<(ConceptId, f32)> {
        let mut pairs: Vec<_> = self.neighbors
            .iter()
            .zip(self.associations.iter())
            .map(|(id, assoc)| (*id, assoc.confidence))
            .collect();
        
        pairs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        pairs
    }
}

/// Immutable graph snapshot
#[derive(Debug, Clone)]
pub struct GraphSnapshot {
    /// All concepts indexed by ID
    pub concepts: Arc<DashMap<ConceptId, ConceptNode>>,
    
    /// Snapshot metadata
    pub sequence: u64,
    pub timestamp: u64,
    pub concept_count: usize,
    pub edge_count: usize,
}

impl GraphSnapshot {
    pub fn new(sequence: u64) -> Self {
        Self {
            concepts: Arc::new(DashMap::new()),
            sequence,
            timestamp: current_timestamp_us(),
            concept_count: 0,
            edge_count: 0,
        }
    }
    
    /// Get a concept by ID
    pub fn get_concept(&self, id: &ConceptId) -> Option<ConceptNode> {
        self.concepts.get(id).map(|e| e.value().clone())
    }
    
    /// Check if concept exists
    pub fn contains(&self, id: &ConceptId) -> bool {
        self.concepts.contains_key(id)
    }
    
    /// Get neighbors of a concept
    pub fn get_neighbors(&self, id: &ConceptId) -> Vec<ConceptId> {
        self.concepts
            .get(id)
            .map(|node| node.neighbors.clone())
            .unwrap_or_default()
    }
    
    /// Get neighbors with strengths
    pub fn get_neighbors_weighted(&self, id: &ConceptId) -> Vec<(ConceptId, f32)> {
        self.concepts
            .get(id)
            .map(|node| node.neighbors_by_strength())
            .unwrap_or_default()
    }
    
    /// Traverse path from start to end (BFS)
    pub fn find_path(&self, start: ConceptId, end: ConceptId, max_depth: usize) -> Option<Vec<ConceptId>> {
        use std::collections::{HashMap, VecDeque};
        
        if start == end {
            return Some(vec![start]);
        }
        
        let mut queue = VecDeque::new();
        let mut visited = HashMap::new();
        
        queue.push_back((start, 0));
        visited.insert(start, None);
        
        while let Some((current, depth)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }
            
            if let Some(neighbors) = self.concepts.get(&current) {
                for &neighbor in &neighbors.neighbors {
                    if !visited.contains_key(&neighbor) {
                        visited.insert(neighbor, Some(current));
                        
                        if neighbor == end {
                            // Reconstruct path
                            let mut path = vec![neighbor];
                            let mut backtrack = current;
                            path.push(backtrack);
                            
                            while let Some(Some(prev)) = visited.get(&backtrack) {
                                path.push(*prev);
                                backtrack = *prev;
                            }
                            
                            path.reverse();
                            return Some(path);
                        }
                        
                        queue.push_back((neighbor, depth + 1));
                    }
                }
            }
        }
        
        None
    }
    
    /// Get all concepts (expensive, for bulk operations)
    pub fn all_concepts(&self) -> Vec<ConceptNode> {
        self.concepts.iter().map(|e| e.value().clone()).collect()
    }
    
    /// Update stats (should be called after modifications)
    pub fn update_stats(&mut self) {
        self.concept_count = self.concepts.len();
        self.edge_count = self.concepts
            .iter()
            .map(|e| e.value().associations.len())
            .sum();
    }
    
    /// Get concept count
    pub fn concept_count(&self) -> usize {
        self.concept_count
    }
    
    /// Get edge count
    pub fn edge_count(&self) -> usize {
        self.edge_count
    }
}

/// Read view with atomic snapshot swapping
pub struct ReadView {
    /// Current snapshot (atomically swappable)
    snapshot: ArcSwap<GraphSnapshot>,
}

impl ReadView {
    /// Create a new read view
    pub fn new() -> Self {
        let initial = GraphSnapshot::new(0);
        
        Self {
            snapshot: ArcSwap::from_pointee(initial),
        }
    }
    
    /// PRODUCTION: Create ReadView from loaded storage data
    pub fn from_loaded_data(
        concepts: std::collections::HashMap<ConceptId, ConceptNode>,
        edges: std::collections::HashMap<ConceptId, Vec<(ConceptId, f32)>>,
    ) -> Self {
        use std::collections::HashMap;
        
        let mut snapshot = GraphSnapshot::new(1); // Start at sequence 1 for loaded data
        
        // Convert HashMap to DashMap and populate concept nodes
        let mut concept_nodes: HashMap<ConceptId, ConceptNode> = HashMap::new();
        
        for (concept_id, mut node) in concepts {
            // Add edges to concept node if they exist
            if let Some(concept_edges) = edges.get(&concept_id) {
                for (target_id, confidence) in concept_edges {
                    // Create association record
                    let assoc = crate::types::AssociationRecord::new(
                        concept_id,
                        *target_id,
                        crate::types::AssociationType::Semantic,
                        *confidence,
                    );
                    node.add_edge(*target_id, assoc);
                }
            }
            
            concept_nodes.insert(concept_id, node);
        }
        
        // Populate DashMap
        for (concept_id, node) in concept_nodes {
            snapshot.concepts.insert(concept_id, node);
        }
        
        // Update statistics
        snapshot.update_stats();
        snapshot.timestamp = current_timestamp_us();
        
        log::info!("🔄 ReadView created from loaded data: {} concepts, {} edges", 
                  snapshot.concept_count, snapshot.edge_count);
        
        Self {
            snapshot: ArcSwap::from_pointee(snapshot),
        }
    }
    
    /// Load current snapshot (zero-copy, lock-free)
    pub fn load(&self) -> Arc<GraphSnapshot> {
        self.snapshot.load_full()
    }
    
    /// Store new snapshot (atomic swap)
    pub fn store(&self, new_snapshot: GraphSnapshot) {
        self.snapshot.store(Arc::new(new_snapshot));
    }
    
    /// Query concept (convenience)
    pub fn get_concept(&self, id: &ConceptId) -> Option<ConceptNode> {
        self.load().get_concept(id)
    }
    
    /// Query neighbors (convenience)
    pub fn get_neighbors(&self, id: &ConceptId) -> Vec<ConceptId> {
        self.load().get_neighbors(id)
    }
    
    /// Traverse path (convenience)
    pub fn find_path(&self, start: ConceptId, end: ConceptId, max_depth: usize) -> Option<Vec<ConceptId>> {
        self.load().find_path(start, end, max_depth)
    }
    
    /// Get snapshot metadata
    pub fn snapshot_info(&self) -> (u64, u64, usize, usize) {
        let snap = self.load();
        (snap.sequence, snap.timestamp, snap.concept_count, snap.edge_count)
    }
}

impl Default for ReadView {
    fn default() -> Self {
        Self::new()
    }
}

/// Get current timestamp in microseconds
fn current_timestamp_us() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_micros() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::AssociationType;
    
    #[test]
    fn test_concept_node() {
        let id = ConceptId([1; 16]);
        let content = b"test".to_vec();
        let node = ConceptNode::new(id, content, None, 1.0, 0.9, 1000);
        
        assert_eq!(node.id, id);
        assert_eq!(node.content.as_ref(), b"test");
        assert_eq!(node.neighbors.len(), 0);
    }
    
    #[test]
    fn test_add_edge() {
        let id1 = ConceptId([1; 16]);
        let id2 = ConceptId([2; 16]);
        
        let mut node = ConceptNode::new(id1, vec![1], None, 1.0, 0.9, 1000);
        
        let assoc = AssociationRecord::new(id1, id2, AssociationType::Semantic, 0.8);
        node.add_edge(id2, assoc);
        
        assert_eq!(node.neighbors.len(), 1);
        assert_eq!(node.neighbors[0], id2);
        assert_eq!(node.associations.len(), 1);
    }
    
    #[test]
    fn test_snapshot_basic() {
        let snapshot = GraphSnapshot::new(0);
        
        let id = ConceptId([1; 16]);
        let node = ConceptNode::new(id, vec![1, 2, 3], None, 1.0, 0.9, 1000);
        
        snapshot.concepts.insert(id, node);
        
        assert!(snapshot.contains(&id));
        
        let retrieved = snapshot.get_concept(&id).unwrap();
        assert_eq!(retrieved.content.as_ref(), &[1, 2, 3]);
    }
    
    #[test]
    fn test_read_view_swap() {
        let view = ReadView::new();
        
        // Initial snapshot
        let snap1 = view.load();
        assert_eq!(snap1.sequence, 0);
        
        // Create new snapshot
        let mut snap2 = GraphSnapshot::new(1);
        let id = ConceptId([1; 16]);
        snap2.concepts.insert(id, ConceptNode::new(id, vec![42], None, 1.0, 0.9, 2000));
        
        // Atomic swap
        view.store(snap2);
        
        // New readers see new snapshot
        let snap_new = view.load();
        assert_eq!(snap_new.sequence, 1);
        assert!(snap_new.contains(&id));
        
        // Old readers still have old snapshot
        assert_eq!(snap1.sequence, 0);
        assert!(!snap1.contains(&id));
    }
    
    #[test]
    fn test_find_path() {
        let snapshot = GraphSnapshot::new(0);
        
        let id1 = ConceptId([1; 16]);
        let id2 = ConceptId([2; 16]);
        let id3 = ConceptId([3; 16]);
        
        // Build graph: 1 -> 2 -> 3
        let mut node1 = ConceptNode::new(id1, vec![1], None, 1.0, 0.9, 1000);
        let assoc12 = AssociationRecord::new(id1, id2, AssociationType::Semantic, 0.8);
        node1.add_edge(id2, assoc12);
        
        let mut node2 = ConceptNode::new(id2, vec![2], None, 1.0, 0.9, 1000);
        let assoc23 = AssociationRecord::new(id2, id3, AssociationType::Semantic, 0.8);
        node2.add_edge(id3, assoc23);
        
        let node3 = ConceptNode::new(id3, vec![3], None, 1.0, 0.9, 1000);
        
        snapshot.concepts.insert(id1, node1);
        snapshot.concepts.insert(id2, node2);
        snapshot.concepts.insert(id3, node3);
        
        // Find path 1 -> 3
        let path = snapshot.find_path(id1, id3, 10).unwrap();
        assert_eq!(path.len(), 3);
        assert_eq!(path[0], id1);
        assert_eq!(path[1], id2);
        assert_eq!(path[2], id3);
    }
    
    #[test]
    fn test_neighbors_by_strength() {
        let id1 = ConceptId([1; 16]);
        let id2 = ConceptId([2; 16]);
        let id3 = ConceptId([3; 16]);
        
        let mut node = ConceptNode::new(id1, vec![1], None, 1.0, 0.9, 1000);
        
        // Add edges with different strengths
        let assoc12 = AssociationRecord::new(id1, id2, AssociationType::Semantic, 0.5);
        let assoc13 = AssociationRecord::new(id1, id3, AssociationType::Semantic, 0.9);
        
        node.add_edge(id2, assoc12);
        node.add_edge(id3, assoc13);
        
        let neighbors = node.neighbors_by_strength();
        
        // Should be sorted by strength descending
        assert_eq!(neighbors.len(), 2);
        assert_eq!(neighbors[0].0, id3);  // 0.9 strength
        assert_eq!(neighbors[1].0, id2);  // 0.5 strength
    }
}
