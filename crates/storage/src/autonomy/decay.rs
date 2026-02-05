//! Knowledge Decay
//!
//! Background loop that applies exponential strength decay to concepts based on
//! time since last access. Reinforces frequently accessed concepts and prunes
//! concepts that fall below a threshold.

use crate::concurrent_memory::ConcurrentMemory;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Configuration for the decay loop
#[derive(Debug, Clone)]
pub struct DecayConfig {
    /// Whether decay is enabled
    pub enabled: bool,
    /// Interval between decay cycles
    pub interval: Duration,
    /// Exponential decay rate (per second)
    pub decay_rate: f64,
    /// Reinforcement bonus multiplier for access count
    pub reinforcement_bonus: f64,
    /// Concepts with strength below this are pruned
    pub prune_threshold: f32,
    /// Number of concepts to process per batch
    pub batch_size: usize,
}

impl Default for DecayConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: Duration::from_secs(5),
            decay_rate: 0.0001, // Very slow decay
            reinforcement_bonus: 0.01,
            prune_threshold: 0.01,
            batch_size: 1000,
        }
    }
}

/// Background decay loop handle
pub struct DecayLoop {
    running: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl DecayLoop {
    pub fn start(config: DecayConfig, storage: Arc<ConcurrentMemory>) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = Arc::clone(&running);

        let handle = thread::spawn(move || {
            decay_loop(config, storage, running_clone);
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

impl Drop for DecayLoop {
    fn drop(&mut self) {
        self.stop();
    }
}

fn decay_loop(config: DecayConfig, storage: Arc<ConcurrentMemory>, running: Arc<AtomicBool>) {
    log::info!(
        "Decay loop started (interval={:?}, rate={}, prune={})",
        config.interval,
        config.decay_rate,
        config.prune_threshold
    );

    while running.load(Ordering::Relaxed) {
        thread::sleep(config.interval);
        if !running.load(Ordering::Relaxed) {
            break;
        }

        let snapshot = storage.get_snapshot();
        let now_us = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64;

        let mut updated = 0usize;
        let mut pruned = 0usize;

        // Process concepts in batches using iterator
        for concept in snapshot.concepts.values() {
            if !running.load(Ordering::Relaxed) {
                break;
            }

            // Skip system-generated concepts (self-monitor, gap-detector, etc.)
            if concept
                .attributes
                .get("sutra:source")
                .is_some_and(|s| s == "self_monitor" || s == "gap_detector")
            {
                continue;
            }

            let seconds_since_access = if now_us > concept.last_accessed {
                (now_us - concept.last_accessed) as f64 / 1_000_000.0
            } else {
                0.0
            };

            // Exponential decay: new_strength = strength * exp(-decay_rate * seconds)
            let decayed =
                concept.strength as f64 * (-config.decay_rate * seconds_since_access).exp();

            // Reinforcement: bonus based on access count
            let reinforcement =
                config.reinforcement_bonus * (1.0 + concept.access_count as f64).ln();

            let new_strength = (decayed + reinforcement).clamp(0.0, 1.0) as f32;

            if new_strength < config.prune_threshold {
                let _ = storage.delete_concept(concept.id);
                pruned += 1;
            } else if (new_strength - concept.strength).abs() > 0.001 {
                let _ = storage.update_strength(concept.id, new_strength);
                updated += 1;
            }
        }

        if updated > 0 || pruned > 0 {
            log::debug!(
                "Decay cycle: {} updated, {} pruned (of {} concepts)",
                updated,
                pruned,
                snapshot.concept_count
            );
        }
    }

    log::info!("Decay loop stopped");
}
