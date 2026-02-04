# High Availability & Replication Design

## Overview
Design for zero-downtime, multi-node deployment of Sutra Storage with automatic failover.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Client Layer                             │
│  (Load Balancer / Service Discovery - routes to leader)        │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            ▼
          ┌─────────────────────────────────┐
          │       PRIMARY (Leader)          │
          │  - Accepts all writes           │
          │  - Streams WAL to replicas      │
          │  - Heartbeat sender             │
          └────────────┬────────────────────┘
                       │ WAL Stream
                       │ (gRPC streaming)
          ┌────────────┴────────────┬───────────────┐
          ▼                         ▼               ▼
    ┌─────────┐             ┌─────────┐      ┌─────────┐
    │REPLICA 1│             │REPLICA 2│      │REPLICA 3│
    │ - Reads │             │ - Reads │      │ - Reads │
    │ - Standby│            │ - Standby│     │ - Standby│
    └─────────┘             └─────────┘      └─────────┘
```

## Replication Model

### Async Replication (Phase 1 - Simpler)
**Characteristics:**
- Leader writes to local WAL + storage, returns success immediately
- Replicates to followers asynchronously via gRPC stream
- **Trade-off:** Possible data loss on leader crash (< 10ms of writes)
- **Benefit:** Low latency writes (no network wait)

**Recovery Point Objective (RPO):** ~10ms (one reconciliation cycle)  
**Recovery Time Objective (RTO):** <5 seconds (failover time)

### Sync Replication (Phase 2 - Optional)
**Characteristics:**
- Leader waits for majority (quorum) of replicas to ACK before returning
- **Trade-off:** Higher write latency (+2-10ms depending on network)
- **Benefit:** Zero data loss on failover

**RPO:** 0 (zero data loss)  
**RTO:** <5 seconds

## Component Design

### 1. WAL Streaming (New Module)

**File:** `packages/sutra-storage/src/replication/wal_streamer.rs`

```rust
/// Streams WAL entries to replica nodes
pub struct WalStreamer {
    wal: Arc<Mutex<WriteAheadLog>>,
    replicas: Vec<ReplicaClient>,
    last_streamed_sequence: AtomicU64,
}

impl WalStreamer {
    /// Start streaming WAL entries to all replicas
    pub async fn stream_loop(&self) {
        loop {
            // Read new WAL entries since last_streamed_sequence
            let entries = self.read_new_entries().await;
            
            // Stream to all replicas concurrently
            for replica in &self.replicas {
                replica.send_entries(&entries).await;
            }
            
            // Update last_streamed_sequence
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }
}
```

### 2. Replica Receiver (New Module)

**File:** `packages/sutra-storage/src/replication/replica_receiver.rs`

```rust
/// Receives and applies WAL entries from leader
pub struct ReplicaReceiver {
    storage: Arc<ConcurrentMemory>,
    leader_address: String,
}

impl ReplicaReceiver {
    /// Connect to leader and consume WAL stream
    pub async fn start_replication(&self) -> Result<()> {
        let mut client = StorageReplicationClient::connect(&self.leader_address).await?;
        
        let mut stream = client.stream_wal_entries(WalStreamRequest {
            start_sequence: self.storage.last_applied_sequence(),
        }).await?.into_inner();
        
        // Consume stream and apply entries
        while let Some(entry) = stream.message().await? {
            self.apply_entry(entry).await?;
        }
        
        Ok(())
    }
    
    /// Apply WAL entry to local storage
    async fn apply_entry(&self, entry: WalEntry) -> Result<()> {
        // Replay into local ConcurrentMemory
        // This is already implemented via replay_wal()
        Ok(())
    }
}
```

### 3. Health Check & Failover (New Module)

**File:** `packages/sutra-storage/src/replication/health.rs`

```rust
/// Health monitoring and leader election
pub struct HealthMonitor {
    is_leader: Arc<AtomicBool>,
    last_heartbeat: Arc<RwLock<Instant>>,
    replicas: Vec<String>,
}

impl HealthMonitor {
    /// Send heartbeat to all replicas (leader only)
    pub async fn send_heartbeats(&self) {
        loop {
            for replica in &self.replicas {
                // Send heartbeat via gRPC
                let _ = self.send_heartbeat(replica).await;
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
    
    /// Monitor leader health (replica only)
    pub async fn monitor_leader(&self) {
        loop {
            tokio::time::sleep(Duration::from_secs(3)).await;
            
            let last_hb = *self.last_heartbeat.read().unwrap();
            if last_hb.elapsed() > Duration::from_secs(5) {
                // Leader is down - initiate election
                self.initiate_election().await;
            }
        }
    }
    
    /// Simple leader election (highest ID wins)
    async fn initiate_election(&self) {
        // In production, use Raft or etcd for consensus
        // For MVP: use pre-configured priority
        log::warn!("Leader failure detected, promoting self to leader");
        self.is_leader.store(true, Ordering::SeqCst);
    }
}
```

### 4. gRPC Service Extensions

**File:** `packages/sutra-storage/proto/replication.proto`

```protobuf
syntax = "proto3";
package sutra.replication;

service ReplicationService {
  // Stream WAL entries to replicas
  rpc StreamWalEntries(WalStreamRequest) returns (stream WalEntry);
  
  // Heartbeat from leader to replica
  rpc Heartbeat(HeartbeatRequest) returns (HeartbeatResponse);
  
  // Promote replica to leader
  rpc Promote(PromoteRequest) returns (PromoteResponse);
}

message WalStreamRequest {
  uint64 start_sequence = 1;
}

message WalEntry {
  uint64 sequence = 1;
  uint64 timestamp = 2;
  bytes operation_data = 3; // Serialized Operation
}

message HeartbeatRequest {
  string leader_id = 1;
  uint64 leader_sequence = 2;
}

message HeartbeatResponse {
  string replica_id = 1;
  uint64 last_applied_sequence = 2;
  bool healthy = 3;
}
```

## Configuration

### Leader Configuration
```bash
# Environment variables
SUTRA_NODE_ID="node-1"
SUTRA_NODE_ROLE="leader"
SUTRA_REPLICAS="node-2:50051,node-3:50051"
SUTRA_STORAGE_SERVER="0.0.0.0:50051"
```

### Replica Configuration
```bash
SUTRA_NODE_ID="node-2"
SUTRA_NODE_ROLE="replica"
SUTRA_LEADER="node-1:50051"
SUTRA_STORAGE_SERVER="0.0.0.0:50052"
```

## Implementation Phases

### Phase 1: WAL Streaming (Week 1-2)
- [ ] Add `replication` module to `src/`
- [ ] Implement `WalStreamer` (leader side)
- [ ] Implement `ReplicaReceiver` (replica side)
- [ ] Add gRPC service definition
- [ ] Test: 1 leader + 1 replica, verify data replication

**Deliverable:** Async replication working, manual failover

### Phase 2: Health Checks (Week 3)
- [ ] Implement `HealthMonitor`
- [ ] Add heartbeat mechanism
- [ ] Test: Detect leader failure

**Deliverable:** Automatic failure detection

### Phase 3: Failover (Week 4)
- [ ] Implement leader election (simple priority-based)
- [ ] Add promotion endpoint
- [ ] Update clients to redirect to new leader
- [ ] Test: Automatic failover on crash

**Deliverable:** Automatic failover with <5s RTO

### Phase 4: Production Hardening (Week 5-6)
- [ ] Add split-brain protection
- [ ] Implement proper consensus (Raft integration)
- [ ] Add replication lag monitoring
- [ ] Load testing with chaos engineering

**Deliverable:** Production-ready HA setup

## Testing Strategy

### Manual Failover Test
```bash
# Terminal 1: Start leader
SUTRA_NODE_ROLE=leader cargo run --bin storage_server

# Terminal 2: Start replica
SUTRA_NODE_ROLE=replica SUTRA_LEADER=localhost:50051 cargo run --bin storage_server -- --port 50052

# Terminal 3: Write data to leader
grpcurl -plaintext -d '{"concept_id": "abc", "content": "test"}' localhost:50051 sutra.storage.StorageService/LearnConcept

# Terminal 4: Read from replica (should have data within 10ms)
grpcurl -plaintext -d '{"concept_id": "abc"}' localhost:50052 sutra.storage.StorageService/QueryConcept

# Kill leader (Ctrl+C in Terminal 1)

# Promote replica
grpcurl -plaintext localhost:50052 sutra.replication.ReplicationService/Promote

# Verify replica accepts writes
grpcurl -plaintext -d '{"concept_id": "def", "content": "new"}' localhost:50052 sutra.storage.StorageService/LearnConcept
```

## Trade-offs

### Async Replication
✅ **Pros:**
- Low write latency
- Simple implementation
- No network dependency for writes

❌ **Cons:**
- Possible data loss on crash (<10ms of writes)
- Not suitable for financial transactions

### Sync Replication
✅ **Pros:**
- Zero data loss
- Strong consistency

❌ **Cons:**
- Higher latency (+2-10ms)
- Writes fail if majority unavailable

**Recommendation:** Start with async (Phase 1-4), add sync as configuration option later.

## Comparison with Alternatives

| Feature | Sutra (Async) | Neo4j (Causal Cluster) | PostgreSQL (Streaming) |
|---------|---------------|------------------------|------------------------|
| RPO | ~10ms | 0 (sync) | Configurable |
| RTO | <5s | <30s | <10s |
| Write Latency | +0ms | +10ms | +5ms |
| Setup Complexity | Simple | Complex | Medium |
| Cost | Low | High | Medium |

## Open Questions

1. **Consensus Algorithm:** Use Raft library or roll our own?
   - **Decision:** Use `openraft` crate for Phase 4

2. **Split-Brain Prevention:** How to prevent two leaders?
   - **Decision:** Use generation number in WAL, reject stale leaders

3. **Read Consistency:** Allow reads from replicas?
   - **Decision:** Yes (eventual consistency), add `read_concern` flag later

4. **Replication Lag:** How much lag is acceptable?
   - **Decision:** <100ms under normal load, alert if >1s

## Next Steps

1. Complete Phase 1 (WAL streaming) this week
2. Deploy 1 leader + 2 replicas in dev environment
3. Run chaos testing (random kills)
4. Measure actual RPO/RTO with synthetic workload
