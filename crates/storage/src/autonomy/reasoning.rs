//! Background Reasoning
//!
//! Discovers associations between similar concepts that aren't yet connected,
//! detects contradictions between neighbors, and strengthens connected pairs.

use crate::concurrent_memory::ConcurrentMemory;
use crate::semantic::{DomainContext, SemanticMetadata, SemanticType};
use crate::types::{AssociationType, ConceptId};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Configuration for the reasoning loop
#[derive(Debug, Clone)]
pub struct ReasoningConfig {
    /// Whether background reasoning is enabled
    pub enabled: bool,
    /// Interval between reasoning cycles
    pub interval: Duration,
    /// Number of random concepts to sample per cycle
    pub sample_size: usize,
    /// Similarity threshold for automatic association discovery
    pub similarity_threshold: f32,
    /// Number of nearest neighbors to check per concept
    pub neighbor_k: usize,
    /// Strength boost for connected pairs per cycle
    pub connection_boost: f32,
}

impl Default for ReasoningConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: Duration::from_secs(10),
            sample_size: 20,
            similarity_threshold: 0.75,
            neighbor_k: 5,
            connection_boost: 0.01,
        }
    }
}

/// Background reasoning loop handle
pub struct ReasoningLoop {
    running: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl ReasoningLoop {
    pub fn start(config: ReasoningConfig, storage: Arc<ConcurrentMemory>) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = Arc::clone(&running);

        let handle = thread::spawn(move || {
            reasoning_loop(config, storage, running_clone);
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

impl Drop for ReasoningLoop {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Simple xorshift PRNG (no external deps needed)
struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 1 } else { seed },
        }
    }

    fn next(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }
}

fn reasoning_loop(
    config: ReasoningConfig,
    storage: Arc<ConcurrentMemory>,
    running: Arc<AtomicBool>,
) {
    log::info!(
        "Reasoning loop started (interval={:?}, sample_size={}, threshold={})",
        config.interval,
        config.sample_size,
        config.similarity_threshold
    );

    let mut rng = XorShift64::new(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64,
    );

    while running.load(Ordering::Relaxed) {
        thread::sleep(config.interval);
        if !running.load(Ordering::Relaxed) {
            break;
        }

        let snapshot = storage.get_snapshot();
        let concept_ids: Vec<ConceptId> = snapshot.concepts.keys().copied().collect();

        if concept_ids.is_empty() {
            continue;
        }

        let mut new_associations = 0usize;
        let mut contradictions = 0usize;
        let mut strengthened = 0usize;

        // Sample random concepts
        let sample_size = config.sample_size.min(concept_ids.len());
        for _ in 0..sample_size {
            if !running.load(Ordering::Relaxed) {
                break;
            }

            let idx = (rng.next() as usize) % concept_ids.len();
            let concept_id = concept_ids[idx];

            let concept = match snapshot.get_concept(&concept_id) {
                Some(c) => c,
                None => continue,
            };

            // Skip system-generated concepts
            if concept.attributes.contains_key("sutra:source") {
                continue;
            }

            // Association discovery: find similar concepts via vector search
            if let Some(ref vector) = concept.vector {
                let results = storage.vector_search(vector, config.neighbor_k, 50);

                for (neighbor_id, similarity) in results {
                    if neighbor_id == concept_id {
                        continue;
                    }

                    if similarity > config.similarity_threshold {
                        // Check if path already exists
                        let path = storage.find_path(concept_id, neighbor_id, 3);
                        if path.is_none() {
                            // Create new semantic association
                            let _ = storage.learn_association(
                                concept_id,
                                neighbor_id,
                                AssociationType::Semantic,
                                similarity,
                            );
                            new_associations += 1;
                        }
                    }
                }
            }

            // Contradiction detection
            if let Some(ref semantic) = concept.semantic {
                for &neighbor_id in &concept.neighbors {
                    if let Some(neighbor) = snapshot.get_concept(&neighbor_id) {
                        if let Some(ref neighbor_semantic) = neighbor.semantic {
                            if semantic.conflicts_with(neighbor_semantic) {
                                // Store contradiction as a Negation concept
                                let content = format!(
                                    "Contradiction detected: {} vs {}",
                                    concept_id.to_hex(),
                                    neighbor_id.to_hex()
                                );
                                let id_source = format!(
                                    "sutra:contradiction:{}:{}",
                                    concept_id.to_hex(),
                                    neighbor_id.to_hex()
                                );
                                let contradiction_id = ConceptId::from_string(&id_source);

                                let mut neg_semantic =
                                    SemanticMetadata::new(SemanticType::Negation);
                                neg_semantic.domain_context = DomainContext::Technical;

                                let _ = storage.learn_concept_with_semantic(
                                    contradiction_id,
                                    content.into_bytes(),
                                    None,
                                    0.8,
                                    0.9,
                                    neg_semantic,
                                );
                                contradictions += 1;
                            }
                        }
                    }
                }
            }

            // Strengthen connected pairs
            if !concept.neighbors.is_empty() {
                let new_strength = (concept.strength + config.connection_boost).min(1.0);
                if (new_strength - concept.strength).abs() > 0.001 {
                    let _ = storage.update_strength(concept_id, new_strength);
                    strengthened += 1;
                }
            }
        }

        if new_associations > 0 || contradictions > 0 || strengthened > 0 {
            log::debug!(
                "Reasoning cycle: {} new associations, {} contradictions, {} strengthened",
                new_associations,
                contradictions,
                strengthened
            );
        }
    }

    log::info!("Reasoning loop stopped");
}
