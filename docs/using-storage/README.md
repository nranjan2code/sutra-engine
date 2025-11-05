# Using Sutra Storage: Developer Guide

## Overview

The Sutra Storage Engine is a high-performance, distributed knowledge graph designed for AI reasoning applications. It stores two fundamental data types: **concepts** and **associations**, which together form a sophisticated knowledge representation system optimized for explainable AI reasoning.

## Architecture at a Glance

```
┌─────────────────────────────────────────────────────────────────┐
│                    Sutra Storage Engine                         │
├─────────────────────────────────────────────────────────────────┤
│  Data Model:                                                    │
│  • Concepts: Knowledge nodes with content + embeddings         │
│  • Associations: Typed relationships between concepts          │
│  • Multi-tenancy: Organization-based data isolation           │
├─────────────────────────────────────────────────────────────────┤
│  Storage:                                                       │
│  • Binary format: SUTRADAT v2 (mmap + WAL)                    │
│  • Performance: Optimized reads and writes                     │
│  • Persistence: Write-Ahead Log with crash recovery           │
├─────────────────────────────────────────────────────────────────┤
│  Interface:                                                     │
│  • TCP Binary Protocol: MessagePack (10-50x faster than gRPC) │
│  • Unified Learning: Server-side embeddings + associations    │
│  • Vector Search: HNSW index with semantic queries            │
│  ⚠️  BREAKING v3.0.0: gRPC removed - TCP Binary Protocol only │
└─────────────────────────────────────────────────────────────────┘
```

## Table of Contents

1. [**Data Model**](./01-data-model.md) - Core concepts and associations
2. [**Storage Format**](./02-storage-format.md) - Binary format and persistence 
3. [**TCP Protocol**](./03-tcp-protocol.md) - Communication interface
4. [**Client Usage**](./04-client-usage.md) - Python client examples
5. [**Learning Pipeline**](./05-learning-pipeline.md) - Unified learning system
6. [**Multi-Tenancy**](./06-multi-tenancy.md) - Organization isolation
7. [**Vector Search**](./07-vector-search.md) - Semantic search capabilities
8. [**Performance Guide**](./08-performance.md) - Optimization strategies
9. [**Troubleshooting**](./09-troubleshooting.md) - Common issues and solutions

## Quick Start

### 1. Connect to Storage

```python
from sutra_storage_client import StorageClient

# Connect to storage server
client = StorageClient("localhost:50051")
```

### 2. Learn Concepts (Unified Pipeline)

```python
# Simple learning with automatic embeddings and associations
concept_id = client.learn_concept_v2(
    content="Machine learning is a subset of artificial intelligence."
)

# Batch learning for efficiency
concept_ids = client.learn_batch_v2([
    "Neural networks are inspired by biological neurons.",
    "Deep learning uses multi-layer neural networks.",
    "Transformers revolutionized natural language processing."
])
```

### 3. Query and Search

```python
# Get a concept by ID
concept = client.query_concept(concept_id)
print(f"Content: {concept['content']}")
print(f"Confidence: {concept['confidence']}")

# Vector search for semantic similarity
results = client.vector_search(
    query_text="What is deep learning?",
    k=5
)
for result in results:
    print(f"Similar concept: {result['content']} (similarity: {result['similarity']})")
```

### 4. Work with Associations

```python
# Manual association creation
client.learn_association(
    source_id=concept_id1,
    target_id=concept_id2,
    assoc_type=0,  # Semantic association
    confidence=0.85
)

# Find related concepts
neighbors = client.get_neighbors(concept_id)
```

## Key Design Principles

### 1. **Performance First**
- Sub-millisecond reads via memory mapping
- Thousands of writes per second with WAL durability
- Zero-copy vector operations with persistent HNSW index

### 2. **Explainable Reasoning**
- Every concept has traceable provenance
- Associations capture explicit reasoning relationships
- Confidence scores enable quality assessment

### 3. **Domain-Specific Knowledge**
- Starts empty, learns from YOUR data
- No pre-trained bias or hallucinations
- Perfect for regulated industries (medical, legal, financial)

### 4. **Production Ready**
- ACID transactions with 2PC across shards
- Automatic failure detection and recovery
- Comprehensive audit trails for compliance

## Core Concepts

### Concept
A **concept** is the fundamental unit of knowledge storage. Each concept contains:
- **Unique ID**: 16-byte MD5 hash for deterministic identification
- **Content**: UTF-8 text (facts, documents, procedures)
- **Embedding**: 768-dimensional semantic vector
- **Metadata**: Strength, confidence, access patterns
- **Multi-tenant**: Optional organization ID for isolation

### Association  
An **association** represents a typed relationship between two concepts:
- **Source/Target**: ConceptId pairs defining the relationship
- **Type**: Semantic, causal, temporal, hierarchical, compositional
- **Confidence**: Strength of the relationship (0.0-1.0)
- **Lifecycle**: Creation time and usage tracking

### Learning Pipeline
The unified learning process that:
1. Generates semantic embeddings for new content
2. Extracts associations to existing concepts
3. Stores everything atomically with full durability
4. Updates HNSW index for fast vector search

## When to Use Sutra Storage

✅ **Perfect For:**
- Domain-specific AI applications (medical protocols, legal precedents)
- Explainable AI systems requiring audit trails
- Real-time learning without model retraining
- Regulated industries needing compliance documentation
- Knowledge bases requiring fine-grained access control

❌ **Not Ideal For:**
- Simple document storage (use traditional databases for non-graph workloads)
- General-purpose chat applications (use existing LLMs)
- Applications needing pre-trained world knowledge
- Simple key-value storage needs

## Next Steps

1. **Start with the [Data Model](./01-data-model.md)** to understand concepts and associations
2. **Review [Client Usage](./04-client-usage.md)** for practical examples
3. **Explore [Learning Pipeline](./05-learning-pipeline.md)** for advanced features
4. **Check [Performance Guide](./08-performance.md)** for optimization tips

## Support

- **Documentation Issues**: Create issues in the repository
- **Performance Questions**: See [Performance Guide](./08-performance.md)
- **Troubleshooting**: Consult [Troubleshooting Guide](./09-troubleshooting.md)

---

*Sutra Storage Engine - Domain-Specific Explainable AI for Production*