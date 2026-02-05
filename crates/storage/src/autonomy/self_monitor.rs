//! Self-Monitoring
//!
//! Background loop that periodically captures engine health stats and stores
//! them as concepts. Maintains a bounded history by pruning old health snapshots.

use crate::concurrent_memory::ConcurrentMemory;
use crate::semantic::{
    DomainContext, SemanticMetadata, SemanticType, TemporalBounds, TemporalRelation,
};
use crate::types::ConceptId;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Configuration for self-monitoring
#[derive(Debug, Clone)]
pub struct SelfMonitorConfig {
    /// Whether self-monitoring is enabled
    pub enabled: bool,
    /// Interval between health snapshots
    pub interval: Duration,
    /// Maximum number of health snapshots to retain
    pub max_history: usize,
    /// Initial strength for health concepts (decays faster than user concepts)
    pub health_concept_strength: f32,
}

impl Default for SelfMonitorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: Duration::from_secs(10),
            max_history: 1000,
            health_concept_strength: 0.5,
        }
    }
}

/// Background self-monitoring loop handle
pub struct SelfMonitorLoop {
    running: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl SelfMonitorLoop {
    pub fn start(config: SelfMonitorConfig, storage: Arc<ConcurrentMemory>) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = Arc::clone(&running);

        let handle = thread::spawn(move || {
            monitor_loop(config, storage, running_clone);
        });

        Self {
            running,
            handle: Some(handle),
        }
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for SelfMonitorLoop {
    fn drop(&mut self) {
        self.stop();
    }
}

fn monitor_loop(
    config: SelfMonitorConfig,
    storage: Arc<ConcurrentMemory>,
    running: Arc<AtomicBool>,
) {
    log::info!(
        "Self-monitor loop started (interval={:?}, max_history={})",
        config.interval,
        config.max_history
    );

    let mut emitted_ids: VecDeque<ConceptId> = VecDeque::new();

    while running.load(Ordering::Relaxed) {
        thread::sleep(config.interval);
        if !running.load(Ordering::Relaxed) {
            break;
        }

        let stats = storage.stats();
        let hnsw_stats = storage.hnsw_stats();

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // Build health snapshot as JSON
        let health = serde_json::json!({
            "concepts": stats.snapshot.concept_count,
            "edges": stats.snapshot.edge_count,
            "pending_writes": stats.write_log.pending,
            "written": stats.write_log.written,
            "dropped": stats.write_log.dropped,
            "reconciliations": stats.reconciler.reconciliations,
            "reconciler_health": stats.reconciler.health_score,
            "hnsw_vectors": hnsw_stats.indexed_vectors,
            "hnsw_dimension": hnsw_stats.dimension,
            "hnsw_ready": hnsw_stats.index_ready,
            "timestamp": now,
        });

        let content = format!("Engine health: {}", health);
        let content_bytes = content.into_bytes();

        // Create concept ID from content + timestamp for uniqueness
        let id_source = format!("sutra:health:{}", now);
        let concept_id = ConceptId::from_string(&id_source);

        let mut semantic = SemanticMetadata::new(SemanticType::Event);
        semantic.domain_context = DomainContext::Technical;
        semantic.temporal_bounds = Some(TemporalBounds::new(
            Some(now),
            Some(now),
            TemporalRelation::At,
        ));

        match storage.learn_concept_with_semantic(
            concept_id,
            content_bytes,
            None,
            config.health_concept_strength,
            1.0,
            semantic,
        ) {
            Ok(_) => {
                emitted_ids.push_back(concept_id);

                // Prune oldest if over limit
                while emitted_ids.len() > config.max_history {
                    if let Some(old_id) = emitted_ids.pop_front() {
                        let _ = storage.delete_concept(old_id);
                    }
                }
            }
            Err(e) => {
                log::warn!("Self-monitor failed to store health concept: {:?}", e);
            }
        }
    }

    log::info!("Self-monitor loop stopped");
}
