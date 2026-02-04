/// LSM-tree compaction for segment management
///
/// Implements a simple LSM-tree with multiple levels:
/// - Level 0: Active segment (writes go here)
/// - Level 1+: Immutable segments (compacted periodically)
///
/// Compaction strategy:
/// - When Level 0 reaches threshold, compact into Level 1
/// - When Level N reaches threshold, compact into Level N+1
/// - Merge overlapping segments to reduce amplification
use crate::index::{ConceptLocation, GraphIndex};
use crate::manifest::{Manifest, SegmentMetadata};
use crate::segment::Segment;
use crate::types::{ConceptId, ConceptRecord};
use crate::wal::{Operation, WriteAheadLog};
use anyhow::Result;
use dashmap::DashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Compaction configuration
#[derive(Debug, Clone)]
pub struct CompactionConfig {
    /// Maximum segments per level before compaction
    pub level_size_multiplier: u32,
    /// Minimum segments to trigger compaction
    pub compaction_threshold: usize,
    /// Maximum file size for a segment (bytes)
    pub max_segment_size: u64,
    /// Compaction check interval (seconds)
    pub check_interval: u64,
}

impl Default for CompactionConfig {
    fn default() -> Self {
        Self {
            level_size_multiplier: 10,
            compaction_threshold: 4,
            max_segment_size: 64 * 1024 * 1024, // 64 MB
            check_interval: 300,                // 5 minutes
        }
    }
}

/// LSM-tree manager with compaction
pub struct LSMTree {
    /// Base directory for segments
    base_path: PathBuf,
    /// Manifest file
    manifest: Arc<parking_lot::RwLock<Manifest>>,
    /// Active segment (Level 0)
    active_segment: Arc<parking_lot::RwLock<Option<Segment>>>,
    /// Open segments for reading (cached)
    segment_cache: Arc<DashMap<u32, Arc<Segment>>>,
    /// Index for fast lookups
    index: Arc<GraphIndex>,
    /// Write-Ahead Log
    wal: Arc<parking_lot::RwLock<WriteAheadLog>>,
    /// Compaction configuration
    config: CompactionConfig,
}

impl LSMTree {
    /// Create or open an LSM-tree
    pub fn open<P: AsRef<Path>>(path: P, config: CompactionConfig) -> Result<Self> {
        let base_path = path.as_ref().to_path_buf();
        std::fs::create_dir_all(&base_path)?;

        let manifest_path = base_path.join("manifest.json");
        let manifest = if manifest_path.exists() {
            Manifest::load(&manifest_path)?
        } else {
            Manifest::new()
        };

        // Open or create WAL
        let wal_path = base_path.join("wal.log");
        let wal = if wal_path.exists() {
            WriteAheadLog::open(&wal_path, true)?
        } else {
            WriteAheadLog::create(&wal_path, true)?
        };

        let lsm = Self {
            base_path,
            manifest: Arc::new(parking_lot::RwLock::new(manifest)),
            active_segment: Arc::new(parking_lot::RwLock::new(None)),
            segment_cache: Arc::new(DashMap::new()),
            index: Arc::new(GraphIndex::new()),
            wal: Arc::new(parking_lot::RwLock::new(wal)),
            config,
        };

        // Replay WAL on startup
        lsm.replay_wal()?;

        // Initialize active segment
        lsm.rotate_active_segment()?;

        // Rebuild index from existing segments
        lsm.rebuild_index()?;

        Ok(lsm)
    }

    /// Get or create the active segment
    fn get_or_create_active(&self) -> Result<()> {
        let active = self.active_segment.read();

        if active.is_none() {
            drop(active); // Release read lock
            self.rotate_active_segment()?;
        }

        Ok(())
    }

    /// Rotate the active segment (seal current, create new)
    fn rotate_active_segment(&self) -> Result<()> {
        let mut active = self.active_segment.write();

        // Seal current active segment if it exists
        if let Some(ref mut segment) = *active {
            segment.sync()?;

            // Update manifest
            let mut manifest = self.manifest.write();
            let segment_id = segment.segment_id();
            manifest.update_segment(segment_id, |meta| {
                let stats = segment.stats();
                meta.concept_count = stats.concept_count;
                meta.association_count = stats.association_count;
                meta.file_size = stats.file_size;
            });
            self.save_manifest(&manifest)?;
        }

        // Create new active segment
        let mut manifest = self.manifest.write();
        let segment_id = manifest.allocate_segment_id();
        let segment_path = self.base_path.join(format!("seg_{:08}.dat", segment_id));

        let new_segment = Segment::create(&segment_path, segment_id)?;

        // Add to manifest
        let metadata = SegmentMetadata::new(
            segment_id,
            segment_path.file_name().unwrap().into(),
            0, // Level 0 (active)
        );
        manifest.add_segment(metadata);
        self.save_manifest(&manifest)?;

        *active = Some(new_segment);

        Ok(())
    }

    /// Write a concept to the active segment
    ///
    /// Note: This is a legacy implementation. Production code uses `ConcurrentMemory`
    /// which provides lock-free writes via `WriteLog` and atomic reconciliation.
    pub fn write_concept(&self, _record: ConceptRecord, _content: &str) -> Result<u64> {
        self.get_or_create_active()?;

        // Legacy: Write locking not implemented in this simplified version
        // Modern architecture uses WriteLog with lock-free append-only writes

        Ok(0) // Placeholder - not used in production
    }

    /// Read a concept by ID (O(1) with index)
    pub fn read_concept(&self, id: ConceptId) -> Result<Option<ConceptRecord>> {
        // Use index for O(1) lookup
        if let Some(location) = self.index.lookup_concept(id) {
            let segment = self.get_segment(location.segment_id)?;
            let record = segment.read_concept(location.offset)?;
            return Ok(Some(record));
        }

        Ok(None)
    }

    /// Rebuild index from all segments
    fn rebuild_index(&self) -> Result<()> {
        let manifest = self.manifest.read();

        for seg_meta in manifest.segments.iter() {
            let segment = self.get_segment(seg_meta.segment_id)?;

            for concept in segment.iter_concepts()? {
                // Note: Offset tracking not implemented in this legacy version
                // Modern ConcurrentMemory uses more efficient indexing
                let location = ConceptLocation::new(seg_meta.segment_id, 0);
                self.index
                    .insert_concept(concept.concept_id, location, concept.created);
            }
        }

        Ok(())
    }

    /// Replay WAL on startup
    fn replay_wal(&self) -> Result<()> {
        let wal_path = self.base_path.join("wal.log");
        if !wal_path.exists() {
            return Ok(());
        }

        let committed = WriteAheadLog::replay(&wal_path)?;

        // Apply committed operations
        // Note: This is a simplified replay - in production we'd need proper recovery
        for entry in committed {
            match entry.operation {
                Operation::WriteConcept {
                    concept_id,
                    created,
                    ..
                } => {
                    // Update index (actual data is in segments)
                    let location = ConceptLocation::new(0, 0); // Placeholder
                    self.index.insert_concept(concept_id, location, created);
                }
                Operation::DeleteConcept { concept_id } => {
                    self.index.remove_concept(concept_id);
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Log operation to WAL before applying
    pub fn log_write_concept(
        &self,
        concept_id: ConceptId,
        content_len: u32,
        vector_len: u32,
        created: u64,
        modified: u64,
    ) -> Result<()> {
        let mut wal = self.wal.write();
        wal.append(Operation::WriteConcept {
            concept_id,
            content_len,
            vector_len,
            created,
            modified,
        })?;
        Ok(())
    }

    /// Get the WAL (for transactions)
    pub fn wal(&self) -> Arc<parking_lot::RwLock<WriteAheadLog>> {
        self.wal.clone()
    }

    /// Checkpoint: flush WAL and truncate after data is persisted
    pub fn checkpoint(&self) -> Result<()> {
        // Sync manifest
        let manifest = self.manifest.read();
        let manifest_path = self.base_path.join("manifest.json");
        manifest.save(&manifest_path)?;
        drop(manifest);

        // Truncate WAL (all data is persisted)
        let mut wal = self.wal.write();
        wal.truncate()?;

        Ok(())
    }

    /// Get the index (for direct queries)
    pub fn index(&self) -> &Arc<GraphIndex> {
        &self.index
    }

    /// Get a segment from cache or open it
    fn get_segment(&self, segment_id: u32) -> Result<Arc<Segment>> {
        if let Some(segment) = self.segment_cache.get(&segment_id) {
            return Ok(Arc::clone(&segment));
        }

        let manifest = self.manifest.read();
        let seg_meta = manifest
            .segments
            .iter()
            .find(|s| s.segment_id == segment_id)
            .ok_or_else(|| anyhow::anyhow!("Segment {} not found", segment_id))?;

        let segment_path = self.base_path.join(&seg_meta.path);
        let segment = Segment::open_read(segment_path)?;
        let segment = Arc::new(segment);

        self.segment_cache.insert(segment_id, Arc::clone(&segment));

        Ok(segment)
    }

    /// Check if compaction is needed
    pub fn needs_compaction(&self) -> bool {
        let manifest = self.manifest.read();

        // Check each level
        for level in 0..10 {
            let segments = manifest.segments_at_level(level);
            let threshold = self.config.compaction_threshold
                * (self.config.level_size_multiplier.pow(level) as usize);

            if segments.len() >= threshold {
                return true;
            }
        }

        false
    }

    /// Perform compaction on a specific level
    pub fn compact_level(&self, level: u32) -> Result<()> {
        let segment_ids: Vec<u32> = {
            let manifest = self.manifest.read();
            let segments = manifest.segments_at_level(level);

            if segments.is_empty() {
                return Ok(());
            }

            // Select segments to compact (oldest ones)
            segments
                .iter()
                .take(self.config.compaction_threshold)
                .map(|s| s.segment_id)
                .collect()
        }; // manifest lock released here

        if segment_ids.is_empty() {
            return Ok(());
        }

        // Perform the merge
        self.merge_segments(&segment_ids, level + 1)?;

        Ok(())
    }

    /// Merge multiple segments into a new segment at target level
    fn merge_segments(&self, segment_ids: &[u32], target_level: u32) -> Result<()> {
        // Create new segment
        let mut manifest = self.manifest.write();
        let new_segment_id = manifest.allocate_segment_id();
        let new_path = self
            .base_path
            .join(format!("seg_{:08}.dat", new_segment_id));

        let mut new_segment = Segment::create(&new_path, new_segment_id)?;

        // Merge concepts from all source segments
        let concept_map: DashMap<ConceptId, ConceptRecord> = DashMap::new();

        for &segment_id in segment_ids {
            let segment = self.get_segment(segment_id)?;

            for concept in segment.iter_concepts()? {
                // Keep newest version (last wins)
                concept_map.insert(concept.concept_id, concept);
            }
        }

        // Write merged concepts to new segment
        for entry in concept_map.iter() {
            new_segment.append_concept(*entry.value())?;
        }

        new_segment.sync()?;

        // Update manifest
        let metadata = SegmentMetadata::new(
            new_segment_id,
            new_path.file_name().unwrap().into(),
            target_level,
        );

        let stats = new_segment.stats();
        manifest.update_segment(new_segment_id, |meta| {
            meta.concept_count = stats.concept_count;
            meta.file_size = stats.file_size;
        });

        manifest.add_segment(metadata);
        manifest.remove_segments(segment_ids);
        manifest.record_compaction();

        self.save_manifest(&manifest)?;

        // Remove old segments from cache
        for &segment_id in segment_ids {
            self.segment_cache.remove(&segment_id);
        }

        // Delete old segment files
        for &segment_id in segment_ids {
            let old_path = self.base_path.join(format!("seg_{:08}.dat", segment_id));
            if old_path.exists() {
                std::fs::remove_file(old_path)?;
            }
        }

        Ok(())
    }

    /// Start background compaction thread
    pub fn start_background_compaction(self: Arc<Self>) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(self.config.check_interval));

                if self.needs_compaction() {
                    // Find first level that needs compaction
                    for level in 0..10 {
                        let needs_compact = {
                            let manifest = self.manifest.read();
                            let segments = manifest.segments_at_level(level);
                            let threshold = self.config.compaction_threshold
                                * (self.config.level_size_multiplier.pow(level) as usize);
                            segments.len() >= threshold
                        }; // manifest lock released here

                        if needs_compact {
                            if let Err(e) = self.compact_level(level) {
                                eprintln!("Compaction error at level {}: {}", level, e);
                            }
                            break;
                        }
                    }
                }
            }
        })
    }

    /// Get LSM-tree statistics
    pub fn stats(&self) -> LSMStats {
        let manifest = self.manifest.read();

        let mut level_counts = vec![0; 10];
        for seg in &manifest.segments {
            if (seg.level as usize) < level_counts.len() {
                level_counts[seg.level as usize] += 1;
            }
        }

        let index_stats = self.index.stats();

        LSMStats {
            total_segments: manifest.segments.len(),
            total_concepts: manifest.total_concepts(),
            total_size: manifest.total_size(),
            level_counts,
            last_compaction: manifest.last_compaction,
            compaction_count: manifest.compaction_count,
            indexed_concepts: index_stats.total_concepts,
            indexed_edges: index_stats.total_edges,
        }
    }

    /// Save manifest to disk
    fn save_manifest(&self, manifest: &Manifest) -> Result<()> {
        let manifest_path = self.base_path.join("manifest.json");
        manifest.save(manifest_path)
    }

    /// Sync all segments to disk
    pub fn sync(&self) -> Result<()> {
        if let Some(ref mut active) = *self.active_segment.write() {
            active.sync()?;
        }

        let manifest = self.manifest.read();
        self.save_manifest(&manifest)?;

        Ok(())
    }
}

/// LSM-tree statistics
#[derive(Debug, Clone)]
pub struct LSMStats {
    pub total_segments: usize,
    pub total_concepts: u64,
    pub total_size: u64,
    pub level_counts: Vec<usize>,
    pub last_compaction: u64,
    pub compaction_count: u64,
    pub indexed_concepts: u64,
    pub indexed_edges: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_lsm_tree() {
        let dir = TempDir::new().unwrap();
        let config = CompactionConfig::default();

        let lsm = LSMTree::open(dir.path(), config).unwrap();
        let stats = lsm.stats();

        assert_eq!(stats.total_segments, 1); // Active segment
        assert_eq!(stats.level_counts[0], 1);
    }

    #[test]
    fn test_rotate_segment() {
        let dir = TempDir::new().unwrap();
        let config = CompactionConfig::default();

        let lsm = LSMTree::open(dir.path(), config).unwrap();
        lsm.rotate_active_segment().unwrap();

        let stats = lsm.stats();
        assert_eq!(stats.total_segments, 2); // Two segments now
    }

    #[test]
    fn test_needs_compaction() {
        let dir = TempDir::new().unwrap();
        let mut config = CompactionConfig::default();
        config.compaction_threshold = 2; // Low threshold for testing

        let lsm = LSMTree::open(dir.path(), config).unwrap();

        // Initially no compaction needed
        assert!(!lsm.needs_compaction());

        // Add more segments
        lsm.rotate_active_segment().unwrap();
        lsm.rotate_active_segment().unwrap();

        // Now compaction should be needed
        assert!(lsm.needs_compaction());
    }

    #[test]
    fn test_index_integration() {
        let dir = TempDir::new().unwrap();
        let config = CompactionConfig::default();

        let lsm = LSMTree::open(dir.path(), config).unwrap();

        // Access index through LSM
        let index = lsm.index();

        // Should be empty initially (no concepts written yet)
        let stats = index.stats();
        assert_eq!(stats.total_concepts, 0);
    }

    #[test]
    fn test_wal_integration() {
        let dir = TempDir::new().unwrap();
        let config = CompactionConfig::default();

        let lsm = LSMTree::open(dir.path(), config).unwrap();

        // Log a write operation
        let concept_id = ConceptId([1; 16]);
        lsm.log_write_concept(concept_id, 100, 384, 1000, 1000)
            .unwrap();

        // WAL should have one entry
        let wal = lsm.wal();
        let wal_guard = wal.read();
        assert_eq!(wal_guard.sequence(), 1);
        drop(wal_guard);

        // Checkpoint should truncate WAL
        lsm.checkpoint().unwrap();

        let wal_guard = wal.read();
        assert_eq!(wal_guard.sequence(), 0);
    }

    #[test]
    fn test_crash_recovery() {
        let dir = TempDir::new().unwrap();
        let config = CompactionConfig::default();

        // Write to WAL
        {
            let lsm = LSMTree::open(dir.path(), config.clone()).unwrap();
            let concept_id = ConceptId([42; 16]);
            lsm.log_write_concept(concept_id, 200, 384, 2000, 2000)
                .unwrap();
            // Simulate crash (don't checkpoint)
        }

        // Reopen - should replay WAL
        let lsm = LSMTree::open(dir.path(), config).unwrap();
        let wal = lsm.wal();
        let wal_guard = wal.read();

        // WAL should still have the entry (not checkpointed)
        assert_eq!(wal_guard.sequence(), 1);
    }
}
