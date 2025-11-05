# Sutra Grid Master

**Control plane for Sutra Grid** - manages agent registration, health monitoring, and storage node orchestration.

## Features

✅ **Agent Registration**: Agents register on startup  
✅ **Heartbeat Monitoring**: Detects failed agents (30s timeout)  
✅ **Health Status**: Tracks agent status (healthy/degraded/offline)  
✅ **Cluster Info**: List agents and cluster status via TCP Binary Protocol  
✅ **Storage Node Control**: Spawn/stop storage nodes (forwarded to agents)  
⚠️ **BREAKING CHANGE v3.0.0**: gRPC completely removed - TCP Binary Protocol only

## Building

```bash
cd packages/sutra-grid-master
cargo build --release
```

## Running

```bash
cargo run
```

**Output:**
```
🚀 Sutra Grid Master v0.1.0 starting on 0.0.0.0:7000
📡 Listening for agent connections...
📊 Cluster: 0 agents (0 healthy), 0 storage nodes (0 running) - healthy
```

## Configuration

**Port:** 7000 (hardcoded, will be configurable in future)  
**Health Check Interval:** 10 seconds  
**Agent Timeout:** 30 seconds (degraded at 15s)  
**Protocol:** TCP Binary with MessagePack serialization

## TCP Binary Protocol API

### Agent Lifecycle

- `RegisterAgent(AgentInfo) → RegistrationResponse`
- `Heartbeat(HeartbeatRequest) → HeartbeatResponse`
- `UnregisterAgent(AgentId) → Empty`

### Storage Node Management

- `SpawnStorageNode(SpawnRequest) → SpawnResponse`
- `StopStorageNode(StopRequest) → StopResponse`
- `GetStorageNodeStatus(NodeId) → NodeStatus`

### Cluster Info

- `ListAgents(Empty) → AgentList`
- `GetClusterStatus(Empty) → ClusterStatus`

## Testing

### Unit Tests (TODO)

```bash
cargo test
```

### Integration Test (with Agent)

1. Start Master: `cargo run`
2. Start Agent (see `sutra-grid-agent/README.md`)
3. Check logs for registration confirmation

## Architecture

```
Master
├── Agent Registry (in-memory HashMap)
│   ├── agent_id → AgentRecord
│   ├── Health tracking (last_heartbeat)
│   └── Storage nodes per agent
│
├── Health Monitor (background task)
│   ├── Check heartbeat staleness every 10s
│   └── Mark agents as degraded/offline
│
└── gRPC Server (port 7000)
    ├── Registration endpoint
    ├── Heartbeat endpoint
    └── Cluster info endpoints
```

## Logging

**Log levels:**
- `INFO`: Agent registration, health changes, cluster status
- `DEBUG`: Heartbeats, status queries
- `WARN`: Agent degraded/offline

**Set log level:**
```bash
RUST_LOG=debug cargo run
```

## Next Steps

- [x] Basic registration and heartbeat
- [ ] Persist agent registry (Sutra storage - no external databases)
- [ ] Forward spawn requests to agents (gRPC client)
- [ ] REST API for UI/CLI access
- [ ] Metrics export (Prometheus)
- [ ] Web UI (React dashboard)

## Related Documentation

- [Phase 1 Plan](../../../docs/sutra-storage/architecture/grid/PHASE1_MASTER_AGENT.md)
- [Unified Architecture](../../../docs/sutra-storage/architecture/grid/SUTRA_GRID_UNIFIED_ARCHITECTURE.md)
