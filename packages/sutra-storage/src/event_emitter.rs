/// Event Emitter - Self-monitoring using Sutra's own event system
/// 
/// This module demonstrates "eating our own dogfood" by using the GridEvent
/// system to monitor storage performance metrics in real-time.

use crate::types::ConceptId;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::{debug, warn};

/// Storage event emitter that sends metrics to Grid event storage
pub struct StorageEventEmitter {
    node_id: String,
    event_storage_addr: Option<String>,
    sender: Option<mpsc::UnboundedSender<StorageEvent>>,
    worker_handle: Option<JoinHandle<()>>,
}

/// Internal event representation before serialization
#[derive(Debug, Clone)]
pub enum StorageEvent {
    Metrics {
        concept_count: usize,
        edge_count: usize,
        write_throughput: u64,
        read_latency_us: u64,
        memory_usage_mb: u64,
    },
    QueryPerformance {
        query_type: String,
        query_depth: u32,
        result_count: usize,
        latency_ms: u64,
        confidence: f32,
    },
    HnswIndexBuilt {
        vector_count: usize,
        build_time_ms: u64,
        dimension: usize,
    },
    HnswIndexLoaded {
        vector_count: usize,
        load_time_ms: u64,
        persisted: bool,
    },
    PathfindingMetrics {
        source_id: String,
        target_id: String,
        path_length: u32,
        paths_explored: u64,
        latency_ms: u64,
        strategy: String,
    },
    ReconciliationComplete {
        entries_processed: u64,
        reconciliation_time_ms: u64,
        disk_flush: bool,
    },
}

impl StorageEventEmitter {
    /// Create new emitter with optional event storage integration
    pub fn new(node_id: String, event_storage_addr: Option<String>) -> Self {
        let (sender, receiver) = if event_storage_addr.is_some() {
            let (tx, rx) = mpsc::unbounded_channel();
            (Some(tx), Some(rx))
        } else {
            (None, None)
        };

        let worker_handle = if let (Some(addr), Some(rx)) = (event_storage_addr.as_ref(), receiver) {
            Some(tokio::spawn(Self::worker_loop(
                node_id.clone(),
                addr.clone(),
                rx,
            )))
        } else {
            None
        };

        Self {
            node_id,
            event_storage_addr,
            sender,
            worker_handle,
        }
    }

    /// Emit storage metrics event
    pub fn emit_metrics(
        &self,
        concept_count: usize,
        edge_count: usize,
        write_throughput: u64,
        read_latency_us: u64,
        memory_usage_mb: u64,
    ) {
        if let Some(sender) = &self.sender {
            let event = StorageEvent::Metrics {
                concept_count,
                edge_count,
                write_throughput,
                read_latency_us,
                memory_usage_mb,
            };
            if sender.send(event).is_err() {
                warn!("Failed to emit storage metrics event");
            }
        }
    }

    /// Emit query performance event
    pub fn emit_query_performance(
        &self,
        query_type: impl Into<String>,
        query_depth: u32,
        result_count: usize,
        latency_ms: u64,
        confidence: f32,
    ) {
        if let Some(sender) = &self.sender {
            let event = StorageEvent::QueryPerformance {
                query_type: query_type.into(),
                query_depth,
                result_count,
                latency_ms,
                confidence,
            };
            if sender.send(event).is_err() {
                warn!("Failed to emit query performance event");
            }
        }
    }

    /// Emit HNSW index built event
    pub fn emit_hnsw_built(&self, vector_count: usize, build_time_ms: u64, dimension: usize) {
        if let Some(sender) = &self.sender {
            let event = StorageEvent::HnswIndexBuilt {
                vector_count,
                build_time_ms,
                dimension,
            };
            if sender.send(event).is_err() {
                warn!("Failed to emit HNSW built event");
            }
        }
    }

    /// Emit HNSW index loaded event
    pub fn emit_hnsw_loaded(&self, vector_count: usize, load_time_ms: u64, persisted: bool) {
        if let Some(sender) = &self.sender {
            let event = StorageEvent::HnswIndexLoaded {
                vector_count,
                load_time_ms,
                persisted,
            };
            if sender.send(event).is_err() {
                warn!("Failed to emit HNSW loaded event");
            }
        }
    }

    /// Emit pathfinding metrics event
    pub fn emit_pathfinding(
        &self,
        source_id: ConceptId,
        target_id: ConceptId,
        path_length: u32,
        paths_explored: u64,
        latency_ms: u64,
        strategy: impl Into<String>,
    ) {
        if let Some(sender) = &self.sender {
            let event = StorageEvent::PathfindingMetrics {
                source_id: source_id.to_hex(),
                target_id: target_id.to_hex(),
                path_length,
                paths_explored,
                latency_ms,
                strategy: strategy.into(),
            };
            if sender.send(event).is_err() {
                warn!("Failed to emit pathfinding metrics event");
            }
        }
    }

    /// Emit reconciliation complete event
    pub fn emit_reconciliation(&self, entries_processed: u64, reconciliation_time_ms: u64, disk_flush: bool) {
        if let Some(sender) = &self.sender {
            let event = StorageEvent::ReconciliationComplete {
                entries_processed,
                reconciliation_time_ms,
                disk_flush,
            };
            if sender.send(event).is_err() {
                warn!("Failed to emit reconciliation event");
            }
        }
    }

    /// Background worker that sends events to Grid event storage
    async fn worker_loop(
        node_id: String,
        event_storage_addr: String,
        mut receiver: mpsc::UnboundedReceiver<StorageEvent>,
    ) {
        debug!(
            "Event emitter worker started for node {} targeting {}",
            node_id, event_storage_addr
        );

        while let Some(event) = receiver.recv().await {
            // Convert to GridEvent JSON and send via TCP to event storage
            // For now, just log (will integrate with sutra-protocol for actual TCP)
            match event {
                StorageEvent::Metrics {
                    concept_count,
                    edge_count,
                    write_throughput,
                    read_latency_us,
                    memory_usage_mb,
                } => {
                    debug!(
                        "📊 Storage Metrics: {} concepts, {} edges, {}K writes/sec, {} μs reads, {} MB memory",
                        concept_count,
                        edge_count,
                        write_throughput / 1000,
                        read_latency_us,
                        memory_usage_mb
                    );
                }
                StorageEvent::QueryPerformance {
                    query_type,
                    query_depth,
                    result_count,
                    latency_ms,
                    confidence,
                } => {
                    debug!(
                        "🔍 Query Performance: type={}, depth={}, {} results, {}ms, conf={:.2}",
                        query_type, query_depth, result_count, latency_ms, confidence
                    );
                }
                StorageEvent::HnswIndexBuilt {
                    vector_count,
                    build_time_ms,
                    dimension,
                } => {
                    debug!(
                        "🏗️  HNSW Built: {} vectors ({}d) in {}ms",
                        vector_count, dimension, build_time_ms
                    );
                }
                StorageEvent::HnswIndexLoaded {
                    vector_count,
                    load_time_ms,
                    persisted,
                } => {
                    let source = if persisted { "disk" } else { "rebuild" };
                    debug!(
                        "📥 HNSW Loaded: {} vectors from {} in {}ms",
                        vector_count, source, load_time_ms
                    );
                }
                StorageEvent::PathfindingMetrics {
                    source_id,
                    target_id,
                    path_length,
                    paths_explored,
                    latency_ms,
                    strategy,
                } => {
                    debug!(
                        "🔎 Pathfinding: {} → {} via {}, len={}, explored={}, {}ms",
                        source_id, target_id, strategy, path_length, paths_explored, latency_ms
                    );
                }
                StorageEvent::ReconciliationComplete {
                    entries_processed,
                    reconciliation_time_ms,
                    disk_flush,
                } => {
                    let flush_status = if disk_flush { "with flush" } else { "memory only" };
                    debug!(
                        "♻️  Reconciliation: {} entries in {}ms ({})",
                        entries_processed, reconciliation_time_ms, flush_status
                    );
                }
            }

            // TODO: Send to actual event storage via sutra-protocol TCP
            // For production, this will use the same TCP binary protocol
            // as the storage server communication
        }

        debug!("Event emitter worker stopped for node {}", node_id);
    }

    /// Shutdown the emitter gracefully
    pub async fn shutdown(self) {
        // Drop sender to signal worker to stop
        drop(self.sender);

        if let Some(handle) = self.worker_handle {
            let _ = tokio::time::timeout(Duration::from_secs(5), handle).await;
        }
    }
}

impl Default for StorageEventEmitter {
    fn default() -> Self {
        Self::new("unknown".to_string(), None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_emit_metrics() {
        let emitter = StorageEventEmitter::new("test-node".to_string(), None);
        
        // Should not crash when no event storage configured
        emitter.emit_metrics(1000, 5000, 50000, 10, 512);
        
        emitter.shutdown().await;
    }

    #[tokio::test]
    async fn test_emit_query_performance() {
        let emitter = StorageEventEmitter::new("test-node".to_string(), None);
        
        emitter.emit_query_performance("semantic", 3, 10, 25, 0.85);
        
        emitter.shutdown().await;
    }
}
