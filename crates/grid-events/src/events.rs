use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Core Grid event types - structured, queryable alternatives to logs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type", rename_all = "snake_case")]
pub enum GridEvent {
    // Agent Lifecycle
    AgentRegistered {
        agent_id: String,
        hostname: String,
        platform: String,
        agent_endpoint: String,
        max_storage_nodes: u32,
        timestamp: DateTime<Utc>,
    },
    
    AgentHeartbeat {
        agent_id: String,
        storage_node_count: u32,
        timestamp: DateTime<Utc>,
    },
    
    AgentDegraded {
        agent_id: String,
        seconds_since_heartbeat: u64,
        timestamp: DateTime<Utc>,
    },
    
    AgentOffline {
        agent_id: String,
        last_seen: DateTime<Utc>,
        timestamp: DateTime<Utc>,
    },
    
    AgentRecovered {
        agent_id: String,
        downtime_seconds: u64,
        timestamp: DateTime<Utc>,
    },
    
    AgentUnregistered {
        agent_id: String,
        timestamp: DateTime<Utc>,
    },
    
    // Storage Node Lifecycle
    SpawnRequested {
        node_id: String,
        agent_id: String,
        port: u32,
        storage_path: String,
        memory_limit_mb: u64,
        timestamp: DateTime<Utc>,
    },
    
    SpawnSucceeded {
        node_id: String,
        agent_id: String,
        pid: u32,
        port: u32,
        timestamp: DateTime<Utc>,
    },
    
    SpawnFailed {
        node_id: String,
        agent_id: String,
        error: String,
        timestamp: DateTime<Utc>,
    },
    
    StopRequested {
        node_id: String,
        agent_id: String,
        timestamp: DateTime<Utc>,
    },
    
    StopSucceeded {
        node_id: String,
        agent_id: String,
        timestamp: DateTime<Utc>,
    },
    
    StopFailed {
        node_id: String,
        agent_id: String,
        error: String,
        timestamp: DateTime<Utc>,
    },
    
    NodeCrashed {
        node_id: String,
        agent_id: String,
        exit_code: Option<i32>,
        timestamp: DateTime<Utc>,
    },
    
    NodeRestarted {
        node_id: String,
        agent_id: String,
        restart_count: u32,
        new_pid: u32,
        timestamp: DateTime<Utc>,
    },
    
    // Cluster Status
    ClusterHealthy {
        total_agents: u32,
        healthy_agents: u32,
        total_nodes: u32,
        running_nodes: u32,
        timestamp: DateTime<Utc>,
    },
    
    ClusterDegraded {
        total_agents: u32,
        healthy_agents: u32,
        reason: String,
        timestamp: DateTime<Utc>,
    },
    
    ClusterCritical {
        total_agents: u32,
        healthy_agents: u32,
        reason: String,
        timestamp: DateTime<Utc>,
    },
    
    // Storage Performance Metrics
    StorageMetrics {
        node_id: String,
        concept_count: usize,
        edge_count: usize,
        write_throughput: u64,  // concepts/sec
        read_latency_us: u64,   // microseconds
        memory_usage_mb: u64,
        timestamp: DateTime<Utc>,
    },
    
    // Query Performance Metrics
    QueryPerformance {
        node_id: String,
        query_type: String,  // "semantic", "graph", "hybrid"
        query_depth: u32,
        result_count: usize,
        latency_ms: u64,
        confidence: f32,
        timestamp: DateTime<Utc>,
    },
    
    // Embedding Service Metrics
    EmbeddingLatency {
        service_instance: String,
        batch_size: usize,
        latency_ms: u64,
        dimension: usize,
        cache_hit: bool,
        timestamp: DateTime<Utc>,
    },
    
    // HNSW Index Operations
    HnswIndexBuilt {
        node_id: String,
        vector_count: usize,
        build_time_ms: u64,
        dimension: usize,
        timestamp: DateTime<Utc>,
    },
    
    HnswIndexLoaded {
        node_id: String,
        vector_count: usize,
        load_time_ms: u64,
        persisted: bool,
        timestamp: DateTime<Utc>,
    },
    
    // Pathfinding Performance
    PathfindingMetrics {
        node_id: String,
        source_id: String,
        target_id: String,
        path_length: u32,
        paths_explored: u64,
        latency_ms: u64,
        strategy: String,  // "best_first", "breadth_first", "bidirectional"
        timestamp: DateTime<Utc>,
    },
    
    // Reconciliation Metrics
    ReconciliationComplete {
        node_id: String,
        entries_processed: u64,
        reconciliation_time_ms: u64,
        disk_flush: bool,
        timestamp: DateTime<Utc>,
    },
    
    // Embedding Service Events
    EmbeddingServiceHealthy {
        instance_id: String,
        endpoint: String,
        success_rate: f32,
        cache_hit_rate: f32,
        timestamp: DateTime<Utc>,
    },
    
    EmbeddingServiceDegraded {
        instance_id: String,
        endpoint: String,
        error_rate: f32,
        reason: String,
        timestamp: DateTime<Utc>,
    },
}

impl GridEvent {
    /// Get the event timestamp
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            GridEvent::AgentRegistered { timestamp, .. } => *timestamp,
            GridEvent::AgentHeartbeat { timestamp, .. } => *timestamp,
            GridEvent::AgentDegraded { timestamp, .. } => *timestamp,
            GridEvent::AgentOffline { timestamp, .. } => *timestamp,
            GridEvent::AgentRecovered { timestamp, .. } => *timestamp,
            GridEvent::AgentUnregistered { timestamp, .. } => *timestamp,
            GridEvent::SpawnRequested { timestamp, .. } => *timestamp,
            GridEvent::SpawnSucceeded { timestamp, .. } => *timestamp,
            GridEvent::SpawnFailed { timestamp, .. } => *timestamp,
            GridEvent::StopRequested { timestamp, .. } => *timestamp,
            GridEvent::StopSucceeded { timestamp, .. } => *timestamp,
            GridEvent::StopFailed { timestamp, .. } => *timestamp,
            GridEvent::NodeCrashed { timestamp, .. } => *timestamp,
            GridEvent::NodeRestarted { timestamp, .. } => *timestamp,
            GridEvent::ClusterHealthy { timestamp, .. } => *timestamp,
            GridEvent::ClusterDegraded { timestamp, .. } => *timestamp,
            GridEvent::ClusterCritical { timestamp, .. } => *timestamp,
            GridEvent::StorageMetrics { timestamp, .. } => *timestamp,
            GridEvent::QueryPerformance { timestamp, .. } => *timestamp,
            GridEvent::EmbeddingLatency { timestamp, .. } => *timestamp,
            GridEvent::HnswIndexBuilt { timestamp, .. } => *timestamp,
            GridEvent::HnswIndexLoaded { timestamp, .. } => *timestamp,
            GridEvent::PathfindingMetrics { timestamp, .. } => *timestamp,
            GridEvent::ReconciliationComplete { timestamp, .. } => *timestamp,
            GridEvent::EmbeddingServiceHealthy { timestamp, .. } => *timestamp,
            GridEvent::EmbeddingServiceDegraded { timestamp, .. } => *timestamp,
        }
    }
    
    /// Get event type as string (for graph concepts)
    pub fn event_type(&self) -> &'static str {
        match self {
            GridEvent::AgentRegistered { .. } => "agent_registered",
            GridEvent::AgentHeartbeat { .. } => "agent_heartbeat",
            GridEvent::AgentDegraded { .. } => "agent_degraded",
            GridEvent::AgentOffline { .. } => "agent_offline",
            GridEvent::AgentRecovered { .. } => "agent_recovered",
            GridEvent::AgentUnregistered { .. } => "agent_unregistered",
            GridEvent::SpawnRequested { .. } => "spawn_requested",
            GridEvent::SpawnSucceeded { .. } => "spawn_succeeded",
            GridEvent::SpawnFailed { .. } => "spawn_failed",
            GridEvent::StopRequested { .. } => "stop_requested",
            GridEvent::StopSucceeded { .. } => "stop_succeeded",
            GridEvent::StopFailed { .. } => "stop_failed",
            GridEvent::NodeCrashed { .. } => "node_crashed",
            GridEvent::NodeRestarted { .. } => "node_restarted",
            GridEvent::ClusterHealthy { .. } => "cluster_healthy",
            GridEvent::ClusterDegraded { .. } => "cluster_degraded",
            GridEvent::ClusterCritical { .. } => "cluster_critical",
            GridEvent::StorageMetrics { .. } => "storage_metrics",
            GridEvent::QueryPerformance { .. } => "query_performance",
            GridEvent::EmbeddingLatency { .. } => "embedding_latency",
            GridEvent::HnswIndexBuilt { .. } => "hnsw_index_built",
            GridEvent::HnswIndexLoaded { .. } => "hnsw_index_loaded",
            GridEvent::PathfindingMetrics { .. } => "pathfinding_metrics",
            GridEvent::ReconciliationComplete { .. } => "reconciliation_complete",
            GridEvent::EmbeddingServiceHealthy { .. } => "embedding_service_healthy",
            GridEvent::EmbeddingServiceDegraded { .. } => "embedding_service_degraded",
        }
    }
    
    /// Get primary entity ID (agent_id or node_id)
    pub fn primary_id(&self) -> String {
        match self {
            GridEvent::AgentRegistered { agent_id, .. } => agent_id.clone(),
            GridEvent::AgentHeartbeat { agent_id, .. } => agent_id.clone(),
            GridEvent::AgentDegraded { agent_id, .. } => agent_id.clone(),
            GridEvent::AgentOffline { agent_id, .. } => agent_id.clone(),
            GridEvent::AgentRecovered { agent_id, .. } => agent_id.clone(),
            GridEvent::AgentUnregistered { agent_id, .. } => agent_id.clone(),
            GridEvent::SpawnRequested { node_id, .. } => node_id.clone(),
            GridEvent::SpawnSucceeded { node_id, .. } => node_id.clone(),
            GridEvent::SpawnFailed { node_id, .. } => node_id.clone(),
            GridEvent::StopRequested { node_id, .. } => node_id.clone(),
            GridEvent::StopSucceeded { node_id, .. } => node_id.clone(),
            GridEvent::StopFailed { node_id, .. } => node_id.clone(),
            GridEvent::NodeCrashed { node_id, .. } => node_id.clone(),
            GridEvent::NodeRestarted { node_id, .. } => node_id.clone(),
            GridEvent::ClusterHealthy { .. } => "cluster".to_string(),
            GridEvent::ClusterDegraded { .. } => "cluster".to_string(),
            GridEvent::ClusterCritical { .. } => "cluster".to_string(),
            GridEvent::StorageMetrics { node_id, .. } => node_id.clone(),
            GridEvent::QueryPerformance { node_id, .. } => node_id.clone(),
            GridEvent::EmbeddingLatency { service_instance, .. } => service_instance.clone(),
            GridEvent::HnswIndexBuilt { node_id, .. } => node_id.clone(),
            GridEvent::HnswIndexLoaded { node_id, .. } => node_id.clone(),
            GridEvent::PathfindingMetrics { node_id, .. } => node_id.clone(),
            GridEvent::ReconciliationComplete { node_id, .. } => node_id.clone(),
            GridEvent::EmbeddingServiceHealthy { instance_id, .. } => instance_id.clone(),
            GridEvent::EmbeddingServiceDegraded { instance_id, .. } => instance_id.clone(),
        }
    }
    
    /// Convert event to human-readable description for storage
    pub fn to_description(&self) -> String {
        match self {
            GridEvent::AgentRegistered { agent_id, hostname, platform, .. } => {
                format!("Agent {} registered from {} ({})", agent_id, hostname, platform)
            }
            GridEvent::AgentHeartbeat { agent_id, storage_node_count, .. } => {
                format!("Agent {} heartbeat with {} nodes", agent_id, storage_node_count)
            }
            GridEvent::AgentDegraded { agent_id, seconds_since_heartbeat, .. } => {
                format!("Agent {} degraded ({}s since heartbeat)", agent_id, seconds_since_heartbeat)
            }
            GridEvent::AgentOffline { agent_id, .. } => {
                format!("Agent {} went offline", agent_id)
            }
            GridEvent::AgentRecovered { agent_id, downtime_seconds, .. } => {
                format!("Agent {} recovered after {}s downtime", agent_id, downtime_seconds)
            }
            GridEvent::AgentUnregistered { agent_id, .. } => {
                format!("Agent {} unregistered", agent_id)
            }
            GridEvent::SpawnRequested { node_id, agent_id, port, .. } => {
                format!("Spawn requested for {} on {} (port {})", node_id, agent_id, port)
            }
            GridEvent::SpawnSucceeded { node_id, agent_id, pid, port, .. } => {
                format!("Node {} spawned on {} (PID: {}, port: {})", node_id, agent_id, pid, port)
            }
            GridEvent::SpawnFailed { node_id, agent_id, error, .. } => {
                format!("Failed to spawn {} on {}: {}", node_id, agent_id, error)
            }
            GridEvent::StopRequested { node_id, agent_id, .. } => {
                format!("Stop requested for {} on {}", node_id, agent_id)
            }
            GridEvent::StopSucceeded { node_id, agent_id, .. } => {
                format!("Node {} stopped on {}", node_id, agent_id)
            }
            GridEvent::StopFailed { node_id, agent_id, error, .. } => {
                format!("Failed to stop {} on {}: {}", node_id, agent_id, error)
            }
            GridEvent::NodeCrashed { node_id, agent_id, exit_code, .. } => {
                match exit_code {
                    Some(code) => format!("Node {} crashed on {} (exit code: {})", node_id, agent_id, code),
                    None => format!("Node {} crashed on {} (killed)", node_id, agent_id),
                }
            }
            GridEvent::NodeRestarted { node_id, agent_id, restart_count, new_pid, .. } => {
                format!("Node {} restarted on {} (attempt {}, new PID: {})", node_id, agent_id, restart_count, new_pid)
            }
            GridEvent::ClusterHealthy { total_agents, healthy_agents, total_nodes, running_nodes, .. } => {
                format!("Cluster healthy: {}/{} agents, {}/{} nodes", healthy_agents, total_agents, running_nodes, total_nodes)
            }
            GridEvent::ClusterDegraded { total_agents, healthy_agents, reason, .. } => {
                format!("Cluster degraded: {}/{} agents - {}", healthy_agents, total_agents, reason)
            }
            GridEvent::ClusterCritical { total_agents, healthy_agents, reason, .. } => {
                format!("Cluster critical: {}/{} agents - {}", healthy_agents, total_agents, reason)
            }
            GridEvent::StorageMetrics { node_id, concept_count, edge_count, write_throughput, read_latency_us, memory_usage_mb, .. } => {
                format!("Storage {} metrics: {} concepts, {} edges, {}K writes/sec, {} μs reads, {} MB memory", 
                    node_id, concept_count, edge_count, write_throughput / 1000, read_latency_us, memory_usage_mb)
            }
            GridEvent::QueryPerformance { node_id, query_type, query_depth, result_count, latency_ms, confidence, .. } => {
                format!("Query on {}: type={}, depth={}, {} results, {}ms, conf={:.2}",
                    node_id, query_type, query_depth, result_count, latency_ms, confidence)
            }
            GridEvent::EmbeddingLatency { service_instance, batch_size, latency_ms, dimension, cache_hit, .. } => {
                let cache_status = if *cache_hit { "cached" } else { "computed" };
                format!("Embedding {} latency: {} vectors ({}d) in {}ms ({})",
                    service_instance, batch_size, dimension, latency_ms, cache_status)
            }
            GridEvent::HnswIndexBuilt { node_id, vector_count, build_time_ms, dimension, .. } => {
                format!("HNSW index built on {}: {} vectors ({}d) in {}ms",
                    node_id, vector_count, dimension, build_time_ms)
            }
            GridEvent::HnswIndexLoaded { node_id, vector_count, load_time_ms, persisted, .. } => {
                let source = if *persisted { "disk" } else { "rebuild" };
                format!("HNSW index loaded on {} from {}: {} vectors in {}ms",
                    node_id, source, vector_count, load_time_ms)
            }
            GridEvent::PathfindingMetrics { node_id, source_id, target_id, path_length, paths_explored, latency_ms, strategy, .. } => {
                format!("Pathfinding on {}: {} → {} via {}, path_len={}, explored={}, {}ms",
                    node_id, source_id, target_id, strategy, path_length, paths_explored, latency_ms)
            }
            GridEvent::ReconciliationComplete { node_id, entries_processed, reconciliation_time_ms, disk_flush, .. } => {
                let flush_status = if *disk_flush { "with flush" } else { "memory only" };
                format!("Reconciliation on {}: {} entries in {}ms ({})",
                    node_id, entries_processed, reconciliation_time_ms, flush_status)
            }
            GridEvent::EmbeddingServiceHealthy { instance_id, endpoint, success_rate, cache_hit_rate, .. } => {
                format!("Embedding service {} healthy at {}: {:.1}% success, {:.1}% cache hits",
                    instance_id, endpoint, success_rate * 100.0, cache_hit_rate * 100.0)
            }
            GridEvent::EmbeddingServiceDegraded { instance_id, endpoint, error_rate, reason, .. } => {
                format!("Embedding service {} degraded at {}: {:.1}% errors - {}",
                    instance_id, endpoint, error_rate * 100.0, reason)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_event_serialization() {
        let event = GridEvent::AgentRegistered {
            agent_id: "agent-001".to_string(),
            hostname: "test-host".to_string(),
            platform: "linux".to_string(),
            agent_endpoint: "localhost:8001".to_string(),
            max_storage_nodes: 5,
            timestamp: Utc::now(),
        };
        
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("agent_registered"));
        assert!(json.contains("agent-001"));
    }
    
    #[test]
    fn test_event_description() {
        let event = GridEvent::SpawnSucceeded {
            node_id: "node-123".to_string(),
            agent_id: "agent-001".to_string(),
            pid: 12345,
            port: 50051,
            timestamp: Utc::now(),
        };
        
        let desc = event.to_description();
        assert!(desc.contains("node-123"));
        assert!(desc.contains("agent-001"));
        assert!(desc.contains("12345"));
        assert!(desc.contains("50051"));
    }
}
