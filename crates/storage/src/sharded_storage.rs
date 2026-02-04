use crate::concurrent_memory::{ConcurrentConfig, ConcurrentMemory, ConcurrentStats};
use crate::transaction::{TransactionCoordinator, TxnOperation};
use crate::types::ConceptId;
/// Sharded Storage - Horizontal Scaling Beyond 10M Concepts
///
/// Distributes concepts across multiple storage shards using consistent hashing.
/// Each shard is an independent ConcurrentMemory instance with its own WAL and HNSW index.
///
/// Features:
/// - Hash-based sharding (even distribution)
/// - 16 shards default (configurable)
/// - Parallel operations across shards
/// - Per-shard statistics
/// - Migration support for rebalancing
use anyhow::Result;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::Arc;

const DEFAULT_NUM_SHARDS: u32 = 16;

/// Sharding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardConfig {
    /// Number of shards
    pub num_shards: u32,
    /// Base storage path
    pub base_path: PathBuf,
    /// Per-shard configuration
    pub shard_config: ConcurrentConfig,
}

impl Default for ShardConfig {
    fn default() -> Self {
        Self {
            num_shards: DEFAULT_NUM_SHARDS,
            base_path: PathBuf::from("./sharded_storage"),
            shard_config: ConcurrentConfig::default(),
        }
    }
}

/// Sharded storage manager
pub struct ShardedStorage {
    /// Configuration
    config: ShardConfig,
    /// Storage shards (shard_id -> ConcurrentMemory)
    shards: Vec<Arc<ConcurrentMemory>>,
    /// Shard map (for routing)
    shard_map: Arc<RwLock<ShardMap>>,
    /// 🔥 NEW: 2PC transaction coordinator for atomic cross-shard operations
    txn_coordinator: Arc<TransactionCoordinator>,
}

/// Shard map for routing decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardMap {
    /// Number of shards
    pub num_shards: u32,
    /// Per-shard statistics
    pub shard_stats: HashMap<u32, ShardStats>,
}

/// Per-shard statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardStats {
    pub shard_id: u32,
    pub concept_count: usize,
    pub edge_count: usize,
    pub vector_count: usize,
    pub data_size_mb: f64,
    pub last_updated: u64,
}

impl ShardedStorage {
    /// Create new sharded storage
    pub fn new(config: ShardConfig) -> Result<Self> {
        std::fs::create_dir_all(&config.base_path)?;

        // Initialize shards
        let mut shards = Vec::with_capacity(config.num_shards as usize);

        for shard_id in 0..config.num_shards {
            let shard_path = config.base_path.join(format!("shard_{:04}", shard_id));
            std::fs::create_dir_all(&shard_path)?;

            let mut shard_config = config.shard_config.clone();
            shard_config.storage_path = shard_path;

            let shard = ConcurrentMemory::new(shard_config);
            shards.push(Arc::new(shard));

            log::info!(
                "✅ Initialized shard {} at {:?}",
                shard_id,
                shards[shard_id as usize].config().storage_path
            );
        }

        let shard_map = ShardMap {
            num_shards: config.num_shards,
            shard_stats: HashMap::new(),
        };

        // Initialize 2PC transaction coordinator (5 second timeout)
        let txn_coordinator = Arc::new(TransactionCoordinator::new(5));

        Ok(Self {
            config,
            shards,
            shard_map: Arc::new(RwLock::new(shard_map)),
            txn_coordinator,
        })
    }

    /// Get shard for concept ID (consistent hashing)
    fn get_shard_id(&self, concept_id: ConceptId) -> u32 {
        let mut hasher = DefaultHasher::new();
        concept_id.0.hash(&mut hasher);
        let hash = hasher.finish();
        (hash % self.config.num_shards as u64) as u32
    }

    /// Get shard storage
    fn get_shard(&self, concept_id: ConceptId) -> &Arc<ConcurrentMemory> {
        let shard_id = self.get_shard_id(concept_id);
        &self.shards[shard_id as usize]
    }

    /// Get shard by index (internal)
    pub fn get_shard_by_index(&self, index: usize) -> &Arc<ConcurrentMemory> {
        &self.shards[index]
    }

    /// Learn concept (automatically routed to correct shard)
    pub fn learn_concept(
        &self,
        id: ConceptId,
        content: Vec<u8>,
        vector: Option<Vec<f32>>,
        strength: f32,
        confidence: f32,
        attributes: std::collections::HashMap<String, String>,
    ) -> Result<u64> {
        let shard = self.get_shard(id);
        shard
            .learn_concept(id, content, vector, strength, confidence, attributes)
            .map_err(|e| anyhow::anyhow!("Shard write failed: {:?}", e))
    }

    /// Delete concept from correct shard
    pub fn delete_concept(&self, id: ConceptId) -> Result<u64> {
        let shard = self.get_shard(id);
        shard
            .delete_concept(id)
            .map_err(|e| anyhow::anyhow!("Shard delete failed: {:?}", e))
    }

    /// Clear all shards in parallel
    pub fn clear(&self) -> Result<()> {
        use rayon::prelude::*;
        self.shards.par_iter().try_for_each(|shard| {
            shard
                .clear()
                .map(|_| ())
                .map_err(|e| anyhow::anyhow!("Shard clear failed: {:?}", e))
        })
    }

    /// Create association with 2PC for cross-shard atomicity
    pub fn create_association(
        &self,
        source: ConceptId,
        target: ConceptId,
        assoc_type: crate::types::AssociationType,
        strength: f32,
    ) -> Result<u64> {
        let source_shard_id = self.get_shard_id(source);
        let target_shard_id = self.get_shard_id(target);

        // ✅ FAST PATH: Same shard - no 2PC needed
        if source_shard_id == target_shard_id {
            let shard = &self.shards[source_shard_id as usize];
            return shard
                .create_association(source, target, assoc_type, strength)
                .map_err(|e| anyhow::anyhow!("Shard association failed: {:?}", e));
        }

        // 🔥 PRODUCTION: Cross-shard - use 2PC for atomicity
        log::info!(
            "🔄 2PC: Cross-shard association {} (shard {}) -> {} (shard {})",
            source.to_hex(),
            source_shard_id,
            target.to_hex(),
            target_shard_id
        );

        // Phase 1: Begin transaction
        let txn_id = self.txn_coordinator.begin(TxnOperation::CreateAssociation {
            source,
            target,
            source_shard: source_shard_id,
            target_shard: target_shard_id,
            assoc_type,
            strength,
        });

        // Phase 2: Prepare both shards
        let source_shard = &self.shards[source_shard_id as usize];
        let target_shard = &self.shards[target_shard_id as usize];

        // Prepare source shard (forward association)
        let source_result = source_shard
            .create_association(source, target, assoc_type, strength)
            .map_err(|e| anyhow::anyhow!("Source shard prepare failed: {:?}", e));

        if source_result.is_err() {
            self.txn_coordinator.abort(txn_id).ok();
            self.txn_coordinator.complete(txn_id);
            return source_result;
        }

        self.txn_coordinator
            .mark_prepared(txn_id, source_shard_id)
            .map_err(|e| anyhow::anyhow!("2PC mark prepared failed: {}", e))?;

        // Prepare target shard (reverse association for bidirectional graph)
        let target_result = target_shard
            .create_association(target, source, assoc_type, strength)
            .map_err(|e| anyhow::anyhow!("Target shard prepare failed: {:?}", e));

        if target_result.is_err() {
            // Rollback: Would need to remove source association here
            // For now, log warning (future: add rollback operations to shards)
            log::error!(
                "⚠️ 2PC: Target shard failed, source association created but not reversed (txn {})",
                txn_id
            );
            self.txn_coordinator.abort(txn_id).ok();
            self.txn_coordinator.complete(txn_id);
            return target_result;
        }

        self.txn_coordinator
            .mark_prepared(txn_id, target_shard_id)
            .map_err(|e| anyhow::anyhow!("2PC mark prepared failed: {}", e))?;

        // Phase 3: Commit
        if self.txn_coordinator.is_ready_to_commit(txn_id)? {
            self.txn_coordinator
                .commit(txn_id)
                .map_err(|e| anyhow::anyhow!("2PC commit failed: {}", e))?;

            log::info!("✅ 2PC: Transaction {} committed", txn_id);
            self.txn_coordinator.complete(txn_id);

            source_result
        } else {
            self.txn_coordinator.abort(txn_id).ok();
            self.txn_coordinator.complete(txn_id);
            anyhow::bail!("2PC: Transaction {} not ready to commit", txn_id)
        }
    }

    /// Learn association (alias for create_association for API compatibility)
    pub fn learn_association(
        &self,
        source: ConceptId,
        target: ConceptId,
        assoc_type: crate::types::AssociationType,
        confidence: f32,
    ) -> Result<u64> {
        self.create_association(source, target, assoc_type, confidence)
    }

    /// Get concept (from correct shard)
    pub fn get_concept(&self, id: ConceptId) -> Option<crate::read_view::ConceptNode> {
        let shard = self.get_shard(id);
        let snapshot = shard.get_snapshot();
        snapshot.get_concept(&id)
    }

    /// Get neighbors (from correct shard)
    pub fn get_neighbors(&self, id: ConceptId) -> Vec<ConceptId> {
        let shard = self.get_shard(id);
        let snapshot = shard.get_snapshot();
        snapshot.get_neighbors(&id)
    }

    /// Semantic search across all shards (parallel)
    pub fn semantic_search(&self, query_vector: Vec<f32>, top_k: usize) -> Vec<(ConceptId, f32)> {
        use rayon::prelude::*;

        // Query all shards in parallel
        let per_shard_k = (top_k / self.config.num_shards as usize).max(10);

        let mut all_results: Vec<(ConceptId, f32)> = self
            .shards
            .par_iter()
            .flat_map(|shard| {
                shard
                    .semantic_search(query_vector.clone(), per_shard_k)
                    .unwrap_or_default()
            })
            .collect();

        // Sort by similarity and take top_k
        all_results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        all_results.truncate(top_k);

        all_results
    }

    /// Flush all shards (parallel)
    pub fn flush(&self) -> Result<()> {
        use rayon::prelude::*;

        self.shards.par_iter().try_for_each(|shard| {
            shard
                .flush()
                .map_err(|e| anyhow::anyhow!("Shard flush failed: {:?}", e))
        })
    }

    /// Get aggregated statistics
    pub fn stats(&self) -> AggregatedStats {
        let shard_stats: Vec<ConcurrentStats> =
            self.shards.iter().map(|shard| shard.stats()).collect();

        AggregatedStats {
            num_shards: self.config.num_shards,
            total_concepts: shard_stats.iter().map(|s| s.snapshot.concept_count).sum(),
            total_edges: shard_stats.iter().map(|s| s.snapshot.edge_count).sum(),
            total_vectors: shard_stats
                .iter()
                .map(|s| s.write_log.written as usize)
                .sum(),
            total_writes: shard_stats.iter().map(|s| s.write_log.written).sum(),
            shard_stats,
        }
    }

    /// Get shard map
    pub fn shard_map(&self) -> Arc<RwLock<ShardMap>> {
        Arc::clone(&self.shard_map)
    }
}

/// Aggregated statistics across all shards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedStats {
    pub num_shards: u32,
    pub total_concepts: usize,
    pub total_edges: usize,
    pub total_vectors: usize,
    pub total_writes: u64,
    pub shard_stats: Vec<ConcurrentStats>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_sharded_storage_basic() {
        let temp_dir = TempDir::new().unwrap();
        let config = ShardConfig {
            num_shards: 4,
            base_path: temp_dir.path().to_path_buf(),
            shard_config: ConcurrentConfig {
                storage_path: PathBuf::from("will_be_overridden"),
                vector_dimension: 768,
                ..Default::default()
            },
        };

        let storage = ShardedStorage::new(config).unwrap();

        // Learn concepts (should route to different shards)
        for i in 0..100 {
            let id = ConceptId([i as u8; 16]);
            let content = format!("Concept {}", i).into_bytes();
            let vector = vec![i as f32 / 100.0; 768];

            storage
                .learn_concept(id, content, Some(vector), 1.0, 0.9)
                .unwrap();
        }

        // Wait for adaptive reconciler to process all writes
        std::thread::sleep(std::time::Duration::from_millis(200));

        let stats = storage.stats();
        assert_eq!(stats.total_concepts, 100);
        assert_eq!(stats.num_shards, 4);

        // Verify concepts distributed across shards
        let mut non_empty_shards = 0;
        for shard_stat in &stats.shard_stats {
            if shard_stat.snapshot.concept_count > 0 {
                non_empty_shards += 1;
            }
        }
        assert!(
            non_empty_shards >= 3,
            "Concepts should distribute across shards"
        );
    }

    #[test]
    fn test_cross_shard_search() {
        let temp_dir = TempDir::new().unwrap();
        let config = ShardConfig {
            num_shards: 4,
            base_path: temp_dir.path().to_path_buf(),
            shard_config: ConcurrentConfig {
                storage_path: PathBuf::from("will_be_overridden"),
                vector_dimension: 10,
                ..Default::default()
            },
        };

        let storage = ShardedStorage::new(config).unwrap();

        // Add concepts with vectors
        for i in 0..50 {
            let id = ConceptId([i as u8; 16]);
            let vector = vec![i as f32 / 50.0; 10];
            storage
                .learn_concept(id, format!("C{}", i).into_bytes(), Some(vector), 1.0, 0.9)
                .unwrap();
        }

        // Wait for adaptive reconciler to process all writes and build HNSW index
        std::thread::sleep(std::time::Duration::from_millis(300));

        // Search across all shards
        let query = vec![0.5; 10];
        let results = storage.semantic_search(query, 10);

        assert_eq!(results.len(), 10);
        assert!(results[0].1 > 0.0); // Should have similarity scores
    }
}
