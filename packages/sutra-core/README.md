# Sutra Core

**Graph-based explainable AI reasoning engine with real-time learning.**

Version: 1.0.0 | Language: Python (42 modules) | License: MIT

---

## Overview

Sutra Core is the reasoning engine that powers explainable AI through graph-based knowledge representation. It learns in real-time without retraining and provides complete reasoning paths for every decision.

### Key Features

- **🧠 100% Explainable**: Complete reasoning paths with confidence scores
- **📈 Real-Time Learning**: Integrate knowledge instantly without model retraining
- **🎯 MPPA Consensus**: Multi-Path Plan Aggregation prevents single-path errors
- **⚡ High Performance**: 100K+ concepts with efficient graph traversal
- **🔍 3 Search Strategies**: Best-first, breadth-first, bidirectional
- **📊 Quality Gates**: Confidence calibration, "I don't know" responses
- **🌊 Progressive Streaming**: Real-time answer refinement
- **📡 Self-Observability**: Event emission for operational queries

---

## Architecture

```
┌────────────────────────────────────────────────────────┐
│                 Sutra Core Architecture                 │
├────────────────────────────────────────────────────────┤
│  ┌──────────────┐        ┌──────────────┐            │
│  │ReasoningEngine│───────▶│Storage Layer │            │
│  │(Orchestrator) │        │(Rust/TCP)    │            │
│  └──────┬───────┘        └──────────────┘            │
│         │                                              │
│    ┌────┴────┬─────────┬─────────┐                   │
│    │         │         │         │                     │
│    ▼         ▼         ▼         ▼                     │
│ ┌────┐  ┌────┐  ┌────┐  ┌─────┐                     │
│ │Path│  │Query│  │MPPA│  │Learn│                     │
│ │Find│  │Proc │  │    │  │     │                     │
│ └────┘  └────┘  └────┘  └─────┘                     │
└────────────────────────────────────────────────────────┘
```

---

## Quick Start

```python
from sutra_core import ReasoningEngine
from sutra_core.config import production_config

# Initialize engine
config = production_config()
engine = ReasoningEngine.from_config(config)

# Learn
engine.learn("The Eiffel Tower is in Paris, France")

# Query
result = engine.query("Where is the Eiffel Tower?")
print(f"Answer: {result.answer} (confidence: {result.confidence:.0%})")

# View reasoning path
for step in result.reasoning_path:
    print(f"  → {step.concept_id}")
```

---

## Core Concepts

### 1. Concepts & Associations

```python
from sutra_core import Concept, Association, AssociationType

# Concepts = nodes
concept = Concept(
    concept_id="eiffel_tower",
    content="The Eiffel Tower",
    strength=1.0,
    confidence=0.9,
)

# Associations = edges
assoc = Association(
    source_id="eiffel_tower",
    target_id="paris",
    association_type=AssociationType.HIERARCHICAL,
    confidence=0.95,
)
```

### 2. Reasoning Paths

```python
from sutra_core import ReasoningPath, ReasoningStep

path = ReasoningPath(
    steps=[
        ReasoningStep("eiffel_tower", AssociationType.HIERARCHICAL, 0.95),
        ReasoningStep("paris", AssociationType.HIERARCHICAL, 0.90),
    ],
    final_confidence=0.855,  # 0.95 * 0.90
)
```

### 3. MPPA Consensus

```python
# Find multiple paths
paths = engine.find_reasoning_paths(start, target, num_paths=3)

# Aggregate via majority voting
result = mppa.aggregate(paths, threshold=0.5)

if result.robust:
    print(f"Consensus: {result.answer}")
```

---

## API Reference

### ReasoningEngine

#### Initialization

```python
from sutra_core import ReasoningEngine
from sutra_core.config import development_config

# Recommended: Use configuration
config = development_config()
engine = ReasoningEngine.from_config(config)

# Or direct initialization
engine = ReasoningEngine(
    storage_path="./knowledge",
    use_rust_storage=True,
    enable_caching=True,
    max_cache_size=1000,
)
```

#### Learning

```python
# Simple learn
engine.learn("Paris is the capital of France")

# With metadata
engine.learn(
    "The Louvre is in Paris",
    source="Wikipedia",
    category="Geography",
)

# Batch learning
texts = ["Fact 1", "Fact 2", "Fact 3"]
for text in texts:
    engine.learn(text)
```

#### Querying

```python
# Simple query
result = engine.query("What is the capital of France?")
print(result.answer)

# Multi-path reasoning
result = engine.query(
    "What caused World War II?",
    num_paths=5,
    search_strategy="best_first",
)

# View paths
for path in result.reasoning_paths:
    print(f"Path (confidence {path.confidence}):")
    for step in path.steps:
        print(f"  → {step.concept_id}")
```

#### Streaming

```python
# Progressive refinement
for update in engine.stream_query("Tell me about AI"):
    print(f"[{update.confidence:.0%}] {update.partial_answer}")
```

### PathFinder

```python
from sutra_core.reasoning import PathFinder

finder = PathFinder(
    storage=engine.storage,
    max_depth=5,
    confidence_decay=0.85,
)

paths = finder.find_reasoning_paths(
    start_concepts=["paris"],
    target_concepts=["france"],
    num_paths=3,
    search_strategy="best_first",
)
```

**Search Strategies**:
- `best_first`: Confidence-optimized (default)
- `breadth_first`: Shortest path
- `bidirectional`: Optimal from both ends

### Quality Gates

```python
from sutra_core.quality_gates import QualityGate, QualityGateConfig

config = QualityGateConfig(
    min_confidence=0.6,
    warning_threshold=0.75,
    high_confidence=0.9,
)

gate = QualityGate(config)
gated = gate.apply(result)

if not gated.passed:
    print(f"Uncertain: {gated.reason}")
```

---

## Configuration

### Presets

```python
from sutra_core.config import (
    minimal_config,
    development_config,
    production_config,
)

engine = ReasoningEngine.from_config(production_config())
```

### Custom Configuration

```python
from sutra_core.config import ReasoningEngineConfig

config = ReasoningEngineConfig.builder() \
    .with_storage_path("./data") \
    .with_rust_storage(True) \
    .with_caching(max_size=2000) \
    .with_parallel_associations(workers=8) \
    .build()
```

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `SUTRA_STORAGE_MODE` | `local` | `local` or `server` |
| `SUTRA_STORAGE_SERVER` | `storage-server:50051` | Server address |

---

## Performance

| Operation | Latency | Notes |
|-----------|---------|-------|
| Simple query (1-2 hops) | 10-50ms | Direct traversal |
| Complex query (3-5 hops) | 50-200ms | Multi-path |
| MPPA (3 paths) | 100-500ms | Consensus |
| Learn concept | ~100/sec | With extraction |

---

## Troubleshooting

### No reasoning path found

```python
# Increase depth
engine.path_finder.max_depth = 6

# Try different strategy
result = engine.query("q", search_strategy="breadth_first")
```

### Slow queries

```python
# Enable caching
config = ReasoningEngineConfig.builder() \
    .with_caching(max_size=2000) \
    .build()

# Use Rust storage
config.use_rust_storage = True
```

---

## License

MIT License

**Built with ❤️ by the Sutra AI Team**
