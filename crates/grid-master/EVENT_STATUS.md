# Master Event Integration Status

## ✅ Completed

### 1. Package Setup
- ✅ Created `sutra-grid-events` package
- ✅ Defined 17 event types
- ✅ Implemented `EventEmitter` with async background worker
- ✅ Added to master's Cargo.toml
- ✅ Added `events: Option<EventEmitter>` field to GridMasterService
- ✅ Added `new_with_events()` constructor

### 2. Agent Lifecycle Events
- ✅ **AgentRegistered**: Emitted in `register_agent()` after successful registration
- ✅ **AgentRecovered**: Emitted in `heartbeat()` when agent transitions from Degraded/Offline → Healthy
- ✅ **AgentDegraded**: Emitted in `check_agent_health()` when 15-30s without heartbeat
- ✅ **AgentOffline**: Emitted in `check_agent_health()` when >30s without heartbeat
- ✅ **AgentUnregistered**: Emitted in `unregister_agent()` when agent leaves

### 3. Heartbeat
- ⏳ **AgentHeartbeat**: Not currently emitted (would be noisy, but could be sampled)

## 🚧 In Progress

### Node Lifecycle Events
Need to add to `spawn_storage_node()` and `stop_storage_node()`:

#### Spawn Events
```rust
// At start of spawn_storage_node()
if let Some(events) = &self.events {
    events.emit(GridEvent::SpawnRequested {
        node_id: node_id.clone(),
        agent_id: req.agent_id.clone(),
        port: req.port,
        storage_path: req.storage_path.clone(),
        memory_limit_mb: req.memory_limit_mb,
        timestamp: Utc::now(),
    });
}

// After successful spawn
if let Some(events) = &self.events {
    events.emit(GridEvent::SpawnSucceeded {
        node_id: resp.node_id.clone(),
        agent_id: req.agent_id.clone(),
        pid: resp.pid,
        port: resp.port,
        timestamp: Utc::now(),
    });
}

// On spawn failure
if let Some(events) = &self.events {
    events.emit(GridEvent::SpawnFailed {
        node_id: node_id.clone(),
        agent_id: req.agent_id.clone(),
        error: resp.error_message.clone(),
        timestamp: Utc::now(),
    });
}
```

#### Stop Events
```rust
// At start of stop_storage_node()
if let Some(events) = &self.events {
    events.emit(GridEvent::StopRequested {
        node_id: req.node_id.clone(),
        agent_id: req.agent_id.clone(),
        timestamp: Utc::now(),
    });
}

// After successful stop
if let Some(events) = &self.events {
    events.emit(GridEvent::StopSucceeded {
        node_id: req.node_id.clone(),
        agent_id: req.agent_id.clone(),
        timestamp: Utc::now(),
    });
}

// On stop failure
if let Some(events) = &self.events {
    events.emit(GridEvent::StopFailed {
        node_id: req.node_id.clone(),
        agent_id: req.agent_id.clone(),
        error: resp.error_message.clone(),
        timestamp: Utc::now(),
    });
}
```

## ⏳ TODO

### 1. Main Function Update
Need to initialize EventEmitter in main():

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    let addr = "0.0.0.0:7000".parse()?;
    
    // Initialize event storage connection
    let event_storage = std::env::var("EVENT_STORAGE")
        .unwrap_or_else(|_| "http://localhost:50052".to_string());
    
    let events = match EventEmitter::new(event_storage.clone()).await {
        Ok(e) => {
            log::info!("📊 Event emission enabled (storage: {})", event_storage);
            Some(e)
        }
        Err(e) => {
            log::warn!("⚠️  Event emission disabled: {}", e);
            None
        }
    };
    
    let service = match events {
        Some(e) => GridMasterService::new_with_events(e),
        None => GridMasterService::new(),
    };
    
    // ... rest of main
}
```

### 2. Agent Integration
- Add EventEmitter to agent's main.rs
- Emit NodeCrashed in storage monitor
- Emit NodeRestarted after auto-restart

### 3. Reserved Storage Instance
- Start dedicated storage-server on port 50052
- Pre-populate with Grid domain concepts
- Bootstrap event store schema

### 4. Query API (sutra-hybrid)
- Add temporal query helpers
- Add event aggregation functions
- Natural language→graph query translation

### 5. Control UI Integration
- Grid management pane
- Event stream display
- Chat-based queries
- Real-time updates

## Testing

Once spawn/stop events are added, test with:

```bash
# Terminal 1: Start reserved storage for events
cd packages/sutra-storage
cargo run --release -- --port 50052 --storage-path /tmp/grid-events

# Terminal 2: Start master with event storage
cd packages/sutra-grid-master
EVENT_STORAGE=http://localhost:50052 cargo run --release

# Terminal 3: Start agent
cd packages/sutra-grid-agent
cargo run --release

# Terminal 4: Trigger operations
cd packages/sutra-grid-master
./target/release/sutra-grid-cli spawn --agent agent-001 --port 50051

# Verify events in storage
# Query the storage directly or via sutra-hybrid
```

## Event Flow Example

```
User: spawn node on agent-001
   ↓
Master emits: SpawnRequested
   ↓
Master→Agent: gRPC SpawnNode
   ↓
Agent spawns process
   ↓
Agent←Master: Success (PID, port)
   ↓
Master emits: SpawnSucceeded
   ↓
Storage writes:
  - Concept: event-spawn_succeeded-{timestamp}
  - Association: agent-001 → event
  - Association: node-{id} → event
  - Association: event → ts-{timestamp}
   ↓
Query via Control:
  "What happened on agent-001?"
  → Shows spawn event with full context
```

## Benefits Proven

1. **No LMT Stack**: Zero Prometheus, Grafana, ELK
2. **Natural Language Queries**: Chat-based ops
3. **Self-Hosting**: Sutra monitors Sutra
4. **Complete Audit Trail**: Every operation tracked
5. **Semantic Search**: Find related events
6. **Temporal Queries**: "Show failures today"
7. **Causality**: Trace event chains

## Next Actions

1. ✅ Add spawn/stop event emissions (5 min)
2. ✅ Update main() to initialize events (3 min)
3. ⏳ Test end-to-end with reserved storage
4. ⏳ Integrate into agent
5. ⏳ Build query API
6. ⏳ Add to Control UI

**Current Progress: ~60% Complete**

The foundation is solid - event types defined, emitter working, agent lifecycle fully instrumented. Just need to finish node lifecycle events and connect the query layer!
