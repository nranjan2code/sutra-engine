//! Goal System
//!
//! Goals are stored as regular concepts with `SemanticType::Goal`. Goal data
//! is serialized as JSON in `attributes["sutra:goal_data"]`.
//! A background evaluator loop checks goal conditions and triggers actions.

use crate::concurrent_memory::ConcurrentMemory;
use crate::semantic::{DomainContext, SemanticMetadata, SemanticType};
use crate::types::ConceptId;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Configuration for the goal evaluator loop
#[derive(Debug, Clone)]
pub struct GoalEvaluatorConfig {
    /// Whether goal evaluation is enabled
    pub enabled: bool,
    /// Interval between evaluation cycles
    pub interval: Duration,
}

impl Default for GoalEvaluatorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: Duration::from_secs(5),
        }
    }
}

/// Condition that triggers a goal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GoalCondition {
    /// A concept matching the description exists
    ConceptExists { content_contains: String },
    /// Total concept count exceeds threshold
    ConceptCountAbove(usize),
    /// A specific concept's strength drops below threshold
    StrengthBelow { concept_id: String, threshold: f32 },
    /// Total association count exceeds threshold
    AssociationCountAbove(usize),
}

/// Action to take when a goal condition is met
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GoalAction {
    /// Log a notification message
    Notify { message: String },
    /// Learn a new concept
    LearnConcept { content: String },
    /// Create an association between two concepts
    CreateAssociation {
        source_id: String,
        target_id: String,
    },
    /// Custom action identifier (for future extension)
    Custom(String),
}

/// Current status of a goal
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GoalStatus {
    Active,
    Triggered,
    Completed,
    Suspended,
}

/// Complete goal data stored as JSON in concept attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalData {
    pub description: String,
    pub condition: GoalCondition,
    pub action: GoalAction,
    pub priority: u8,
    pub status: GoalStatus,
    pub created_at: i64,
    pub triggered_at: Option<i64>,
}

/// Summary info about a goal (for listing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalSummary {
    pub goal_id: String,
    pub description: String,
    pub status: String,
    pub priority: u8,
}

/// Background goal evaluator loop handle
pub struct GoalEvaluatorLoop {
    running: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl GoalEvaluatorLoop {
    pub fn start(config: GoalEvaluatorConfig, storage: Arc<ConcurrentMemory>) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = Arc::clone(&running);

        let handle = thread::spawn(move || {
            goal_evaluator_loop(config, storage, running_clone);
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

impl Drop for GoalEvaluatorLoop {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Create a new goal and store it as a concept
pub fn create_goal(
    storage: &Arc<ConcurrentMemory>,
    namespace: Option<&str>,
    description: &str,
    condition_str: &str,
    action_str: &str,
    priority: u8,
) -> Result<String, String> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    // Parse condition from string
    let condition = parse_condition(condition_str);
    let action = parse_action(action_str);

    let goal_data = GoalData {
        description: description.to_string(),
        condition,
        action,
        priority,
        status: GoalStatus::Active,
        created_at: now,
        triggered_at: None,
    };

    let goal_json = serde_json::to_string(&goal_data)
        .map_err(|e| format!("Failed to serialize goal: {}", e))?;

    let content = format!("Goal: {}", description);
    let id_source = format!(
        "sutra:goal:{}:{}",
        namespace.unwrap_or("default"),
        description
    );
    let concept_id = ConceptId::from_string(&id_source);

    let mut semantic = SemanticMetadata::new(SemanticType::Goal);
    semantic.domain_context = DomainContext::Technical;

    // We need to store goal_data in attributes, but learn_concept_with_semantic
    // doesn't take attributes. So we learn the concept first, then the data
    // is stored in the content itself.
    // Instead, we use learn_concept with attributes.
    let mut attributes = std::collections::HashMap::new();
    attributes.insert("sutra:goal_data".to_string(), goal_json);
    attributes.insert("sutra:source".to_string(), "goal_system".to_string());
    if let Some(ns) = namespace {
        attributes.insert("sutra:namespace".to_string(), ns.to_string());
    }

    storage
        .learn_concept(concept_id, content.into_bytes(), None, 1.0, 1.0, attributes)
        .map_err(|e| format!("Failed to store goal: {:?}", e))?;

    Ok(concept_id.to_hex())
}

/// List all goals, optionally filtered by namespace
pub fn list_goals(storage: &Arc<ConcurrentMemory>, namespace: Option<&str>) -> Vec<GoalSummary> {
    let snapshot = storage.get_snapshot();
    let mut goals = Vec::new();

    for concept in snapshot.concepts.values() {
        if let Some(ref semantic) = concept.semantic {
            if semantic.semantic_type == SemanticType::Goal {
                // Check namespace filter
                if let Some(ns) = namespace {
                    if let Some(stored_ns) = concept.attributes.get("sutra:namespace") {
                        if stored_ns != ns {
                            continue;
                        }
                    }
                }

                let goal_data: Option<GoalData> = concept
                    .attributes
                    .get("sutra:goal_data")
                    .and_then(|json| serde_json::from_str(json).ok());

                if let Some(data) = goal_data {
                    goals.push(GoalSummary {
                        goal_id: concept.id.to_hex(),
                        description: data.description,
                        status: format!("{:?}", data.status),
                        priority: data.priority,
                    });
                } else {
                    // Goal without proper data, still list it
                    goals.push(GoalSummary {
                        goal_id: concept.id.to_hex(),
                        description: String::from_utf8_lossy(&concept.content).to_string(),
                        status: "Unknown".to_string(),
                        priority: 0,
                    });
                }
            }
        }
    }

    // Sort by priority descending
    goals.sort_by(|a, b| b.priority.cmp(&a.priority));
    goals
}

/// Cancel (delete) a goal by ID
pub fn cancel_goal(storage: &Arc<ConcurrentMemory>, goal_id: &str) -> Result<(), String> {
    let concept_id = ConceptId::from_string(goal_id);
    storage
        .delete_concept(concept_id)
        .map(|_| ())
        .map_err(|e| format!("Failed to cancel goal: {:?}", e))
}

fn parse_condition(s: &str) -> GoalCondition {
    let lower = s.to_lowercase();
    if lower.starts_with("count above") || lower.starts_with("concepts above") {
        let num = lower
            .split_whitespace()
            .last()
            .and_then(|n| n.parse::<usize>().ok())
            .unwrap_or(100);
        GoalCondition::ConceptCountAbove(num)
    } else if lower.starts_with("associations above") {
        let num = lower
            .split_whitespace()
            .last()
            .and_then(|n| n.parse::<usize>().ok())
            .unwrap_or(50);
        GoalCondition::AssociationCountAbove(num)
    } else {
        // Default: treat as content match
        GoalCondition::ConceptExists {
            content_contains: s.to_string(),
        }
    }
}

fn parse_action(s: &str) -> GoalAction {
    let lower = s.to_lowercase();
    if lower.starts_with("learn:") || lower.starts_with("learn ") {
        GoalAction::LearnConcept {
            content: s[6..].trim().to_string(),
        }
    } else if lower.starts_with("notify:") || lower.starts_with("notify ") {
        GoalAction::Notify {
            message: s[7..].trim().to_string(),
        }
    } else {
        GoalAction::Notify {
            message: format!("Goal triggered: {}", s),
        }
    }
}

fn goal_evaluator_loop(
    config: GoalEvaluatorConfig,
    storage: Arc<ConcurrentMemory>,
    running: Arc<AtomicBool>,
) {
    log::info!(
        "Goal evaluator loop started (interval={:?})",
        config.interval
    );

    while running.load(Ordering::Relaxed) {
        thread::sleep(config.interval);
        if !running.load(Ordering::Relaxed) {
            break;
        }

        let snapshot = storage.get_snapshot();

        // Find all active goals
        for concept in snapshot.concepts.values() {
            if !running.load(Ordering::Relaxed) {
                break;
            }

            if let Some(ref semantic) = concept.semantic {
                if semantic.semantic_type != SemanticType::Goal {
                    continue;
                }
            } else {
                continue;
            }

            let goal_data: Option<GoalData> = concept
                .attributes
                .get("sutra:goal_data")
                .and_then(|json| serde_json::from_str(json).ok());

            let data = match goal_data {
                Some(d) if d.status == GoalStatus::Active => d,
                _ => continue,
            };

            // Evaluate condition
            let condition_met = evaluate_condition(&data.condition, &snapshot);

            if condition_met {
                log::info!(
                    "Goal triggered: {} (id={})",
                    data.description,
                    concept.id.to_hex()
                );

                // Execute action
                execute_action(&data.action, &storage);

                // Update goal status to Triggered
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64;

                let mut updated_data = data.clone();
                updated_data.status = GoalStatus::Triggered;
                updated_data.triggered_at = Some(now);

                if let Ok(json) = serde_json::to_string(&updated_data) {
                    let mut new_attrs = concept.attributes.clone();
                    new_attrs.insert("sutra:goal_data".to_string(), json);

                    // Re-store the concept with updated attributes
                    let _ = storage.learn_concept(
                        concept.id,
                        concept.content.to_vec(),
                        None,
                        concept.strength,
                        concept.confidence,
                        new_attrs,
                    );
                }
            }
        }
    }

    log::info!("Goal evaluator loop stopped");
}

fn evaluate_condition(
    condition: &GoalCondition,
    snapshot: &Arc<crate::read_view::GraphSnapshot>,
) -> bool {
    match condition {
        GoalCondition::ConceptExists { content_contains } => {
            let lower_search = content_contains.to_lowercase();
            snapshot.concepts.values().any(|c| {
                String::from_utf8_lossy(&c.content)
                    .to_lowercase()
                    .contains(&lower_search)
            })
        }
        GoalCondition::ConceptCountAbove(threshold) => snapshot.concept_count > *threshold,
        GoalCondition::StrengthBelow {
            concept_id,
            threshold,
        } => {
            let id = ConceptId::from_string(concept_id);
            snapshot
                .get_concept(&id)
                .is_some_and(|c| c.strength < *threshold)
        }
        GoalCondition::AssociationCountAbove(threshold) => snapshot.edge_count > *threshold,
    }
}

fn execute_action(action: &GoalAction, storage: &Arc<ConcurrentMemory>) {
    match action {
        GoalAction::Notify { message } => {
            log::info!("Goal notification: {}", message);
        }
        GoalAction::LearnConcept { content } => {
            let id = ConceptId::from_string(&format!("goal:learn:{}", content));
            let _ = storage.learn_concept(
                id,
                content.clone().into_bytes(),
                None,
                1.0,
                1.0,
                std::collections::HashMap::new(),
            );
        }
        GoalAction::CreateAssociation {
            source_id,
            target_id,
        } => {
            let source = ConceptId::from_string(source_id);
            let target = ConceptId::from_string(target_id);
            let _ = storage.learn_association(
                source,
                target,
                crate::types::AssociationType::Semantic,
                0.8,
            );
        }
        GoalAction::Custom(action_str) => {
            log::info!("Custom goal action: {}", action_str);
        }
    }
}
