//! Autonomy Engine
//!
//! Makes Sutra Engine self-directed through 7 features:
//! - Knowledge decay (exponential strength decay + reinforcement)
//! - Self-monitoring (health stats stored as concepts)
//! - Background reasoning (association discovery + contradiction detection)
//! - Goal system (condition/action evaluation)
//! - Subscriptions (push notifications on concept changes)
//! - Gap detection (isolated concepts, near-misses, incomplete chains)
//! - Feedback integration (accept/reject signals adjust strengths)

pub mod decay;
pub mod feedback;
pub mod gap_detector;
pub mod goals;
pub mod reasoning;
pub mod self_monitor;
pub mod subscriptions;

pub use decay::{DecayConfig, DecayLoop};
pub use feedback::{FeedbackConfig, FeedbackProcessor};
pub use gap_detector::{GapDetectorConfig, GapDetectorLoop};
pub use goals::{GoalData, GoalEvaluatorConfig, GoalEvaluatorLoop, GoalSummary};
pub use reasoning::{ReasoningConfig, ReasoningLoop};
pub use self_monitor::{SelfMonitorConfig, SelfMonitorLoop};
pub use subscriptions::{SubscriptionConfig, SubscriptionInfo, SubscriptionManager};

use crate::concurrent_memory::ConcurrentMemory;
use std::sync::Arc;

/// Master configuration for all autonomy features
#[derive(Debug, Clone)]
pub struct AutonomyConfig {
    /// Master switch
    pub enabled: bool,
    /// Knowledge decay configuration
    pub decay: DecayConfig,
    /// Self-monitoring configuration
    pub self_monitor: SelfMonitorConfig,
    /// Background reasoning configuration
    pub reasoning: ReasoningConfig,
    /// Goal evaluator configuration
    pub goals: GoalEvaluatorConfig,
    /// Gap detector configuration
    pub gap_detector: GapDetectorConfig,
    /// Feedback configuration
    pub feedback: FeedbackConfig,
    /// Subscription configuration
    pub subscriptions: SubscriptionConfig,
}

impl Default for AutonomyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            decay: DecayConfig::default(),
            self_monitor: SelfMonitorConfig::default(),
            reasoning: ReasoningConfig::default(),
            goals: GoalEvaluatorConfig::default(),
            gap_detector: GapDetectorConfig::default(),
            feedback: FeedbackConfig::default(),
            subscriptions: SubscriptionConfig::default(),
        }
    }
}

impl AutonomyConfig {
    /// Create a config with all features disabled
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            decay: DecayConfig {
                enabled: false,
                ..Default::default()
            },
            self_monitor: SelfMonitorConfig {
                enabled: false,
                ..Default::default()
            },
            reasoning: ReasoningConfig {
                enabled: false,
                ..Default::default()
            },
            goals: GoalEvaluatorConfig {
                enabled: false,
                ..Default::default()
            },
            gap_detector: GapDetectorConfig {
                enabled: false,
                ..Default::default()
            },
            feedback: FeedbackConfig::default(),
            subscriptions: SubscriptionConfig {
                enabled: false,
                ..Default::default()
            },
        }
    }
}

/// Central manager for all autonomy features.
///
/// Lifecycle: `new()` → `start()` → `stop()` (or Drop)
pub struct AutonomyManager {
    config: AutonomyConfig,
    storage: Arc<ConcurrentMemory>,
    decay_loop: Option<DecayLoop>,
    self_monitor_loop: Option<SelfMonitorLoop>,
    reasoning_loop: Option<ReasoningLoop>,
    goal_evaluator_loop: Option<GoalEvaluatorLoop>,
    gap_detector_loop: Option<GapDetectorLoop>,
    subscription_manager: Arc<SubscriptionManager>,
    feedback_processor: FeedbackProcessor,
}

impl AutonomyManager {
    pub fn new(config: AutonomyConfig, storage: Arc<ConcurrentMemory>) -> Self {
        let subscription_manager = Arc::new(SubscriptionManager::new());
        let feedback_processor = FeedbackProcessor::new(config.feedback.clone());

        Self {
            config,
            storage,
            decay_loop: None,
            self_monitor_loop: None,
            reasoning_loop: None,
            goal_evaluator_loop: None,
            gap_detector_loop: None,
            subscription_manager,
            feedback_processor,
        }
    }

    /// Start all enabled background loops
    pub fn start(&mut self) {
        if !self.config.enabled {
            log::info!("Autonomy engine disabled");
            return;
        }

        log::info!("Starting autonomy engine...");

        if self.config.decay.enabled {
            self.decay_loop = Some(DecayLoop::start(
                self.config.decay.clone(),
                Arc::clone(&self.storage),
            ));
        }

        if self.config.self_monitor.enabled {
            self.self_monitor_loop = Some(SelfMonitorLoop::start(
                self.config.self_monitor.clone(),
                Arc::clone(&self.storage),
            ));
        }

        if self.config.reasoning.enabled {
            self.reasoning_loop = Some(ReasoningLoop::start(
                self.config.reasoning.clone(),
                Arc::clone(&self.storage),
            ));
        }

        if self.config.goals.enabled {
            self.goal_evaluator_loop = Some(GoalEvaluatorLoop::start(
                self.config.goals.clone(),
                Arc::clone(&self.storage),
            ));
        }

        if self.config.subscriptions.enabled {
            // The subscription manager's poll loop needs to be started separately
            let mut sub_mgr = SubscriptionManager::new();
            sub_mgr.start(self.config.subscriptions.clone(), Arc::clone(&self.storage));
            self.subscription_manager = Arc::new(sub_mgr);
        }

        if self.config.gap_detector.enabled {
            self.gap_detector_loop = Some(GapDetectorLoop::start(
                self.config.gap_detector.clone(),
                Arc::clone(&self.storage),
                Some(Arc::clone(&self.subscription_manager)),
            ));
        }

        log::info!("Autonomy engine started");
    }

    /// Stop all background loops
    pub fn stop(&mut self) {
        if let Some(ref mut loop_) = self.decay_loop {
            loop_.stop();
        }
        if let Some(ref mut loop_) = self.self_monitor_loop {
            loop_.stop();
        }
        if let Some(ref mut loop_) = self.reasoning_loop {
            loop_.stop();
        }
        if let Some(ref mut loop_) = self.goal_evaluator_loop {
            loop_.stop();
        }
        if let Some(ref mut loop_) = self.gap_detector_loop {
            loop_.stop();
        }
        // subscription_manager is behind Arc, stop via its own Drop

        self.decay_loop = None;
        self.self_monitor_loop = None;
        self.reasoning_loop = None;
        self.goal_evaluator_loop = None;
        self.gap_detector_loop = None;

        log::info!("Autonomy engine stopped");
    }

    /// Get the subscription manager (for protocol handler)
    pub fn subscription_manager(&self) -> &Arc<SubscriptionManager> {
        &self.subscription_manager
    }

    /// Get the feedback processor (for protocol handler)
    pub fn feedback_processor(&self) -> &FeedbackProcessor {
        &self.feedback_processor
    }

    /// Get the storage reference (for goal/subscription operations)
    pub fn storage(&self) -> &Arc<ConcurrentMemory> {
        &self.storage
    }

    /// Get autonomy stats as a JSON string
    pub fn stats(&self) -> String {
        let snapshot = self.storage.get_snapshot();
        let stats = self.storage.stats();
        let hnsw_stats = self.storage.hnsw_stats();

        let sub_count = self.subscription_manager.list().len();
        let goal_count = goals::list_goals(&self.storage, None).len();

        serde_json::json!({
            "autonomy_enabled": self.config.enabled,
            "decay_enabled": self.config.decay.enabled,
            "self_monitor_enabled": self.config.self_monitor.enabled,
            "reasoning_enabled": self.config.reasoning.enabled,
            "goals_enabled": self.config.goals.enabled,
            "gap_detector_enabled": self.config.gap_detector.enabled,
            "subscriptions_enabled": self.config.subscriptions.enabled,
            "active_subscriptions": sub_count,
            "active_goals": goal_count,
            "concepts": snapshot.concept_count,
            "edges": snapshot.edge_count,
            "vectors": hnsw_stats.indexed_vectors,
            "reconciler_health": stats.reconciler.health_score,
            "pending_writes": stats.write_log.pending,
        })
        .to_string()
    }
}

impl Drop for AutonomyManager {
    fn drop(&mut self) {
        self.stop();
    }
}
