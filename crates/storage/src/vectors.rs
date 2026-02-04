/// Dense vector storage with compression
///
/// Stores float32 vectors efficiently with optional Product Quantization.
/// Supports incremental updates and persistence.
use anyhow::{Context, Result};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::quantization::ProductQuantizer;
use crate::types::ConceptId;

/// Vector storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorConfig {
    /// Vector dimensionality
    pub dimension: usize,
    /// Whether to use compression
    pub use_compression: bool,
    /// Number of subvectors for Product Quantization
    pub num_subvectors: usize,
    /// Number of centroids per subvector
    pub num_centroids: usize,
}

impl Default for VectorConfig {
    fn default() -> Self {
        Self {
            dimension: 384,
            use_compression: true,
            num_subvectors: 48, // 384 / 8
            num_centroids: 256,
        }
    }
}

/// Vector metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorMetadata {
    /// Concept ID
    pub concept_id: ConceptId,
    /// Vector dimension
    pub dimension: usize,
    /// Whether compressed
    pub compressed: bool,
    /// Timestamp
    pub timestamp: u64,
}

/// Vector storage with optional compression
pub struct VectorStore {
    /// Configuration
    config: VectorConfig,
    /// Base path
    path: PathBuf,
    /// Raw vectors (concept_id -> vector)
    raw_vectors: Arc<RwLock<HashMap<ConceptId, Vec<f32>>>>,
    /// Product quantizer (optional)
    quantizer: Arc<RwLock<Option<ProductQuantizer>>>,
    /// Compressed vectors (concept_id -> codes)
    compressed_vectors: Arc<RwLock<HashMap<ConceptId, Vec<u8>>>>,
    /// Metadata
    metadata: Arc<RwLock<HashMap<ConceptId, VectorMetadata>>>,
}

impl VectorStore {
    /// Create a new vector store
    pub fn new<P: AsRef<Path>>(path: P, config: VectorConfig) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        std::fs::create_dir_all(&path)?;

        Ok(Self {
            config,
            path,
            raw_vectors: Arc::new(RwLock::new(HashMap::new())),
            quantizer: Arc::new(RwLock::new(None)),
            compressed_vectors: Arc::new(RwLock::new(HashMap::new())),
            metadata: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Load from disk
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        // Load config
        let config_path = path.join("config.json");
        let config: VectorConfig = if config_path.exists() {
            let file = File::open(&config_path)?;
            serde_json::from_reader(file)?
        } else {
            VectorConfig::default()
        };

        let store = Self::new(&path, config)?;

        // Load quantizer if exists
        let quantizer_path = path.join("quantizer.bin");
        if quantizer_path.exists() {
            let quantizer = ProductQuantizer::load(&quantizer_path)?;
            *store.quantizer.write() = Some(quantizer);
        }

        // Load vectors
        let vectors_path = path.join("vectors.bin");
        if vectors_path.exists() {
            store.load_vectors(&vectors_path)?;
        }

        Ok(store)
    }

    /// Add a vector
    pub fn add_vector(&self, concept_id: ConceptId, vector: Vec<f32>) -> Result<()> {
        if vector.len() != self.config.dimension {
            anyhow::bail!(
                "Vector dimension mismatch: expected {}, got {}",
                self.config.dimension,
                vector.len()
            );
        }

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64;

        // Store raw vector
        self.raw_vectors.write().insert(concept_id, vector.clone());

        // Compress if enabled and quantizer is trained
        if self.config.use_compression {
            if let Some(quantizer) = self.quantizer.read().as_ref() {
                let codes = quantizer.encode(&vector)?;
                self.compressed_vectors.write().insert(concept_id, codes);
            }
        }

        // Store metadata
        let metadata = VectorMetadata {
            concept_id,
            dimension: vector.len(),
            compressed: self.config.use_compression && self.quantizer.read().is_some(),
            timestamp,
        };
        self.metadata.write().insert(concept_id, metadata);

        Ok(())
    }

    /// Get a vector (returns raw vector)
    pub fn get_vector(&self, concept_id: ConceptId) -> Option<Vec<f32>> {
        self.raw_vectors.read().get(&concept_id).cloned()
    }

    /// Get compressed codes
    pub fn get_compressed(&self, concept_id: ConceptId) -> Option<Vec<u8>> {
        self.compressed_vectors.read().get(&concept_id).cloned()
    }

    /// Remove a vector
    pub fn remove_vector(&self, concept_id: ConceptId) -> Result<()> {
        self.raw_vectors.write().remove(&concept_id);
        self.compressed_vectors.write().remove(&concept_id);
        self.metadata.write().remove(&concept_id);
        Ok(())
    }

    /// Train the quantizer on existing vectors
    pub fn train_quantizer(&self, training_vectors: Option<Vec<Vec<f32>>>) -> Result<()> {
        if !self.config.use_compression {
            return Ok(());
        }

        let vectors = if let Some(vecs) = training_vectors {
            vecs
        } else {
            // Use all stored vectors for training
            self.raw_vectors.read().values().cloned().collect()
        };

        if vectors.is_empty() {
            anyhow::bail!("No vectors available for training");
        }

        let mut quantizer = ProductQuantizer::new(
            self.config.dimension,
            self.config.num_subvectors,
            self.config.num_centroids,
        );

        quantizer.train(&vectors)?;

        *self.quantizer.write() = Some(quantizer);

        // Compress all existing vectors
        self.compress_all()?;

        Ok(())
    }

    /// Compress all stored vectors
    fn compress_all(&self) -> Result<()> {
        let quantizer = self.quantizer.read();
        let quantizer = quantizer.as_ref().context("Quantizer not trained")?;

        let raw = self.raw_vectors.read();
        let mut compressed = self.compressed_vectors.write();

        for (id, vector) in raw.iter() {
            let codes = quantizer.encode(vector)?;
            compressed.insert(*id, codes);
        }

        Ok(())
    }

    /// Compute distance between two concepts
    pub fn distance(&self, id1: ConceptId, id2: ConceptId) -> Result<f32> {
        // Use raw vectors for accurate distance
        let v1 = self
            .raw_vectors
            .read()
            .get(&id1)
            .context("Vector 1 not found")?
            .clone();
        let v2 = self
            .raw_vectors
            .read()
            .get(&id2)
            .context("Vector 2 not found")?
            .clone();

        Ok(Self::cosine_distance(&v1, &v2))
    }

    /// Compute approximate distance using compressed vectors
    pub fn approximate_distance(&self, id1: ConceptId, id2: ConceptId) -> Result<f32> {
        let quantizer = self.quantizer.read();
        let quantizer = quantizer.as_ref().context("Quantizer not trained")?;

        let codes1 = self
            .compressed_vectors
            .read()
            .get(&id1)
            .context("Compressed vector 1 not found")?
            .clone();
        let codes2 = self
            .compressed_vectors
            .read()
            .get(&id2)
            .context("Compressed vector 2 not found")?
            .clone();

        quantizer.compute_distance(&codes1, &codes2)
    }

    /// Cosine distance between two vectors
    fn cosine_distance(v1: &[f32], v2: &[f32]) -> f32 {
        let dot: f32 = v1.iter().zip(v2.iter()).map(|(a, b)| a * b).sum();
        let norm1: f32 = v1.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm2: f32 = v2.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm1 == 0.0 || norm2 == 0.0 {
            return 1.0; // Maximum distance
        }

        1.0 - (dot / (norm1 * norm2))
    }

    /// Save to disk
    pub fn save(&self) -> Result<()> {
        // Save config
        let config_path = self.path.join("config.json");
        let file = File::create(&config_path)?;
        serde_json::to_writer_pretty(file, &self.config)?;

        // Save quantizer if trained
        if let Some(quantizer) = self.quantizer.read().as_ref() {
            let quantizer_path = self.path.join("quantizer.bin");
            quantizer.save(&quantizer_path)?;
        }

        // Save vectors
        let vectors_path = self.path.join("vectors.bin");
        self.save_vectors(&vectors_path)?;

        Ok(())
    }

    /// Save vectors to binary file
    fn save_vectors(&self, path: &Path) -> Result<()> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;
        let mut writer = BufWriter::new(file);

        let raw = self.raw_vectors.read();
        let compressed = self.compressed_vectors.read();
        let metadata_map = self.metadata.read();

        // Write count
        let count = raw.len() as u32;
        writer.write_all(&count.to_le_bytes())?;

        // Write each vector
        for (id, vector) in raw.iter() {
            // Write concept ID
            writer.write_all(&id.0)?;

            // Write metadata
            if let Some(meta) = metadata_map.get(id) {
                let meta_json = serde_json::to_string(meta)?;
                let meta_len = meta_json.len() as u32;
                writer.write_all(&meta_len.to_le_bytes())?;
                writer.write_all(meta_json.as_bytes())?;
            } else {
                writer.write_all(&0u32.to_le_bytes())?;
            }

            // Write raw vector
            let vec_len = vector.len() as u32;
            writer.write_all(&vec_len.to_le_bytes())?;
            for &val in vector {
                writer.write_all(&val.to_le_bytes())?;
            }

            // Write compressed codes if available
            if let Some(codes) = compressed.get(id) {
                let codes_len = codes.len() as u32;
                writer.write_all(&codes_len.to_le_bytes())?;
                writer.write_all(codes)?;
            } else {
                writer.write_all(&0u32.to_le_bytes())?;
            }
        }

        writer.flush()?;
        Ok(())
    }

    /// Load vectors from binary file
    fn load_vectors(&self, path: &Path) -> Result<()> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        // Read count
        let mut count_bytes = [0u8; 4];
        reader.read_exact(&mut count_bytes)?;
        let count = u32::from_le_bytes(count_bytes);

        let mut raw = self.raw_vectors.write();
        let mut compressed = self.compressed_vectors.write();
        let mut metadata_map = self.metadata.write();

        // Read each vector
        for _ in 0..count {
            // Read concept ID
            let mut id_bytes = [0u8; 16];
            reader.read_exact(&mut id_bytes)?;
            let concept_id = ConceptId(id_bytes);

            // Read metadata
            let mut meta_len_bytes = [0u8; 4];
            reader.read_exact(&mut meta_len_bytes)?;
            let meta_len = u32::from_le_bytes(meta_len_bytes);

            if meta_len > 0 {
                let mut meta_bytes = vec![0u8; meta_len as usize];
                reader.read_exact(&mut meta_bytes)?;
                let meta: VectorMetadata = serde_json::from_slice(&meta_bytes)?;
                metadata_map.insert(concept_id, meta);
            }

            // Read raw vector
            let mut vec_len_bytes = [0u8; 4];
            reader.read_exact(&mut vec_len_bytes)?;
            let vec_len = u32::from_le_bytes(vec_len_bytes);

            let mut vector = Vec::with_capacity(vec_len as usize);
            for _ in 0..vec_len {
                let mut val_bytes = [0u8; 4];
                reader.read_exact(&mut val_bytes)?;
                vector.push(f32::from_le_bytes(val_bytes));
            }
            raw.insert(concept_id, vector);

            // Read compressed codes if available
            let mut codes_len_bytes = [0u8; 4];
            reader.read_exact(&mut codes_len_bytes)?;
            let codes_len = u32::from_le_bytes(codes_len_bytes);

            if codes_len > 0 {
                let mut codes = vec![0u8; codes_len as usize];
                reader.read_exact(&mut codes)?;
                compressed.insert(concept_id, codes);
            }
        }

        Ok(())
    }

    /// Get statistics
    pub fn stats(&self) -> VectorStats {
        let raw = self.raw_vectors.read();
        let compressed = self.compressed_vectors.read();

        let raw_count = raw.len();
        let compressed_count = compressed.len();

        // Estimate memory usage
        let raw_size = raw.values().map(|v| v.len() * 4).sum::<usize>();
        let compressed_size = compressed.values().map(|c| c.len()).sum::<usize>();

        let compression_ratio = if compressed_size > 0 {
            raw_size as f32 / compressed_size as f32
        } else {
            1.0
        };

        VectorStats {
            total_vectors: raw_count,
            compressed_vectors: compressed_count,
            dimension: self.config.dimension,
            raw_size_bytes: raw_size,
            compressed_size_bytes: compressed_size,
            compression_ratio,
            quantizer_trained: self.quantizer.read().is_some(),
        }
    }

    /// Clear all vectors
    pub fn clear(&self) {
        self.raw_vectors.write().clear();
        self.compressed_vectors.write().clear();
        self.metadata.write().clear();
    }
}

/// Vector statistics
#[derive(Debug, Clone)]
pub struct VectorStats {
    pub total_vectors: usize,
    pub compressed_vectors: usize,
    pub dimension: usize,
    pub raw_size_bytes: usize,
    pub compressed_size_bytes: usize,
    pub compression_ratio: f32,
    pub quantizer_trained: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_concept_id(id: u64) -> ConceptId {
        let mut bytes = [0u8; 16];
        bytes[0..8].copy_from_slice(&id.to_le_bytes());
        ConceptId(bytes)
    }

    fn random_vector(dim: usize) -> Vec<f32> {
        (0..dim).map(|i| (i as f32) * 0.01).collect()
    }

    #[test]
    fn test_create_vector_store() {
        let dir = TempDir::new().unwrap();
        let config = VectorConfig::default();

        let store = VectorStore::new(dir.path(), config).unwrap();
        let stats = store.stats();

        assert_eq!(stats.total_vectors, 0);
        assert_eq!(stats.dimension, 384);
    }

    #[test]
    fn test_add_get_vector() {
        let dir = TempDir::new().unwrap();
        let config = VectorConfig {
            dimension: 128,
            use_compression: false,
            ..Default::default()
        };

        let store = VectorStore::new(dir.path(), config).unwrap();
        let vector = random_vector(128);
        let id = test_concept_id(1);

        store.add_vector(id, vector.clone()).unwrap();

        let retrieved = store.get_vector(id).unwrap();
        assert_eq!(retrieved.len(), 128);
        assert_eq!(retrieved, vector);
    }

    #[test]
    fn test_remove_vector() {
        let dir = TempDir::new().unwrap();
        let config = VectorConfig {
            dimension: 64,
            use_compression: false,
            ..Default::default()
        };

        let store = VectorStore::new(dir.path(), config).unwrap();
        let vector = random_vector(64);
        let id = test_concept_id(1);

        store.add_vector(id, vector).unwrap();
        assert!(store.get_vector(id).is_some());

        store.remove_vector(id).unwrap();
        assert!(store.get_vector(id).is_none());
    }

    #[test]
    fn test_distance_computation() {
        let dir = TempDir::new().unwrap();
        let config = VectorConfig {
            dimension: 64,
            use_compression: false,
            ..Default::default()
        };

        let store = VectorStore::new(dir.path(), config).unwrap();

        let v1 = vec![1.0; 64];
        let v2 = vec![1.0; 64];
        let v3 = vec![-1.0; 64];

        let id1 = test_concept_id(1);
        let id2 = test_concept_id(2);
        let id3 = test_concept_id(3);

        store.add_vector(id1, v1).unwrap();
        store.add_vector(id2, v2).unwrap();
        store.add_vector(id3, v3).unwrap();

        // Same vectors should have 0 distance
        let dist12 = store.distance(id1, id2).unwrap();
        assert!(dist12 < 0.001);

        // Opposite vectors should have max distance
        let dist13 = store.distance(id1, id3).unwrap();
        assert!(dist13 > 1.99); // ~2.0
    }

    #[test]
    fn test_save_load() {
        let dir = TempDir::new().unwrap();
        let config = VectorConfig {
            dimension: 64,
            use_compression: false,
            ..Default::default()
        };

        let vector = random_vector(64);
        let id = test_concept_id(42);

        {
            let store = VectorStore::new(dir.path(), config.clone()).unwrap();
            store.add_vector(id, vector.clone()).unwrap();
            store.save().unwrap();
        }

        // Load and verify
        let store = VectorStore::load(dir.path()).unwrap();
        let retrieved = store.get_vector(id).unwrap();
        assert_eq!(retrieved, vector);
    }

    #[test]
    fn test_stats() {
        let dir = TempDir::new().unwrap();
        let config = VectorConfig {
            dimension: 64,
            use_compression: false,
            ..Default::default()
        };

        let store = VectorStore::new(dir.path(), config).unwrap();

        for i in 0..10 {
            let vector = random_vector(64);
            store.add_vector(test_concept_id(i), vector).unwrap();
        }

        let stats = store.stats();
        assert_eq!(stats.total_vectors, 10);
        assert_eq!(stats.dimension, 64);
        assert_eq!(stats.raw_size_bytes, 10 * 64 * 4); // 10 vectors * 64 dims * 4 bytes
    }

    #[test]
    fn test_compression() {
        let dir = TempDir::new().unwrap();
        let config = VectorConfig {
            dimension: 64,
            use_compression: true,
            num_subvectors: 8,
            num_centroids: 16,
        };

        let store = VectorStore::new(dir.path(), config).unwrap();

        // Add vectors
        for i in 0..50 {
            let vector = random_vector(64);
            store.add_vector(test_concept_id(i), vector).unwrap();
        }

        // Train quantizer
        store.train_quantizer(None).unwrap();

        // Check stats
        let stats = store.stats();
        assert_eq!(stats.total_vectors, 50);
        assert_eq!(stats.compressed_vectors, 50);
        assert!(stats.compression_ratio > 1.0);

        // Verify approximate distance works
        let id1 = test_concept_id(0);
        let id2 = test_concept_id(1);

        let exact_dist = store.distance(id1, id2).unwrap();
        let approx_dist = store.approximate_distance(id1, id2).unwrap();

        // Distances should be in same ballpark
        assert!(exact_dist >= 0.0);
        assert!(approx_dist >= 0.0);
    }
}
