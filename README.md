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

**12-Service Production Ecosystem** with TCP binary protocol and containerized deployment. All services communicate via high-performance TCP with a secure React-based control center for monitoring.

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                        Docker Network (sutra-network)                         │
├─────────────────────────────────────────────────────────────────────────────────┤
│  ┌───────────────┐    ┌───────────────┐    ┌──────────────────┐  │
│  │  sutra-control │    │  sutra-client  │    │ sutra-markdown-web │  │
│  │  (React + Fast │    │   (Streamlit   │    │   (Markdown API)   │  │
│  │   API Gateway) │    │    UI Client)  │    │    Port: 8002     │  │
│  │   Port: 9000   │    │   Port: 8080   │    └──────┴────────────┘  │
│  └──────┴─────────┘    └──────┴─────────┘                        │
│            │                     │            TCP                        │
│            └─────────────────────┴───── Binary                   │
│                                     │  Protocol                      │
│  ┌───────────────┐              │  ┌─────────────────────────────┐  │
│  │   sutra-api     │◀─────────────┴──▶│       storage-server         │  │
│  │   (FastAPI)     │              │  │    (Rust TCP Server)        │  │
│  │   Port: 8000    │              │  │      Port: 50051            │  │
│  └──────┴─────────┘              │  └──────┴────────────────────────┘  │
│            │                     │            │                       │
│            └─────────────────────┴─────┴────────────────────────┘  │
│                                     │            │                       │
│  ┌───────────────┐              │  ┌─────┴───────────────────────┐  │
│  │ sutra-hybrid  │◀─────────────┴──│◀┴────  sutra-ollama         │  │
│  │ (Embeddings + │              │  │   (Local LLM Server)      │  │
│  │ Orchestration)│              │  │      Port: 11434           │  │
│  │   Port: 8001   │              │  └───────────────────────┘  │
│  └───────────────┘              │                           │
│                                     │                           │
│  ┌────────────────────────────────┐◀─────────────┴─────────────────────────────  │
│  │      sutra-bulk-ingester       │            🔥 NEW SERVICE        │
│  │   (High-Performance Rust)      │            Port: 8005           │
│  │      Port: 8005               │         (Production Ready)      │
│  └────────────────────────────────┘                                   │
│                                                                    │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                   Sutra Grid (Distributed Layer)                    │  │
│  │  Grid Master (7001 HTTP, 7002 TCP) ◀──TCP──▶ Grid Agents (8001)        │  │
│  │  Event Storage (50052 TCP)                                         │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────────────────────────────────┘
```

### Core Services
- **sutra-control**: React-based monitoring center with Grid management and bulk ingester UI
- **sutra-client**: Streamlit web interface for interactive queries  
- **sutra-api**: Primary REST API for AI operations
- **sutra-hybrid**: Semantic embeddings and orchestration
- **storage-server**: Rust TCP core storage engine (57K writes/sec)
- **sutra-bulk-ingester**: 🔥 **NEW** High-performance Rust bulk data ingestion (1K-10K articles/min)
- **sutra-markdown-web**: Document processing API
- **sutra-ollama**: Local LLM inference server

All services communicate via gRPC internally, with REST APIs for external access. The control center provides secure monitoring without exposing internal implementation details.

### Sutra Grid - Distributed Storage Orchestration

**NEW**: Production-ready distributed infrastructure with complete Docker deployment and web UI integration.

Sutra Grid manages storage nodes across multiple agents with:
- **Bidirectional gRPC**: Master ↔ Agent communication (ports 7001 HTTP, 7002 gRPC)
- **Event-Driven Monitoring**: 17 structured events → knowledge graph (port 50052)
- **Auto-Recovery**: Crashed nodes restart automatically (up to 3 times)
- **Production Features**: Retry logic, timeouts, health monitoring, graceful degradation
- **Web UI**: Complete Grid management via Sutra Control Center (port 9000)

**Key Innovation**: Grid monitors itself using Sutra's own platform - proving event-driven observability works without external LMT (Logs/Metrics/Telemetry) stack.

**Status**: Production-Ready ✅  
- Master: 11 events emitted
- Agent: 2 node lifecycle events  
- Storage: Events as queryable concepts
- Docker: Complete containerized deployment
- Control Center: Grid management UI integrated
- Testing: End-to-end verified

**Architecture Details**: See [docs/grid/architecture/GRID_ARCHITECTURE.md](docs/grid/architecture/GRID_ARCHITECTURE.md) and [DEPLOYMENT.md](DEPLOYMENT.md) for complete documentation.

## 🚨 CRITICAL PRODUCTION REQUIREMENTS

**⚠️ Before deployment, you MUST read:**
- [`PRODUCTION_CHECKLIST.md`](PRODUCTION_CHECKLIST.md) - Mandatory pre-deployment verification
- [`docs/EMBEDDING_TROUBLESHOOTING.md`](docs/EMBEDDING_TROUBLESHOOTING.md) - Critical fixes applied

**The system will NOT function without:**
1. Ollama service with `granite-embedding:30m` model
2. Proper TCP protocol implementation
3. Environment variables correctly configured

## Quick Start

### 1. Deploy with Docker (Recommended)

**⚡ Single command deployment:**

```bash
# First-time installation
./sutra-deploy.sh install

# Or start existing services
./sutra-deploy.sh up
```

**Access services:**
```bash
open http://localhost:9000    # Control Center (monitoring + Grid + bulk ingester)
open http://localhost:8080    # Interactive Client (queries)
open http://localhost:8000    # Primary API
```

**Manage deployment:**
```bash
./sutra-deploy.sh status      # Check system status
./sutra-deploy.sh logs        # View all logs
./sutra-deploy.sh maintenance # Interactive menu
./sutra-deploy.sh down        # Stop all services
```

**See [DEPLOYMENT.md](DEPLOYMENT.md) for complete documentation.**

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
