# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Project Overview

Sutra AI is an explainable graph-based AI system that learns in real-time without retraining. It provides complete reasoning paths for every decision, making it a transparent alternative to black-box LLMs.

**Core Value Proposition:**
- Shows reasoning for every answer
- Learns incrementally from new information  
- Provides audit trails for compliance
- Works without GPUs or massive compute
- 100% explainable reasoning paths

## Architecture

gRPC-first monorepo — all app services connect to the storage server.

```
┌───────────────┐         gRPC          ┌─────────────────────┐
│  sutra-api    │ ───────────────────▶  │  storage-server     │
│  (FastAPI)    │ ◀───────────────────  │  (Rust, gRPC)       │
└───────────────┘                       └─────────────────────┘
        ▲                                        ▲
        │                                        │
        └──────────── gRPC ──────────────────────┘

┌───────────────┐
│ sutra-hybrid  │  (embeddings + orchestration)
└───────────────┘
```

**Key Architectural Principles:**
- All graph/vector ops run in storage-server; no in-process storage in API/Hybrid
- Rust storage provides zero-copy memory-mapped files and lock-free concurrency
- Optional semantic embeddings enhance reasoning but remain transparent
- Temporal log-structured storage for time-travel queries

### Package Structure

- **sutra-core**: Core graph reasoning engine with concepts, associations, and multi-path plan aggregation (MPPA)
- **sutra-storage**: Production-ready Rust storage with ConcurrentStorage (57K writes/sec, <0.01ms reads), single-file architecture, and lock-free concurrency  
- **sutra-hybrid**: Semantic embeddings integration (SutraAI class) that combines graph reasoning with optional similarity matching
- **sutra-api**: Production REST API with FastAPI, rate limiting, and comprehensive endpoints
- **sutra-cli**: Command-line interface (placeholder)

## Development Commands

### Environment Setup
```bash
# One-command setup (creates venv, installs packages)
make setup

# Manual setup (alternative)
python3 -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate
pip install -e packages/sutra-core/
pip install -r requirements-dev.txt
```

### Testing
```bash
# Core package tests
make test-core

# End-to-end demos
python demo_simple.py
python demo_end_to_end.py
python demo_mass_learning.py
python demo_wikipedia_learning.py

# Integration tests (root tests directory)
python -m pytest tests/ -v

# Verify storage performance
python verify_concurrent_storage.py
```

### Code Quality
```bash
# Format code (black + isort)
make format

# Lint core package
make lint

# Full quality check
make check  # Runs format, lint, and test
```

### Demos and Development
```bash
# Local stack (recommended)
DEPLOY=local VERSION=v2 bash deploy-optimized.sh

# Or run API locally (requires storage server)
export SUTRA_STORAGE_SERVER=localhost:50051
uvicorn sutra_api.main:app --host 0.0.0.0 --port 8000
```

### Build and Distribution
```bash
# Build all packages
make build

# Clean build artifacts
make clean
```

## Key Components

### ReasoningEngine (sutra-core)
The main AI interface that orchestrates reasoning components:
- Natural language query processing
- Real-time learning without retraining
- Query result caching for performance  
- Multi-path plan aggregation (MPPA) for consensus
- Complete audit trails

### PathFinder (sutra-core/reasoning)
Advanced graph traversal with multiple search strategies:
- **Best-first**: Confidence-optimized with heuristics
- **Breadth-first**: Shortest path exploration
- **Bidirectional**: Optimal path finding from both ends
- Confidence decay (0.85 default) for realistic propagation
- Cycle detection and path diversification

### MultiPathAggregator (MPPA) (sutra-core/reasoning)
Consensus-based reasoning that prevents single-path derailment:
- Path clustering by answer similarity (0.8 threshold)
- Majority voting with configurable thresholds
- Diversity bonus for varied reasoning approaches
- Robustness analysis with multiple metrics

### SutraAI (sutra-hybrid)
High-level interface combining graph reasoning with semantic embeddings:
- Optional semantic similarity matching
- Multi-strategy comparison (graph-only vs semantic-enhanced)
- Agreement scoring between strategies
- Knowledge persistence and audit trails

### ConcurrentStorage (sutra-storage - Rust)
Production-ready storage architecture:
- **57,412 writes/sec** (25,000× faster than JSON baseline)
- **<0.01ms read latency** (zero-copy memory-mapped files)
- Lock-free write log with background reconciliation
- Single-file storage (`storage.dat`) with 512MB initial size
- Immutable read snapshots for burst-tolerant performance
- Path finding and graph traversal (BFS)
- 100% test pass rate with verified accuracy
### Configuration

### Environment Variables
```bash
# Storage server address (all services)
export SUTRA_STORAGE_SERVER="storage-server:50051"

# API settings
export SUTRA_API_PORT="8000"

# Rate limits
export SUTRA_RATE_LIMIT_LEARN="30"
export SUTRA_RATE_LIMIT_REASON="60"
```
```

### Reasoning Configuration
- **Confidence decay**: 0.85 per reasoning step
- **Max reasoning depth**: 6 hops
- **Consensus threshold**: 50% agreement for multi-path aggregation
- **Path similarity threshold**: 0.7 maximum overlap for diversification

## Performance Characteristics

Based on production testing with ConcurrentStorage:
- **Learning**: **0.02ms per concept** (57,412 concepts/sec) — 25,000× faster than old system
- **Query**: **<0.01ms** with zero-copy memory-mapped access
- **Path finding**: ~1ms for 3-hop BFS traversal
- **Memory**: ~0.1KB per concept (excluding embeddings)
- **Storage**: Single `storage.dat` file (512MB for 1K concepts)
- **Vector search**: Product quantization for 4× compression (in development)
- **Accuracy**: 100% verified with comprehensive test suite

## Testing Strategy

The system has comprehensive testing at multiple levels:

1. **Integration tests**: `tests/` directory with cross-package functionality
2. **End-to-end demos**: `demo_simple.py`, `demo_end_to_end.py`, `demo_mass_learning.py`
3. **API tests**: `packages/sutra-api/tests/` and `packages/sutra-hybrid/tests/`
4. **Performance verification**: `verify_concurrent_storage.py` (production benchmarks)
5. **Storage tests**: Rust unit and integration tests in `packages/sutra-storage/`

### Test Locations
- **Root `tests/`**: Integration tests and query processor tests
- **`demo_*.py`**: End-to-end workflow demonstrations
- **`verify_concurrent_storage.py`**: Performance benchmarking (57K writes/sec verified)
- **Package-specific tests**: In respective `packages/*/tests/` directories

## Common Development Tasks

### Adding New Reasoning Strategies
1. Implement in `sutra_core/reasoning/paths.py`
2. Add to PathFinder class
3. Update QueryProcessor to support new strategy
4. Add comprehensive tests

### Extending Storage Format
1. Update Rust structures in `packages/sutra-storage/src/`
2. Modify Python bindings via PyO3 in `lib.rs`
3. Update memory layout documentation in `docs/sutra-storage/`
4. Run `cargo build --release` and verify with `verify_concurrent_storage.py`
5. Ensure single-file `storage.dat` format compatibility

### Adding API Endpoints
1. Define request/response models in `sutra_api/models.py`
2. Implement endpoint in `sutra_api/main.py`
3. Add rate limiting configuration
4. Update OpenAPI documentation

## Troubleshooting

### Import Errors
```bash
# Solution: Set PYTHONPATH for core tests
PYTHONPATH=packages/sutra-core python -m pytest tests/ -v

# Or run integration tests from root
python -m pytest tests/ -v
```

### Virtual Environment Issues
```bash
# Recreate environment
python3 -m venv venv
source venv/bin/activate
pip install -e packages/sutra-core/
pip install -r requirements-dev.txt
```

### Storage/Persistence Issues
- **ConcurrentStorage**: Data written to single `storage.dat` file
- Check storage path permissions (default: `./knowledge/storage.dat`)
- Call `storage.flush()` manually before shutdown
- Auto-flush triggers at 50K concepts (configurable)
- Verify performance with `verify_concurrent_storage.py`
- Monitor stats: `storage.stats()` shows writes, drops, concepts, edges

## Code Style

- **Line length**: 88 characters (black default)
- **Import order**: stdlib, third-party, local (isort with black profile)
- **Type hints**: Required for all public functions
- **Docstrings**: Google style for all public classes and methods
- **Testing**: pytest with descriptive test names and docstrings

## Research Foundation

Built on published research:
- **Adaptive Focus Learning**: "LLM-Oriented Token-Adaptive Knowledge Distillation" (Oct 2024)
- **Multi-Path Plan Aggregation (MPPA)**: Consensus-based reasoning
- **Graph-based reasoning**: Decades of knowledge representation research

No proprietary techniques - all methods are from published work.