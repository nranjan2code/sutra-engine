//! Sutra Storage Explorer Library
//!
//! Read-only binary parser for exploring Sutra storage.dat files
//! with graph query and visualization capabilities.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

/// Concept ID: 16-byte identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConceptId(pub [u8; 16]);

impl ConceptId {
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
    
    pub fn from_hex(s: &str) -> Result<Self> {
        let bytes = hex::decode(s)?;
        if bytes.len() != 16 {
            return Err(anyhow!("Invalid ConceptId length: expected 16, got {}", bytes.len()));
        }
        let mut arr = [0u8; 16];
        arr.copy_from_slice(&bytes);
        Ok(Self(arr))
    }
}

/// Association types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
    
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Semantic => "semantic",
            Self::Causal => "causal",
            Self::Temporal => "temporal",
            Self::Hierarchical => "hierarchical",
            Self::Compositional => "compositional",
        }
    }
}

/// Concept with content and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Concept {
    pub id: String,
    pub content: String,
    pub strength: f32,
    pub confidence: f32,
    pub access_count: u32,
    pub created: u64,
    pub vector_dimension: Option<usize>,
    pub neighbors: Vec<String>,
}

/// Association between concepts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Association {
    pub source_id: String,
    pub target_id: String,
    pub confidence: f32,
}

/// Storage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub version: u32,
    pub total_concepts: usize,
    pub total_edges: usize,
    pub total_vectors: usize,
    pub file_size_mb: f64,
    pub timestamp: u64,
}

/// Graph path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphPath {
    pub concepts: Vec<String>,
    pub length: usize,
    pub confidence: f32,
}

/// Storage Explorer - Read-only access to storage.dat
pub struct StorageExplorer {
    concepts: HashMap<ConceptId, ConceptData>,
    edges: HashMap<ConceptId, Vec<EdgeData>>,
    vectors: HashMap<ConceptId, Vec<f32>>,
    stats: StorageStats,
}

#[derive(Debug, Clone)]
struct ConceptData {
    id: ConceptId,
    content: Vec<u8>,
    strength: f32,
    confidence: f32,
    access_count: u32,
    created: u64,
}

#[derive(Debug, Clone)]
struct EdgeData {
    target_id: ConceptId,
    confidence: f32,
}

impl StorageExplorer {
    /// Load a storage.dat file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let file_size = std::fs::metadata(path)?.len();
        
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        
        // Read header (64 bytes)
        let mut header = vec![0u8; 64];
        reader.read_exact(&mut header)?;
        
        // Validate magic bytes
        if &header[0..8] != b"SUTRADAT" {
            return Err(anyhow!("Invalid storage format: expected SUTRADAT magic bytes"));
        }
        
        // Parse header
        let version = u32::from_le_bytes([header[8], header[9], header[10], header[11]]);
        let concept_count = u32::from_le_bytes([header[12], header[13], header[14], header[15]]) as usize;
        let edge_count = u32::from_le_bytes([header[16], header[17], header[18], header[19]]) as usize;
        let vector_count = u32::from_le_bytes([header[20], header[21], header[22], header[23]]) as usize;
        let timestamp = u64::from_le_bytes([
            header[24], header[25], header[26], header[27],
            header[28], header[29], header[30], header[31],
        ]);
        
        if version != 2 {
            return Err(anyhow!("Unsupported storage version: {}. Expected version 2.", version));
        }
        
        let mut concepts = HashMap::new();
        let mut edges: HashMap<ConceptId, Vec<EdgeData>> = HashMap::new();
        let mut vectors = HashMap::new();
        
        // Read concepts
        for _ in 0..concept_count {
            let mut concept_header = vec![0u8; 36];
            reader.read_exact(&mut concept_header)?;
            
            let mut id_bytes = [0u8; 16];
            id_bytes.copy_from_slice(&concept_header[0..16]);
            let id = ConceptId(id_bytes);
            
            let content_len = u32::from_le_bytes([concept_header[16], concept_header[17], concept_header[18], concept_header[19]]) as usize;
            let strength = f32::from_le_bytes([concept_header[20], concept_header[21], concept_header[22], concept_header[23]]);
            let confidence = f32::from_le_bytes([concept_header[24], concept_header[25], concept_header[26], concept_header[27]]);
            let access_count = u32::from_le_bytes([concept_header[28], concept_header[29], concept_header[30], concept_header[31]]);
            let created = u64::from_le_bytes([
                concept_header[32], concept_header[33], concept_header[34], concept_header[35],
                0, 0, 0, 0
            ]);
            
            let mut content = vec![0u8; content_len];
            reader.read_exact(&mut content)?;
            
            concepts.insert(id, ConceptData {
                id,
                content,
                strength,
                confidence,
                access_count,
                created,
            });
        }
        
        // Read edges
        for _ in 0..edge_count {
            let mut edge_data = vec![0u8; 36];
            reader.read_exact(&mut edge_data)?;
            
            let mut source_bytes = [0u8; 16];
            source_bytes.copy_from_slice(&edge_data[0..16]);
            let source_id = ConceptId(source_bytes);
            
            let mut target_bytes = [0u8; 16];
            target_bytes.copy_from_slice(&edge_data[16..32]);
            let target_id = ConceptId(target_bytes);
            
            let confidence = f32::from_le_bytes([edge_data[32], edge_data[33], edge_data[34], edge_data[35]]);
            
            edges.entry(source_id)
                .or_insert_with(Vec::new)
                .push(EdgeData { target_id, confidence });
        }
        
        // Read vectors
        for _ in 0..vector_count {
            let mut vector_header = vec![0u8; 20];
            reader.read_exact(&mut vector_header)?;
            
            let mut id_bytes = [0u8; 16];
            id_bytes.copy_from_slice(&vector_header[0..16]);
            let concept_id = ConceptId(id_bytes);
            
            let dimension = u32::from_le_bytes([vector_header[16], vector_header[17], vector_header[18], vector_header[19]]) as usize;
            
            let mut vector_data = Vec::with_capacity(dimension);
            for _ in 0..dimension {
                let mut component = [0u8; 4];
                reader.read_exact(&mut component)?;
                vector_data.push(f32::from_le_bytes(component));
            }
            
            vectors.insert(concept_id, vector_data);
        }
        
        let stats = StorageStats {
            version,
            total_concepts: concepts.len(),
            total_edges: edge_count,
            total_vectors: vectors.len(),
            file_size_mb: file_size as f64 / (1024.0 * 1024.0),
            timestamp,
        };
        
        Ok(Self {
            concepts,
            edges,
            vectors,
            stats,
        })
    }
    
    /// Get storage statistics
    pub fn stats(&self) -> &StorageStats {
        &self.stats
    }
    
    /// Get a concept by ID
    pub fn get_concept(&self, id: &str) -> Result<Concept> {
        let concept_id = ConceptId::from_hex(id)?;
        let data = self.concepts.get(&concept_id)
            .ok_or_else(|| anyhow!("Concept not found: {}", id))?;
        
        let content = String::from_utf8_lossy(&data.content).to_string();
        let vector_dimension = self.vectors.get(&concept_id).map(|v| v.len());
        
        let neighbors = self.edges.get(&concept_id)
            .map(|edges| edges.iter().map(|e| e.target_id.to_hex()).collect())
            .unwrap_or_default();
        
        Ok(Concept {
            id: id.to_string(),
            content,
            strength: data.strength,
            confidence: data.confidence,
            access_count: data.access_count,
            created: data.created,
            vector_dimension,
            neighbors,
        })
    }
    
    /// List all concept IDs
    pub fn list_concepts(&self, limit: Option<usize>) -> Vec<String> {
        self.concepts.keys()
            .take(limit.unwrap_or(usize::MAX))
            .map(|id| id.to_hex())
            .collect()
    }
    
    /// Search concepts by content substring
    pub fn search_content(&self, query: &str, limit: Option<usize>) -> Vec<Concept> {
        let query_lower = query.to_lowercase();
        
        self.concepts.iter()
            .filter(|(_, data)| {
                String::from_utf8_lossy(&data.content)
                    .to_lowercase()
                    .contains(&query_lower)
            })
            .take(limit.unwrap_or(usize::MAX))
            .filter_map(|(id, _)| self.get_concept(&id.to_hex()).ok())
            .collect()
    }
    
    /// Get associations for a concept
    pub fn get_associations(&self, id: &str) -> Result<Vec<Association>> {
        let concept_id = ConceptId::from_hex(id)?;
        
        let edges = self.edges.get(&concept_id)
            .ok_or_else(|| anyhow!("No associations found for concept: {}", id))?;
        
        Ok(edges.iter()
            .map(|edge| Association {
                source_id: id.to_string(),
                target_id: edge.target_id.to_hex(),
                confidence: edge.confidence,
            })
            .collect())
    }
    
    /// Find path between two concepts (BFS)
    pub fn find_path(&self, start_id: &str, end_id: &str, max_depth: usize) -> Result<Option<GraphPath>> {
        use std::collections::{HashMap, VecDeque};
        
        let start = ConceptId::from_hex(start_id)?;
        let end = ConceptId::from_hex(end_id)?;
        
        if start == end {
            return Ok(Some(GraphPath {
                concepts: vec![start_id.to_string()],
                length: 0,
                confidence: 1.0,
            }));
        }
        
        let mut queue = VecDeque::new();
        let mut visited: HashMap<ConceptId, Option<ConceptId>> = HashMap::new();
        
        queue.push_back((start, 0));
        visited.insert(start, None);
        
        while let Some((current, depth)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }
            
            if let Some(neighbors) = self.edges.get(&current) {
                for edge in neighbors {
                    if !visited.contains_key(&edge.target_id) {
                        visited.insert(edge.target_id, Some(current));
                        
                        if edge.target_id == end {
                            // Reconstruct path
                            let mut path = vec![end];
                            let mut backtrack = current;
                            path.push(backtrack);
                            
                            while let Some(Some(prev)) = visited.get(&backtrack) {
                                path.push(*prev);
                                backtrack = *prev;
                            }
                            
                            path.reverse();
                            
                            return Ok(Some(GraphPath {
                                concepts: path.iter().map(|id| id.to_hex()).collect(),
                                length: path.len() - 1,
                                confidence: 1.0, // Could calculate from edge confidences
                            }));
                        }
                        
                        queue.push_back((edge.target_id, depth + 1));
                    }
                }
            }
        }
        
        Ok(None)
    }
    
    /// Get neighborhood graph (concept + N-hop neighbors)
    pub fn get_neighborhood(&self, id: &str, depth: usize) -> Result<NeighborhoodGraph> {
        use std::collections::{HashSet, VecDeque};
        
        let start = ConceptId::from_hex(id)?;
        
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        
        queue.push_back((start, 0));
        visited.insert(start);
        
        while let Some((current, current_depth)) = queue.pop_front() {
            // Add node
            if let Ok(concept) = self.get_concept(&current.to_hex()) {
                nodes.push(concept);
            }
            
            // Add edges and neighbors
            if current_depth < depth {
                if let Some(neighbors) = self.edges.get(&current) {
                    for edge in neighbors {
                        edges.push(Association {
                            source_id: current.to_hex(),
                            target_id: edge.target_id.to_hex(),
                            confidence: edge.confidence,
                        });
                        
                        if !visited.contains(&edge.target_id) {
                            visited.insert(edge.target_id);
                            queue.push_back((edge.target_id, current_depth + 1));
                        }
                    }
                }
            }
        }
        
        Ok(NeighborhoodGraph { nodes, edges })
    }
    
    /// Get vector similarity (cosine)
    pub fn vector_similarity(&self, id1: &str, id2: &str) -> Result<f32> {
        let concept_id1 = ConceptId::from_hex(id1)?;
        let concept_id2 = ConceptId::from_hex(id2)?;
        
        let vec1 = self.vectors.get(&concept_id1)
            .ok_or_else(|| anyhow!("No vector for concept: {}", id1))?;
        let vec2 = self.vectors.get(&concept_id2)
            .ok_or_else(|| anyhow!("No vector for concept: {}", id2))?;
        
        if vec1.len() != vec2.len() {
            return Err(anyhow!("Vector dimension mismatch: {} vs {}", vec1.len(), vec2.len()));
        }
        
        let dot: f32 = vec1.iter().zip(vec2.iter()).map(|(a, b)| a * b).sum();
        let norm1: f32 = vec1.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm2: f32 = vec2.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        Ok(dot / (norm1 * norm2))
    }
}

/// Neighborhood graph result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeighborhoodGraph {
    pub nodes: Vec<Concept>,
    pub edges: Vec<Association>,
}
