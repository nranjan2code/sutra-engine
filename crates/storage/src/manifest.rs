/// Manifest file for tracking segments in LSM-tree
/// 
/// The manifest is a simple JSON file that tracks all active segments,
/// their metadata, and compaction history. It's atomically updated
/// to ensure consistency.
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

/// Metadata about a segment in the LSM-tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentMetadata {
    /// Segment ID
    pub segment_id: u32,
    /// File path relative to store directory
    pub path: PathBuf,
    /// Number of concepts in segment
    pub concept_count: u32,
    /// Number of associations in segment
    pub association_count: u32,
    /// File size in bytes
    pub file_size: u64,
    /// Creation timestamp
    pub created_at: u64,
    /// Last compaction timestamp (0 if never compacted)
    pub compacted_at: u64,
    /// Level in LSM-tree (0 = newest)
    pub level: u32,
}

impl SegmentMetadata {
    pub fn new(segment_id: u32, path: PathBuf, level: u32) -> Self {
        Self {
            segment_id,
            path,
            concept_count: 0,
            association_count: 0,
            file_size: 0,
            created_at: current_timestamp(),
            compacted_at: 0,
            level,
        }
    }
}

/// Manifest tracks all segments and compaction history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// Format version
    pub version: u32,
    /// Next segment ID to allocate
    pub next_segment_id: u32,
    /// All active segments (ordered by level, then age)
    pub segments: Vec<SegmentMetadata>,
    /// Last compaction timestamp
    pub last_compaction: u64,
    /// Total compactions performed
    pub compaction_count: u64,
}

impl Manifest {
    /// Create a new empty manifest
    pub fn new() -> Self {
        Self {
            version: 1,
            next_segment_id: 0,
            segments: Vec::new(),
            last_compaction: 0,
            compaction_count: 0,
        }
    }
    
    /// Load manifest from file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path.as_ref())
            .context("Failed to open manifest file")?;
        let reader = BufReader::new(file);
        let manifest = serde_json::from_reader(reader)
            .context("Failed to parse manifest JSON")?;
        Ok(manifest)
    }
    
    /// Save manifest to file atomically
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        let temp_path = path.with_extension("tmp");
        
        // Write to temporary file
        {
            let file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&temp_path)
                .context("Failed to create temp manifest file")?;
            
            let mut writer = BufWriter::new(file);
            serde_json::to_writer_pretty(&mut writer, self)
                .context("Failed to serialize manifest")?;
            writer.flush()?;
            writer.get_ref().sync_all()?;
        }
        
        // Atomic rename
        std::fs::rename(&temp_path, path)
            .context("Failed to rename manifest file")?;
        
        Ok(())
    }
    
    /// Allocate a new segment ID
    pub fn allocate_segment_id(&mut self) -> u32 {
        let id = self.next_segment_id;
        self.next_segment_id += 1;
        id
    }
    
    /// Add a new segment to the manifest
    pub fn add_segment(&mut self, metadata: SegmentMetadata) {
        self.segments.push(metadata);
        self.sort_segments();
    }
    
    /// Remove segments by ID
    pub fn remove_segments(&mut self, segment_ids: &[u32]) {
        self.segments.retain(|s| !segment_ids.contains(&s.segment_id));
    }
    
    /// Update segment metadata
    pub fn update_segment(&mut self, segment_id: u32, update: impl FnOnce(&mut SegmentMetadata)) {
        if let Some(segment) = self.segments.iter_mut().find(|s| s.segment_id == segment_id) {
            update(segment);
        }
    }
    
    /// Get all segments at a specific level
    pub fn segments_at_level(&self, level: u32) -> Vec<&SegmentMetadata> {
        self.segments
            .iter()
            .filter(|s| s.level == level)
            .collect()
    }
    
    /// Get total number of concepts across all segments
    pub fn total_concepts(&self) -> u64 {
        self.segments.iter().map(|s| s.concept_count as u64).sum()
    }
    
    /// Get total file size across all segments
    pub fn total_size(&self) -> u64 {
        self.segments.iter().map(|s| s.file_size).sum()
    }
    
    /// Sort segments by level (ascending) and age (descending)
    fn sort_segments(&mut self) {
        self.segments.sort_by(|a, b| {
            a.level.cmp(&b.level)
                .then_with(|| b.created_at.cmp(&a.created_at))
        });
    }
    
    /// Record a compaction
    pub fn record_compaction(&mut self) {
        self.last_compaction = current_timestamp();
        self.compaction_count += 1;
    }
}

impl Default for Manifest {
    fn default() -> Self {
        Self::new()
    }
}

/// Get current timestamp in milliseconds
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_create_manifest() {
        let manifest = Manifest::new();
        assert_eq!(manifest.version, 1);
        assert_eq!(manifest.next_segment_id, 0);
        assert_eq!(manifest.segments.len(), 0);
    }
    
    #[test]
    fn test_allocate_segment_id() {
        let mut manifest = Manifest::new();
        assert_eq!(manifest.allocate_segment_id(), 0);
        assert_eq!(manifest.allocate_segment_id(), 1);
        assert_eq!(manifest.allocate_segment_id(), 2);
    }
    
    #[test]
    fn test_add_remove_segments() {
        let mut manifest = Manifest::new();
        
        let seg1 = SegmentMetadata::new(0, PathBuf::from("seg0.dat"), 0);
        let seg2 = SegmentMetadata::new(1, PathBuf::from("seg1.dat"), 0);
        
        manifest.add_segment(seg1);
        manifest.add_segment(seg2);
        assert_eq!(manifest.segments.len(), 2);
        
        manifest.remove_segments(&[0]);
        assert_eq!(manifest.segments.len(), 1);
        assert_eq!(manifest.segments[0].segment_id, 1);
    }
    
    #[test]
    fn test_save_load_manifest() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("manifest.json");
        
        let mut manifest = Manifest::new();
        manifest.add_segment(SegmentMetadata::new(0, PathBuf::from("seg0.dat"), 0));
        manifest.add_segment(SegmentMetadata::new(1, PathBuf::from("seg1.dat"), 1));
        
        manifest.save(&path).unwrap();
        
        let loaded = Manifest::load(&path).unwrap();
        assert_eq!(loaded.version, manifest.version);
        assert_eq!(loaded.segments.len(), 2);
        assert_eq!(loaded.segments[0].segment_id, 0);
    }
    
    #[test]
    fn test_segment_sorting() {
        let mut manifest = Manifest::new();
        
        // Add segments out of order
        let seg2 = SegmentMetadata::new(2, PathBuf::from("seg2.dat"), 1);
        let seg0 = SegmentMetadata::new(0, PathBuf::from("seg0.dat"), 0);
        let seg1 = SegmentMetadata::new(1, PathBuf::from("seg1.dat"), 0);
        
        manifest.add_segment(seg2);
        manifest.add_segment(seg0);
        manifest.add_segment(seg1);
        
        // Should be sorted by level, then by age
        assert_eq!(manifest.segments[0].level, 0);
        assert_eq!(manifest.segments[1].level, 0);
        assert_eq!(manifest.segments[2].level, 1);
    }
    
    #[test]
    fn test_update_segment() {
        let mut manifest = Manifest::new();
        manifest.add_segment(SegmentMetadata::new(0, PathBuf::from("seg0.dat"), 0));
        
        manifest.update_segment(0, |seg| {
            seg.concept_count = 100;
            seg.file_size = 1024;
        });
        
        assert_eq!(manifest.segments[0].concept_count, 100);
        assert_eq!(manifest.segments[0].file_size, 1024);
    }
}
