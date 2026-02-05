//! Feedback Integration
//!
//! Processes accept/reject signals from users to adjust concept strengths.
//! Synchronous processor invoked directly from the protocol handler.

use crate::concurrent_memory::ConcurrentMemory;
use crate::types::ConceptId;
use std::sync::Arc;

/// Configuration for feedback processing
#[derive(Debug, Clone)]
pub struct FeedbackConfig {
    /// Strength boost for accepted results
    pub accept_boost: f32,
    /// Strength penalty for rejected results
    pub reject_penalty: f32,
    /// Maximum proportional ranking boost
    pub max_ranking_boost: f32,
}

impl Default for FeedbackConfig {
    fn default() -> Self {
        Self {
            accept_boost: 0.1,
            reject_penalty: 0.05,
            max_ranking_boost: 0.15,
        }
    }
}

/// Processes feedback signals to adjust concept strengths
pub struct FeedbackProcessor {
    config: FeedbackConfig,
}

impl FeedbackProcessor {
    pub fn new(config: FeedbackConfig) -> Self {
        Self { config }
    }

    /// Process feedback for a set of query results.
    ///
    /// - `result_concept_ids`: IDs of concepts returned in a query result
    /// - `accepted`: parallel bool vec indicating whether each result was accepted
    /// - `ranking`: optional ranking order (lower = better) for accepted results
    ///
    /// Returns the number of adjustments made.
    pub fn process(
        &self,
        storage: &Arc<ConcurrentMemory>,
        result_concept_ids: &[String],
        accepted: &[bool],
        ranking: Option<&[u32]>,
    ) -> usize {
        let mut adjustments = 0;
        let snapshot = storage.get_snapshot();

        for (i, id_str) in result_concept_ids.iter().enumerate() {
            let concept_id = ConceptId::from_string(id_str);

            let current_strength = match snapshot.get_concept(&concept_id) {
                Some(node) => node.strength,
                None => continue,
            };

            let is_accepted = accepted.get(i).copied().unwrap_or(false);

            if is_accepted {
                // Base boost for accepted results
                let mut boost = self.config.accept_boost;

                // Proportional ranking boost if ranking provided
                if let Some(ranks) = ranking {
                    if let Some(&rank) = ranks.get(i) {
                        let total = ranks.len() as f32;
                        if total > 1.0 {
                            // Higher ranked (lower number) gets more boost
                            let rank_factor = 1.0 - (rank as f32 / total);
                            boost += self.config.max_ranking_boost * rank_factor;
                        }
                    }
                }

                let new_strength = (current_strength + boost).min(1.0);
                if (new_strength - current_strength).abs() > 0.001 {
                    let _ = storage.update_strength(concept_id, new_strength);
                    adjustments += 1;
                }

                // Record access for accepted results
                let _ = storage.record_access(concept_id);
            } else {
                // Penalize rejected results
                let new_strength = (current_strength - self.config.reject_penalty).max(0.0);
                if (new_strength - current_strength).abs() > 0.001 {
                    let _ = storage.update_strength(concept_id, new_strength);
                    adjustments += 1;
                }
            }
        }

        adjustments
    }
}
