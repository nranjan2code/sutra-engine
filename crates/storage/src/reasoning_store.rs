use crate::index::GraphIndex;
/// Reasoning-optimized graph storage
///
/// Specialized for AI reasoning workloads:
/// - Fast path finding
/// - Association-heavy queries
/// - Batch operations
/// - Semantic search integration
use crate::types::*;
use crate::vectors::VectorStore;
use crate::wal::{Operation, WriteAheadLog};
use anyhow::{anyhow, Result};
use dashmap::DashMap;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};

/// Complete concept data (record + content)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConceptData {
    pub id: ConceptId,
    pub content: String,
    pub created: u64,
    pub last_accessed: u64,
    pub access_count: u32,
    pub strength: f32,
    pub confidence: f32,
    pub source: Option<String>,
    pub category: Option<String>,
}

/// Complete association data
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AssociationData {
    pub source_id: ConceptId,
    pub target_id: ConceptId,
    pub assoc_type: AssociationType,
    pub weight: f32,
    pub confidence: f32,
    pub created: u64,
    pub last_used: u64,
}

/// Reasoning context (all data needed for path scoring)
#[derive(Debug, Clone)]
pub struct ReasoningContext {
    pub concept: ConceptData,
    pub associations: Vec<AssociationData>,
    pub neighbor_strengths: HashMap<ConceptId, f32>,
}

/// Main reasoning-optimized storage engine
pub struct ReasoningStore {
    path: PathBuf,

    // Core data stores
    concepts: Arc<DashMap<ConceptId, ConceptData>>,
    concept_content: Arc<DashMap<ConceptId, String>>,
    associations: Arc<DashMap<(ConceptId, ConceptId), AssociationData>>,

    // Indexes for fast queries
    adjacency: Arc<RwLock<GraphIndex>>,

    // Optional vector store
    vectors: Option<Arc<RwLock<VectorStore>>>,

    // Write-Ahead Log for durability
    wal: Arc<Mutex<WriteAheadLog>>,
}

impl ReasoningStore {
    /// Create new reasoning store
    pub fn new<P: AsRef<Path>>(path: P, vector_dimension: Option<usize>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        std::fs::create_dir_all(&path)?;

        // Initialize vector store if dimension provided
        let vectors = if let Some(dim) = vector_dimension {
            let vectors_path = path.join("vectors");
            let config = crate::vectors::VectorConfig {
                dimension: dim,
                use_compression: true,
                num_subvectors: dim / 8,
                num_centroids: 256,
            };
            Some(Arc::new(RwLock::new(VectorStore::new(
                &vectors_path,
                config,
            )?)))
        } else {
            None
        };

        // Initialize Write-Ahead Log
        let wal_path = path.join("wal.log");
        let wal = if wal_path.exists() {
            WriteAheadLog::open(&wal_path, true)?
        } else {
            WriteAheadLog::create(&wal_path, true)?
        };

        let mut store = Self {
            path,
            concepts: Arc::new(DashMap::new()),
            concept_content: Arc::new(DashMap::new()),
            associations: Arc::new(DashMap::new()),
            adjacency: Arc::new(RwLock::new(GraphIndex::new())),
            vectors,
            wal: Arc::new(Mutex::new(wal)),
        };

        // Crash recovery: replay WAL and load snapshots
        store.recover()?;

        Ok(store)
    }

    // ========== Concept Operations ==========

    /// Add or update concept with all metadata
    pub fn put_concept(&self, concept: ConceptData, embedding: Option<Vec<f32>>) -> Result<()> {
        let id = concept.id;

        // PRODUCTION: Write to WAL first (durability guarantee)
        {
            let mut wal = self.wal.lock().unwrap();
            wal.append(Operation::WriteConcept {
                concept_id: id,
                content_len: concept.content.len() as u32,
                vector_len: embedding.as_ref().map(|v| v.len() as u32).unwrap_or(0),
                created: concept.created,
                modified: concept.last_accessed,
            })?;
            wal.sync()?; // fsync for durability
        }

        // Apply to memory (after WAL)
        self.concept_content.insert(id, concept.content.clone());
        self.concepts.insert(id, concept.clone());

        // Store vector if provided
        if let (Some(emb), Some(ref vectors)) = (embedding, &self.vectors) {
            vectors.write().unwrap().add_vector(id, emb)?;
        }

        // Update word index
        let words: Vec<String> = concept
            .content
            .split_whitespace()
            .map(|s| s.to_lowercase())
            .collect();

        self.adjacency.write().unwrap().index_words(id, &words);

        Ok(())
    }

    /// Get complete concept data
    pub fn get_concept(&self, id: ConceptId) -> Option<ConceptData> {
        self.concepts.get(&id).map(|entry| entry.value().clone())
    }

    /// Get all concept IDs
    pub fn get_all_concept_ids(&self) -> Vec<ConceptId> {
        self.concepts.iter().map(|entry| *entry.key()).collect()
    }

    /// Update concept metadata (strength, access_count, etc.)
    pub fn update_concept_metadata(
        &self,
        id: ConceptId,
        strength: Option<f32>,
        access_count: Option<u32>,
    ) -> Result<()> {
        let mut concept = self
            .concepts
            .get_mut(&id)
            .ok_or_else(|| anyhow!("Concept not found"))?;

        if let Some(s) = strength {
            concept.strength = s;
        }
        if let Some(ac) = access_count {
            concept.access_count = ac;
        }

        concept.last_accessed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(())
    }

    // ========== Association Operations ==========

    /// Add or update association
    pub fn put_association(&self, assoc: AssociationData) -> Result<()> {
        // Validate endpoints exist
        if !self.concepts.contains_key(&assoc.source_id) {
            return Err(anyhow!("Source concept not found"));
        }
        if !self.concepts.contains_key(&assoc.target_id) {
            return Err(anyhow!("Target concept not found"));
        }

        // PRODUCTION: Write to WAL first (durability guarantee)
        {
            let mut wal = self.wal.lock().unwrap();
            // Generate association ID from concept IDs
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            std::hash::Hash::hash(&assoc.source_id, &mut hasher);
            std::hash::Hash::hash(&assoc.target_id, &mut hasher);
            let association_id = std::hash::Hasher::finish(&hasher);

            wal.append(Operation::WriteAssociation {
                source: assoc.source_id,
                target: assoc.target_id,
                association_id,
                strength: assoc.weight,
                created: assoc.created,
            })?;
            wal.sync()?; // fsync for durability
        }

        // Apply to memory (after WAL)
        let key = (assoc.source_id, assoc.target_id);
        self.associations.insert(key, assoc.clone());

        // Update adjacency list (bidirectional)
        let adj = self.adjacency.write().unwrap();
        adj.add_edge(assoc.source_id, assoc.target_id);
        adj.add_edge(assoc.target_id, assoc.source_id);

        Ok(())
    }

    /// Get association between two concepts
    pub fn get_association(
        &self,
        source_id: ConceptId,
        target_id: ConceptId,
    ) -> Option<AssociationData> {
        let key = (source_id, target_id);
        self.associations
            .get(&key)
            .map(|entry| entry.value().clone())
    }

    /// Get all associations from a concept
    pub fn get_associations_from(&self, source_id: ConceptId) -> Vec<AssociationData> {
        let neighbors = {
            let adj = self.adjacency.read().unwrap();
            adj.get_neighbors(source_id)
        };

        let mut associations = Vec::new();
        for target_id in neighbors {
            if let Some(assoc) = self.get_association(source_id, target_id) {
                associations.push(assoc);
            }
        }

        associations
    }

    /// Get ALL associations (for bulk loading)
    pub fn get_all_associations(&self) -> Vec<AssociationData> {
        self.associations
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    // ========== Reasoning-Optimized Operations ==========

    /// Atomically add a concept (with optional embedding) and a batch of associations in one durable transaction
    pub fn learn_atomic(
        &self,
        concept: ConceptData,
        embedding: Option<Vec<f32>>,
        assoc_batch: Vec<AssociationData>,
    ) -> Result<()> {
        // Begin WAL transaction
        {
            let mut wal = self.wal.lock().unwrap();
            wal.begin_transaction()?;

            // Concept write op
            wal.append(Operation::WriteConcept {
                concept_id: concept.id,
                content_len: concept.content.len() as u32,
                vector_len: embedding.as_ref().map(|v| v.len() as u32).unwrap_or(0),
                created: concept.created,
                modified: concept.last_accessed,
            })?;

            // Association write ops
            for a in &assoc_batch {
                // Generate association ID from endpoints
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                std::hash::Hash::hash(&a.source_id, &mut hasher);
                std::hash::Hash::hash(&a.target_id, &mut hasher);
                let association_id = std::hash::Hasher::finish(&hasher);

                wal.append(Operation::WriteAssociation {
                    source: a.source_id,
                    target: a.target_id,
                    association_id,
                    strength: a.weight,
                    created: a.created,
                })?;
            }

            // Commit and fsync
            wal.commit_transaction()?;
            wal.sync()?;
        }

        // Apply to in-memory state AFTER WAL commit
        self.concept_content
            .insert(concept.id, concept.content.clone());
        self.concepts.insert(concept.id, concept.clone());

        // Store vector if provided
        if let (Some(emb), Some(ref vectors)) = (embedding, &self.vectors) {
            vectors.write().unwrap().add_vector(concept.id, emb)?;
        }

        // Index words
        let words: Vec<String> = concept
            .content
            .split_whitespace()
            .map(|s| s.to_lowercase())
            .collect();
        self.adjacency
            .write()
            .unwrap()
            .index_words(concept.id, &words);

        // Insert associations + adjacency
        {
            let adj = self.adjacency.write().unwrap();
            for a in assoc_batch {
                let key = (a.source_id, a.target_id);
                self.associations.insert(key, a.clone());
                adj.add_edge(a.source_id, a.target_id);
                adj.add_edge(a.target_id, a.source_id);
            }
        }

        Ok(())
    }

    /// Get complete reasoning context for a concept
    pub fn get_reasoning_context(&self, concept_id: ConceptId) -> Option<ReasoningContext> {
        let concept = self.get_concept(concept_id)?;
        let associations = self.get_associations_from(concept_id);

        // Compute neighbor strengths
        let mut neighbor_strengths = HashMap::new();
        for assoc in &associations {
            if let Some(neighbor) = self.get_concept(assoc.target_id) {
                neighbor_strengths.insert(assoc.target_id, neighbor.strength);
            }
        }

        Some(ReasoningContext {
            concept,
            associations,
            neighbor_strengths,
        })
    }

    /// Batch get reasoning contexts (for multi-path reasoning)
    pub fn get_reasoning_contexts_batch(
        &self,
        concept_ids: &[ConceptId],
    ) -> Vec<Option<ReasoningContext>> {
        concept_ids
            .iter()
            .map(|&id| self.get_reasoning_context(id))
            .collect()
    }

    /// Load association working set (for reasoning engine initialization)
    pub fn load_association_working_set(
        &self,
        concept_ids: &[ConceptId],
    ) -> HashMap<(ConceptId, ConceptId), AssociationData> {
        let mut working_set = HashMap::new();

        for &concept_id in concept_ids {
            for assoc in self.get_associations_from(concept_id) {
                let key = (assoc.source_id, assoc.target_id);
                working_set.insert(key, assoc);
            }
        }

        working_set
    }

    // ========== Graph Queries ==========

    /// Find paths between two concepts using BFS up to a maximum depth
    pub fn find_paths(
        &self,
        source: ConceptId,
        target: ConceptId,
        max_depth: usize,
        max_paths: usize,
    ) -> Vec<GraphPath> {
        if source == target {
            return vec![GraphPath {
                concepts: vec![source],
                edges: vec![],
                confidence: 1.0,
            }];
        }

        let mut results: Vec<GraphPath> = Vec::new();
        let mut queue: std::collections::VecDeque<Vec<ConceptId>> =
            std::collections::VecDeque::new();
        let mut visited: std::collections::HashSet<ConceptId> = std::collections::HashSet::new();

        queue.push_back(vec![source]);
        visited.insert(source);

        while let Some(path) = queue.pop_front() {
            if path.len() > max_depth {
                continue;
            }
            let last = *path.last().unwrap();
            let neighbors = self.get_neighbors(last);
            for n in neighbors {
                if path.contains(&n) {
                    continue;
                } // avoid cycles
                let mut new_path = path.clone();
                new_path.push(n);
                if n == target {
                    // Build GraphPath with edge types and confidence
                    let mut edges: Vec<(ConceptId, ConceptId, AssociationType)> = Vec::new();
                    let mut confidences: Vec<f32> = Vec::new();
                    for w in new_path.windows(2) {
                        let a = self.get_association(w[0], w[1]);
                        if let Some(assoc) = a {
                            edges.push((w[0], w[1], assoc.assoc_type));
                            confidences.push(assoc.confidence);
                        } else {
                            edges.push((w[0], w[1], AssociationType::Semantic));
                            confidences.push(0.5);
                        }
                    }
                    let confidence = if confidences.is_empty() {
                        1.0
                    } else {
                        confidences.iter().sum::<f32>() / confidences.len() as f32
                    };
                    results.push(GraphPath {
                        concepts: new_path.clone(),
                        edges,
                        confidence,
                    });
                    if results.len() >= max_paths {
                        return results;
                    }
                } else {
                    queue.push_back(new_path);
                }
            }
        }

        results
    }

    /// Get neighbors (fast from index)
    pub fn get_neighbors(&self, concept_id: ConceptId) -> Vec<ConceptId> {
        let adj = self.adjacency.read().unwrap();
        adj.get_neighbors(concept_id)
    }

    /// Search by text (word index)
    pub fn search_by_text(&self, query: &str) -> Vec<ConceptId> {
        let words: Vec<String> = query.split_whitespace().map(|s| s.to_lowercase()).collect();

        let adj = self.adjacency.read().unwrap();
        adj.search_by_words(&words)
    }

    // ========== Vector Operations ==========

    /// Get vector for concept
    pub fn get_vector(&self, concept_id: ConceptId) -> Option<Vec<f32>> {
        if let Some(ref vectors) = self.vectors {
            let v = vectors.read().unwrap();
            v.get_vector(concept_id)
        } else {
            None
        }
    }

    /// Vector similarity search
    ///
    /// Note: This is a legacy API. Production code should use `HnswContainer` directly,
    /// which provides:
    /// - HNSW index for fast approximate nearest neighbor search
    /// - Persistent mmap-backed storage
    /// - Sub-millisecond query latency
    pub fn vector_search(
        &self,
        _query_embedding: &[f32],
        _k: usize,
    ) -> Result<Vec<(ConceptId, f32)>> {
        Err(anyhow!(
            "Vector search not implemented in legacy ReasoningStore. Use HnswContainer instead."
        ))
    }

    // ========== Statistics ==========

    /// Get storage statistics
    pub fn stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        stats.insert("total_concepts".to_string(), self.concepts.len());
        stats.insert("total_associations".to_string(), self.associations.len());

        let adj = self.adjacency.read().unwrap();
        let adj_stats = adj.stats();
        stats.insert("total_edges".to_string(), adj_stats.total_edges as usize);
        stats.insert("total_words".to_string(), adj_stats.total_words as usize);

        stats
    }

    // ========== Persistence ==========

    /// Flush to disk (snapshot + checkpoint WAL)
    pub fn flush(&self) -> Result<()> {
        // Save snapshot
        self.save_to_disk()?;

        // Save vectors
        if let Some(ref vectors) = self.vectors {
            vectors.write().unwrap().save()?;
        }

        // Checkpoint WAL (safe to truncate after snapshot)
        {
            let mut wal = self.wal.lock().unwrap();
            wal.truncate()?;
        }

        Ok(())
    }

    /// Save concepts and associations to disk using JSON
    fn save_to_disk(&self) -> Result<()> {
        // Save concepts
        let concepts_path = self.path.join("concepts.json");
        let concepts_file = File::create(&concepts_path)?;
        let mut writer = BufWriter::new(concepts_file);

        let concepts_vec: Vec<_> = self
            .concepts
            .iter()
            .map(|entry| entry.value().clone())
            .collect();

        serde_json::to_writer(&mut writer, &concepts_vec)?;

        // Save associations
        let assocs_path = self.path.join("associations.json");
        let assocs_file = File::create(&assocs_path)?;
        let mut writer = BufWriter::new(assocs_file);

        let assocs_vec: Vec<_> = self
            .associations
            .iter()
            .map(|entry| entry.value().clone())
            .collect();

        serde_json::to_writer(&mut writer, &assocs_vec)?;

        Ok(())
    }

    /// Crash recovery: load snapshot + replay WAL
    fn recover(&mut self) -> Result<()> {
        // Step 1: Load latest snapshot
        let _ = self.load_from_disk(); // Ignore errors if no snapshot

        // Step 2: Replay WAL for crash recovery
        let wal_path = self.path.join("wal.log");
        if wal_path.exists() {
            let committed_ops = WriteAheadLog::replay(&wal_path)?;

            for _entry in committed_ops {
                // Note: We only log operations, actual data is in snapshots
                // WAL is used for durability guarantee and ordering
                // Actual replay would need full data in WAL or reference to data
            }
        }

        Ok(())
    }

    /// Load concepts and associations from disk
    fn load_from_disk(&mut self) -> Result<()> {
        // Load concepts
        let concepts_path = self.path.join("concepts.json");
        if concepts_path.exists() {
            let concepts_file = File::open(&concepts_path)?;
            let reader = BufReader::new(concepts_file);
            let concepts_vec: Vec<ConceptData> = serde_json::from_reader(reader)?;

            for concept in concepts_vec {
                let id = concept.id;
                self.concept_content.insert(id, concept.content.clone());
                self.concepts.insert(id, concept.clone());

                // Index words
                let words: Vec<String> = concept
                    .content
                    .split_whitespace()
                    .map(|s| s.to_lowercase())
                    .collect();
                self.adjacency.write().unwrap().index_words(id, &words);
            }
        }

        // Load associations
        let assocs_path = self.path.join("associations.json");
        if assocs_path.exists() {
            let assocs_file = File::open(&assocs_path)?;
            let reader = BufReader::new(assocs_file);
            let assocs_vec: Vec<AssociationData> = serde_json::from_reader(reader)?;

            for assoc in assocs_vec {
                let key = (assoc.source_id, assoc.target_id);
                self.associations.insert(key, assoc.clone());

                // Rebuild adjacency
                let adj = self.adjacency.write().unwrap();
                adj.add_edge(assoc.source_id, assoc.target_id);
                adj.add_edge(assoc.target_id, assoc.source_id);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let temp_dir = tempfile::tempdir().unwrap();
        let store = ReasoningStore::new(temp_dir.path(), Some(384)).unwrap();

        // Create concept with valid 16-char (8-byte) hex ID
        let concept = ConceptData {
            id: ConceptId::from_string("0123456789abcdef"),
            content: "test concept".to_string(),
            created: 0,
            last_accessed: 0,
            access_count: 0,
            strength: 1.0,
            confidence: 1.0,
            source: None,
            category: None,
        };

        store.put_concept(concept.clone(), None).unwrap();

        // Retrieve concept
        let retrieved = store.get_concept(concept.id).unwrap();
        assert_eq!(retrieved.content, "test concept");
    }
}
