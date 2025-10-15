# Sutra AI - Design Decisions & Principles

**Version**: 1.0  
**Last Updated**: October 15, 2025

---

## Table of Contents

1. [Design Philosophy](#design-philosophy)
2. [Core Design Decisions](#core-design-decisions)
3. [Temporal Dynamics](#temporal-dynamics)
4. [Knowledge Representation](#knowledge-representation)
5. [Reasoning Strategies](#reasoning-strategies)
6. [Learning Mechanisms](#learning-mechanisms)
7. [Contradiction Handling](#contradiction-handling)
8. [Performance Optimizations](#performance-optimizations)
9. [Trade-offs and Rationale](#trade-offs-and-rationale)

---

## Design Philosophy

> **Related Documentation**:  
> - [ARCHITECTURE.md](ARCHITECTURE.md) - System architecture implementing these design principles  
> - [ALGORITHMS.md](ALGORITHMS.md) - Algorithms realizing these design decisions  
> - [CONTRIBUTING.md](CONTRIBUTING.md) - Development practices aligned with these principles

### Core Tenets

1. **Glass Box over Black Box**
   - Every reasoning decision must be traceable
   - Confidence scores at every step
   - Alternative paths preserved
   - User can inspect full reasoning chain

2. **Continuous over Batch**
   - Learning happens in real-time
   - No train/test separation
   - Knowledge evolves with use
   - System never stops adapting

3. **Temporal over Static**
   - Time is first-class
   - Memory decay like biological systems
   - Versioning enables history queries
   - Recency influences decisions

4. **Consensus over Single-Path**
   - Multiple reasoning paths explored
   - Voting prevents derailment
   - Diversity bonus for varied approaches
   - Outlier detection and handling

5. **Adaptive over Fixed**
   - Resource allocation based on difficulty
   - Dynamic threshold adjustment
   - Context-sensitive processing
   - Self-optimizing structures

---

## Core Design Decisions

### Decision 1: Graph-Based Knowledge Representation

> See [ARCHITECTURE.md#graph-layer](ARCHITECTURE.md#graph-layer) for architectural implementation.

**Choice**: Explicit typed graph vs. embedding space

**We Chose**: Typed knowledge graph with explicit associations

**Rationale**:
- **Explainability**: Can trace exact reasoning path
- **Editability**: Can modify individual facts
- **Compositionality**: Clear relationship types
- **Efficiency**: Sparse storage vs. dense embeddings

**Trade-off**: Less expressive than continuous embeddings, but gains in interpretability outweigh this

**Mitigation**: Hybrid approach with optional embeddings for semantic search

### Decision 2: Continuous Learning Architecture

**Choice**: Batch training vs. online learning

**We Chose**: Pure online learning with no retraining

**Rationale**:
- **Responsiveness**: Knowledge available immediately
- **Efficiency**: No expensive retraining cycles
- **Scalability**: Linear cost per new fact
- **Simplicity**: No model versioning complexity

**Trade-off**: Can't leverage batch optimization techniques

**Mitigation**: Adaptive focus learning allocates more resources to difficult concepts

### Decision 3: Multi-Path Consensus

> See [ALGORITHMS.md#multi-path-aggregation](ALGORITHMS.md#multi-path-aggregation) for algorithm details.

**Choice**: Single best path vs. multiple path voting

**We Chose**: Generate multiple paths and use consensus

**Rationale**:
- **Robustness**: Single paths can derail easily
- **Quality**: Majority voting filters noise
- **Confidence**: Agreement indicates reliability
- **Discovery**: Different paths reveal alternatives

**Trade-off**: 3-5x more computation per query

**Mitigation**: Caching, parallel path finding, early termination

### Decision 4: Temporal Strength Dynamics

**Choice**: Static weights vs. adaptive strengthening

**We Chose**: Time-based strengthening and decay

**Rationale**:
- **Biological Plausibility**: Mimics human memory
- **Self-Organization**: Important concepts emerge
- **Pruning**: Unused knowledge fades naturally
- **Recency Bias**: Recent info weighs more

**Trade-off**: Requires careful tuning of decay rates

**Mitigation**: Health monitoring and manual decay controls

### Decision 5: Rust Storage Layer

**Choice**: Python native vs. Rust backend

**We Chose**: Rust for storage, Python for logic

**Rationale**:
- **Performance**: 10-100x speedup for I/O
- **Safety**: Memory safety without GC
- **Concurrency**: Lock-free data structures
- **Efficiency**: Zero-copy operations

**Trade-off**: Additional complexity and build dependencies

**Mitigation**: Clear FFI boundaries, PyO3 bindings, fallback Python storage

---

## Temporal Dynamics

> See [ALGORITHMS.md#temporal-dynamics-algorithms](ALGORITHMS.md#temporal-dynamics-algorithms) for implementation details.

### Concept Strengthening

**Design**: Concepts strengthen with each access

**Formula**: `strength_new = min(10.0, strength_old * 1.02)`

**Rationale**:
- **Gradual Growth**: 2% per access prevents spikes
- **Cap at 10.0**: Prevents runaway growth
- **Exponential**: Frequently accessed concepts grow faster
- **Bounded**: All concepts in [1.0, 10.0] range

**Alternatives Considered**:
- Linear growth: Too slow for frequent concepts
- Unbounded: Causes numerical instability
- Fixed increment: Doesn't model usage patterns

**Why This Works**:
- After 100 accesses: strength ≈ 7.2
- After 200 accesses: strength ≈ 10.0 (capped)
- Natural frequency-based weighting

### Concept Decay

**Design**: Unused concepts decay over time

**Formula**: `strength = strength * (decay_rate ** days_inactive)`

**Default**: decay_rate = 0.995 (0.5% daily decay)

**Rationale**:
- **Biological**: Models forgetting curves
- **Pruning**: Reduces noise over time
- **Configurable**: Can adjust per use case
- **Reversible**: Re-access re-strengthens

**Impact**:
- After 30 days unused: strength ≈ 0.86 × original
- After 90 days unused: strength ≈ 0.64 × original
- After 180 days unused: strength ≈ 0.41 × original

### Time-Based Contradictions

**Design**: Newer information can override older

**Rationale**:
- **Scientific Progress**: Old facts become outdated
- **Error Correction**: Mistakes can be fixed
- **Evolution**: Knowledge improves over time
- **Source Quality**: Better sources emerge

**Conflict Resolution**:
```
if resolve_by_recency:
    winner = newer_concept
elif resolve_by_confidence:
    winner = higher_confidence
elif resolve_by_source:
    winner = more_reliable_source
```

---

## Knowledge Representation

### Concept Structure

**Fixed Fields**:
- `id`: 16-byte hash (MD5 of content)
- `content`: UTF-8 text (variable length)
- `strength`: Float [1.0, 10.0]
- `confidence`: Float [0.0, 1.0]
- `created`: Unix timestamp
- `last_accessed`: Unix timestamp
- `access_count`: Integer
- `source`: Optional string
- `category`: Optional string

**Design Rationale**:
- **Content-Based ID**: Same content = same concept
- **Temporal Metadata**: Enables decay and analytics
- **Confidence Separate from Strength**: Strength = frequency, confidence = reliability
- **Optional Metadata**: Extensible without breaking changes

### Association Types

**Five Types**:
1. **Semantic**: Meaning-based (similar, related)
2. **Causal**: Cause-effect (A causes B)
3. **Temporal**: Time-based (A before B)
4. **Hierarchical**: Parent-child (A is-a B)
5. **Compositional**: Part-whole (A contains B)

**Why These Five**:
- Cover most knowledge relationships
- Clear semantics for reasoning
- Extractable from natural language
- Intuitive for users

**Why Not More**:
- Too many types = confusion
- Similar relationships can share types
- Can extend later if needed

**Future Consideration**: Learned association types via GNN

### Association Confidence

**Range**: [0.0, 1.0]

**Sources**:
- Pattern extraction: 0.8-0.9 (explicit phrases)
- Co-occurrence: 0.5 (word proximity)
- User-defined: 0.9 (high trust)
- Semantic similarity: Variable (0.0-1.0)

**Evolution**:
- Strengthens with use: `confidence = min(1.0, confidence + 0.02)`
- Validated by traversal
- Weak links pruned over time

---

## Reasoning Strategies

### Path Finding Strategies

#### Best-First Search

**When to Use**: Default for most queries

**Characteristics**:
- Greedy confidence maximization
- Target proximity heuristics
- Fast but not guaranteed optimal
- Good for quick answers

**Heuristic**: 
```
score = path_confidence × (1.0 + target_proximity_bonus)
```

#### Breadth-First Search

**When to Use**: Need shortest path guarantee

**Characteristics**:
- Level-by-level exploration
- Guaranteed shortest path
- No heuristics needed
- More comprehensive

**Use Case**: Relationship distance queries

#### Bidirectional Search

**When to Use**: Clear start and end points

**Characteristics**:
- Search from both ends
- Meet in the middle
- Optimal for known targets
- Efficient for distant concepts

**Use Case**: "How is A related to B?"

### Multi-Path Aggregation

**Clustering Algorithm**:

1. **Group by answer similarity** (threshold: 0.8)
2. **Select representative** (highest individual confidence)
3. **Calculate cluster confidence** (weighted average)
4. **Apply consensus bonus** (majority agreement boost)
5. **Penalize outliers** (singleton paths: -30%)
6. **Add diversity bonus** (varied paths: +20%)

**Consensus Threshold**: 50% (majority)

**Why This Works**:
- Multiple paths catch errors
- Clustering handles equivalent answers
- Outliers filtered naturally
- Diversity rewarded

### Query Decomposition

**When to Decompose**:
- Conjunction/disjunction detected
- Causal chain identified
- Comparison requested
- Temporal sequence present
- Hypothetical reasoning needed

**Decomposition Process**:
1. Pattern match query structure
2. Extract sub-questions
3. Build dependency graph
4. Topological sort for order
5. Execute sequentially or parallel

**Example**:
```
"Why is photosynthesis important?"
    ↓
1. "What is photosynthesis?" (factual)
2. "What does photosynthesis produce?" (relational)
3. "Why is that important?" (causal, depends on 2)
```

---

## Learning Mechanisms

### Adaptive Focus Learning

**Core Principle**: Allocate resources based on difficulty

**Difficulty Assessment**:
- Difficult: strength < 4.0
- Moderate: strength 4.0-7.0
- Easy: strength > 7.0

**Resource Allocation**:

| Difficulty | Reinforcement | Extraction Depth | Rationale |
|-----------|---------------|------------------|-----------|
| Difficult | 1.15× | 2 (deep) | Need more help |
| Moderate | 1.02× | 1 (normal) | Standard growth |
| Easy | 1.01× | 1 (normal) | Minimal effort |

**Inspiration**: AdaKD (Loss-Driven Adaptive Token Focusing)

**Key Difference**: We use strength as proxy for difficulty, not loss

### Association Extraction

**Two-Stage Process**:

**Stage 1: Pattern Matching**
- Regex patterns for explicit relationships
- High confidence (0.8-0.9)
- Fast and precise
- Language-specific

**Stage 2: Co-occurrence (Adaptive)**
- Sliding window (3 words)
- Lower confidence (0.5)
- Only for difficult concepts
- Discovers implicit links

**Why Two Stages**:
- Pattern matching catches explicit facts
- Co-occurrence adds context
- Adaptive depth saves compute
- Balanced precision/recall

### Central Link Strategy

**Design**: Link learned concept to extracted phrases

**Configuration**:
- Enable by default
- Confidence: 0.6 (moderate)
- Type: Compositional (whole-part)

**Purpose**:
- Tie content to components
- Enable better traversal
- Provide context
- Aid in reasoning

**Example**:
```
"Photosynthesis converts sunlight to energy"
    ↓
Central: "Photosynthesis converts..."
    ├─> "photosynthesis" (compositional, 0.6)
    ├─> "sunlight" (compositional, 0.6)
    └─> "energy" (compositional, 0.6)
```

---

## Contradiction Handling

### Detection Strategy

**Three Types of Contradictions**:

1. **Direct Negation** (confidence: 0.9)
   - One statement has negation keywords
   - Otherwise similar text
   - Example: "X is Y" vs "X is not Y"

2. **Semantic Opposites** (confidence: 0.7)
   - Opposite word pairs detected
   - Example: "hot water" vs "cold water"

3. **Quantitative Conflict** (confidence: 0.8)
   - Same structure, different numbers
   - Example: "Earth is 100M years old" vs "Earth is 4.5B years old"

### Resolution Strategies

**Five Strategies Available**:

1. **Recency**: Prefer newer information
   - Default for fast-changing domains
   - Assumes progress over time
   - Simple to implement

2. **Confidence**: Prefer higher confidence
   - Default general strategy
   - Trusts more certain sources
   - Works for most cases

3. **Source Reliability**: Prefer trusted sources
   - Learns over time
   - Tracks source quality
   - Best for multi-source knowledge

4. **Consensus**: Majority wins
   - Multiple sources voting
   - Democratic resolution
   - Requires multiple instances

5. **Probabilistic**: Keep both
   - No resolution
   - Maintain uncertainty
   - For genuine ambiguity

### Source Reliability Learning

**Scoring Formula**:
```
reliability = 0.5 + (correct - incorrect - 0.5×contradicted) / total
```

**Bounded**: [0.0, 1.0]

**Updates**:
- Correct answer: +1
- Incorrect answer: -1
- Contradicted: -0.5

**Bootstrap**: Start at 0.5 (neutral)

---

## Performance Optimizations

### Query Caching

**Strategy**: LRU with optional TTL

**Design**:
- OrderedDict for LRU
- Timestamp for TTL
- Move-to-end for access
- Size-bounded eviction

**Parameters**:
- Max size: 1000 (default)
- TTL: None (default, no expiry)
- Configurable per instance

**Impact**: 8.5x speedup on repeated queries

### Neighbor Indexing

**Structure**: Dictionary of sets

**Update Strategy**:
- Bidirectional on creation
- Symmetric after load
- O(1) neighbor lookup
- Async rebuild option

**Why This Matters**:
- Graph traversal is core operation
- Adjacency list is essential
- Hash map gives O(1) access
- Sets prevent duplicates

### Word-to-Concept Index

**Purpose**: Fast concept lookup by keywords

**Structure**: 
```
word → set of concept IDs
```

**Maintenance**:
- Built during learning
- Updated on new concepts
- Rebuilt on load
- Case-insensitive

**Impact**: O(1) concept finding vs O(n) scan

### Path Caching

**Future Enhancement**: Cache common paths

**Design**:
```
(start_id, target_id, strategy) → cached_paths
```

**Invalidation**: On learning events affecting path

**Benefit**: Avoid re-traversal of stable subgraphs

---

## Trade-offs and Rationale

### Python vs. Rust Split

**Trade-off**:
- **Pro Python**: Rapid iteration, easy debugging
- **Pro Rust**: Performance, safety, concurrency
- **Decision**: Python for logic, Rust for storage

**Rationale**: Best of both worlds
- Hot path (storage) in Rust
- Cold path (reasoning) in Python
- Clear interface boundary
- Independent evolution

### Memory vs. Computation

**Trade-off**: Store more vs. compute more

**Decisions**:
- **Cache results**: Memory for speed
- **Index structures**: Memory for O(1) lookup
- **Embeddings**: Optional (memory/quality balance)
- **Path storage**: Limited to top-k

**Rationale**: Memory is cheap, user time is not

### Accuracy vs. Speed

**Trade-off**: More thorough vs. faster response

**Tuning Knobs**:
- Number of reasoning paths (default: 5)
- Max traversal depth (default: 6)
- Cache size (default: 1000)
- Extraction depth (adaptive: 1-2)

**Default Philosophy**: Favor speed with quality guardrails
- Caching for common queries
- Adaptive depth for learning
- Bounded exploration
- Early termination on consensus

### Explainability vs. Compactness

**Trade-off**: Full traces vs. minimal storage

**Decision**: Store full reasoning paths

**Rationale**: Explainability is core value
- Space cost is acceptable
- Compression if needed
- Debugging benefit
- Trust building

---

## Future Design Considerations

### Phase 2: Neural Integration

**Challenge**: Integrate learned representations

**Design Direction**:
- GNN for concept embeddings
- Relation type learning
- Path quality prediction
- Keep explainability

### Phase 3: Distributed System

**Challenge**: Scale beyond single machine

**Design Direction**:
- Graph partitioning strategies
- Consensus protocols
- Distributed caching
- Federated learning

### Phase 4: Multi-Modal

**Challenge**: Beyond text

**Design Direction**:
- Image embeddings as concepts
- Multi-modal associations
- Cross-modal reasoning
- Unified representation

---

## Design Principles Summary

1. **Temporal First**: Time is not an afterthought
2. **Explainable Always**: Every decision traceable
3. **Continuous Learning**: Never stops adapting
4. **Consensus Over Single**: Multiple paths better than one
5. **Adaptive Resources**: More effort for harder problems
6. **Graph Native**: Relationships are explicit
7. **Contradiction Aware**: Conflicts detected and resolved
8. **Performance Conscious**: Fast enough for interactive use
9. **Extensible Design**: Components can evolve independently
10. **Data-Driven**: Metrics guide optimization

---

**Related Documents**:
- ARCHITECTURE.md - System structure
- ALGORITHMS.md - Core algorithms
- packages/sutra-storage/ARCHITECTURE.md - Storage details
