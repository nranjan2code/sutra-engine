/// Core types for the Sutra storage engine
use bytemuck::{Pod, Zeroable};
use std::fmt;

/// Concept ID: 16-byte MD5 hash
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Pod, Zeroable)]
#[repr(C)]
pub struct ConceptId(pub [u8; 16]);

impl ConceptId {
    pub fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }
    
    pub fn from_string(s: &str) -> Self {
        use std::convert::TryInto;
        let bytes = hex::decode(s).expect("Invalid hex string");
        Self(bytes[..16].try_into().expect("Invalid length"))
    }
    
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
}

impl fmt::Display for ConceptId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Association type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum AssociationType {
    Semantic = 0,
    Causal = 1,
    Temporal = 2,
    Hierarchical = 3,
    Compositional = 4,
}

impl AssociationType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Semantic),
            1 => Some(Self::Causal),
            2 => Some(Self::Temporal),
            3 => Some(Self::Hierarchical),
            4 => Some(Self::Compositional),
            _ => None,
        }
    }
}

/// Fixed-size concept record (128 bytes)
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C, align(16))]
pub struct ConceptRecord {
    pub concept_id: ConceptId,
    pub strength: f32,
    pub confidence: f32,
    pub access_count: u32,
    pub created: u64,
    pub last_accessed: u64,
    pub content_offset: u64,
    pub content_length: u32,
    pub embedding_offset: u64,
    pub source_hash: u32,
    pub flags: u32,
    pub reserved: [u8; 40],
}

impl ConceptRecord {
    pub fn new(
        id: ConceptId,
        content_offset: u64,
        content_length: u32,
        embedding_offset: u64,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            concept_id: id,
            strength: 1.0,
            confidence: 1.0,
            access_count: 0,
            created: now,
            last_accessed: now,
            content_offset,
            content_length,
            embedding_offset,
            source_hash: 0,
            flags: 0,
            reserved: [0; 40],
        }
    }
}

/// Fixed-size association record (64 bytes)
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C, align(8))]
pub struct AssociationRecord {
    pub source_id: ConceptId,
    pub target_id: ConceptId,
    pub assoc_type: u8,
    pub confidence: f32,
    pub weight: f32,
    pub created: u64,
    pub last_used: u64,
    pub flags: u8,
    pub reserved: [u8; 7],
}

impl AssociationRecord {
    pub fn new(
        source: ConceptId,
        target: ConceptId,
        assoc_type: AssociationType,
        confidence: f32,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            source_id: source,
            target_id: target,
            assoc_type: assoc_type as u8,
            confidence,
            weight: 1.0,
            created: now,
            last_used: now,
            flags: 0,
            reserved: [0; 7],
        }
    }
}

/// Segment header for memory-mapped regions
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct SegmentHeader {
    pub magic: [u8; 8],           // "SUTRASEG"
    pub version: u32,
    pub segment_id: u64,
    pub created_at: u64,
    pub concept_count: u64,
    pub association_count: u64,
    pub content_size: u64,
    pub embedding_size: u64,
    pub checksum: u64,
    pub reserved: [u8; 216],
}

impl SegmentHeader {
    pub fn new(segment_id: u64) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            magic: *b"SUTRASEG",
            version: 1,
            segment_id,
            created_at: now,
            concept_count: 0,
            association_count: 0,
            content_size: 0,
            embedding_size: 0,
            checksum: 0,
            reserved: [0; 216],
        }
    }
    
    pub fn validate(&self) -> bool {
        &self.magic == b"SUTRASEG"
    }
}

/// Path through the knowledge graph
#[derive(Debug, Clone)]
pub struct GraphPath {
    pub concepts: Vec<ConceptId>,
    pub edges: Vec<(ConceptId, ConceptId, AssociationType)>,
    pub confidence: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_concept_record_size() {
        assert_eq!(std::mem::size_of::<ConceptRecord>(), 128);
    }
    
    #[test]
    fn test_association_record_size() {
        assert_eq!(std::mem::size_of::<AssociationRecord>(), 64);
    }
    
    #[test]
    fn test_segment_header_size() {
        assert_eq!(std::mem::size_of::<SegmentHeader>(), 256);
    }
}
