/// Concurrent Memory - Main coordinator for burst-tolerant storage
/// 
/// Unified API that hides write/read plane separation.
/// Optimized for unpredictable burst patterns.
///
/// Architecture:
/// - Writes → WriteLog (lock-free, never blocks)
/// - Reads → ReadView (immutable snapshot, never blocks)
/// - Background reconciler merges continuously

use crate::read_view::{ConceptNode, ReadView};
use crate::reconciler::{Reconciler, ReconcilerConfig, ReconcilerStats};
use crate::types::{AssociationRecord, AssociationType, ConceptId};
use crate::write_log::{WriteLog, WriteLogError, WriteLogStats};
use hnsw_rs::prelude::*;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// Concurrent memory configuration
#[derive(Debug, Clone)]
pub struct ConcurrentConfig {
    /// Storage base path
    pub storage_path: PathBuf,
    
    /// Reconciliation interval (milliseconds)
    pub reconcile_interval_ms: u64,
    
    /// Memory threshold before disk flush (number of concepts)
    pub memory_threshold: usize,
    
    /// Vector dimension for HNSW index
    pub vector_dimension: usize,
}

impl Default for ConcurrentConfig {
    fn default() -> Self {
        Self {
            storage_path: PathBuf::from("./storage"),
            reconcile_interval_ms: 10, // 10ms
            memory_threshold: 50_000,
            vector_dimension: 768, // Default: EmbeddingGemma dimension
        }
    }
}

/// Main concurrent memory system
pub struct ConcurrentMemory {
    /// Write plane (append-only log)
    write_log: Arc<WriteLog>,
    
    /// Read plane (immutable snapshots)
    read_view: Arc<ReadView>,
    
    /// Background reconciler
    reconciler: Reconciler,
    
    /// Vectors stored for HNSW indexing
    vectors: Arc<RwLock<HashMap<ConceptId, Vec<f32>>>>,
    
    /// Configuration
    config: ConcurrentConfig,
}

impl ConcurrentMemory {
    /// Create and start a new concurrent memory system
    pub fn new(config: ConcurrentConfig) -> Self {
        let write_log = Arc::new(WriteLog::new());
        let read_view = Arc::new(ReadView::new());
        
        let reconciler_config = ReconcilerConfig {
            reconcile_interval_ms: config.reconcile_interval_ms,
            disk_flush_threshold: config.memory_threshold,
            storage_path: config.storage_path.clone(),
            ..Default::default()
        };
        
        let mut reconciler = Reconciler::new(
            reconciler_config,
            Arc::clone(&write_log),
            Arc::clone(&read_view),
        );
        
        // Start reconciler thread immediately
        reconciler.start();
        
        Self {
            write_log,
            read_view,
            reconciler,
            vectors: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }
    
    // ========================
    // WRITE API (never blocks)
    // ========================
    
    /// Learn a new concept
    pub fn learn_concept(
        &self,
        id: ConceptId,
        content: Vec<u8>,
        vector: Option<Vec<f32>>,
        strength: f32,
        confidence: f32,
    ) -> Result<u64, WriteLogError> {
        let seq = self.write_log.append_concept(id, content, vector.clone(), strength, confidence)?;
        
        // Auto-index vector in HNSW if provided
        if let Some(vec) = vector {
            if vec.len() == self.config.vector_dimension {
                let _ = self.index_vector(id, vec);
            }
        }
        
        Ok(seq)
    }
    
    /// Learn an association between concepts
    pub fn learn_association(
        &self,
        source: ConceptId,
        target: ConceptId,
        assoc_type: AssociationType,
        confidence: f32,
    ) -> Result<u64, WriteLogError> {
        let record = AssociationRecord::new(source, target, assoc_type, confidence);
        self.write_log.append_association(record)
    }
    
    /// Update concept strength (for temporal decay)
    pub fn update_strength(&self, id: ConceptId, strength: f32) -> Result<u64, WriteLogError> {
        self.write_log.append(crate::write_log::WriteEntry::UpdateStrength { id, strength })
    }
    
    /// Record concept access (for heat tracking)
    pub fn record_access(&self, id: ConceptId) -> Result<u64, WriteLogError> {
        let timestamp = current_timestamp_us();
        self.write_log.append(crate::write_log::WriteEntry::RecordAccess { id, timestamp })
    }
    
    // ========================
    // READ API (never blocks)
    // ========================
    
    /// Query a concept by ID
    pub fn query_concept(&self, id: &ConceptId) -> Option<ConceptNode> {
        self.read_view.get_concept(id)
    }
    
    /// Get neighbors of a concept
    pub fn query_neighbors(&self, id: &ConceptId) -> Vec<ConceptId> {
        self.read_view.get_neighbors(id)
    }
    
    /// Get neighbors with association strengths
    pub fn query_neighbors_weighted(&self, id: &ConceptId) -> Vec<(ConceptId, f32)> {
        let snapshot = self.read_view.load();
        snapshot.get_neighbors_weighted(id)
    }
    
    /// Find path between two concepts (BFS)
    pub fn find_path(&self, start: ConceptId, end: ConceptId, max_depth: usize) -> Option<Vec<ConceptId>> {
        self.read_view.find_path(start, end, max_depth)
    }
    
    /// Check if concept exists
    pub fn contains(&self, id: &ConceptId) -> bool {
        self.read_view.load().contains(id)
    }
    
    /// Get current snapshot stats
    pub fn snapshot_info(&self) -> SnapshotInfo {
        let (sequence, timestamp, concepts, edges) = self.read_view.snapshot_info();
        SnapshotInfo {
            sequence,
            timestamp,
            concept_count: concepts,
            edge_count: edges,
        }
    }
    
    // ========================
    // VECTOR SEARCH API
    // ========================
    
    /// Store vector for indexing (internal, called automatically by learn_concept)
    fn index_vector(&self, concept_id: ConceptId, vector: Vec<f32>) -> Result<(), String> {
        self.vectors.write().insert(concept_id, vector);
        Ok(())
    }
    
    /// Vector similarity search (k-NN) - builds HNSW on demand
    pub fn vector_search(&self, query: &[f32], k: usize, _ef_search: usize) -> Vec<(ConceptId, f32)> {
        let vectors_guard = self.vectors.read();
        
        if vectors_guard.is_empty() {
            return Vec::new();
        }
        
        // Build HNSW index from stored vectors
        let data_with_ids: Vec<(&Vec<f32>, ConceptId)> = vectors_guard
            .iter()
            .map(|(id, vec)| (vec, *id))
            .collect();
        
        // Build HNSW
        let hnsw = Hnsw::<f32, DistCosine>::new(
            16,
            data_with_ids.len(),
            16,
            100,
            DistCosine {},
        );
        
        // Insert all vectors
        for (idx, (vec, _concept_id)) in data_with_ids.iter().enumerate() {
            hnsw.insert((vec.as_slice(), idx));
        }
        
        // Search
        let results = hnsw.search(query, k, 50);
        
        // Convert to (ConceptId, similarity)
        results
            .into_iter()
            .filter_map(|neighbor| {
                data_with_ids.get(neighbor.d_id).map(|(_, concept_id)| {
                    // Convert distance to similarity
                    (*concept_id, 1.0 - neighbor.distance)
                })
            })
            .collect()
    }
    
    /// Get HNSW statistics
    pub fn hnsw_stats(&self) -> HnswStats {
        let indexed_count = self.vectors.read().len();
        
        HnswStats {
            indexed_vectors: indexed_count,
            dimension: self.config.vector_dimension,
            index_ready: indexed_count > 0,
        }
    }
    
    // ========================
    // SYSTEM API
    // ========================
    
    /// Get write log statistics
    pub fn write_stats(&self) -> WriteLogStats {
        self.write_log.stats()
    }
    
    /// Get reconciler statistics
    pub fn reconciler_stats(&self) -> ReconcilerStats {
        self.reconciler.stats()
    }
    
    /// Get complete system statistics
    pub fn stats(&self) -> ConcurrentStats {
        ConcurrentStats {
            write_log: self.write_stats(),
            reconciler: self.reconciler_stats(),
            snapshot: self.snapshot_info(),
        }
    }
    
    /// Force immediate flush to disk
    pub fn flush(&self) -> anyhow::Result<()> {
        // Get current snapshot
        let snap = self.read_view.load();
        
        // Flush to disk (flush_to_disk creates storage.dat inside the path)
        crate::reconciler::flush_to_disk(&snap, &self.config.storage_path, 0)?;
        
        Ok(())
    }
    
    /// Stop the system gracefully
    pub fn shutdown(mut self) {
        // Flush before stopping
        let _ = self.flush();
        self.reconciler.stop();
    }
}

impl Drop for ConcurrentMemory {
    fn drop(&mut self) {
        // Reconciler will be dropped and stopped automatically
    }
}

/// Snapshot metadata
#[derive(Debug, Clone, Copy)]
pub struct SnapshotInfo {
    pub sequence: u64,
    pub timestamp: u64,
    pub concept_count: usize,
    pub edge_count: usize,
}

/// Complete system statistics
#[derive(Debug, Clone, Copy)]
pub struct ConcurrentStats {
    pub write_log: WriteLogStats,
    pub reconciler: ReconcilerStats,
    pub snapshot: SnapshotInfo,
}

/// HNSW index statistics
#[derive(Debug, Clone, Copy)]
pub struct HnswStats {
    pub indexed_vectors: usize,
    pub dimension: usize,
    pub index_ready: bool,
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
    use std::thread;
    use std::time::Duration;
    use tempfile::TempDir;
    
    #[test]
    fn test_basic_operations() {
        let dir = TempDir::new().unwrap();
        let config = ConcurrentConfig {
            storage_path: dir.path().to_path_buf(),
            reconcile_interval_ms: 50,
            ..Default::default()
        };
        
        let memory = ConcurrentMemory::new(config);
        
        // Learn concept
        let id = ConceptId([1; 16]);
        memory.learn_concept(id, b"test concept".to_vec(), None, 1.0, 0.9).unwrap();
        
        // Wait for reconciliation
        thread::sleep(Duration::from_millis(100));
        
        // Query concept
        let concept = memory.query_concept(&id).unwrap();
        assert_eq!(concept.content.as_ref(), b"test concept");
        assert_eq!(concept.strength, 1.0);
        assert_eq!(concept.confidence, 0.9);
    }
    
    #[test]
    fn test_associations() {
        let dir = TempDir::new().unwrap();
        let config = ConcurrentConfig {
            storage_path: dir.path().to_path_buf(),
            reconcile_interval_ms: 50,
            ..Default::default()
        };
        
        let memory = ConcurrentMemory::new(config);
        
        let id1 = ConceptId([1; 16]);
        let id2 = ConceptId([2; 16]);
        
        // Learn concepts
        memory.learn_concept(id1, vec![1], None, 1.0, 0.9).unwrap();
        memory.learn_concept(id2, vec![2], None, 1.0, 0.9).unwrap();
        
        // Learn association
        memory.learn_association(id1, id2, AssociationType::Semantic, 0.8).unwrap();
        
        // Wait for reconciliation
        thread::sleep(Duration::from_millis(100));
        
        // Query neighbors
        let neighbors = memory.query_neighbors(&id1);
        assert!(neighbors.contains(&id2));
        
        // Query with weights
        let weighted = memory.query_neighbors_weighted(&id1);
        assert_eq!(weighted.len(), 1);
        assert_eq!(weighted[0].0, id2);
        assert_eq!(weighted[0].1, 0.8);
    }
    
    #[test]
    fn test_path_finding() {
        let dir = TempDir::new().unwrap();
        let config = ConcurrentConfig {
            storage_path: dir.path().to_path_buf(),
            reconcile_interval_ms: 50,
            ..Default::default()
        };
        
        let memory = ConcurrentMemory::new(config);
        
        let id1 = ConceptId([1; 16]);
        let id2 = ConceptId([2; 16]);
        let id3 = ConceptId([3; 16]);
        
        // Build chain: 1 -> 2 -> 3
        memory.learn_concept(id1, vec![1], None, 1.0, 0.9).unwrap();
        memory.learn_concept(id2, vec![2], None, 1.0, 0.9).unwrap();
        memory.learn_concept(id3, vec![3], None, 1.0, 0.9).unwrap();
        
        memory.learn_association(id1, id2, AssociationType::Semantic, 0.8).unwrap();
        memory.learn_association(id2, id3, AssociationType::Semantic, 0.8).unwrap();
        
        // Wait for reconciliation
        thread::sleep(Duration::from_millis(150));
        
        // Find path 1 -> 3
        let path = memory.find_path(id1, id3, 10).unwrap();
        assert_eq!(path.len(), 3);
        assert_eq!(path[0], id1);
        assert_eq!(path[1], id2);
        assert_eq!(path[2], id3);
    }
    
    #[test]
    fn test_burst_writes() {
        let dir = TempDir::new().unwrap();
        let config = ConcurrentConfig {
            storage_path: dir.path().to_path_buf(),
            reconcile_interval_ms: 50,
            ..Default::default()
        };
        
        let memory = ConcurrentMemory::new(config);
        
        // Simulate write burst
        for i in 0..1000 {
            let id = ConceptId([i as u8; 16]);
            memory.learn_concept(id, vec![i as u8], None, 1.0, 0.9).unwrap();
        }
        
        // Wait for reconciliation
        thread::sleep(Duration::from_millis(200));
        
        // Verify some concepts
        let id1 = ConceptId([1; 16]);
        let id2 = ConceptId([100; 16]);
        
        assert!(memory.contains(&id1));
        assert!(memory.contains(&id2));
        
        let stats = memory.stats();
        assert!(stats.snapshot.concept_count >= 1000);
    }
    
    #[test]
    fn test_concurrent_read_write() {
        let dir = TempDir::new().unwrap();
        let config = ConcurrentConfig {
            storage_path: dir.path().to_path_buf(),
            reconcile_interval_ms: 20,
            ..Default::default()
        };
        
        let memory = Arc::new(ConcurrentMemory::new(config));
        
        // Writer thread
        let memory_writer = Arc::clone(&memory);
        let write_handle = thread::spawn(move || {
            for i in 0..100 {
                let id = ConceptId([i; 16]);
                memory_writer.learn_concept(id, vec![i], None, 1.0, 0.9).ok();
                thread::sleep(Duration::from_millis(1));
            }
        });
        
        // Reader thread
        let memory_reader = Arc::clone(&memory);
        let read_handle = thread::spawn(move || {
            let mut found_count = 0;
            for _ in 0..100 {
                for i in 0..100 {
                    let id = ConceptId([i; 16]);
                    if memory_reader.contains(&id) {
                        found_count += 1;
                    }
                }
                thread::sleep(Duration::from_millis(1));
            }
            found_count
        });
        
        write_handle.join().unwrap();
        let found = read_handle.join().unwrap();
        
        // Should have found many concepts (not necessarily all due to timing)
        assert!(found > 0);
    }
    
    #[test]
    fn test_stats() {
        let dir = TempDir::new().unwrap();
        let config = ConcurrentConfig {
            storage_path: dir.path().to_path_buf(),
            reconcile_interval_ms: 50,
            ..Default::default()
        };
        
        let memory = ConcurrentMemory::new(config);
        
        // Write some data
        for i in 0..10 {
            let id = ConceptId([i; 16]);
            memory.learn_concept(id, vec![i], None, 1.0, 0.9).unwrap();
        }
        
        // Wait for reconciliation
        thread::sleep(Duration::from_millis(100));
        
        let stats = memory.stats();
        assert!(stats.write_log.written >= 10);
        assert!(stats.reconciler.entries_processed >= 10);
        assert!(stats.snapshot.concept_count >= 10);
    }
}
