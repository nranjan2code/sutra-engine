# Sutra AI - System Architecture

**Version**: 1.0  
**Last Updated**: October 15, 2025

---

## Table of Contents

1. [Overview](#overview)
2. [Architectural Principles](#architectural-principles)
3. [System Layers](#system-layers)
4. [Component Architecture](#component-architecture)
5. [Data Flow](#data-flow)
6. [Storage Architecture](#storage-architecture)
7. [Scalability Strategy](#scalability-strategy)
8. [Design Patterns](#design-patterns)

---

## Overview

Sutra AI employs a **layered, modular architecture** designed specifically for continuous learning and explainable reasoning. Unlike traditional neural network architectures that rely on frozen weights, our system uses a **dynamic knowledge graph** that evolves in real-time.

> **Related Documentation**:  
> - [DESIGN.md](DESIGN.md) - Design decisions and trade-offs behind this architecture  
> - [ALGORITHMS.md](ALGORITHMS.md) - Core algorithms implementing these components  
> - [CONTRIBUTING.md](CONTRIBUTING.md) - Development workflow and guidelines

### Core Architectural Vision

```
Traditional AI:          Sutra AI:
┌─────────────────┐    ┌──────────────────────┐
│  Neural Weights │    │  Living Knowledge    │
│   (Frozen)      │    │     Graph            │
│                 │    │   (Evolving)         │
└─────────────────┘    └──────────────────────┘
        ↓                        ↓
   Black Box              Glass Box
   Retraining          Instant Learning
   Context Limits       Infinite Memory
```

---

## Architectural Principles

### 1. **Separation of Concerns**

The system is divided into distinct layers, each with a single responsibility:

- **Storage Layer**: Manages persistent knowledge representation
- **Graph Layer**: Handles graph structure and traversal
- **Learning Layer**: Processes new information and updates
- **Reasoning Layer**: Performs inference and query answering
- **Interface Layer**: Provides APIs for external access

### 2. **Temporal by Design**

Time is a first-class citizen:
- All entities track creation and access timestamps
- Strength and confidence evolve over time
- Decay mechanisms model forgetting curves
- Versioning enables time-travel queries

### 3. **Explainability First**

Every decision leaves a trace:
- Reasoning paths are explicitly constructed
- Confidence scores propagate through graph
- Alternative answers are tracked
- Contradictions are detected and recorded

### 4. **Continuous Evolution**

The system never stops adapting:
- Learning happens in real-time
- No training/inference separation
- Knowledge strengthens with use
- Adaptive resource allocation

### 5. **Composition Over Inheritance**

Components are assembled, not derived:
- Interfaces define contracts
- Dependency injection enables testing
- Loose coupling supports evolution
- Clear boundaries between modules

---

## System Layers

```
┌──────────────────────────────────────────────────────────────┐
│                      Interface Layer                          │
│  ┌──────────────┐ ┌──────────────┐ ┌─────────────────────┐ │
│  │ Python API   │ │   REST API   │ │       CLI           │ │
│  └──────────────┘ └──────────────┘ └─────────────────────┘ │
└────────────────────────────┬─────────────────────────────────┘
                             │
┌────────────────────────────┴─────────────────────────────────┐
│                      Reasoning Layer                          │
│  ┌───────────────────────────────────────────────────────┐  │
│  │            ReasoningEngine (Orchestrator)             │  │
│  └───────────────────────────────────────────────────────┘  │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────┐  │
│  │ Query    │ │ Path     │ │  MPPA    │ │ Contradiction│  │
│  │ Planner  │ │ Finder   │ │          │ │  Resolver    │  │
│  └──────────┘ └──────────┘ └──────────┘ └──────────────┘  │
└────────────────────────────┬─────────────────────────────────┘
                             │
┌────────────────────────────┴─────────────────────────────────┐
│                      Learning Layer                           │
│  ┌──────────────┐ ┌──────────────────┐ ┌─────────────────┐ │
│  │  Adaptive    │ │   Association    │ │  Contradiction  │ │
│  │  Learner     │ │   Extractor      │ │   Detection     │ │
│  └──────────────┘ └──────────────────┘ └─────────────────┘ │
└────────────────────────────┬─────────────────────────────────┘
                             │
┌────────────────────────────┴─────────────────────────────────┐
│                      Graph Layer                              │
│  ┌──────────────┐ ┌──────────────────┐ ┌─────────────────┐ │
│  │  Concepts    │ │   Associations   │ │   Traversal     │ │
│  │  (Nodes)     │ │    (Edges)       │ │   Algorithms    │ │
│  └──────────────┘ └──────────────────┘ └─────────────────┘ │
└────────────────────────────┬─────────────────────────────────┘
                             │
┌────────────────────────────┴─────────────────────────────────┐
│                      Storage Layer                            │
│  ┌──────────────────────────────────────────────────────┐   │
│  │       Temporal Log-Structured Store (Rust)           │   │
│  │  - Memory-mapped segments                            │   │
│  │  - Zero-copy access                                  │   │
│  │  - Lock-free concurrency                             │   │
│  └──────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────┘
```

---

## Component Architecture

### ReasoningEngine (Orchestrator)

**Purpose**: Central coordination point for all reasoning operations.

**Responsibilities**:
- Accepts natural language queries
- Coordinates learning operations
- Manages component lifecycle
- Provides unified API surface
- Handles caching and optimization

**Key Design Decisions**:
- **Single entry point** simplifies usage
- **Dependency injection** enables testing
- **Event-driven** architecture for extensibility
- **Stateful** but thread-safe design

**Component Interactions**:
```
User Query
    ↓
ReasoningEngine.ask()
    ↓
    ├──> QueryPlanner (decompose if complex)
    ├──> QueryProcessor (classify intent)
    ├──> PathFinder (find reasoning paths)
    ├──> MPPA (aggregate consensus)
    └──> ContradictionResolver (check conflicts)
    ↓
Consensus Result
```

### Query Planner

**Purpose**: Decomposes complex queries into manageable steps.

**Architecture Pattern**: **Strategy + Builder**

**Key Components**:
- **Pattern Matcher**: Identifies query structures
- **Dependency Analyzer**: Builds execution graph
- **Optimizer**: Reorders for efficiency
- **Executor**: Coordinates step execution

**Decomposition Strategy**:
```
Complex Query → Pattern Match → Sub-Questions
                     ↓
            Dependency Graph
                     ↓
            Topological Sort
                     ↓
            Execution Plan
```

### Path Finder

**Purpose**: Graph traversal for multi-hop reasoning.

> See [ALGORITHMS.md#graph-traversal-algorithms](ALGORITHMS.md#graph-traversal-algorithms) for detailed algorithm descriptions.

**Architecture Pattern**: **Strategy Pattern**

**Search Strategies**:
1. **Best-First**: Greedy confidence maximization
2. **Breadth-First**: Shortest path guarantees
3. **Bidirectional**: Optimal path finding

**State Management**:
- Visited set prevents cycles
- Priority queue for best-first
- Path history for reconstruction
- Confidence propagation

### Multi-Path Aggregator (MPPA)

**Purpose**: Consensus-based answer selection.

> See [ALGORITHMS.md#multi-path-aggregation](ALGORITHMS.md#multi-path-aggregation) for the consensus algorithm details and [DESIGN.md#decision-3-multi-path-consensus](DESIGN.md#decision-3-multi-path-consensus) for design rationale.

**Architecture Pattern**: **Pipeline + Voting**

**Processing Pipeline**:
```
Multiple Paths
    ↓
Answer Clustering
    ↓
Similarity Scoring
    ↓
Consensus Voting
    ↓
Outlier Detection
    ↓
Confidence Adjustment
    ↓
Final Answer
```

### Contradiction Resolver

**Purpose**: Detect and resolve conflicting knowledge.

**Architecture Pattern**: **Observer + Strategy**

**Resolution Strategies**:
- **Recency**: Newer information wins
- **Confidence**: Higher confidence wins
- **Source Reliability**: Trustworthy sources win
- **Probabilistic**: Maintain multiple versions

**Conflict Detection**:
```
New Concept
    ↓
Semantic Analysis
    ↓
    ├──> Direct Negation Detection
    ├──> Opposite Word Detection
    └──> Quantitative Conflict Detection
    ↓
Contradiction Found?
    ↓
Resolution Strategy
    ↓
Updated Knowledge
```

---

## Data Flow

### Learning Flow

```
User Input (Text)
    ↓
[Learning Layer]
    ↓
Adaptive Learner
    ├──> Concept Creation/Update
    ├──> Strength Adjustment
    └──> Difficulty Assessment
    ↓
Association Extractor
    ├──> Pattern Matching
    ├──> Co-occurrence Analysis
    └──> Association Creation
    ↓
Contradiction Resolver
    ├──> Conflict Detection
    └──> Resolution (if needed)
    ↓
[Storage Layer]
    ↓
Persistent Storage
    ├──> Concept Record
    ├──> Association Record
    └──> Index Updates
```

### Query Flow

```
User Query
    ↓
[Reasoning Layer]
    ↓
Query Planner
    ├──> Decomposition
    └──> Execution Plan
    ↓
For Each Step:
    ↓
Query Processor
    ├──> Intent Classification
    ├──> Concept Extraction
    └──> Context Expansion
    ↓
Path Finder
    ├──> Graph Traversal
    ├──> Multiple Strategies
    └──> Path Collection
    ↓
MPPA
    ├──> Path Clustering
    ├──> Consensus Voting
    └──> Answer Selection
    ↓
Result Aggregation
    ↓
Explainable Answer
```

---

## Storage Architecture

### Temporal Log-Structured Design

**Core Concept**: Append-only immutable segments.

```
Time ──────────────────────────────────►

┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐
│Segment 1│ │Segment 2│ │Segment 3│ │Segment 4│ ← Active
│ (Old)   │ │         │ │         │ │         │
└─────────┘ └─────────┘ └─────────┘ └─────────┘
     ↓           ↓           ↓           ↓
  Compact ──→ Compact ──→ Compact    Write
```

**Benefits**:
- **Immutability**: Enables safe concurrent reads
- **Time-Travel**: Query historical states
- **Compaction**: Merge old segments for efficiency
- **Crash Recovery**: Atomic segment creation

### Memory Layout

```
┌───────────────────────────────────────────────────┐
│              Segment File (.sutra)                 │
├───────────────────────────────────────────────────┤
│  Header (256 bytes)                                │
│  - Magic: "SUTRASEG"                              │
│  - Version, Timestamps, Checksums                  │
├───────────────────────────────────────────────────┤
│  Concept Block (Fixed-size: 128 bytes each)        │
│  - Concept ID (16 bytes MD5)                       │
│  - Strength, Confidence, Access metadata           │
│  - Offsets to content and embeddings               │
├───────────────────────────────────────────────────┤
│  Association Block (Fixed-size: 64 bytes each)     │
│  - Source/Target IDs                               │
│  - Type, Weight, Confidence                        │
│  - Temporal metadata                               │
├───────────────────────────────────────────────────┤
│  Vector Block (Product Quantized)                  │
│  - Quantized embeddings (384D → 96D)              │
│  - HNSW graph structures                           │
├───────────────────────────────────────────────────┤
│  Content Block (Variable-length)                   │
│  - UTF-8 text with length prefix                   │
└───────────────────────────────────────────────────┘
```

### Indexing Strategy

**Multi-Level Indices**:

1. **Concept Map**: ID → Offset (O(1) lookup)
2. **Adjacency List**: ID → Neighbors (O(1) neighbor access)
3. **Inverted Index**: Word → Concepts (O(1) word search)
4. **Vector Index**: HNSW for semantic search (O(log n))
5. **Temporal Index**: Time → Segment (O(log n) historical queries)

---

## Scalability Strategy

### Horizontal Scaling

**Partitioning Strategies**:

1. **Temporal Partitioning**: Segments by time ranges
2. **Concept Partitioning**: Hash-based distribution
3. **Query Partitioning**: Route by topic/domain

### Vertical Scaling

**Resource Optimization**:

1. **Memory Tiering**:
   - Hot concepts in RAM
   - Warm concepts in memory-mapped files
   - Cold concepts compressed on disk

2. **Compute Allocation**:
   - Difficult concepts get more resources
   - Easy concepts use cached results
   - Dynamic priority adjustment

3. **I/O Optimization**:
   - Batch writes for efficiency
   - Prefetch for graph traversal
   - Zero-copy when possible

---

## Design Patterns

### 1. **Repository Pattern** (Storage)
- Abstract storage implementation
- Swappable backends
- Consistent interface

### 2. **Strategy Pattern** (Reasoning)
- Multiple path-finding algorithms
- Pluggable contradiction resolution
- Extensible query processing

### 3. **Observer Pattern** (Events)
- Learning event notifications
- Contradiction detection triggers
- Cache invalidation signals

### 4. **Builder Pattern** (Query Plans)
- Step-by-step plan construction
- Validation before execution
- Immutable plans once built

### 5. **Decorator Pattern** (Caching)
- Transparent query caching
- LRU eviction policy
- Optional TTL support

### 6. **Pipeline Pattern** (MPPA)
- Sequential processing stages
- Each stage transforms data
- Clear separation of concerns

---

## Evolution and Extension

### Adding New Components

**Pattern to Follow**:

1. **Define Interface**: Clear contract for new component
2. **Implement Core**: Core functionality without dependencies
3. **Wire Dependencies**: Inject required components
4. **Add Tests**: Unit and integration tests
5. **Update Documentation**: Architecture and API docs

### Maintaining Backward Compatibility

**Strategies**:

1. **Version Fields**: Track schema versions
2. **Migration Functions**: Convert old to new formats
3. **Deprecation Warnings**: Gradual phase-out
4. **Feature Flags**: Toggle new functionality

---

## Performance Considerations

### Critical Paths

1. **Query Processing**: Must be <50ms
2. **Learning**: Must be <10ms per concept
3. **Graph Traversal**: Limit depth to avoid explosion
4. **Caching**: Hit rate >70% target

### Bottleneck Prevention

1. **Index Maintenance**: Async where possible
2. **Compaction**: Background process only
3. **Lock Contention**: Lock-free data structures
4. **Memory Pressure**: Bounded cache sizes

---

## Security Considerations

### Data Protection

- Concept content sanitization
- Source validation
- Rate limiting on API
- Input validation throughout

### Access Control

- Query authorization (future)
- Learning permissions (future)
- Audit logging (future)

---

## Monitoring and Observability

### Key Metrics

1. **Performance**:
   - Query latency (p50, p95, p99)
   - Learning throughput
   - Cache hit rates

2. **Quality**:
   - Reasoning confidence scores
   - Contradiction resolution rate
   - Path diversity metrics

3. **Health**:
   - Concept count and growth
   - Association density
   - Source reliability scores

---

## Future Architecture Evolution

### Phase 2: Distributed Architecture

- Multi-node deployment
- Consensus protocols
- Distributed caching
- Load balancing

### Phase 3: Neural Integration

- Graph neural networks
- Learned embeddings
- Hybrid reasoning
- Automated association discovery

### Phase 4: Federated Learning

- Privacy-preserving updates
- Distributed knowledge graphs
- Cross-domain reasoning
- Collaborative learning

---

**For Implementation Details**: See DESIGN.md and ALGORITHMS.md  
**For Storage Specifics**: See packages/sutra-storage/ARCHITECTURE.md
