# HA Phase 1 Implementation Plan: WAL Streaming

**Goal:** Implement asynchronous WAL streaming from leader to replicas  
**Duration:** 2 weeks  
**Deliverable:** 1 leader + N replicas with automatic data replication

---

## Week 1: Core Replication Infrastructure

### Day 1-2: gRPC Service Definition & Code Generation

**Tasks:**
1. Create `packages/sutra-storage/proto/replication.proto`
2. Define replication service
3. Generate Rust code with tonic-build
4. Update build.rs for proto compilation

**Files to Create:**
```
packages/sutra-storage/
  proto/
    replication.proto        # New service definition
  src/
    replication/
      mod.rs                 # Module entry point
      proto.rs               # Generated protobuf code (auto)
```

**Proto Definition:**
```protobuf
syntax = "proto3";
package sutra.replication;

service ReplicationService {
  // Stream WAL entries from leader to replica
  rpc StreamWalEntries(WalStreamRequest) returns (stream WalEntry);
  
  // Heartbeat from leader to replica
  rpc Heartbeat(HeartbeatRequest) returns (HeartbeatResponse);
}

message WalStreamRequest {
  uint64 start_sequence = 1;
  string replica_id = 2;
}

message WalEntry {
  uint64 sequence = 1;
  uint64 timestamp = 2;
  bytes operation_data = 3;  // Serialized Operation enum
  bytes transaction_id = 4;  // Optional transaction ID
}

message HeartbeatRequest {
  string leader_id = 1;
  uint64 current_sequence = 2;
  uint64 timestamp = 3;
}

message HeartbeatResponse {
  string replica_id = 1;
  uint64 last_applied_sequence = 2;
  bool healthy = 3;
  uint64 lag_ms = 4;
}
```

**Build Configuration:**
Update `packages/sutra-storage/build.rs`:
```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Existing proto compilation...
    
    // Add replication proto
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile(
            &["proto/replication.proto"],
            &["proto"],
        )?;
    
    Ok(())
}
```

**Acceptance Criteria:**
- [ ] Proto compiles without errors
- [ ] Generated Rust code accessible
- [ ] Basic client/server stubs available

---

### Day 3-4: WalStreamer Implementation (Leader Side)

**Create:** `packages/sutra-storage/src/replication/wal_streamer.rs`

**Structure:**
```rust
use crate::wal::{WriteAheadLog, LogEntry};
use tokio::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};

/// Streams WAL entries to replica nodes
pub struct WalStreamer {
    /// Reference to WAL
    wal: Arc<Mutex<WriteAheadLog>>,
    
    /// Last sequence number streamed
    last_streamed: Arc<AtomicU64>,
    
    /// Replica connections
    replicas: Arc<Mutex<Vec<ReplicaClient>>>,
    
    /// Shutdown signal
    shutdown: Arc<tokio::sync::Notify>,
}

impl WalStreamer {
    pub fn new(wal: Arc<Mutex<WriteAheadLog>>) -> Self {
        Self {
            wal,
            last_streamed: Arc::new(AtomicU64::new(0)),
            replicas: Arc::new(Mutex::new(Vec::new())),
            shutdown: Arc::new(tokio::sync::Notify::new()),
        }
    }
    
    /// Register a replica connection
    pub async fn add_replica(&self, addr: String) -> Result<(), Box<dyn std::error::Error>> {
        let client = ReplicaClient::connect(addr).await?;
        self.replicas.lock().unwrap().push(client);
        Ok(())
    }
    
    /// Start streaming loop (runs in background task)
    pub async fn start(&self) {
        loop {
            tokio::select! {
                _ = self.shutdown.notified() => {
                    log::info!("WalStreamer shutdown requested");
                    break;
                }
                _ = tokio::time::sleep(Duration::from_millis(10)) => {
                    if let Err(e) = self.stream_batch().await {
                        log::error!("Stream batch error: {}", e);
                    }
                }
            }
        }
    }
    
    /// Stream a batch of WAL entries to all replicas
    async fn stream_batch(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 1. Read new entries from WAL
        let last_seq = self.last_streamed.load(Ordering::Relaxed);
        let entries = self.read_wal_since(last_seq)?;
        
        if entries.is_empty() {
            return Ok(());
        }
        
        // 2. Send to all replicas concurrently
        let replicas = self.replicas.lock().unwrap().clone();
        for replica in replicas {
            // Non-blocking send (don't wait for replica)
            tokio::spawn(async move {
                for entry in &entries {
                    if let Err(e) = replica.send_entry(entry).await {
                        log::warn!("Failed to send entry to replica: {}", e);
                    }
                }
            });
        }
        
        // 3. Update last_streamed
        if let Some(last_entry) = entries.last() {
            self.last_streamed.store(last_entry.sequence, Ordering::Relaxed);
        }
        
        Ok(())
    }
    
    /// Read WAL entries since given sequence
    fn read_wal_since(&self, seq: u64) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let wal = self.wal.lock().unwrap();
        let all_entries = WriteAheadLog::read_entries(wal.path())?;
        
        let filtered: Vec<_> = all_entries
            .into_iter()
            .filter(|e| e.sequence > seq)
            .collect();
        
        Ok(filtered)
    }
}

/// Client for sending data to a replica
struct ReplicaClient {
    addr: String,
    // TODO: Add gRPC client
}

impl ReplicaClient {
    async fn connect(addr: String) -> Result<Self, Box<dyn std::error::Error>> {
        // TODO: Implement gRPC connection
        Ok(Self { addr })
    }
    
    async fn send_entry(&self, entry: &LogEntry) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Implement gRPC send
        Ok(())
    }
}
```

**Tests:**
Create `packages/sutra-storage/src/replication/tests.rs`:
```rust
#[tokio::test]
async fn test_wal_streamer_basic() {
    let dir = TempDir::new().unwrap();
    let wal_path = dir.path().join("wal.log");
    let wal = WriteAheadLog::create(&wal_path, true).unwrap();
    let wal = Arc::new(Mutex::new(wal));
    
    let streamer = WalStreamer::new(Arc::clone(&wal));
    
    // Write to WAL
    {
        let mut w = wal.lock().unwrap();
        w.append(Operation::WriteConcept { ... }).unwrap();
    }
    
    // Stream should pick up entry
    streamer.stream_batch().await.unwrap();
    
    assert_eq!(streamer.last_streamed.load(Ordering::Relaxed), 1);
}
```

**Acceptance Criteria:**
- [ ] WalStreamer compiles
- [ ] Can read new entries from WAL
- [ ] Updates last_streamed correctly
- [ ] Tests pass

---

### Day 5-6: ReplicaReceiver Implementation (Replica Side)

**Create:** `packages/sutra-storage/src/replication/replica_receiver.rs`

**Structure:**
```rust
/// Receives and applies WAL entries from leader
pub struct ReplicaReceiver {
    /// Local storage instance
    storage: Arc<Mutex<ConcurrentMemory>>,
    
    /// Leader address
    leader_addr: String,
    
    /// Last applied sequence
    last_applied: Arc<AtomicU64>,
    
    /// Shutdown signal
    shutdown: Arc<tokio::sync::Notify>,
}

impl ReplicaReceiver {
    pub fn new(storage: Arc<Mutex<ConcurrentMemory>>, leader_addr: String) -> Self {
        Self {
            storage,
            leader_addr,
            last_applied: Arc::new(AtomicU64::new(0)),
            shutdown: Arc::new(tokio::sync::Notify::new()),
        }
    }
    
    /// Start replication (connect to leader and consume stream)
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("ReplicaReceiver connecting to leader at {}", self.leader_addr);
        
        // TODO: Connect to leader via gRPC
        // let mut client = ReplicationServiceClient::connect(&self.leader_addr).await?;
        
        loop {
            tokio::select! {
                _ = self.shutdown.notified() => {
                    log::info!("ReplicaReceiver shutdown requested");
                    break;
                }
                // TODO: Receive from stream
                // Some(entry) = stream.message().await => {
                //     self.apply_entry(entry).await?;
                // }
            }
        }
        
        Ok(())
    }
    
    /// Apply a WAL entry to local storage
    async fn apply_entry(&self, entry: LogEntry) -> Result<(), Box<dyn std::error::Error>> {
        log::debug!("Applying entry seq={}", entry.sequence);
        
        // Apply operation to local ConcurrentMemory
        // This is similar to replay_wal() logic
        match entry.operation {
            Operation::WriteConcept { concept_id, .. } => {
                // Note: Full data needs to be in WAL for replication
                // TODO: Decide on replication format (metadata only or full data)
            }
            Operation::WriteAssociation { source, target, .. } => {
                // Apply association
            }
            _ => {}
        }
        
        self.last_applied.store(entry.sequence, Ordering::Relaxed);
        Ok(())
    }
}
```

**Acceptance Criteria:**
- [ ] ReplicaReceiver compiles
- [ ] Can apply entries to local storage
- [ ] Tracks last_applied sequence
- [ ] Tests pass

---

### Day 7: Integration & End-to-End Testing

**Test Setup:**
```rust
#[tokio::test]
async fn test_leader_replica_replication() {
    // 1. Start leader
    let leader_dir = TempDir::new().unwrap();
    let leader_config = ConcurrentConfig {
        storage_path: leader_dir.path().to_path_buf(),
        ..Default::default()
    };
    let leader = Arc::new(ConcurrentMemory::new(leader_config));
    
    // 2. Start WAL streamer on leader
    let streamer = WalStreamer::new(leader.wal.clone());
    tokio::spawn(async move {
        streamer.start().await;
    });
    
    // 3. Start replica
    let replica_dir = TempDir::new().unwrap();
    let replica_config = ConcurrentConfig {
        storage_path: replica_dir.path().to_path_buf(),
        ..Default::default()
    };
    let replica = Arc::new(ConcurrentMemory::new(replica_config));
    
    // 4. Start replica receiver
    let receiver = ReplicaReceiver::new(replica.clone(), "localhost:50051".to_string());
    tokio::spawn(async move {
        receiver.start().await.unwrap();
    });
    
    // 5. Write to leader
    let id = ConceptId([1; 16]);
    leader.learn_concept(id, b"test".to_vec(), None, 1.0, 0.9).unwrap();
    
    // 6. Wait for replication
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // 7. Verify data on replica
    assert!(replica.contains(&id));
}
```

**Acceptance Criteria:**
- [ ] Leader streams to replica
- [ ] Replica receives and applies entries
- [ ] Data visible on replica within 100ms
- [ ] No data loss under normal operation

---

## Week 2: Production Readiness

### Day 8-9: Error Handling & Retry Logic

**Tasks:**
1. Add connection retry for replicas
2. Handle network failures gracefully
3. Add exponential backoff
4. Implement replica reconnection

**Error Scenarios:**
- Replica unreachable → Log warning, continue streaming to others
- Network timeout → Retry with backoff
- Replica lag too high → Alert, but don't block leader

**Code Addition (wal_streamer.rs):**
```rust
async fn send_with_retry(
    &self,
    replica: &ReplicaClient,
    entry: &LogEntry,
    max_retries: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut retries = 0;
    let mut backoff = Duration::from_millis(10);
    
    loop {
        match replica.send_entry(entry).await {
            Ok(()) => return Ok(()),
            Err(e) if retries < max_retries => {
                log::warn!("Retry {}/{} after error: {}", retries + 1, max_retries, e);
                tokio::time::sleep(backoff).await;
                backoff *= 2; // Exponential backoff
                retries += 1;
            }
            Err(e) => return Err(e),
        }
    }
}
```

---

### Day 10-11: Monitoring & Metrics

**Tasks:**
1. Add replication lag metrics
2. Track replica health status
3. Expose metrics via stats endpoint

**Metrics to Track:**
```rust
pub struct ReplicationMetrics {
    pub leader_sequence: u64,
    pub replicas: Vec<ReplicaStatus>,
}

pub struct ReplicaStatus {
    pub replica_id: String,
    pub last_applied: u64,
    pub lag_ms: u64,
    pub healthy: bool,
    pub last_heartbeat: u64,
}
```

**Expose in gRPC Service:**
```rust
async fn get_replication_status(
    &self,
    _request: Request<ReplicationStatusRequest>,
) -> Result<Response<ReplicationStatusResponse>, Status> {
    let metrics = self.get_metrics();
    Ok(Response::new(ReplicationStatusResponse {
        leader_sequence: metrics.leader_sequence,
        replicas: metrics.replicas.into_iter().map(|r| {
            ReplicaInfo {
                id: r.replica_id,
                lag: r.lag_ms,
                healthy: r.healthy,
            }
        }).collect(),
    }))
}
```

---

### Day 12-13: Documentation & Deployment

**Documentation to Create:**
1. Replication setup guide
2. Troubleshooting guide
3. Performance tuning guide

**Setup Guide Example:**
```markdown
# Setting Up Replication

## Leader Configuration
```bash
# Environment variables
export SUTRA_NODE_ROLE=leader
export SUTRA_REPLICAS=replica1:50052,replica2:50053
export SUTRA_STORAGE_SERVER=0.0.0.0:50051

# Start leader
cargo run --release --bin storage_server
```

## Replica Configuration
```bash
# Replica 1
export SUTRA_NODE_ROLE=replica
export SUTRA_LEADER=leader:50051
export SUTRA_STORAGE_SERVER=0.0.0.0:50052

# Start replica
cargo run --release --bin storage_server
```

## Verification
```bash
# Check replication status
grpcurl -plaintext localhost:50051 sutra.replication.ReplicationService/GetReplicationStatus
```
```

---

### Day 14: Load Testing & Validation

**Load Test Script:**
```rust
#[tokio::test]
async fn test_replication_under_load() {
    // Setup leader + 3 replicas
    // ...
    
    // Generate 10K writes/sec for 1 minute
    for i in 0..600_000 {
        let id = ConceptId::from_u64(i);
        leader.learn_concept(id, vec![i as u8], None, 1.0, 0.9).unwrap();
        
        if i % 1000 == 0 {
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    }
    
    // Verify all replicas have data
    tokio::time::sleep(Duration::from_secs(5)).await;
    
    for replica in &replicas {
        let stats = replica.stats();
        assert!(stats.snapshot.concept_count >= 600_000);
    }
}
```

**Success Criteria:**
- [ ] Replication keeps up with 10K writes/sec
- [ ] Replica lag < 100ms under load
- [ ] No crashes or data loss
- [ ] Memory usage stable

---

## Deliverables Checklist

### Code
- [ ] `proto/replication.proto` - gRPC service definition
- [ ] `src/replication/mod.rs` - Module entry
- [ ] `src/replication/wal_streamer.rs` - Leader streaming
- [ ] `src/replication/replica_receiver.rs` - Replica receiving
- [ ] `src/replication/tests.rs` - Integration tests

### Documentation
- [ ] Replication setup guide
- [ ] Troubleshooting guide
- [ ] Architecture diagram
- [ ] API documentation

### Tests
- [ ] Unit tests for WalStreamer
- [ ] Unit tests for ReplicaReceiver
- [ ] Integration test: 1 leader + 1 replica
- [ ] Load test: 10K writes/sec
- [ ] Failure test: Network partition

### Deployment
- [ ] Docker Compose configuration for multi-node
- [ ] Environment variable documentation
- [ ] Monitoring dashboard (Grafana/Prometheus)

---

## Risk Mitigation

### Technical Risks
1. **WAL entries too large**
   - Mitigation: Only stream metadata, fetch full data on-demand
   
2. **Network congestion**
   - Mitigation: Implement flow control, async streaming
   
3. **Replica falls too far behind**
   - Mitigation: Alert on high lag, support full snapshot sync

### Schedule Risks
1. **gRPC integration complexity**
   - Buffer: 2 extra days for debugging
   
2. **Testing infrastructure setup**
   - Mitigation: Use Docker Compose for easy multi-node testing

---

## Next Phase Preview

**Phase 2: Health Checks & Heartbeat (Week 3)**
- Implement HealthMonitor
- Add heartbeat mechanism
- Detect leader failures automatically

**Phase 3: Automatic Failover (Week 4)**
- Implement leader election
- Promote replica to leader
- Client redirection
