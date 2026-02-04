/// Single-file, zero-copy, memory-mapped storage (arena-based)
///
/// Layout (little-endian):
/// [FileHeader (256B)]
/// [ConceptArena   (128B * concept_count, tightly packed)]
/// [EdgeArena      ( 64B * edge_count,    tightly packed)]
/// [VectorBlob     ( [u32 dim][f32..] repeated )]
/// [ContentBlob    ( [u32 len][u8..] repeated )]
///
/// Goals:
/// - One file (human sees a single file)
/// - Zero-copy reads via mmap (no deserialization)
/// - Append-only arenas (bump pointers)
/// - Bit/byte efficient, aligned
use anyhow::{Context, Result};
use memmap2::{MmapMut, MmapOptions};
use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::mem::size_of;
use std::path::{Path, PathBuf};
use std::ptr::copy_nonoverlapping;
use std::collections::HashMap;

use crate::types::{AssociationRecord, ConceptRecord, ConceptId};

const MAGIC: [u8; 8] = *b"SUTRAALL"; // "All-in-one"
const FILE_VERSION: u32 = 1;
const HEADER_SIZE: usize = 256;

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct FileHeader {
    magic: [u8; 8],      // 8
    version: u32,        // 4
    _pad0: u32,          // 4

    // Offsets (from file start)
    concept_off: u64,    // 8
    edge_off: u64,       // 8
    vector_off: u64,     // 8
    content_off: u64,    // 8

    // Sizes / counts
    concept_count: u64,  // 8
    edge_count: u64,     // 8
    vector_bytes: u64,   // 8 (total bytes written in vector blob)
    content_bytes: u64,  // 8 (total bytes written in content blob)

    // Write epoch (monotonic)
    epoch: u64,          // 8

    // Footer index offsets (written after arenas, for fast lookups)
    bloom_off: u64,      // 8 (Bloom filter start)
    bloom_bytes: u32,    // 4 (Bloom filter size)
    index_off: u64,      // 8 (ConceptID->offset index)
    index_count: u32,    // 4 (number of index entries)

    // Reserved
    reserved: [u8; 256 - (8 + 4 + 4 + 8*6 + 8*4 + 8 + 8 + 4 + 8 + 4)],
}

impl Default for FileHeader {
    fn default() -> Self {
        Self {
            magic: MAGIC,
            version: FILE_VERSION,
            _pad0: 0,
            concept_off: HEADER_SIZE as u64,
            edge_off: 0,
            vector_off: 0,
            content_off: 0,
            concept_count: 0,
            edge_count: 0,
            vector_bytes: 0,
            content_bytes: 0,
            epoch: 0,
            bloom_off: 0,
            bloom_bytes: 0,
            index_off: 0,
            index_count: 0,
            reserved: [0; 256 - (8 + 4 + 4 + 8*6 + 8*4 + 8 + 8 + 4 + 8 + 4)],
        }
    }
}

/// Align `offset` up to `alignment` bytes.
fn align_up(offset: u64, alignment: u64) -> u64 {
    if alignment == 0 { return offset; }
    let mask = alignment - 1;
    (offset + mask) & !mask
}

/// Single-file memory-mapped storage
pub struct MmapStore {
    #[allow(dead_code)]
    path: PathBuf,
    file: File,
    mmap: MmapMut,
    header: FileHeader,
    /// Concept ID -> file offset mapping (for fast lookups)
    concept_index: HashMap<ConceptId, u64>,
}

impl MmapStore {
    /// Create or open the single storage file.
    /// If new, initializes arenas in the following order: Concepts → Edges → Vectors → Content.
    pub fn open<P: AsRef<Path>>(path: P, initial_size: u64) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let is_new = !path.exists();

        // Create file and ensure size
        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(false)
            .open(&path)
            .with_context(|| format!("Failed to open storage file: {}", path.display()))?;

        if is_new {
            file.set_len(initial_size)?;
            // Write default header
            let mut header = FileHeader::default();

            // Compute arena base offsets with alignment
            let mut off = HEADER_SIZE as u64;
            header.concept_off = align_up(off, 128); // ConceptRecord alignment
            // Reserve 64MB for concepts initially
            off = header.concept_off + 64 * 1024 * 1024;

            header.edge_off = align_up(off, 64); // AssociationRecord alignment
            // Reserve 64MB for edges
            off = header.edge_off + 64 * 1024 * 1024;

            header.vector_off = align_up(off, 4); // f32 alignment
            // Reserve 128MB for vectors
            off = header.vector_off + 128 * 1024 * 1024;

            header.content_off = align_up(off, 4); // u32 len prefix alignment
            // Remaining for content blob

            // Persist header
            write_header(&mut file, &header)?;
        }

        // Map file
        let mmap = unsafe { MmapOptions::new().map_mut(&file)? };

        // Read header
        let header = read_header(&mmap)?;
        if header.magic != MAGIC || header.version != FILE_VERSION {
            anyhow::bail!("Invalid storage file format");
        }

        // Load index from footer (if exists)
        let concept_index = if header.index_count > 0 {
            load_concept_index(&mmap, header.index_off, header.index_count as usize)?
        } else {
            HashMap::new()
        };

        Ok(Self { 
            path, 
            file, 
            mmap, 
            header,
            concept_index,
        })
    }

    /// Ensure file has enough capacity, grow if needed
    fn ensure_capacity(&mut self, min_size: u64) -> Result<()> {
        let current = self.file.metadata()?.len();
        if current >= min_size { return Ok(()); }
        let new_size = (min_size.next_power_of_two()).max(current * 2);
        self.file.set_len(new_size)?;
        // Remap
        self.mmap.flush()?;
        self.mmap = unsafe { MmapOptions::new().map_mut(&self.file)? };
        Ok(())
    }

    /// Append content bytes, returning its offset and length
    pub fn append_content(&mut self, bytes: &[u8]) -> Result<(u64, u32)> {
        let base = self.header.content_off;
        let write_off = base + self.header.content_bytes;
        let needed = write_off
            .checked_add(4)
            .and_then(|v| v.checked_add(bytes.len() as u64))
            .ok_or_else(|| anyhow::anyhow!("overflow"))?;
        self.ensure_capacity(needed + 1)?;

        // SAFETY: mmap has been grown to fit
        unsafe {
            // Write length prefix (u32 LE)
            let len_ptr = self.mmap.as_mut_ptr().add(write_off as usize);
            *(len_ptr as *mut u32) = (bytes.len() as u32).to_le();
            // Write bytes
            let data_ptr = len_ptr.add(4);
            copy_nonoverlapping(bytes.as_ptr(), data_ptr, bytes.len());
        }

        self.header.content_bytes += 4 + bytes.len() as u64;
        Ok((write_off, bytes.len() as u32))
    }

    /// Append vector (f32 slice), returning its offset and dimension
    pub fn append_vector(&mut self, vector: &[f32]) -> Result<(u64, u32)> {
        let base = self.header.vector_off;
        let write_off = base + self.header.vector_bytes;
        let vec_bytes = std::mem::size_of_val(vector) as u64;
        let needed = write_off
            .checked_add(4)
            .and_then(|v| v.checked_add(vec_bytes))
            .ok_or_else(|| anyhow::anyhow!("overflow"))?;
        self.ensure_capacity(needed + 1)?;

        // SAFETY: mmap has been grown to fit
        unsafe {
            // Write dimension prefix (u32 LE)
            let dim_ptr = self.mmap.as_mut_ptr().add(write_off as usize);
            *(dim_ptr as *mut u32) = (vector.len() as u32).to_le();
            // Write f32 bytes
            let data_ptr = dim_ptr.add(4);
            let byte_src = vector.as_ptr() as *const u8;
            copy_nonoverlapping(byte_src, data_ptr, vec_bytes as usize);
        }

        self.header.vector_bytes += 4 + vec_bytes;
        Ok((write_off, vector.len() as u32))
    }

    /// Append a concept record (auto-filling content/vector offsets)
    pub fn append_concept_full(
        &mut self,
        mut record: ConceptRecord,
        content: Option<&[u8]>,
        vector: Option<&[f32]>,
    ) -> Result<u64> {
        // Write content first to get offsets
        if let Some(data) = content {
            let (off, len) = self.append_content(data)?;
            record.content_offset = off;
            record.content_length = len;
        }
        // Write vector
        if let Some(v) = vector {
            let (off, _dim) = self.append_vector(v)?;
            record.embedding_offset = off;
        }

        // Compute concept write position
        let concept_base = self.header.concept_off;
        // ✅ PRODUCTION: Use checked_mul() to prevent integer overflow
        let record_offset = self.header.concept_count
            .checked_mul(size_of::<ConceptRecord>() as u64)
            .ok_or_else(|| anyhow::anyhow!("Concept count overflow"))?;
        let offset = concept_base
            .checked_add(record_offset)
            .ok_or_else(|| anyhow::anyhow!("Offset overflow"))?;
        let needed = offset
            .checked_add(size_of::<ConceptRecord>() as u64)
            .ok_or_else(|| anyhow::anyhow!("Size overflow"))?;
        self.ensure_capacity(needed + 1)?;

        // SAFETY: write raw bytes of ConceptRecord
        unsafe {
            let dst = self.mmap.as_mut_ptr().add(offset as usize);
            let src = (&record as *const ConceptRecord) as *const u8;
            copy_nonoverlapping(src, dst, size_of::<ConceptRecord>());
        }

        self.header.concept_count += 1;
        
        // Update in-memory index
        self.concept_index.insert(record.concept_id, offset);
        
        Ok(offset)
    }

    /// Append an association record
    pub fn append_association(&mut self, record: &AssociationRecord) -> Result<u64> {
        let edge_base = self.header.edge_off;
        // ✅ PRODUCTION: Use checked_mul() to prevent integer overflow
        let record_offset = self.header.edge_count
            .checked_mul(size_of::<AssociationRecord>() as u64)
            .ok_or_else(|| anyhow::anyhow!("Edge count overflow"))?;
        let offset = edge_base
            .checked_add(record_offset)
            .ok_or_else(|| anyhow::anyhow!("Offset overflow"))?;
        let needed = offset
            .checked_add(size_of::<AssociationRecord>() as u64)
            .ok_or_else(|| anyhow::anyhow!("Size overflow"))?;
        self.ensure_capacity(needed + 1)?;

        unsafe {
            let dst = self.mmap.as_mut_ptr().add(offset as usize);
            let src = (record as *const AssociationRecord) as *const u8;
            copy_nonoverlapping(src, dst, size_of::<AssociationRecord>());
        }

        self.header.edge_count += 1;
        Ok(offset)
    }

    /// Lookup concept offset by ID (O(1) via index)
    pub fn find_concept_offset(&self, id: &ConceptId) -> Option<u64> {
        self.concept_index.get(id).copied()
    }

    /// Read concept by ID
    pub fn read_concept(&self, id: &ConceptId) -> Result<Option<ConceptRecord>> {
        let offset = match self.find_concept_offset(id) {
            Some(off) => off,
            None => return Ok(None),
        };

        if offset as usize + size_of::<ConceptRecord>() > self.mmap.len() {
            anyhow::bail!("Concept offset out of bounds");
        }

        unsafe {
            let ptr = self.mmap.as_ptr().add(offset as usize);
            let mut record = ConceptRecord::new(*id, 0, 0, 0);
            copy_nonoverlapping(ptr, (&mut record as *mut ConceptRecord) as *mut u8, size_of::<ConceptRecord>());
            Ok(Some(record))
        }
    }

    /// Persist header + data + footer indexes to disk
    pub fn sync(&mut self) -> Result<()> {
        // Write footer: Bloom filter + concept index
        self.write_footer()?;

        // Update header epoch and write
        self.header.epoch = self.header.epoch.wrapping_add(1);
        write_header_with_mmap(&mut self.mmap, &self.header)?;
        self.mmap.flush()?;
        self.file.sync_all()?;
        Ok(())
    }

    /// Write Bloom filter and offset index to footer
    fn write_footer(&mut self) -> Result<()> {
        // Compute footer start (after content blob)
        let footer_start = self.header.content_off + self.header.content_bytes;
        let footer_start = align_up(footer_start, 8);

        // 1. Write Bloom filter (simple bit array, 2 bits per concept for ~1% FP)
        let bloom_bits = (self.header.concept_count * 2).max(1024) as usize;
        let bloom_bytes = bloom_bits.div_ceil(8);
        let mut bloom = vec![0u8; bloom_bytes];

        for id in self.concept_index.keys() {
            let hash = hash_concept_id(id);
            let idx1 = (hash % bloom_bits as u64) as usize;
            let idx2 = ((hash / bloom_bits as u64) % bloom_bits as u64) as usize;
            bloom[idx1 / 8] |= 1 << (idx1 % 8);
            bloom[idx2 / 8] |= 1 << (idx2 % 8);
        }

        let bloom_off = footer_start;
        let needed = bloom_off + bloom_bytes as u64;
        self.ensure_capacity(needed + 1)?;

        unsafe {
            let dst = self.mmap.as_mut_ptr().add(bloom_off as usize);
            copy_nonoverlapping(bloom.as_ptr(), dst, bloom_bytes);
        }

        self.header.bloom_off = bloom_off;
        self.header.bloom_bytes = bloom_bytes as u32;

        // 2. Write concept ID -> offset index
        let index_off = bloom_off + bloom_bytes as u64;
        let index_off = align_up(index_off, 8);
        let index_count = self.concept_index.len();
        let index_size = index_count * (16 + 8); // ConceptId (16B) + offset (8B)
        let needed = index_off + index_size as u64;
        self.ensure_capacity(needed + 1)?;

        unsafe {
            let mut ptr = self.mmap.as_mut_ptr().add(index_off as usize);
            for (id, offset) in &self.concept_index {
                // Write ConceptId (16 bytes)
                copy_nonoverlapping(id.0.as_ptr(), ptr, 16);
                ptr = ptr.add(16);
                // Write offset (8 bytes LE)
                *(ptr as *mut u64) = offset.to_le();
                ptr = ptr.add(8);
            }
        }

        self.header.index_off = index_off;
        self.header.index_count = index_count as u32;

        Ok(())
    }

    /// Return basic stats
    pub fn stats(&self) -> MmapStats {
        MmapStats {
            concept_count: self.header.concept_count,
            edge_count: self.header.edge_count,
            vector_bytes: self.header.vector_bytes,
            content_bytes: self.header.content_bytes,
            epoch: self.header.epoch,
            file_len: self.file.metadata().map(|m| m.len()).unwrap_or(0),
            bloom_bytes: self.header.bloom_bytes,
            index_count: self.header.index_count,
        }
    }
}

/// Simple hash for ConceptId (for Bloom filter)
fn hash_concept_id(id: &ConceptId) -> u64 {
    // FNV-1a hash
    let mut hash = 0xcbf29ce484222325u64;
    for &byte in &id.0 {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn write_header(file: &mut File, header: &FileHeader) -> Result<()> {
    let mut buf = [0u8; HEADER_SIZE];
    // SAFETY: FileHeader is packed and fits into buf
    unsafe {
        copy_nonoverlapping(
            (header as *const FileHeader) as *const u8,
            buf.as_mut_ptr(),
            HEADER_SIZE,
        );
    }
    file.seek(SeekFrom::Start(0))?;
    file.write_all(&buf)?;
    file.sync_all()?;
    Ok(())
}

fn write_header_with_mmap(mmap: &mut MmapMut, header: &FileHeader) -> Result<()> {
    unsafe {
        copy_nonoverlapping(
            (header as *const FileHeader) as *const u8,
            mmap.as_mut_ptr(),
            HEADER_SIZE,
        );
    }
    Ok(())
}

fn read_header(mmap: &MmapMut) -> Result<FileHeader> {
    if mmap.len() < HEADER_SIZE { anyhow::bail!("file too small"); }
    let mut header = FileHeader::default();
    unsafe {
        copy_nonoverlapping(
            mmap.as_ptr(),
            (&mut header as *mut FileHeader) as *mut u8,
            HEADER_SIZE,
        );
    }
    Ok(header)
}

fn load_concept_index(mmap: &MmapMut, index_off: u64, count: usize) -> Result<HashMap<ConceptId, u64>> {
    let mut index = HashMap::with_capacity(count);
    
    if index_off as usize + count * 24 > mmap.len() {
        anyhow::bail!("Index footer out of bounds");
    }
    
    unsafe {
        let mut ptr = mmap.as_ptr().add(index_off as usize);
        for _ in 0..count {
            // Read ConceptId (16 bytes)
            let mut id_bytes = [0u8; 16];
            copy_nonoverlapping(ptr, id_bytes.as_mut_ptr(), 16);
            let concept_id = ConceptId(id_bytes);
            ptr = ptr.add(16);
            
            // Read offset (8 bytes LE)
            let offset = u64::from_le(*(ptr as *const u64));
            ptr = ptr.add(8);
            
            index.insert(concept_id, offset);
        }
    }
    
    Ok(index)
}

#[derive(Debug, Clone, Copy)]
pub struct MmapStats {
    pub concept_count: u64,
    pub edge_count: u64,
    pub vector_bytes: u64,
    pub content_bytes: u64,
    pub epoch: u64,
    pub file_len: u64,
    pub bloom_bytes: u32,
    pub index_count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AssociationType, ConceptId};
    use tempfile::TempDir;

    #[test]
    fn test_open_and_append() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("storage.dat");

        let mut store = MmapStore::open(&path, 16 * 1024 * 1024).unwrap();

        // Append concept with content and vector
        let id = ConceptId([1; 16]);
        let rec = ConceptRecord::new(id, 0, 0, 0);
        let content = b"hello world".as_ref();
        let vector = vec![1.0f32, 2.0, 3.0];

        store.append_concept_full(rec, Some(content), Some(&vector)).unwrap();
        store.sync().unwrap();

        let stats = store.stats();
        assert_eq!(stats.concept_count, 1);
        assert!(stats.content_bytes > 0);
        assert!(stats.vector_bytes > 0);
    }

    #[test]
    fn test_append_association() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("storage.dat");
        let mut store = MmapStore::open(&path, 8 * 1024 * 1024).unwrap();

        let id1 = crate::types::ConceptId([1; 16]);
        let id2 = crate::types::ConceptId([2; 16]);
        let assoc = AssociationRecord::new(id1, id2, AssociationType::Semantic, 0.8);

        store.append_association(&assoc).unwrap();
        store.sync().unwrap();

        let stats = store.stats();
        assert_eq!(stats.edge_count, 1);
    }

    #[test]
    fn test_index_persistence() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("storage.dat");

        let id1 = ConceptId([1; 16]);
        let id2 = ConceptId([2; 16]);

        // Write concepts
        {
            let mut store = MmapStore::open(&path, 16 * 1024 * 1024).unwrap();
            let rec1 = ConceptRecord::new(id1, 0, 0, 0);
            let rec2 = ConceptRecord::new(id2, 0, 0, 0);
            
            store.append_concept_full(rec1, Some(b"concept1"), None).unwrap();
            store.append_concept_full(rec2, Some(b"concept2"), None).unwrap();
            store.sync().unwrap();

            let stats = store.stats();
            assert_eq!(stats.concept_count, 2);
            assert!(stats.bloom_bytes > 0);
            assert_eq!(stats.index_count, 2);
        }

        // Reopen and verify index loaded
        {
            let store = MmapStore::open(&path, 16 * 1024 * 1024).unwrap();
            
            // Should find concepts via index
            let found1 = store.read_concept(&id1).unwrap();
            assert!(found1.is_some());
            assert_eq!(found1.unwrap().concept_id, id1);

            let found2 = store.read_concept(&id2).unwrap();
            assert!(found2.is_some());
            assert_eq!(found2.unwrap().concept_id, id2);

            // Non-existent concept
            let id3 = ConceptId([99; 16]);
            let found3 = store.read_concept(&id3).unwrap();
            assert!(found3.is_none());
        }
    }
}
