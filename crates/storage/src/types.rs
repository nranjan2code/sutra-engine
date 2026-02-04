/// Core types for the Sutra storage engine
use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Concept ID: 16-byte MD5 hash
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Pod, Zeroable, Serialize, Deserialize)]
#[repr(C)]
pub struct ConceptId(pub [u8; 16]);

/// Association ID: 64-bit unique identifier
pub type AssociationId = u64;

impl ConceptId {
    pub fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }
    
    pub fn from_string(s: &str) -> Self {
        use std::convert::TryInto;
        
        // Handle odd-length hex strings by padding with leading zero
        let hex_str = if s.len() % 2 == 1 {
            format!("0{}", s)
        } else {
            s.to_string()
        };
        
        let bytes = hex::decode(&hex_str).unwrap_or_else(|e| {
            log::warn!("Failed to decode hex '{}', using MD5 hash instead: {}", s, e);
            // Fallback: use MD5 hash of the string
            let hash = md5::compute(s.as_bytes());
            hash.to_vec()
        });
        
        // Handle different byte lengths
        if bytes.len() <= 8 {
            // Pad to 16 bytes with zeros
            let mut padded = [0u8; 16];
            padded[..bytes.len()].copy_from_slice(&bytes);
            Self(padded)
        } else if bytes.len() <= 16 {
            let mut padded = [0u8; 16];
            padded[..bytes.len()].copy_from_slice(&bytes);
            Self(padded)
        } else {
            // Take first 16 bytes if too long
            Self(bytes[..16].try_into().expect("Invalid length"))
        }
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
#[repr(C, packed)]  // packed to avoid padding
pub struct ConceptRecord {
    pub concept_id: ConceptId,       // 16 bytes
    pub strength: f32,                // 4 bytes
    pub confidence: f32,              // 4 bytes
    pub access_count: u32,            // 4 bytes
    pub created: u64,                 // 8 bytes
    pub last_accessed: u64,           // 8 bytes
    pub content_offset: u64,          // 8 bytes
    pub content_length: u32,          // 4 bytes
    pub embedding_offset: u64,        // 8 bytes
    pub source_hash: u32,             // 4 bytes
    pub flags: u32,                   // 4 bytes
    pub reserved1: [u8; 32],          // 32 bytes
    pub reserved2: [u8; 24],          // 24 bytes to reach 128
}  // Total: 128 bytes

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
            reserved1: [0; 32],
            reserved2: [0; 24],
        }
    }
}

/// Fixed-size association record (64 bytes)
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C, packed)]  // packed to avoid padding
pub struct AssociationRecord {
    pub source_id: ConceptId,         // 16 bytes
    pub target_id: ConceptId,         // 16 bytes
    pub assoc_type: u8,               // 1 byte
    pub confidence: f32,              // 4 bytes
    pub weight: f32,                  // 4 bytes
    pub created: u64,                 // 8 bytes
    pub last_used: u64,               // 8 bytes
    pub flags: u8,                    // 1 byte
    pub reserved: [u8; 6],            // 6 bytes padding to reach 64
}  // Total: 64 bytes

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
            reserved: [0; 6],
        }
    }
}

/// Segment header for memory-mapped regions
/// Note: Full implementation is in segment.rs
/// This is kept for backward compatibility with existing code
pub use crate::segment::SegmentHeader;

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
