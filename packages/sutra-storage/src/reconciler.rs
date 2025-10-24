/// Background reconciler - continuously merges write log into read view
/// 
/// Runs in dedicated thread, invisible to readers/writers.
/// Drains write log, applies to snapshot, atomically swaps.
/// Also flushes to disk segments periodically.

use crate::read_view::{ConceptNode, GraphSnapshot, ReadView};
use crate::write_log::{WriteEntry, WriteLog};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Reconciler configuration
#[derive(Debug, Clone)]
pub struct ReconcilerConfig {
    /// How often to reconcile (milliseconds)
    pub reconcile_interval_ms: u64,
    
    /// Max batch size per reconciliation
    pub max_batch_size: usize,
    
    /// Flush to disk threshold (number of concepts in memory)
    pub disk_flush_threshold: usize,
    
    /// Base path for segments
    pub storage_path: PathBuf,
}

impl Default for ReconcilerConfig {
    fn default() -> Self {
        Self {
            reconcile_interval_ms: 10, // 10ms reconciliation
            max_batch_size: 10_000,
            disk_flush_threshold: 50_000,
            storage_path: PathBuf::from("./storage"),
        }
    }
}

/// Background reconciler
pub struct Reconciler {
    config: ReconcilerConfig,
    write_log: Arc<WriteLog>,
    read_view: Arc<ReadView>,
    
    /// Control
    running: Arc<AtomicBool>,
    thread_handle: Option<JoinHandle<()>>,
    
    /// Metrics
    reconciliations: Arc<AtomicU64>,
    entries_processed: Arc<AtomicU64>,
    disk_flushes: Arc<AtomicU64>,
}

impl Reconciler {
    /// Create a new reconciler (does not start thread)
    pub fn new(
        config: ReconcilerConfig,
        write_log: Arc<WriteLog>,
        read_view: Arc<ReadView>,
    ) -> Self {
        std::fs::create_dir_all(&config.storage_path).ok();
        
        Self {
            config,
            write_log,
            read_view,
            running: Arc::new(AtomicBool::new(false)),
            thread_handle: None,
            reconciliations: Arc::new(AtomicU64::new(0)),
            entries_processed: Arc::new(AtomicU64::new(0)),
            disk_flushes: Arc::new(AtomicU64::new(0)),
        }
    }
    
    /// Start reconciliation thread
    pub fn start(&mut self) {
        if self.running.load(Ordering::Relaxed) {
            return; // Already running
        }
        
        self.running.store(true, Ordering::Relaxed);
        
        let config = self.config.clone();
        let write_log = Arc::clone(&self.write_log);
        let read_view = Arc::clone(&self.read_view);
        let running = Arc::clone(&self.running);
        let reconciliations = Arc::clone(&self.reconciliations);
        let entries_processed = Arc::clone(&self.entries_processed);
        let disk_flushes = Arc::clone(&self.disk_flushes);
        
        let handle = thread::spawn(move || {
            reconcile_loop(
                config,
                write_log,
                read_view,
                running,
                reconciliations,
                entries_processed,
                disk_flushes,
            );
        });
        
        self.thread_handle = Some(handle);
    }
    
    /// Stop reconciliation thread
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        
        if let Some(handle) = self.thread_handle.take() {
            handle.join().ok();
        }
    }
    
    /// Get reconciler statistics
    pub fn stats(&self) -> ReconcilerStats {
        ReconcilerStats {
            reconciliations: self.reconciliations.load(Ordering::Relaxed),
            entries_processed: self.entries_processed.load(Ordering::Relaxed),
            disk_flushes: self.disk_flushes.load(Ordering::Relaxed),
            running: self.running.load(Ordering::Relaxed),
        }
    }
}

impl Drop for Reconciler {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Reconciler statistics
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct ReconcilerStats {
    pub reconciliations: u64,
    pub entries_processed: u64,
    pub disk_flushes: u64,
    pub running: bool,
}

/// Main reconciliation loop
fn reconcile_loop(
    config: ReconcilerConfig,
    write_log: Arc<WriteLog>,
    read_view: Arc<ReadView>,
    running: Arc<AtomicBool>,
    reconciliations: Arc<AtomicU64>,
    entries_processed: Arc<AtomicU64>,
    disk_flushes: Arc<AtomicU64>,
) {
    let interval = Duration::from_millis(config.reconcile_interval_ms);
    let mut next_segment_id = 0u32;
    
    while running.load(Ordering::Relaxed) {
        // Drain write log
        let batch = write_log.drain_batch(config.max_batch_size);
        
        if !batch.is_empty() {
            // Load current snapshot
            let current_snapshot = read_view.load();
            
            // CRITICAL: Build a truly new immutable snapshot
            // Clone the entire im::HashMap (cheap due to structural sharing)
            let mut new_snapshot = GraphSnapshot {
                concepts: current_snapshot.concepts.clone(),
                sequence: current_snapshot.sequence + 1,
                timestamp: current_timestamp_us(),
                concept_count: current_snapshot.concept_count,
                edge_count: current_snapshot.edge_count,
            };
            
            // Apply batch to the new mutable copy
            for entry in &batch {
                apply_entry(&mut new_snapshot, entry);
            }
            
            // Update stats
            new_snapshot.update_stats();
            
            // Atomic swap
            read_view.store(new_snapshot);
            
            // Update metrics
            reconciliations.fetch_add(1, Ordering::Relaxed);
            entries_processed.fetch_add(batch.len() as u64, Ordering::Relaxed);
            
            // Check if we need to flush to disk
            let snap = read_view.load();
            if snap.concept_count >= config.disk_flush_threshold {
                if flush_to_disk(&snap, &config.storage_path, next_segment_id).is_ok() {
                    next_segment_id += 1;
                    disk_flushes.fetch_add(1, Ordering::Relaxed);
                }
            }
        }
        
        // Sleep before next reconciliation
        thread::sleep(interval);
    }
}

/// Apply a single write entry to the snapshot
/// CRITICAL: snapshot is now &mut to allow proper immutable updates
fn apply_entry(snapshot: &mut GraphSnapshot, entry: &WriteEntry) {
    match entry {
        WriteEntry::AddConcept {
            id,
            content,
            vector,
            strength,
            confidence,
            timestamp,
        } => {
            let node = ConceptNode::new(
                *id,
                content.to_vec(),
                vector.as_ref().map(|v| v.to_vec()),
                *strength,
                *confidence,
                *timestamp,
            );
            
            snapshot.concepts.insert(*id, node);
        }
        
        WriteEntry::AddAssociation { record } => {
            // Add edge to source concept (clone-update pattern for immutable map)
            if let Some(mut source_node) = snapshot.concepts.get(&record.source_id).cloned() {
                source_node.add_edge(record.target_id, *record);
                snapshot.concepts.insert(record.source_id, source_node);
            }
            
            // Add reverse edge to target concept (for bidirectional queries)
            if let Some(mut target_node) = snapshot.concepts.get(&record.target_id).cloned() {
                target_node.add_edge(record.source_id, *record);
                snapshot.concepts.insert(record.target_id, target_node);
            }
        }
        
        WriteEntry::UpdateStrength { id, strength } => {
            if let Some(mut node) = snapshot.concepts.get(id).cloned() {
                node.strength = *strength;
                snapshot.concepts.insert(*id, node);
            }
        }
        
        WriteEntry::RecordAccess { id, timestamp } => {
            if let Some(mut node) = snapshot.concepts.get(id).cloned() {
                node.last_accessed = *timestamp;
                node.access_count += 1;
                snapshot.concepts.insert(*id, node);
            }
        }
        
        WriteEntry::BatchMarker { .. } => {
            // Marker only, no action
        }
    }
}

/// PRODUCTION: Flush snapshot to disk with complete SUTRA binary format including vectors
pub fn flush_to_disk(
    snapshot: &GraphSnapshot,
    base_path: &PathBuf,
    _segment_id: u32,
) -> anyhow::Result<()> {
    use std::fs::File;
    use std::io::{BufWriter, Write};
    
    // Single-file storage: storage.dat with SUTRA binary format
    let path = base_path.join("storage.dat");
    
    // Create backup of existing file if it exists
    if path.exists() {
        let backup_path = base_path.join("storage.dat.backup");
        std::fs::copy(&path, &backup_path)?;
    }
    
    let file = File::create(&path)?;
    let mut writer = BufWriter::new(file);
    
    // Collect data before writing
    let concepts: Vec<_> = snapshot.concepts.iter().map(|(_id, node)| node.clone()).collect();
    let mut edges = Vec::new();
    let mut vectors = Vec::new();
    
    // Collect all edges and vectors
    for concept in &concepts {
        // Collect edges
        for (i, neighbor) in concept.neighbors.iter().enumerate() {
            if i < concept.associations.len() {
                let confidence = concept.associations[i].confidence;
                edges.push((concept.id, *neighbor, confidence));
            }
        }
        
        // Collect vectors (CRITICAL: vectors must be persisted)
        if let Some(vector) = concept.vector.as_ref() {
            let v: Vec<f32> = vector.iter().copied().collect();
            vectors.push((concept.id, v));
        }
    }
    
    // Write SUTRA binary format header (64 bytes) - VERSION 2 with vectors
    writer.write_all(b"SUTRADAT")?; // Magic bytes (8 bytes)
    writer.write_all(&2u32.to_le_bytes())?; // Version 2 - includes vectors (4 bytes)
    writer.write_all(&(concepts.len() as u32).to_le_bytes())?; // Concept count (4 bytes)
    writer.write_all(&(edges.len() as u32).to_le_bytes())?; // Edge count (4 bytes)
    writer.write_all(&(vectors.len() as u32).to_le_bytes())?; // Vector count (4 bytes)
    writer.write_all(&current_timestamp_us().to_le_bytes())?; // Timestamp (8 bytes)
    writer.write_all(&[0u8; 32])?; // Reserved (32 bytes) = Total 64 bytes
    
    log::info!("📄 Writing SUTRA binary format v2: {} concepts, {} edges, {} vectors", 
               concepts.len(), edges.len(), vectors.len());
    
    // Write concepts section (metadata only)
    for concept in &concepts {
        // Concept header (36 bytes): ID(16) + content_len(4) + strength(4) + confidence(4) + access_count(4) + created(4)
        writer.write_all(&concept.id.0)?; // ID (16 bytes)
        writer.write_all(&(concept.content.len() as u32).to_le_bytes())?; // Content length (4 bytes)
        writer.write_all(&concept.strength.to_le_bytes())?; // Strength (4 bytes)
        writer.write_all(&concept.confidence.to_le_bytes())?; // Confidence (4 bytes)
        writer.write_all(&concept.access_count.to_le_bytes())?; // Access count (4 bytes)
        writer.write_all(&(concept.created as u32).to_le_bytes())?; // Created timestamp (4 bytes)
        
        // Content data (variable length)
        writer.write_all(&concept.content)?;
    }
    
    // Write edges section
    for (source_id, target_id, confidence) in &edges {
        writer.write_all(&source_id.0)?; // Source ID (16 bytes)
        writer.write_all(&target_id.0)?; // Target ID (16 bytes)
        writer.write_all(&confidence.to_le_bytes())?; // Confidence (4 bytes)
    }
    
    // PRODUCTION: Write vectors section (NEW in v2)
    for (concept_id, vector) in &vectors {
        writer.write_all(&concept_id.0)?; // Concept ID (16 bytes)
        writer.write_all(&(vector.len() as u32).to_le_bytes())?; // Vector dimension (4 bytes)
        
        // Write vector data (dimension * 4 bytes for f32)
        for &component in vector.iter() {
            writer.write_all(&component.to_le_bytes())?; // f32 component (4 bytes)
        }
    }
    
    // Ensure data is written to disk
    writer.flush()?;
    drop(writer); // Close file
    
    // Sync to disk
    let file = File::open(&path)?;
    file.sync_all()?;
    
    let file_size_mb = path.metadata()?.len() as f64 / (1024.0 * 1024.0);
    log::info!("✅ PRODUCTION: Wrote {:.2} MB to storage.dat (v2 with {} vectors)", file_size_mb, vectors.len());
    
    Ok(())
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
    use crate::types::{AssociationType, ConceptId};
    use crate::write_log::WriteLog;
    use crate::read_view::ReadView;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;
    use tempfile::TempDir;
    
    #[test]
    fn test_reconciler_basic() {
        let write_log = Arc::new(WriteLog::new());
        let read_view = Arc::new(ReadView::new());
        
        let dir = TempDir::new().unwrap();
        let config = ReconcilerConfig {
            reconcile_interval_ms: 50,
            storage_path: dir.path().to_path_buf(),
            ..Default::default()
        };
        
        let mut reconciler = Reconciler::new(config, Arc::clone(&write_log), Arc::clone(&read_view));
        
        reconciler.start();
        
        // Write some entries
        let id = ConceptId([1; 16]);
        write_log.append_concept(id, vec![1, 2, 3], None, 1.0, 0.9).unwrap();
        
        // Wait for reconciliation
        thread::sleep(Duration::from_millis(100));
        
        // Check read view
        let snapshot = read_view.load();
        assert!(snapshot.contains(&id));
        
        reconciler.stop();
    }
    
    #[test]
    fn test_association_reconciliation() {
        let write_log = Arc::new(WriteLog::new());
        let read_view = Arc::new(ReadView::new());
        
        let dir = TempDir::new().unwrap();
        let config = ReconcilerConfig {
            reconcile_interval_ms: 50,
            storage_path: dir.path().to_path_buf(),
            ..Default::default()
        };
        
        let mut reconciler = Reconciler::new(config, Arc::clone(&write_log), Arc::clone(&read_view));
        reconciler.start();
        
        let id1 = ConceptId([1; 16]);
        let id2 = ConceptId([2; 16]);
        
        // Add concepts
        write_log.append_concept(id1, vec![1], None, 1.0, 0.9).unwrap();
        write_log.append_concept(id2, vec![2], None, 1.0, 0.9).unwrap();
        
        // Add association
        let assoc = crate::types::AssociationRecord::new(
            id1,
            id2,
            AssociationType::Semantic,
            0.8,
        );
        write_log.append_association(assoc).unwrap();
        
        // Wait for reconciliation
        thread::sleep(Duration::from_millis(150));
        
        // Check neighbors
        let neighbors = read_view.get_neighbors(&id1);
        assert!(neighbors.contains(&id2));
        
        reconciler.stop();
    }
    
    #[test]
    fn test_reconciler_stats() {
        let write_log = Arc::new(WriteLog::new());
        let read_view = Arc::new(ReadView::new());
        
        let dir = TempDir::new().unwrap();
        let config = ReconcilerConfig {
            reconcile_interval_ms: 50,
            storage_path: dir.path().to_path_buf(),
            ..Default::default()
        };
        
        let mut reconciler = Reconciler::new(config, Arc::clone(&write_log), Arc::clone(&read_view));
        reconciler.start();
        
        // Write entries
        for i in 0..10 {
            let id = ConceptId([i; 16]);
            write_log.append_concept(id, vec![i], None, 1.0, 0.9).unwrap();
        }
        
        // Wait for reconciliation
        thread::sleep(Duration::from_millis(150));
        
        let stats = reconciler.stats();
        assert!(stats.reconciliations > 0);
        assert!(stats.entries_processed >= 10);
        assert!(stats.running);
        
        reconciler.stop();
    }
}
