# Sutra AI Quick Reference

**Version 2.0 - Production Features**

---

## Deployment

```bash
# One command
./sutra-deploy.sh up

# Status check
./sutra-deploy.sh status

# View logs
./sutra-deploy.sh logs [service_name]

# Stop all
./sutra-deploy.sh down
```

**Services:**
- Control Center: http://localhost:9000
- API: http://localhost:8000
- Hybrid (Streaming): http://localhost:8001
- Client UI: http://localhost:8080

---

## Python Quick Start

### Basic Usage

```python
from sutra_core import ReasoningEngine

# Initialize (auto-enables events + quality gates)
engine = ReasoningEngine()

# Learn
engine.learn("Machine learning is a subset of AI")

# Query
result = engine.ask("What is machine learning?")
print(f"{result.primary_answer} (confidence: {result.confidence:.2f})")
```

### Streaming Responses

```python
import asyncio
from sutra_core.streaming import create_async_engine

async def main():
    engine = ReasoningEngine()
    async_engine = create_async_engine(engine)
    
    async for chunk in async_engine.ask_stream("What is AI?"):
        print(f"[{chunk.stage}] {chunk.answer} ({chunk.confidence:.2f})")

asyncio.run(main())
```

### Observability Queries

```python
from sutra_core.observability_query import create_observability_interface

obs = create_observability_interface(engine.storage)

# Performance
obs.query("Show me slow queries in the last hour")
obs.query("What's the average latency today?")

# Errors
obs.query("Show me failed queries")
obs.query("What errors occurred today?")

# Confidence
obs.query("Show me low confidence queries")
obs.query("How many queries failed quality gates?")
```

### Quality Gates

```python
from sutra_core.quality_gates import create_quality_validator, MODERATE_QUALITY_GATE

validator = create_quality_validator(engine.storage, MODERATE_QUALITY_GATE)

result = engine.ask("What is quantum computing?")
assessment = validator.validate(
    raw_confidence=result.confidence,
    consensus_strength=result.consensus_strength,
    num_paths=len(result.supporting_paths),
    has_evidence=bool(result.supporting_paths)
)

if not assessment.passed:
    print(f"❌ {assessment.recommendation}")  # "I don't know"
```

---

## REST API Quick Start

### Standard Query

```bash
curl -X POST http://localhost:8001/sutra/query \
  -H "Content-Type: application/json" \
  -d '{
    "query": "What is AI?",
    "semantic_boost": true,
    "max_paths": 5
  }'
```

**Response:**
```json
{
  "answer": "Artificial intelligence is...",
  "confidence": 0.85,
  "reasoning_paths": [...],
  "explanation": "..."
}
```

### Streaming Query (SSE)

```bash
curl -X POST http://localhost:8001/sutra/stream/query \
  -H "Content-Type: application/json" \
  -d '{
    "query": "What is AI?",
    "max_concepts": 10,
    "enable_quality_gates": true
  }'
```

**Response:** Server-Sent Events
```
event: chunk
data: {"stage": "initial", "answer": "...", "confidence": 0.6}

event: chunk
data: {"stage": "refining", "answer": "...", "confidence": 0.75}

event: chunk
data: {"stage": "complete", "answer": "...", "confidence": 0.85, "is_final": true}
```

### Learn

```bash
curl -X POST http://localhost:8001/sutra/learn \
  -H "Content-Type: application/json" \
  -d '{
    "text": "Machine learning is a subset of AI",
    "source": "training_data",
    "category": "ai_concepts"
  }'
```

### Health Check

```bash
curl http://localhost:8001/sutra/health
```

---

## JavaScript/React/Vue

### Browser/Node.js

```javascript
import { streamQuery } from './examples/streaming_client';

await streamQuery('What is AI?', {
    onChunk: (chunk) => {
        console.log(`${chunk.stage}: ${chunk.answer}`);
    },
    onComplete: () => console.log('Done!')
});
```

### React Hook

```jsx
import { useStreamingQuery } from './examples/streaming_client';

function ChatBot() {
    const { answer, confidence, stage, stream } = useStreamingQuery();
    
    return (
        <div>
            <button onClick={() => stream("What is AI?")}>Ask</button>
            <div>{stage}: {answer} ({confidence}%)</div>
        </div>
    );
}
```

### Vue Composable

```vue
<script setup>
import { useStreamingQuery } from './examples/streaming_client';

const { answer, confidence, stage, stream } = useStreamingQuery();
</script>

<template>
  <div>
    <button @click="stream('What is AI?')">Ask</button>
    <div>{{ stage }}: {{ answer }} ({{ confidence }}%)</div>
  </div>
</template>
```

---

## Configuration Presets

### Quality Gates

```python
from sutra_core.quality_gates import (
    STRICT_QUALITY_GATE,      # Medical/legal
    MODERATE_QUALITY_GATE,    # General chatbot
    LENIENT_QUALITY_GATE      # Exploratory
)
```

| Preset | Min Confidence | Min Consensus | Min Paths | Require Evidence |
|--------|---------------|---------------|-----------|------------------|
| STRICT | 0.5 | 0.6 | 2 | Yes |
| MODERATE | 0.3 | 0.5 | 1 | Yes |
| LENIENT | 0.2 | 0.3 | 1 | No |

### Custom Quality Gate

```python
from sutra_core.quality_gates import QualityGate

CUSTOM_GATE = QualityGate(
    min_confidence=0.7,
    min_consensus=0.8,
    min_paths=2,
    require_evidence=True
)
```

### Streaming Configuration

```python
from sutra_core.streaming import StreamingQueryProcessor

processor = StreamingQueryProcessor(
    query_processor=engine.query_processor,
    path_finder=engine.path_finder,
    mppa=engine.mppa,
    storage=engine.storage,
    target_paths=5,              # Total paths to find
    min_paths_for_refinement=2,  # When to emit "refining" chunk
)
```

---

## Common Observability Queries

### Performance Monitoring

```python
obs.query("Show me slow queries in the last hour")
obs.query("What's the average response time today?")
obs.query("Show me queries > 500ms")
```

### Error Analysis

```python
obs.query("Show me failed queries today")
obs.query("What errors occurred in the last hour?")
obs.query("Show me storage errors")
```

### Confidence Analysis

```python
obs.query("Show me low confidence queries")
obs.query("How many queries failed quality gates?")
obs.query("What knowledge gaps do we have?")
```

### System Health

```python
obs.query("How many events in the last hour?")
obs.query("Show me system errors")
obs.query("What's the system status?")
```

---

## Environment Variables

### Storage

```bash
export SUTRA_STORAGE_SERVER="storage-server:50051"
export SUTRA_STORAGE_MODE="server"  # or "local"
```

### API Ports

```bash
export SUTRA_API_PORT="8000"
export SUTRA_HYBRID_PORT="8001"
export SUTRA_CONTROL_PORT="9000"
```

### Embeddings

```bash
export SUTRA_OLLAMA_URL="http://ollama:11434"
export OLLAMA_MODEL="granite-embedding:30m"
```

### NLG

```bash
export SUTRA_NLG_ENABLED="true"
export SUTRA_NLG_TONE="friendly"  # friendly|formal|concise|regulatory
```

### Rate Limits

```bash
export SUTRA_RATE_LIMIT_LEARN="30"
export SUTRA_RATE_LIMIT_REASON="60"
```

---

## Troubleshooting

### Events Not Appearing

```python
# Check if event emitter is enabled
print(engine._event_emitter.enabled)  # Should be True

# Query recent events
obs.query("Show me events in the last minute")
```

### Quality Gates Too Strict

```python
# Use lenient gate
from sutra_core.quality_gates import LENIENT_QUALITY_GATE
validator = create_quality_validator(engine.storage, LENIENT_QUALITY_GATE)
```

### Streaming Slow

```python
# Reduce target paths
processor = StreamingQueryProcessor(
    target_paths=3,  # Default is 5
    min_paths_for_refinement=1
)
```

### High Latency

```python
# Check specific components
obs.query("Show me slow storage operations")
obs.query("Show me slow embedding generation")
obs.query("Show me path finding operations > 500ms")
```

---

## Performance Benchmarks

### Storage (Rust)

- **Writes:** 57,412 concepts/sec (0.02ms per concept)
- **Reads:** <0.01ms (zero-copy memory-mapped)
- **Path finding:** ~1ms for 3-hop BFS
- **Durability:** Zero data loss (Write-Ahead Log)

### Streaming Stages

- **Initial:** 60ms (first path found)
- **Refining:** 160ms (accumulating paths 2-4)
- **Consensus:** 360ms (building multi-path consensus)
- **Complete:** 500ms (final answer + quality check)

### Comparison

- Non-streaming: 500-1000ms to first byte
- Streaming: 60ms to first byte (**10x faster perceived performance**)

---

## Event Types

| Category | Event Types | Use Case |
|----------|-------------|----------|
| **Query** | received, completed, failed, low_confidence, high_latency | Query performance tracking |
| **Learning** | received, completed, failed, batch operations | Knowledge ingestion monitoring |
| **Storage** | read, write, errors, slow | Storage performance |
| **Embedding** | generated, failed, slow | Embedding service health |
| **Pathfinding** | started, completed, no_results, failed | Reasoning performance |
| **NLG** | generated, failed | NLG service health |
| **System** | healthy, degraded, error | Overall health |

---

## Documentation Index

- **[Production Guide](PRODUCTION_GUIDE.md)** - Complete production documentation
- **[Streaming Guide](STREAMING.md)** - Detailed streaming API and examples
- **[Production Enhancements](PRODUCTION_ENHANCEMENTS.md)** - Self-observability, quality gates, NL queries
- **[Deployment Guide](./deployment/DEPLOYMENT.md)** - Docker, Kubernetes, and manual deployment
- **[Architecture](../WARP.md)** - Complete system architecture and development guide

---

**Status:** Production-Ready ✅  
**Version:** 2.0.0  
**Last Updated:** 2025-10-19
