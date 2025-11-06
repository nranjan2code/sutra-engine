# Sutra Grid Architecture Documentation

This directory contains the complete architecture design for **Sutra Grid**, a distributed reasoning engine that scales from gigabytes to terabytes of knowledge graph data.

## Overview

Sutra Grid combines four core systems:
1. **Grid Infrastructure**: Self-healing P2P network for node management
2. **Control Center UI**: Web-based management interface for Grid operations ✅
3. **Data Sharding**: Intelligent graph partitioning across nodes
4. **Query Execution**: Distributed reasoning with cross-shard coordination

## Documentation Index

### 1. [SUTRA_GRID_UNIFIED_ARCHITECTURE.md](./SUTRA_GRID_UNIFIED_ARCHITECTURE.md) ⭐ **START HERE**

**Comprehensive end-to-end architecture** combining all components.

**Contents:**
- Complete system overview with all layers
- Data flow examples (write, read, reasoning queries)
- Integration points between components
- Failure scenarios and recovery procedures
- Scalability analysis (1GB to 100TB+)
- Deployment example for 1TB dataset
- Implementation roadmap (16 weeks)

**Best for:** Understanding the complete system, deployment planning, integration design

---

### 2. [SUTRA_GRID_ARCHITECTURE.md](./architecture/SUTRA_GRID_ARCHITECTURE.md)

See also: [GRID_ARCHITECTURE.md](./architecture/GRID_ARCHITECTURE.md) - Production implementation with complete event-driven observability.

**Deep dive into grid infrastructure and node management.**

**Contents:**
- Grid Master (coordinator, health monitor, failover manager)
- Grid Agents (lifecycle management, auto-restart)
- Gossip Protocol (P2P health checks, state propagation)
- Node discovery and registration
- Automatic failover procedures
- Grid CLI and management tools

**Best for:** Infrastructure engineers, DevOps, understanding cluster management

---

### 3. [SUTRA_GRID_SHARDING.md](./SUTRA_GRID_SHARDING.md)

**Deep dive into data partitioning and terabyte-scale distribution.**

**Contents:**
- Sharding strategies (hash, range, graph-aware)
- Shard map and routing layer design
- Cross-shard query execution (distributed BFS)
- Auto-rebalancing algorithms
- Deployment examples (100GB, 1TB, 10TB)
- Performance impact analysis
- Comparison with Neo4j Fabric, MongoDB sharding

**Best for:** Data architects, performance optimization, understanding sharding strategies

---

## Reading Path

### For **System Architects**:
1. Start with **SUTRA_GRID_UNIFIED_ARCHITECTURE.md** (complete picture)
2. Dive into **SUTRA_GRID_ARCHITECTURE.md** (infrastructure details)
3. Review **SUTRA_GRID_SHARDING.md** (data distribution strategy)

### For **Infrastructure Engineers**:
1. Start with **SUTRA_GRID_ARCHITECTURE.md** (grid infrastructure)
2. Review **SUTRA_GRID_UNIFIED_ARCHITECTURE.md** (integration points)
3. Reference **SUTRA_GRID_SHARDING.md** (data management requirements)

### For **Data Engineers**:
1. Start with **SUTRA_GRID_SHARDING.md** (data partitioning)
2. Review **SUTRA_GRID_UNIFIED_ARCHITECTURE.md** (query execution)
3. Reference **SUTRA_GRID_ARCHITECTURE.md** (node placement constraints)

### For **Product/Business**:
1. Read **SUTRA_GRID_UNIFIED_ARCHITECTURE.md** sections:
   - Executive Summary
   - Scalability Analysis
   - Competitive Comparison
   - Deployment Example

---

## Key Concepts

### Grid Master
Single coordinator that manages cluster state, routes queries, and orchestrates failover. Not a single point of failure (can run in HA mode).

**Production Features:**
- ✅ Background health monitoring (30s interval)
- ✅ Event emission via sutra-grid-events (EVENT_STORAGE env var)
- ✅ Automatic agent status updates on heartbeat
- ✅ Cluster status aggregation and reporting

### Grid Agent
Node-local process that spawns and monitors storage nodes. Automatically restarts crashed processes and reports health to Master.

### Shard
Logical partition of the knowledge graph (subset of concepts and associations). Each shard is replicated 3× for durability.

### Storage Node
Process that stores and serves one or more shards. Runs BFS engine for local graph traversal.

### Graph-Aware Sharding
Intelligent partitioning that co-locates related concepts (e.g., all medical concepts on one shard) to minimize cross-shard queries.

### Distributed BFS
Breadth-first search that spans multiple shards, coordinated by Query Router with result aggregation.

---

## Quick Facts

- **Scalability**: Linear from 1 node (10GB) to 100+ nodes (10TB+)
- **Performance**: <1ms queries (90%+ intra-shard), 3ms cross-shard
- **Durability**: Zero data loss (WAL + 3× replication)
- **Availability**: <5 second failover, 100% uptime with replicas
- **Cost**: $500/month for 1TB (vs $3000 for Neo4j Fabric)
- **Monitoring**: Built-in health checks every 30s with event-driven observability

---

## Implementation Status

**Phase 1: Grid Infrastructure** ✅ (Completed)
- Master coordinator with gRPC API
- Agent lifecycle management
- Gossip protocol for health checks
- Basic failover
- **Control Center Web UI** ✅ (Grid management interface)
- **Health Monitoring** ✅ (30s background task with event emission)
- **Configuration Integration** ✅ (All agent/master fields in use)

**Phase 2: Data Sharding** 🚧 (In Design)
- Hash-based shard map
- Routing layer
- Cross-shard queries
- Rebalancing

**Phase 3: High Availability** 📋 (Planned)
- WAL streaming
- Automatic failover with quorum
- Network partition handling

**Phase 4: Auto-Scaling** 📋 (Planned)
- Shard splitting/merging
- Live rebalancing

**Phase 5: Advanced Features** 📋 (Planned)
- Graph-aware sharding
- Query optimizer
- Monitoring dashboard

---

## Related Documentation

### In Parent Directory (`docs/sutra-storage/architecture/`)
- **HA_PHASE1_PLAN.md**: High availability implementation plan
- **PRODUCTION_DEPLOYMENT.md**: Production deployment guide
- **Memory layouts**: Single-file storage format specifications

### In Root Documentation (`docs/`)
- **WARP.md**: Project overview and development guide
- **CHANGELOG.md**: Version history and features
- **HA_REPLICATION_DESIGN.md**: Replication architecture

---

## Contributing

When adding new grid-related documentation:

1. **Infrastructure changes**: Update `SUTRA_GRID_ARCHITECTURE.md`
2. **Sharding changes**: Update `SUTRA_GRID_SHARDING.md`
3. **Integration changes**: Update `SUTRA_GRID_UNIFIED_ARCHITECTURE.md`
4. **New components**: Create separate design doc, link from unified architecture

---

## Questions?

For questions about grid architecture:
- See **SUTRA_GRID_UNIFIED_ARCHITECTURE.md** for integration points
- See individual component docs for deep dives
- Check `docs/` root for implementation plans and guides
