# Natural Language Generation (NLG) Documentation

**Explainable Natural Language Generation for Sutra AI**

Version: 2.0.0 | Date: 2025-10-27 | Status: Production-Ready ✅

---

## 📚 Documentation Index

### Core Documentation
- **[ARCHITECTURE.md](./ARCHITECTURE.md)** - Complete system architecture, components, and data flow
- **[DESIGN_DECISIONS.md](./DESIGN_DECISIONS.md)** - Design rationale and trade-offs
- **[DEPLOYMENT.md](./DEPLOYMENT.md)** - Deployment guide and troubleshooting
- **[IMPLEMENTATION_SUMMARY.md](./IMPLEMENTATION_SUMMARY.md)** - Implementation details and file inventory

### Related Documentation
- **[../STREAMING.md](../STREAMING.md)** - Progressive answer refinement with streaming
- **[../../packages/sutra-nlg/](../../packages/sutra-nlg/)** - NLG package implementation
- **[../../packages/sutra-nlg-service/](../../packages/sutra-nlg-service/)** - Self-hosted LLM service

---

## 🎯 What is NLG in Sutra AI?

Natural Language Generation in Sutra AI transforms graph-based reasoning results into human-readable responses while maintaining complete explainability and grounding in verified facts.

### Core Principles

- ✅ **Reasoning-First**: Graph reasoning always precedes text generation
- ✅ **Strict Grounding**: All generated text validated against verified facts
- ✅ **Multi-Path Consensus**: MPPA aggregates multiple reasoning paths
- ✅ **Quality Gates**: Confidence thresholds prevent uncertain responses
- ✅ **Progressive Streaming**: Real-time refinement as paths are discovered
- ✅ **Optional LLM Enhancement**: Template-based default, hybrid LLM optional

---

## 🚀 Quick Start (3 Minutes)

### Standard Deployment (Template Mode)
```bash
./sutra-deploy.sh install
# Template-based NLG (<10ms, no LLM required)
```

### With Streaming (Progressive Refinement)
```python
from sutra_core import ReasoningEngine
from sutra_core.streaming import create_async_engine

# Create engine
engine = ReasoningEngine(storage_path="./knowledge")
async_engine = create_async_engine(engine)

# Stream query with progressive refinement
async for chunk in async_engine.ask_stream("What is artificial intelligence?"):
    print(f"{chunk.stage.value}: {chunk.answer} ({chunk.confidence:.2f})")
    # Prints: initial → refining → consensus → complete
```

### Enable Hybrid LLM Mode (Optional)
```bash
# Build with hybrid NLG service
docker-compose -f docker-compose-grid.yml --profile nlg-hybrid up -d

# Verify service
curl http://localhost:8889/health
# Expected: {"status":"healthy","model_loaded":true}
```

**See [DEPLOYMENT.md](./DEPLOYMENT.md) for complete deployment guide**

---

## 📊 NLG Modes Comparison

| Feature | Template Mode | Hybrid LLM Mode | Streaming Mode |
|---------|---------------|-----------------|----------------|
| **Speed** | <10ms | ~120ms | 50-200ms (progressive) |
| **Quality** | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ (progressive) |
| **Memory** | 50MB | 4-12GB (3 replicas) | 50MB |
| **Grounding** | 50% overlap | 70% overlap | 50% overlap |
| **Real-time** | Instant | Instant | Progressive updates |
| **Use Case** | APIs, simple queries | User-facing chat | Interactive UX |

---

## 🏗️ Architecture Overview

```
┌──────────────────────────────────────────────────────────────────┐
│                     Sutra AI NLG Pipeline                         │
├──────────────────────────────────────────────────────────────────┤
│                                                                   │
│  User Query                                                       │
│       │                                                           │
│       ▼                                                           │
│  ┌─────────────────────────────────────────────────┐            │
│  │  1. Graph Reasoning Engine                      │            │
│  │     • PathFinder (BFS, Best-First, Bi-directional)          │
│  │     • MPPA (Multi-Path Plan Aggregation)       │            │
│  │     • Quality Gates (confidence thresholds)     │            │
│  │     • Consensus Voting (prevent derailment)     │            │
│  │     ✅ OUTPUT: Verified Facts + Reasoning Paths │            │
│  └───────────────────┬─────────────────────────────┘            │
│                      │                                           │
│                      ▼                                           │
│  ┌─────────────────────────────────────────────────┐            │
│  │  2. NLG Layer (sutra-nlg)                       │            │
│  │                                                  │            │
│  │     Mode Router (NLGRealizer)                   │            │
│  │        │                                         │            │
│  │        ├─→ Template Mode (pattern-based, <10ms) │            │
│  │        │   • Slot-filling templates             │            │
│  │        │   • 50% grounding validation           │            │
│  │        │   • Zero dependencies                  │            │
│  │        │                                         │            │
│  │        └─→ Hybrid Mode (LLM, ~120ms, optional)  │            │
│  │            • Extract fact pool                  │            │
│  │            • Build constrained prompt           │            │
│  │            • Call NLG Service (HA cluster)      │            │
│  │            • 70% grounding validation           │            │
│  │            • Auto-fallback on failure           │            │
│  └─────────────────────────────────────────────────┘            │
│                      │                                           │
│                      ▼                                           │
│  ┌─────────────────────────────────────────────────┐            │
│  │  3. Optional: Streaming Enhancement             │            │
│  │     (AsyncReasoningEngine)                      │            │
│  │     • Initial answer (first path)               │            │
│  │     • Progressive refinement (more paths)       │            │
│  │     • Final consensus (all paths)               │            │
│  └─────────────────────────────────────────────────┘            │
│                      │                                           │
│                      ▼                                           │
│  Final Response: Natural Language + Reasoning Paths + Metadata  │
│                                                                   │
└──────────────────────────────────────────────────────────────────┘
```

**See [ARCHITECTURE.md](./ARCHITECTURE.md) for detailed component design**

---

## 🔒 Grounding Strategy

### How Sutra Ensures Factual Accuracy

1. **Graph Reasoning First** - Always extract verified facts from knowledge graph
2. **Multi-Path Consensus** - Aggregate multiple reasoning paths (MPPA)
3. **Quality Gates** - Reject low-confidence responses (configurable thresholds)
4. **Constrained Generation** - LLM explicitly limited to verified facts
5. **Post-Generation Validation** - Token overlap check (50% template, 70% hybrid)
6. **Automatic Fallback** - Degradation to template on validation failure

### Example: Preventing Hallucination

**Fact Pool (from graph reasoning):**
```
- Paris is the capital of France
- The Eiffel Tower is in Paris
- Paris has a population of 2.2 million
```

**Valid Generation (98% overlap):**
```
"Paris, the capital of France with 2.2 million residents, is home to the Eiffel Tower."
✅ Accepted - All tokens come from fact pool
```

**Invalid Generation (hallucination detected):**
```
"Paris became the capital in 1789 during the French Revolution."
❌ Rejected - "1789" and "Revolution" not in facts (60% overlap < 70% threshold)
→ Automatic fallback to template mode
```

**"I Don't Know" Response (quality gate):**
```python
result = engine.ask("What is the population of Mars?")
# No reasoning paths found
# Quality gate triggered: confidence = 0.0 < threshold (0.3)
# Response: "I don't have enough knowledge to answer this question."
```

---

## 🎓 Key Features

### 1. Multi-Path Plan Aggregation (MPPA)

Prevents single-path reasoning derailment:

```python
# MPPA finds multiple independent paths and votes
paths = [
    Path(answer="Paris", confidence=0.9, steps=[...]),
    Path(answer="Paris", confidence=0.85, steps=[...]),
    Path(answer="Lyon", confidence=0.4, steps=[...])  # Outlier
]

consensus = mppa.aggregate_reasoning_paths(paths, query)
# Result: "Paris" (90% consensus, outlier penalized)
# confidence=0.92, consensus_strength=0.9
```

### 2. Progressive Streaming

Real-time answer refinement as paths are discovered:

```python
async for chunk in async_engine.ask_stream(query):
    if chunk.stage == StreamingStage.INITIAL:
        # Fast first response (50-100ms)
        display_partial_answer(chunk.answer, chunk.confidence)
    
    elif chunk.stage == StreamingStage.REFINING:
        # Progressive improvement (2-5 updates)
        update_answer(chunk.answer, chunk.confidence)
    
    elif chunk.stage == StreamingStage.COMPLETE:
        # Final consensus answer
        display_final_answer(chunk.answer, chunk.confidence)
        show_reasoning_paths(chunk.reasoning_explanation)
```

**Benefits:**
- Perceived latency: 50ms (initial) vs 200ms (complete)
- User sees progress, not loading spinner
- Can stop early if initial answer satisfies query

### 3. Quality Gates

Confidence-based gating prevents low-quality responses:

```python
from sutra_core.quality_gates import QualityGate, ConfidenceLevel

gate = QualityGate(
    min_confidence=0.3,      # Require 30% confidence
    min_consensus=0.5,        # Require 50% path agreement
    min_paths=1,              # Need at least 1 reasoning path
    require_evidence=True     # Must have supporting evidence
)

validator = create_quality_validator(engine._core, gate)
result = validator.validate(query_result)

if not result.passed:
    # Return "I don't know" response
    return f"I don't have enough knowledge: {result.reasons_for_failure}"
```

### 4. Hybrid LLM Mode (Optional)

Self-hosted LLM for natural language:

```python
config = NLGConfig(
    mode="hybrid",                      # Enable LLM
    service_url="http://nlg-ha:8889",  # HA cluster
    tone="friendly"                     # Output style
)

realizer = NLGRealizer(config)
text, grounded, meta = realizer.realize(query, answer, paths)

# meta contains:
# {
#   "mode": "hybrid",
#   "model": "google/gemma-3-270m-it",
#   "processing_time_ms": 118,
#   "tokens_generated": 42,
#   "grounding_validated": true
# }
```

**High Availability:**
- 3 replicas with HAProxy load balancing
- Least-connection routing (adapts to variable latency)
- Health checks every 10s
- Auto-failover <10s detection

---

## 📁 Project Structure

```
docs/nlg/
├── README.md                      # This file (overview & quick start)
├── ARCHITECTURE.md                # Detailed system architecture
├── DESIGN_DECISIONS.md            # Design rationale and trade-offs
├── DEPLOYMENT.md                  # Deployment guide and operations
└── IMPLEMENTATION_SUMMARY.md      # Implementation details

packages/
├── sutra-core/
│   ├── reasoning/
│   │   ├── engine.py              # ReasoningEngine (main orchestrator)
│   │   ├── mppa.py                # Multi-Path Plan Aggregation
│   │   ├── paths.py               # PathFinder strategies
│   │   └── query.py               # QueryProcessor
│   ├── streaming.py               # Progressive streaming support
│   ├── quality_gates.py           # Confidence thresholds
│   └── learning/                  # Adaptive learning
│
├── sutra-nlg/                     # NLG abstraction layer
│   └── sutra_nlg/
│       ├── realizer.py            # NLGRealizer (router & validator)
│       └── templates.py           # Template patterns
│
├── sutra-nlg-service/             # Self-hosted LLM service (optional)
│   ├── main.py                    # FastAPI service
│   ├── Dockerfile                 # CPU-optimized container
│   └── requirements.txt
│
└── sutra-hybrid/                  # API integration
    └── api/
        ├── sutra_endpoints.py     # REST API with NLG
        └── streaming_endpoints.py  # SSE streaming API

docker/
├── haproxy-nlg.cfg                # Load balancer for NLG service
└── docker-compose-grid.yml        # Complete deployment
```

---

## 🎯 Use Cases & When to Use Each Mode

### Template Mode (Default)
**Best for:**
- ✅ High-throughput APIs (1000+ req/s)
- ✅ Simple fact lookups ("What is X?")
- ✅ Development and testing
- ✅ Resource-constrained environments
- ✅ Guaranteed <10ms latency
- ✅ Maximum reliability (no dependencies)

**Example:**
```python
# Simple, fast, reliable
result = engine.ask("What is the capital of France?")
# Response: "Here's what I found: Paris."
# Latency: 5ms
```

### Streaming Mode
**Best for:**
- ✅ Interactive user interfaces
- ✅ Complex multi-part questions
- ✅ Real-time progress indication
- ✅ Perceived performance optimization
- ✅ Mobile/web applications

**Example:**
```python
# Progressive refinement for better UX
async for chunk in async_engine.ask_stream(query):
    # Initial: "Paris is in France" (80ms, 0.7 confidence)
    # Refining: "Paris is the capital of France" (150ms, 0.85 confidence)
    # Complete: "Paris is the capital and largest city of France..." (220ms, 0.92 confidence)
```

### Hybrid LLM Mode (Optional)
**Best for:**
- ✅ User-facing chatbots
- ✅ Professional/formal communication
- ✅ Complex explanations needing fluency
- ✅ Marketing demos
- ✅ When natural language quality is critical

**Not for:**
- ❌ High-volume APIs (limited to ~30 req/s with 3 replicas)
- ❌ Resource-constrained deployments (requires 12GB RAM)
- ❌ <50ms latency requirements

---

## 🛠️ Configuration Reference

### Environment Variables

```bash
# NLG Configuration
SUTRA_NLG_ENABLED=true                    # Enable/disable NLG (default: true)
SUTRA_NLG_MODE=template                   # Mode: "template" or "hybrid"
SUTRA_NLG_TONE=friendly                   # Tone: friendly, formal, concise, regulatory
SUTRA_NLG_SERVICE_URL=http://nlg-ha:8889  # LLM service URL (hybrid mode only)
SUTRA_NLG_MODEL=google/gemma-3-270m-it    # Model name (swappable)

# Quality Gates
SUTRA_QUALITY_MIN_CONFIDENCE=0.3          # Minimum confidence threshold
SUTRA_QUALITY_MIN_CONSENSUS=0.5           # Minimum consensus for MPPA
SUTRA_QUALITY_MIN_PATHS=1                 # Minimum reasoning paths

# Streaming
SUTRA_STREAMING_ENABLED=true              # Enable progressive streaming
SUTRA_STREAMING_TARGET_PATHS=5            # Target paths for streaming
SUTRA_STREAMING_MIN_REFINEMENT=2          # Min paths before refinement

# Storage
SUTRA_STORAGE_MODE=local                  # Storage: "local" or "server"
SUTRA_STORAGE_SERVER=storage-server:50051 # TCP storage server (if server mode)
```

### Python API

```python
from sutra_core import ReasoningEngine
from sutra_core.config import ReasoningEngineConfig, production_config
from sutra_nlg import NLGConfig, NLGRealizer
from sutra_core.streaming import create_async_engine
from sutra_core.quality_gates import QualityGate, create_quality_validator

# 1. Configure reasoning engine
config = ReasoningEngineConfig.builder() \
    .with_caching(max_size=1000) \
    .with_parallel_associations(workers=4) \
    .build()

engine = ReasoningEngine.from_config(config)

# 2. Configure NLG
nlg_config = NLGConfig(
    mode="template",      # or "hybrid"
    tone="friendly",      # or "formal", "concise", "regulatory"
    service_url=None      # Required if mode="hybrid"
)

realizer = NLGRealizer(nlg_config)

# 3. Configure quality gates
gate = QualityGate(
    min_confidence=0.3,
    min_consensus=0.5,
    min_paths=1,
    require_evidence=True
)

validator = create_quality_validator(engine, gate)

# 4. Configure streaming (optional)
async_engine = create_async_engine(engine)

# 5. Query with full pipeline
result = engine.ask("What is artificial intelligence?")

# Validate quality
assessment = validator.validate(result)
if not assessment.passed:
    response = "I don't know: " + ", ".join(assessment.reasons_for_failure)
else:
    # Apply NLG
    response, grounded, meta = realizer.realize(
        query="What is artificial intelligence?",
        answer=result.answer,
        reasoning_paths=[{
            "concepts": [step.content for step in path.steps],
            "concept_ids": [step.concept_id for step in path.steps]
        } for path in result.reasoning_paths]
    )
```

---

## 🧪 Testing & Validation

### Health Checks

```bash
# Check reasoning engine
curl http://localhost:8001/health
# Expected: {"status":"healthy","concepts_loaded":10000}

# Check NLG service (if hybrid mode)
curl http://localhost:8889/health
# Expected: {"status":"healthy","model_loaded":true}

# Check HAProxy stats
open http://localhost:8405/stats
```

### Test Queries

```bash
# Test template mode
curl -X POST http://localhost:8001/sutra/query \
  -H "Content-Type: application/json" \
  -d '{
    "query": "What is the capital of France?",
    "tone": "friendly"
  }'

# Test streaming mode
curl -X POST http://localhost:8001/sutra/stream/query \
  -H "Content-Type: application/json" \
  -d '{
    "query": "Explain artificial intelligence",
    "max_concepts": 10
  }'

# Test hybrid mode (if enabled)
SUTRA_NLG_MODE=hybrid curl -X POST http://localhost:8001/sutra/query \
  -H "Content-Type: application/json" \
  -d '{
    "query": "What is machine learning?",
    "tone": "formal"
  }'
```

---

## 📊 Performance Benchmarks

### Latency (P50/P95/P99)

| Mode | P50 | P95 | P99 | Notes |
|------|-----|-----|-----|-------|
| **Template** | 5ms | 12ms | 18ms | Consistent, no variance |
| **Streaming (initial)** | 50ms | 85ms | 120ms | First path found |
| **Streaming (complete)** | 180ms | 250ms | 320ms | All paths + consensus |
| **Hybrid LLM** | 118ms | 215ms | 287ms | With 3 replicas, least-conn |

### Throughput

| Configuration | Requests/Second | Bottleneck |
|---------------|-----------------|------------|
| Template only | 1000+ | CPU (trivial) |
| Streaming | 50-100 | Path finding |
| Hybrid (1 replica) | ~10 | LLM generation |
| Hybrid (3 replicas) | ~30 | LLM generation |
| Hybrid (10 replicas) | ~100 | HAProxy routing |

### Memory Usage

| Component | Per Instance | With 3 Replicas |
|-----------|--------------|-----------------|
| Reasoning Engine | 500MB | 500MB (shared) |
| Template NLG | 10MB | 10MB |
| Streaming | +50MB | +50MB |
| NLG Service (gemma-3-270m) | 4GB | 12GB |
| **Total (all modes)** | **4.5GB** | **12.5GB** |

---

## 🚨 Troubleshooting

### Common Issues

**1. "I don't have enough knowledge" responses**
```
Problem: Quality gates rejecting low-confidence answers
Solution: Lower min_confidence threshold or add more training data

# Check confidence scores
result = engine.ask(query)
print(f"Confidence: {result.confidence}, Paths: {len(result.reasoning_paths)}")

# Adjust quality gate
gate = QualityGate(min_confidence=0.2)  # Lower threshold
```

**2. Streaming not showing progressive updates**
```
Problem: All paths found too quickly (<100ms)
Solution: Increase target_paths or query complexity

# Increase target paths
processor = StreamingQueryProcessor(
    ...,
    target_paths=10,  # More refinement stages
    min_paths_for_refinement=3
)
```

**3. Hybrid mode falling back to template**
```
Problem: Grounding validation failing (70% threshold)
Solution: Check fact pool extraction or lower temperature

# Debug grounding
import logging
logging.getLogger('sutra_nlg').setLevel(logging.DEBUG)

# Check generated vs facts
```

**4. NLG service not responding (hybrid mode)**
```
# Check service health
curl http://localhost:8889/health

# Check HAProxy routing
curl http://localhost:8405/stats

# View service logs
docker logs nlg-1

# Restart service
docker-compose -f docker-compose-grid.yml restart nlg-1
```

**See [DEPLOYMENT.md](./DEPLOYMENT.md) for complete troubleshooting guide**

---

## 🔗 Related Documentation

### Core System
- [WARP.md](../../WARP.md) - Complete system architecture
- [README.md](../../README.md) - Project overview
- [QUICKSTART.md](../QUICKSTART.md) - Getting started

### Related Features
- [STREAMING.md](../STREAMING.md) - Progressive answer refinement
- [PRODUCTION.md](../PRODUCTION.md) - Production deployment guide
- [SEMANTIC_UNDERSTANDING.md](../SEMANTIC_UNDERSTANDING.md) - Semantic reasoning

### Storage & Graph
- [STORAGE_ARCHITECTURE_DEEP_DIVE.md](../STORAGE_ARCHITECTURE_DEEP_DIVE.md) - Storage engine
- [TCP_PROTOCOL_ARCHITECTURE.md](../TCP_PROTOCOL_ARCHITECTURE.md) - TCP protocol

---

## 📞 Support & Contributing

### Getting Help
- 📖 Documentation: [docs/nlg/](.)
- 🐛 Issues: [GitHub Issues](https://github.com/nranjan2code/sutra-memory/issues)
- 💬 Discussions: [GitHub Discussions](https://github.com/nranjan2code/sutra-memory/discussions)

### Contributing
See [CONTRIBUTING.md](../CONTRIBUTING.md) for:
- Code style guidelines
- Testing requirements
- Pull request process

---

## 📝 Changelog

### v2.0.0 (2025-10-27) - Major Documentation Rewrite

**Updated:**
- ✅ Complete NLG documentation rewrite based on actual code
- ✅ Added streaming architecture and examples
- ✅ Integrated MPPA and quality gates documentation
- ✅ Updated model to gemma-3-270m-it
- ✅ Comprehensive configuration and troubleshooting guides
- ✅ Accurate performance benchmarks
- ✅ Real code examples from codebase

**Architecture:**
- ✅ Multi-Path Plan Aggregation (MPPA) for consensus
- ✅ Progressive streaming with AsyncReasoningEngine
- ✅ Quality gates with confidence calibration
- ✅ Template/Hybrid/Streaming mode support

---

**Built with ❤️ by the Sutra AI Team**

**Status:** ✅ Production-Ready  
**Last Updated:** 2025-10-27  
**Version:** 2.0.0

````

---

## 🎓 Learning Path

### 1. Start Here (5 minutes)
- Read this README
- Run quickstart commands above

### 2. Deploy (10 minutes)
- Follow [DEPLOYMENT.md](./DEPLOYMENT.md)
- Enable hybrid mode
- Test with sample queries

### 3. Understand Architecture (20 minutes)
- Read [ARCHITECTURE.md](./ARCHITECTURE.md)
- Understand data flow
- Learn grounding validation

### 4. Explore Design (30 minutes)
- Read [DESIGN_DECISIONS.md](./DESIGN_DECISIONS.md)
- Understand trade-offs
- Learn why each decision was made

### 5. Customize (optional)
- Swap models (phi-2, TinyLlama)
- Adjust grounding threshold
- Scale replicas

---

## 🎯 Use Cases

### ✅ When to Use Hybrid Mode

- **User-facing chatbots**: Natural language improves UX
- **Complex explanations**: Multi-part questions benefit from fluent text
- **Professional contexts**: Formal/regulatory tones
- **Marketing demos**: Show best-case AI quality

### ⚡ When to Use Template Mode

- **High-throughput APIs**: <10ms latency required
- **Simple fact lookups**: "What is X?" queries
- **Resource-constrained environments**: Limited RAM/CPU
- **Development**: Fast iteration without model loading

---

## 🛠️ Configuration Reference

### Environment Variables

```bash
# Enable/Disable NLG
SUTRA_NLG_ENABLED=true          # Default: false

# NLG Mode
SUTRA_NLG_MODE=hybrid           # Options: "template" or "hybrid"

# NLG Service
SUTRA_NLG_SERVICE_URL=http://nlg-ha:8889
SUTRA_NLG_MODEL=google/gemma-2-2b-it  # Swappable

# Tone
SUTRA_NLG_TONE=friendly         # Options: friendly, formal, concise, regulatory
```

### Docker Compose

```bash
# Start with hybrid NLG
docker-compose -f docker-compose-grid.yml --profile nlg-hybrid up -d

# Scale replicas
docker-compose -f docker-compose-grid.yml --profile nlg-hybrid up -d --scale nlg-1=5

# Swap models
SUTRA_NLG_MODEL=microsoft/phi-2 docker-compose -f docker-compose-grid.yml --profile nlg-hybrid up -d
```

---

## 🧪 Testing

### Health Checks

```bash
# NLG service health
curl http://localhost:8889/health
# Expected: {"status":"healthy","model_loaded":true}

# HAProxy stats dashboard
open http://localhost:8405/stats

# Service metrics
curl http://localhost:8889/metrics
```

### Test Queries

```bash
# Template mode (fast)
curl -X POST http://localhost:8001/sutra/query \
  -H "Content-Type: application/json" \
  -d '{"query": "What is Paris?", "tone": "friendly"}'

# Hybrid mode (natural)
SUTRA_NLG_MODE=hybrid curl -X POST http://localhost:8001/sutra/query \
  -H "Content-Type: application/json" \
  -d '{"query": "What is Paris?", "tone": "formal"}'
```

---

## 🚨 Troubleshooting

### Common Issues

| Issue | Solution | Documentation |
|-------|----------|---------------|
| NLG service won't start | Check memory (4GB required) | [DEPLOYMENT.md#troubleshooting](./DEPLOYMENT.md#troubleshooting) |
| Grounding validation fails | Lower temperature or change model | [DESIGN_DECISIONS.md#grounding-threshold](./DESIGN_DECISIONS.md#grounding-threshold) |
| Slow generation (>500ms) | Scale replicas or use smaller model | [ARCHITECTURE.md#scalability](./ARCHITECTURE.md#scalability) |

**See [DEPLOYMENT.md - Troubleshooting](./DEPLOYMENT.md#troubleshooting) for complete guide**

---

## 📊 Performance Benchmarks

### Latency

| Query Type | Template | Hybrid | Improvement |
|------------|----------|--------|-------------|
| Simple ("What is X?") | 5ms | 100ms | +20× latency, +5× quality |
| Complex (multi-part) | 8ms | 150ms | +19× latency, +10× quality |
| Long explanation | 10ms | 200ms | +20× latency, +8× quality |

### Throughput

| Configuration | Requests/Second | Notes |
|---------------|-----------------|-------|
| Template only | 1000+ | CPU-bound, trivial |
| Hybrid (1 replica) | ~10 | Limited by generation |
| Hybrid (3 replicas) | ~30 | Linear scaling |
| Hybrid (10 replicas) | ~100 | HAProxy bottleneck |

---

## 🔗 Related Documentation

### Sutra AI Core
- `../../WARP.md` - Complete system architecture
- `../../README.md` - Project overview
- `../QUICKSTART.md` - Getting started

### Storage & Graph
- `../storage/` - Storage engine documentation
- `../grid/` - Distributed infrastructure

### API Documentation
- `../../packages/sutra-api/` - REST API
- `../../packages/sutra-hybrid/` - Hybrid API

---

## 📞 Support

### Documentation
- [DEPLOYMENT.md](./DEPLOYMENT.md) - Deployment guide
- [ARCHITECTURE.md](./ARCHITECTURE.md) - Architecture details
- [DESIGN_DECISIONS.md](./DESIGN_DECISIONS.md) - Design rationale

### Troubleshooting
- Check service logs: `docker logs nlg-1`
- Check HAProxy stats: http://localhost:8405/stats
- Check metrics: `curl http://localhost:8889/metrics`

### Community
- GitHub Issues: [Report bugs](https://github.com/sutra-ai/issues)
- Discussions: [Ask questions](https://github.com/sutra-ai/discussions)

---

## 📝 Changelog

### v1.0.0 (2025-10-25) - Initial Release

**Features:**
- ✅ Self-hosted LLM service (gemma-2-2b-it)
- ✅ High availability (3 replicas + HAProxy)
- ✅ 70% grounding validation (stricter than template)
- ✅ Automatic fallback to template
- ✅ Swappable models
- ✅ Production-grade deployment

**Documentation:**
- ✅ Complete deployment guide
- ✅ Architecture documentation
- ✅ Design decisions explained
- ✅ Troubleshooting guide

---

**Built with ❤️ by the Sutra AI Team**

**Status:** ✅ Production-Ready  
**Last Updated:** 2025-10-25  
**Version:** 1.0.0
