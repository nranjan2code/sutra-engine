# Sutra AI

An explainable AI system that learns in real-time without retraining. Every decision includes reasoning paths showing how it arrived at an answer.

## Why This Exists

Current AI systems (LLMs) are black boxes:
- You can't see how they make decisions
- You can't verify their reasoning
- You can't update them without complete retraining
- You can't use them in regulated industries that require explainability

We're building an alternative that:
- Shows its reasoning for every answer
- Learns incrementally from new information
- Provides audit trails for compliance
- Works without requiring GPUs or massive compute

## What It Does

Sutra AI combines graph-based reasoning with semantic embeddings:

1. **Graph reasoning**: Concepts connected by typed relationships (semantic, causal, temporal, hierarchical, compositional)
2. **Semantic embeddings**: Optional similarity matching to enhance reasoning
3. **Multi-strategy comparison**: Compare different reasoning approaches and see agreement scores
4. **Real-time learning**: Learn from new information without retraining
5. **Full audit trails**: Every decision logged with timestamps, confidence scores, and reasoning paths

## What Works (Proven End-to-End)

✅ **Learn new knowledge** - Add concepts and relationships  
✅ **Query with reasoning paths** - Get answers with explanations  
✅ **Save to disk** - Persist knowledge (concepts, associations, embeddings)  
✅ **Reload from disk** - Restore complete state after restart  
✅ **Multi-strategy reasoning** - Compare graph-only vs semantic-enhanced  
✅ **Audit trails** - Full compliance tracking  
✅ **REST API** - Production-ready HTTP interface  

Tested with 5 concepts, ~100ms query latency, full persistence verified.

## Architecture

gRPC-first, service-oriented design. All application services talk to the Storage Server over gRPC.

```
┌───────────────┐       gRPC        ┌─────────────────────┐
│  sutra-api    │ ───────────────▶  │  storage-server     │
│ (FastAPI)     │ ◀───────────────  │  (Rust, gRPC)       │
└───────────────┘                   └─────────────────────┘
       ▲   ▲                                  ▲
       │   │                                  │
       │   └─────────────── gRPC ─────────────┘
       │
┌───────────────┐
│ sutra-hybrid  │  (embeddings + orchestration, gRPC client)
└───────────────┘
```

- sutra-api: Thin REST-to-gRPC proxy (no local reasoning)
- sutra-hybrid: Embeddings + orchestration, forwards graph ops via gRPC
- storage-server: Single source of truth for graph + vectors

Only `sutra-api` and `sutra-hybrid` are user-facing HTTP services. No component accesses storage directly in-process.

## Quick Start

### 1. Run locally (Docker Compose)

```bash
# From repo root
DEPLOY=local VERSION=v2 bash deploy-optimized.sh

# Health checks
curl -s http://localhost:8000/health     # sutra-api
curl -s http://localhost:8001/ping       # sutra-hybrid
```

### 2. Test End-to-End

```bash
# Run the end-to-end test
python test_direct_workflow.py
```

This tests: Learn → Save → Reload → Query → Multi-strategy → Audit

### 2. Use the API

```bash
# Learn (thin proxy → storage via gRPC)
curl -X POST http://localhost:8000/learn \
  -H "Content-Type: application/json" \
  -d '{"content": "Python is a programming language"}'

# Health
curl -s http://localhost:8000/health
```

## What We're Working Toward

**Short-term** (Working now):
- Graph-based reasoning with explainability ✅
- Real-time learning without retraining ✅
- Semantic similarity enhancement ✅
- REST API ✅

**Mid-term** (In progress):
- Replace LLM-style interfaces completely
- Streaming responses
- Multi-modal support (text + structured data)
- Distributed reasoning

**Long-term** (Research):
- Replace all black-box neural networks with explainable alternatives
- Provable correctness for critical decisions
- Zero-trust AI systems where every output is verifiable

## Performance Characteristics

Storage-server benchmarks (production):

- **Learning**: 0.02ms per concept (57,412/sec)
- **Query (read)**: <0.01ms via in-memory snapshot
- **Path finding**: ~1ms for 3-hop BFS (server-side)
- **Storage**: Single file, memory-mapped, lock-free writes
- **Vector search**: HNSW O(log N)

## Key Design Decisions

### Why Graph-Based?

Graphs are inherently explainable. You can trace every reasoning path. LLMs are not.

### Why Rust for Storage?

Python is great for logic but slow for I/O. Rust gives us:
- Zero-copy memory-mapped files
- Lock-free concurrency
- Predictable performance

### Why Optional Embeddings?

Pure graph reasoning is 100% explainable. Embeddings enhance it but add some opacity. We make it optional and always show contribution.

### Why REST API as Sole Interface?

Clean separation. Internal implementation can change without breaking users.

## Project Structure

```
sutra-models/
├── packages/
│   ├── sutra-core/          # Graph reasoning engine
│   ├── sutra-storage/        # Rust storage backend  
│   ├── sutra-hybrid/         # Semantic embeddings
│   └── sutra-api/            # REST API (FastAPI)
├── test_direct_workflow.py   # End-to-end test
├── test_api_workflow.py      # API integration test
├── QUICK_START.md            # How to run and test
└── README.md                 # This file
```

## Testing

```bash
# Test core package
make test-core

# Test end-to-end workflow (no API)
python test_direct_workflow.py

# Test API workflow (requires API server running)
python test_api_workflow.py

# Format code
make format

# Lint
make lint
```

## Configuration

Via environment variables or config files:

```bash
# Storage location
export SUTRA_STORAGE_PATH="./knowledge"

# Enable semantic embeddings
export SUTRA_USE_SEMANTIC_EMBEDDINGS="true"

# API settings
export SUTRA_API_PORT="8000"

# Rate limits
export SUTRA_RATE_LIMIT_LEARN="30"
export SUTRA_RATE_LIMIT_REASON="60"
```

## Dependencies

**Core**:
- Python 3.8+
- numpy
- sutra-storage (Rust, compiled to Python extension)

**Optional**:
- sentence-transformers (for semantic embeddings)
- spaCy (for enhanced NLP)
- FastAPI + uvicorn (for API server)

## What This Is Not

- **Not an LLM replacement yet** - We're working toward it, but not there yet
- **Not trained on massive datasets** - Learns from what you give it
- **Not a general knowledge base** - Specialized for your domain
- **Not "AI magic"** - Deterministic reasoning with explainable paths

## Current Limitations

Honest assessment of what doesn't work yet:

1. **Limited reasoning depth** - Works well for 2-3 hops, gets expensive beyond that
2. **No natural language generation** - Returns concept content, not fluent text
3. **Requires structured input** - Works best with clear factual statements
4. **No common sense reasoning** - Only knows what you teach it
5. **English-only** - NLP components are English-centric

## Contributing

We welcome contributions that align with the mission of explainable, accountable AI.

Before contributing:
1. Read the architecture docs in WARP.md
2. Run tests to verify your changes
3. Follow the existing code style (black + isort)
4. Add tests for new features

## Research Foundation

Built on published research:

- **Adaptive Focus Learning**: "LLM-Oriented Token-Adaptive Knowledge Distillation" (Oct 2024)
- **Multi-Path Plan Aggregation (MPPA)**: Consensus-based reasoning
- **Graph-based reasoning**: Decades of knowledge representation research

No proprietary "secret sauce" - all techniques are from published work.

## License

MIT License - see LICENSE file

## Contact

This is an active research project. We're figuring things out as we go.

Issues and pull requests welcome.

---

**Status**: Production-ready for internal use. API tested end-to-end. Full persistence verified.  
**Version**: 2.0.0  
**Last tested**: 2025-10-16
