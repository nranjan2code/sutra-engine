/// Segment-based storage with memory-mapped reads
/// 
/// File Format:
/// ┌─────────────────┬──────────────┬─────────────────┬──────────────┬──────────────┐
/// │ SegmentHeader   │ Concept[]    │ Association[]   │ Vector[]     │ Content[]    │
/// │ (256 bytes)     │ (128B each)  │ (64B each)      │ (variable)   │ (variable)   │
/// └─────────────────┴──────────────┴─────────────────┴──────────────┴──────────────┘
use crate::types::*;
use anyhow::{Context, Result};
use bytemuck::{cast_slice, from_bytes, Pod, Zeroable};
use memmap2::{Mmap, MmapOptions};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Magic bytes to identify Sutra segment files: "SUTRA" in hex
const MAGIC_BYTES: [u8; 8] = [0x53, 0x55, 0x54, 0x52, 0x41, 0x00, 0x00, 0x00];
const VERSION: u32 = 1;
const HEADER_SIZE: usize = 256;

/// Segment file header (256 bytes, aligned)
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C, packed)]
pub struct SegmentHeader {
    /// Magic bytes for file identification
    pub magic: [u8; 8],               // 8 bytes
    /// Format version
    pub version: u32,                 // 4 bytes
    /// Segment ID (monotonic counter)
    pub segment_id: u32,              // 4 bytes
    
    // Block offsets and sizes
    pub concept_offset: u64,          // 8 bytes
    pub concept_count: u32,           // 4 bytes
    pub association_offset: u64,      // 8 bytes
    pub association_count: u32,       // 4 bytes
    pub vector_offset: u64,           // 8 bytes
    pub vector_count: u32,            // 4 bytes
    pub content_offset: u64,          // 8 bytes
    pub content_length: u32,          // 4 bytes
    
    // Timestamps
    pub created_at: u64,              // 8 bytes
    pub compacted_at: u64,            // 8 bytes
    
    // Checksums (CRC32)
    pub concept_checksum: u32,        // 4 bytes
    pub association_checksum: u32,    // 4 bytes
    pub content_checksum: u32,        // 4 bytes
    pub padding: u32,                 // 4 bytes for alignment
    
    // Reserved for future use (split into multiple arrays for bytemuck)
    pub reserved1: [u8; 32],
    pub reserved2: [u8; 32],
    pub reserved3: [u8; 32],
    pub reserved4: [u8; 32],
    pub reserved5: [u8; 32],
}  // Total: 256 bytes

impl SegmentHeader {
    pub fn new(segment_id: u32) -> Self {
        Self {
            magic: MAGIC_BYTES,
            version: VERSION,
            segment_id,
            concept_offset: HEADER_SIZE as u64,
            concept_count: 0,
            association_offset: HEADER_SIZE as u64,
            association_count: 0,
            vector_offset: HEADER_SIZE as u64,
            vector_count: 0,
            content_offset: HEADER_SIZE as u64,
            content_length: 0,
            created_at: current_timestamp(),
            compacted_at: 0,
            concept_checksum: 0,
            association_checksum: 0,
            content_checksum: 0,
            padding: 0,
            reserved1: [0; 32],
            reserved2: [0; 32],
            reserved3: [0; 32],
            reserved4: [0; 32],
            reserved5: [0; 32],
        }
    }
    
    pub fn validate(&self) -> Result<()> {
        let version = self.version;
        if self.magic != MAGIC_BYTES {
            anyhow::bail!("Invalid magic bytes");
        }
        if version != VERSION {
            anyhow::bail!("Unsupported version: {}", version);
        }
        Ok(())
    }
}

/// Segment file for append-only storage
pub struct Segment {
    path: PathBuf,
    header: SegmentHeader,
    // Memory-mapped read access (lazy initialized)
    mmap: Option<Arc<Mmap>>,
    // Write handle (only for active segment)
    writer: Option<BufWriter<File>>,
    // Current write position
    write_pos: u64,
}

impl Segment {
    /// Create a new segment file for writing
    pub fn create<P: AsRef<Path>>(path: P, segment_id: u32) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .read(true)
            .open(&path)
            .context("Failed to create segment file")?;
        
        let mut writer = BufWriter::new(file);
        let header = SegmentHeader::new(segment_id);
        
        // Write header
        let header_bytes = bytemuck::bytes_of(&header);
        writer.write_all(header_bytes)?;
        writer.flush()?;
        
        Ok(Self {
            path,
            header,
            mmap: None,
            writer: Some(writer),
            write_pos: HEADER_SIZE as u64,
        })
    }
    
    /// Open an existing segment for reading
    pub fn open_read<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        
        let file = File::open(&path)
            .context("Failed to open segment file")?;
        
        let mmap = unsafe {
            MmapOptions::new()
                .map(&file)
                .context("Failed to mmap segment")?
        };
        
        // Read and validate header
        if mmap.len() < HEADER_SIZE {
            anyhow::bail!("Segment file too small");
        }
        
        let header: SegmentHeader = *from_bytes(&mmap[0..HEADER_SIZE]);
        header.validate()?;
        
        Ok(Self {
            path,
            header,
            mmap: Some(Arc::new(mmap)),
            writer: None,
            write_pos: 0,
        })
    }
    
    /// Append a concept record to the segment
    pub fn append_concept(&mut self, record: ConceptRecord) -> Result<u64> {
        let writer = self.writer.as_mut()
            .ok_or_else(|| anyhow::anyhow!("Segment is read-only"))?;
        
        let offset = self.write_pos;
        let record_bytes = bytemuck::bytes_of(&record);
        
        writer.write_all(record_bytes)?;
        
        self.write_pos += record_bytes.len() as u64;
        self.header.concept_count += 1;
        
        Ok(offset)
    }
    
    /// Append an association record to the segment
    pub fn append_association(&mut self, record: AssociationRecord) -> Result<u64> {
        let writer = self.writer.as_mut()
            .ok_or_else(|| anyhow::anyhow!("Segment is read-only"))?;
        
        let offset = self.write_pos;
        let record_bytes = bytemuck::bytes_of(&record);
        
        writer.write_all(record_bytes)?;
        
        self.write_pos += record_bytes.len() as u64;
        self.header.association_count += 1;
        
        Ok(offset)
    }
    
    /// Append content (variable-length string) to the segment
    pub fn append_content(&mut self, content: &str) -> Result<(u64, u32)> {
        let writer = self.writer.as_mut()
            .ok_or_else(|| anyhow::anyhow!("Segment is read-only"))?;
        
        let offset = self.write_pos;
        let content_bytes = content.as_bytes();
        let length = content_bytes.len() as u32;
        
        // Write length prefix (4 bytes)
        writer.write_all(&length.to_le_bytes())?;
        // Write content
        writer.write_all(content_bytes)?;
        
        self.write_pos += 4 + content_bytes.len() as u64;
        
        Ok((offset, length))
    }
    
    /// Append a vector (f32 array) to the segment
    pub fn append_vector(&mut self, vector: &[f32]) -> Result<(u64, u32)> {
        let writer = self.writer.as_mut()
            .ok_or_else(|| anyhow::anyhow!("Segment is read-only"))?;
        
        let offset = self.write_pos;
        let dim = vector.len() as u32;
        
        // Write dimension (4 bytes)
        writer.write_all(&dim.to_le_bytes())?;
        
        // Write vector data
        let vector_bytes = bytemuck::cast_slice(vector);
        writer.write_all(vector_bytes)?;
        
        self.write_pos += 4 + vector_bytes.len() as u64;
        
        Ok((offset, dim))
    }
    
    /// Read a concept record at the given offset
    pub fn read_concept(&self, offset: u64) -> Result<ConceptRecord> {
        let mmap = self.mmap.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Segment not opened for reading"))?;
        
        let start = offset as usize;
        let end = start + std::mem::size_of::<ConceptRecord>();
        
        if end > mmap.len() {
            anyhow::bail!("Offset out of bounds");
        }
        
        Ok(*from_bytes(&mmap[start..end]))
    }
    
    /// Read an association record at the given offset
    pub fn read_association(&self, offset: u64) -> Result<AssociationRecord> {
        let mmap = self.mmap.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Segment not opened for reading"))?;
        
        let start = offset as usize;
        let end = start + std::mem::size_of::<AssociationRecord>();
        
        if end > mmap.len() {
            anyhow::bail!("Offset out of bounds");
        }
        
        Ok(*from_bytes(&mmap[start..end]))
    }
    
    /// Read content at the given offset
    pub fn read_content(&self, offset: u64) -> Result<String> {
        let mmap = self.mmap.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Segment not opened for reading"))?;
        
        let start = offset as usize;
        
        // Read length prefix
        if start + 4 > mmap.len() {
            anyhow::bail!("Offset out of bounds");
        }
        
        let length = u32::from_le_bytes([
            mmap[start],
            mmap[start + 1],
            mmap[start + 2],
            mmap[start + 3],
        ]);
        
        let content_start = start + 4;
        let content_end = content_start + length as usize;
        
        if content_end > mmap.len() {
            anyhow::bail!("Content length exceeds segment size");
        }
        
        String::from_utf8(mmap[content_start..content_end].to_vec())
            .context("Invalid UTF-8 in content")
    }
    
    /// Read a vector at the given offset
    pub fn read_vector(&self, offset: u64) -> Result<Vec<f32>> {
        let mmap = self.mmap.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Segment not opened for reading"))?;
        
        let start = offset as usize;
        
        // Read dimension
        if start + 4 > mmap.len() {
            anyhow::bail!("Offset out of bounds");
        }
        
        let dim = u32::from_le_bytes([
            mmap[start],
            mmap[start + 1],
            mmap[start + 2],
            mmap[start + 3],
        ]);
        
        let vector_start = start + 4;
        let vector_end = vector_start + (dim as usize * 4);
        
        if vector_end > mmap.len() {
            anyhow::bail!("Vector length exceeds segment size");
        }
        
        let vector_slice: &[f32] = cast_slice(&mmap[vector_start..vector_end]);
        Ok(vector_slice.to_vec())
    }
    
    /// Iterate over all concepts in the segment
    pub fn iter_concepts(&self) -> Result<ConceptIterator> {
        let mmap = self.mmap.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Segment not opened for reading"))?;
        
        Ok(ConceptIterator {
            mmap: Arc::clone(mmap),
            offset: self.header.concept_offset as usize,
            end: self.header.concept_offset as usize + 
                 (self.header.concept_count as usize * std::mem::size_of::<ConceptRecord>()),
        })
    }
    
    /// Flush all pending writes and sync to disk
    pub fn sync(&mut self) -> Result<()> {
        if let Some(writer) = self.writer.as_mut() {
            writer.flush()?;
            writer.get_ref().sync_all()?;
            
            // Update header with current counts
            writer.seek(SeekFrom::Start(0))?;
            let header_bytes = bytemuck::bytes_of(&self.header);
            writer.write_all(header_bytes)?;
            writer.flush()?;
            writer.get_ref().sync_all()?;
        }
        Ok(())
    }
    
    /// Close the segment and finalize writes
    pub fn close(mut self) -> Result<()> {
        self.sync()?;
        Ok(())
    }
    
    /// Get segment statistics
    pub fn stats(&self) -> SegmentStats {
        SegmentStats {
            segment_id: self.header.segment_id,
            concept_count: self.header.concept_count,
            association_count: self.header.association_count,
            vector_count: self.header.vector_count,
            content_length: self.header.content_length,
            file_size: self.write_pos,
            created_at: self.header.created_at,
        }
    }
    
    pub fn segment_id(&self) -> u32 {
        self.header.segment_id
    }
    
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// Iterator over concepts in a segment
pub struct ConceptIterator {
    mmap: Arc<Mmap>,
    offset: usize,
    end: usize,
}

impl Iterator for ConceptIterator {
    type Item = ConceptRecord;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.end {
            return None;
        }
        
        let record_size = std::mem::size_of::<ConceptRecord>();
        let record = from_bytes::<ConceptRecord>(&self.mmap[self.offset..self.offset + record_size]);
        self.offset += record_size;
        
        Some(*record)
    }
}

/// Segment statistics
#[derive(Debug, Clone)]
pub struct SegmentStats {
    pub segment_id: u32,
    pub concept_count: u32,
    pub association_count: u32,
    pub vector_count: u32,
    pub content_length: u32,
    pub file_size: u64,
    pub created_at: u64,
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
    fn test_create_segment() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.seg");
        
        let segment = Segment::create(&path, 0).unwrap();
        assert_eq!(segment.segment_id(), 0);
    }
    
    #[test]
    fn test_write_read_concept() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.seg");
        
        // Create concept record
        let concept_id = ConceptId::from_bytes([1u8; 16]);
        let record = ConceptRecord::new(concept_id, 0, 0, 0);
        
        // Write
        let mut segment = Segment::create(&path, 0).unwrap();
        let offset = segment.append_concept(record).unwrap();
        segment.sync().unwrap();
        drop(segment);
        
        // Read
        let segment = Segment::open_read(&path).unwrap();
        let read_record = segment.read_concept(offset).unwrap();
        
        assert_eq!(read_record.concept_id, concept_id);
    }
    
    #[test]
    fn test_write_read_content() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.seg");
        
        let content = "Hello, Sutra Storage!";
        
        // Write
        let mut segment = Segment::create(&path, 0).unwrap();
        let (offset, length) = segment.append_content(content).unwrap();
        segment.sync().unwrap();
        drop(segment);
        
        // Read
        let segment = Segment::open_read(&path).unwrap();
        let read_content = segment.read_content(offset).unwrap();
        
        assert_eq!(read_content, content);
        assert_eq!(length, content.len() as u32);
    }
    
    #[test]
    fn test_write_read_vector() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.seg");
        
        let vector = vec![1.0, 2.0, 3.0, 4.0];
        
        // Write
        let mut segment = Segment::create(&path, 0).unwrap();
        let (offset, dim) = segment.append_vector(&vector).unwrap();
        segment.sync().unwrap();
        drop(segment);
        
        // Read
        let segment = Segment::open_read(&path).unwrap();
        let read_vector = segment.read_vector(offset).unwrap();
        
        assert_eq!(read_vector, vector);
        assert_eq!(dim, 4);
    }
    
    #[test]
    fn test_iterate_concepts() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.seg");
        
        // Write multiple concepts
        let mut segment = Segment::create(&path, 0).unwrap();
        for i in 0..10 {
            let id = ConceptId::from_bytes([i; 16]);
            let record = ConceptRecord::new(id, 0, 0, 0);
            segment.append_concept(record).unwrap();
        }
        segment.sync().unwrap();
        drop(segment);
        
        // Read and iterate
        let segment = Segment::open_read(&path).unwrap();
        let concepts: Vec<_> = segment.iter_concepts().unwrap().collect();
        
        assert_eq!(concepts.len(), 10);
        assert_eq!(concepts[0].concept_id.0[0], 0);
        assert_eq!(concepts[9].concept_id.0[0], 9);
    }
    
    #[test]
    fn test_segment_stats() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.seg");
        
        let mut segment = Segment::create(&path, 42).unwrap();
        
        let id = ConceptId::from_bytes([1; 16]);
        let record = ConceptRecord::new(id, 0, 0, 0);
        segment.append_concept(record).unwrap();
        
        let stats = segment.stats();
        assert_eq!(stats.segment_id, 42);
        assert_eq!(stats.concept_count, 1);
    }
}
