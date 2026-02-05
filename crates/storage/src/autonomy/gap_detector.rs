//! Gap Detection
//!
//! Identifies knowledge gaps: isolated concepts, near-miss pairs (similar but
//! not connected), and incomplete causal chains. Stores gaps as concepts and
//! optionally notifies through the subscription system.

use super::subscriptions::SubscriptionManager;
use crate::concurrent_memory::ConcurrentMemory;
use crate::semantic::{DomainContext, SemanticMetadata, SemanticType};
use crate::types::ConceptId;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Configuration for gap detection
#[derive(Debug, Clone)]
pub struct GapDetectorConfig {
    /// Whether gap detection is enabled
    pub enabled: bool,
    /// Interval between detection cycles
    pub interval: Duration,
    /// Minimum neighbors to not be considered isolated
    pub isolation_threshold: usize,
    /// Lower similarity bound for near-miss detection
    pub near_miss_low: f32,
    /// Upper similarity bound for near-miss detection
    pub near_miss_high: f32,
    /// Number of concepts to sample per cycle
    pub sample_size: usize,
}

impl Default for GapDetectorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: Duration::from_secs(30),
            isolation_threshold: 1,
            near_miss_low: 0.6,
            near_miss_high: 0.75,
            sample_size: 50,
        }
    }
}

/// Background gap detection loop handle
pub struct GapDetectorLoop {
    running: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl GapDetectorLoop {
    pub fn start(
        config: GapDetectorConfig,
        storage: Arc<ConcurrentMemory>,
        subscriptions: Option<Arc<SubscriptionManager>>,
    ) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = Arc::clone(&running);

        let handle = thread::spawn(move || {
            gap_detection_loop(config, storage, subscriptions, running_clone);
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

impl Drop for GapDetectorLoop {
    fn drop(&mut self) {
        self.stop();
    }
}

fn gap_detection_loop(
    config: GapDetectorConfig,
    storage: Arc<ConcurrentMemory>,
    subscriptions: Option<Arc<SubscriptionManager>>,
    running: Arc<AtomicBool>,
) {
    log::info!(
        "Gap detector loop started (interval={:?}, isolation_threshold={})",
        config.interval,
        config.isolation_threshold
    );

    while running.load(Ordering::Relaxed) {
        thread::sleep(config.interval);
        if !running.load(Ordering::Relaxed) {
            break;
        }

        let snapshot = storage.get_snapshot();
        let mut isolated = 0usize;
        let mut near_misses = 0usize;
        let mut processed = 0usize;

        for concept in snapshot.concepts.values() {
            if !running.load(Ordering::Relaxed) {
                break;
            }

            if processed >= config.sample_size {
                break;
            }

            // Skip system-generated concepts
            if concept.attributes.contains_key("sutra:source") {
                continue;
            }

            processed += 1;

            // Detect isolated concepts
            if concept.neighbors.len() < config.isolation_threshold {
                let content = format!(
                    "Knowledge gap: isolated concept '{}' (id={})",
                    String::from_utf8_lossy(&concept.content)
                        .chars()
                        .take(100)
                        .collect::<String>(),
                    concept.id.to_hex()
                );

                store_gap(&storage, &content, &subscriptions);
                isolated += 1;
            }

            // Detect near-miss pairs via vector search
            if let Some(ref vector) = concept.vector {
                let results = storage.vector_search(vector, 5, 50);

                for (neighbor_id, similarity) in results {
                    if neighbor_id == concept.id {
                        continue;
                    }

                    if similarity >= config.near_miss_low
                        && similarity < config.near_miss_high
                        && !concept.neighbors.contains(&neighbor_id)
                    {
                        let content = format!(
                            "Knowledge gap: near-miss pair {} <-> {} (similarity={:.3})",
                            concept.id.to_hex(),
                            neighbor_id.to_hex(),
                            similarity
                        );

                        store_gap(&storage, &content, &subscriptions);
                        near_misses += 1;
                    }
                }
            }

            // Detect incomplete causal chains
            if let Some(ref semantic) = concept.semantic {
                if semantic.semantic_type == SemanticType::Causal && concept.neighbors.is_empty() {
                    let content = format!(
                        "Knowledge gap: causal leaf node '{}' (id={}) has no connections",
                        String::from_utf8_lossy(&concept.content)
                            .chars()
                            .take(100)
                            .collect::<String>(),
                        concept.id.to_hex()
                    );

                    store_gap(&storage, &content, &subscriptions);
                }
            }
        }

        if isolated > 0 || near_misses > 0 {
            log::debug!(
                "Gap detection cycle: {} isolated, {} near-misses (of {} sampled)",
                isolated,
                near_misses,
                processed
            );
        }
    }

    log::info!("Gap detector loop stopped");
}

fn store_gap(
    storage: &Arc<ConcurrentMemory>,
    content: &str,
    subscriptions: &Option<Arc<SubscriptionManager>>,
) {
    let id_source = format!("sutra:gap:{}", md5_hash(content));
    let concept_id = ConceptId::from_string(&id_source);

    let mut semantic = SemanticMetadata::new(SemanticType::Event);
    semantic.domain_context = DomainContext::Technical;

    let mut attributes = std::collections::HashMap::new();
    attributes.insert("sutra:source".to_string(), "gap_detector".to_string());
    attributes.insert("sutra:category".to_string(), "knowledge_gap".to_string());

    let _ = storage.learn_concept(
        concept_id,
        content.as_bytes().to_vec(),
        None,
        0.5,
        0.8,
        attributes,
    );

    // Notify subscribers
    if let Some(ref subs) = subscriptions {
        subs.notify(&concept_id.to_hex(), content, Some("event"));
    }
}

/// Simple hash for generating unique IDs
fn md5_hash(s: &str) -> String {
    let hash = md5::compute(s.as_bytes());
    format!("{:x}", hash)
}
