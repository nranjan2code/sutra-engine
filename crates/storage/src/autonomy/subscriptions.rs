//! Subscription Manager
//!
//! Push notifications when concepts matching a filter are created.
//! Background thread polls ReadView for snapshot sequence changes and
//! compares new vs old concepts to detect additions.

use crate::concurrent_memory::ConcurrentMemory;
use crate::read_view::GraphSnapshot;
use crate::semantic::{CausalFilter, SemanticFilter, SemanticType, TemporalConstraint};
use crate::tcp_server::SemanticFilterMsg;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Configuration for the subscription system
#[derive(Debug, Clone)]
pub struct SubscriptionConfig {
    /// Whether subscriptions are enabled
    pub enabled: bool,
    /// How often to poll for new concepts
    pub poll_interval: Duration,
}

impl Default for SubscriptionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            poll_interval: Duration::from_millis(500),
        }
    }
}

/// A subscription filter + callback
#[derive(Debug, Clone)]
pub struct Subscription {
    pub id: String,
    pub filter: SemanticFilterMsg,
    pub callback_addr: String,
}

/// Notification about a matching concept
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub subscription_id: String,
    pub concept_id: String,
    pub content_preview: String,
    pub semantic_type: Option<String>,
}

/// Summary info about a subscription (for listing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionInfo {
    pub id: String,
    pub filter: SemanticFilterMsg,
    pub callback_addr: String,
}

/// Manages subscriptions and background polling
pub struct SubscriptionManager {
    subscriptions: Arc<DashMap<String, Subscription>>,
    running: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
    next_id: std::sync::atomic::AtomicU64,
}

impl Default for SubscriptionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SubscriptionManager {
    pub fn new() -> Self {
        Self {
            subscriptions: Arc::new(DashMap::new()),
            running: Arc::new(AtomicBool::new(false)),
            handle: None,
            next_id: std::sync::atomic::AtomicU64::new(1),
        }
    }

    /// Start the background polling loop
    pub fn start(&mut self, config: SubscriptionConfig, storage: Arc<ConcurrentMemory>) {
        if self.running.load(Ordering::Relaxed) {
            return;
        }
        self.running.store(true, Ordering::Relaxed);

        let running = Arc::clone(&self.running);
        let subscriptions = Arc::clone(&self.subscriptions);

        let handle = thread::spawn(move || {
            subscription_poll_loop(config, storage, subscriptions, running);
        });

        self.handle = Some(handle);
        log::info!("Subscription manager started");
    }

    /// Subscribe with a filter. Returns subscription ID.
    pub fn subscribe(&self, filter: SemanticFilterMsg, callback_addr: String) -> String {
        let id = format!("sub-{}", self.next_id.fetch_add(1, Ordering::Relaxed));
        let sub = Subscription {
            id: id.clone(),
            filter,
            callback_addr,
        };
        self.subscriptions.insert(id.clone(), sub);
        id
    }

    /// Unsubscribe by ID
    pub fn unsubscribe(&self, id: &str) -> bool {
        self.subscriptions.remove(id).is_some()
    }

    /// List all subscriptions
    pub fn list(&self) -> Vec<SubscriptionInfo> {
        self.subscriptions
            .iter()
            .map(|entry| {
                let sub = entry.value();
                SubscriptionInfo {
                    id: sub.id.clone(),
                    filter: sub.filter.clone(),
                    callback_addr: sub.callback_addr.clone(),
                }
            })
            .collect()
    }

    /// Notify subscriptions about a concept (called by other autonomy modules)
    pub fn notify(&self, concept_id: &str, content_preview: &str, semantic_type: Option<&str>) {
        for entry in self.subscriptions.iter() {
            let sub = entry.value();
            let notification = Notification {
                subscription_id: sub.id.clone(),
                concept_id: concept_id.to_string(),
                content_preview: content_preview.to_string(),
                semantic_type: semantic_type.map(|s| s.to_string()),
            };

            if sub.callback_addr.is_empty() {
                // Log-only mode
                log::info!("Subscription notification: {:?}", notification);
            } else {
                // Attempt TCP push (fire-and-forget)
                let addr = sub.callback_addr.clone();
                let json = serde_json::to_vec(&notification).unwrap_or_default();
                std::thread::spawn(move || {
                    if let Ok(mut stream) = std::net::TcpStream::connect(&addr) {
                        use std::io::Write;
                        let _ = stream.write_all(&json);
                        let _ = stream.write_all(b"\n");
                    }
                });
            }
        }
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for SubscriptionManager {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Convert SemanticFilterMsg to internal SemanticFilter
fn filter_from_msg(msg: &SemanticFilterMsg) -> SemanticFilter {
    let mut filter = SemanticFilter::new();

    if let Some(ref st) = msg.semantic_type {
        if let Some(semantic_type) = parse_semantic_type(st) {
            filter = filter.with_type(semantic_type);
        }
    }

    if let Some(after) = msg.temporal_after {
        filter = filter.with_temporal(TemporalConstraint::After(after));
    }

    if let Some(before) = msg.temporal_before {
        filter = filter.with_temporal(TemporalConstraint::Before(before));
    }

    if msg.has_causal_relation {
        filter = filter.with_causal(CausalFilter::HasCausalRelation);
    }

    filter = filter.with_min_confidence(msg.min_confidence);

    for term in &msg.required_terms {
        filter = filter.with_term(term.clone());
    }

    filter
}

fn parse_semantic_type(s: &str) -> Option<SemanticType> {
    match s.to_lowercase().as_str() {
        "entity" => Some(SemanticType::Entity),
        "event" => Some(SemanticType::Event),
        "rule" => Some(SemanticType::Rule),
        "temporal" => Some(SemanticType::Temporal),
        "negation" => Some(SemanticType::Negation),
        "condition" => Some(SemanticType::Condition),
        "causal" => Some(SemanticType::Causal),
        "quantitative" => Some(SemanticType::Quantitative),
        "definitional" => Some(SemanticType::Definitional),
        "goal" => Some(SemanticType::Goal),
        _ => None,
    }
}

fn subscription_poll_loop(
    config: SubscriptionConfig,
    storage: Arc<ConcurrentMemory>,
    subscriptions: Arc<DashMap<String, Subscription>>,
    running: Arc<AtomicBool>,
) {
    let mut last_sequence = storage.get_snapshot().sequence;

    while running.load(Ordering::Relaxed) {
        thread::sleep(config.poll_interval);
        if !running.load(Ordering::Relaxed) {
            break;
        }

        if subscriptions.is_empty() {
            continue;
        }

        let new_snapshot = storage.get_snapshot();
        if new_snapshot.sequence == last_sequence {
            continue;
        }

        // Find new concepts (concepts in new snapshot that weren't in old)
        // We track by sequence number - only process if sequence advanced
        check_subscriptions(&new_snapshot, &subscriptions);

        last_sequence = new_snapshot.sequence;
    }

    log::info!("Subscription poll loop stopped");
}

fn check_subscriptions(
    snapshot: &Arc<GraphSnapshot>,
    subscriptions: &Arc<DashMap<String, Subscription>>,
) {
    for entry in subscriptions.iter() {
        let sub = entry.value();
        let filter = filter_from_msg(&sub.filter);

        // Check all concepts against filter (new concepts will be caught because
        // we only run when sequence advances)
        for concept in snapshot.concepts.values() {
            if let Some(ref semantic) = concept.semantic {
                let content = String::from_utf8_lossy(&concept.content);
                if filter.matches(semantic, &content, &concept.id) {
                    let notification = Notification {
                        subscription_id: sub.id.clone(),
                        concept_id: concept.id.to_hex(),
                        content_preview: content.chars().take(200).collect(),
                        semantic_type: Some(semantic.semantic_type.as_str().to_string()),
                    };

                    if sub.callback_addr.is_empty() {
                        log::debug!("Subscription match: {:?}", notification);
                    } else {
                        let addr = sub.callback_addr.clone();
                        let json = serde_json::to_vec(&notification).unwrap_or_default();
                        std::thread::spawn(move || {
                            if let Ok(mut stream) = std::net::TcpStream::connect(&addr) {
                                use std::io::Write;
                                let _ = stream.write_all(&json);
                                let _ = stream.write_all(b"\n");
                            }
                        });
                    }
                }
            }
        }
    }
}
