# Sutra Engine Architecture

The Sutra Engine is built for low-latency, high-concurrency knowledge storage and reasoning. This document details the internal components that make it effective as a standalone system.

## Performance Philosophy
Sutra Engine follows a **Lock-Free** and **Memory-First** architecture. By utilizing Rust's safety and performance features, it achieves sub-10ms latencies even under heavy load.

## Core Components

### 1. Lock-Free Concurrent Memory
The heart of the engine is a sharded, concurrent hash map (`DashMap`) and atomic pointers (`ArcSwap`). This allows millions of read operations per second without write contention.

### 2. HNSW Vector Index
For semantic search, the engine integrates an optimized **Hierarchical Navigable Small World (HNSW)** index. 
- **Persistence**: Vectors are memory-mapped (`mmap`) for instant startup.
- **Precision**: Supports various distance metrics (Cosine, L2, IP).

### 3. Graph Logic
Sutra Engine treates data as a **Weighted Directed Graph**.
- **Nodes**: Concepts.
- **Edges**: Associations (Semantic, Temporal, Causal).
- **Consensus**: Multi-Path Plan Aggregation (MPPA) happens at the reasoning layer.

### 4. Storage & Persistence
- **Binary Snapshots**: Periodic flushes of the graph to disk in a compact binary format.
- **Write-Ahead Log (WAL)**: Every write is logged before acknowledgment, ensuring data safety across crashes.
- **Adaptive Reconciler**: A background process that optimizes graph structure and maintains consistency.

## Data Flow (Ingestion)

1.  **Request**: Binary packet received via TCP.
2.  **Preprocessing**: Text normalization and (optional) association extraction.
3.  **Embedding**: Vector generation (if internal model or external service is linked).
4.  **WAL**: Write operation logged to disk.
5.  **Memory Update**: Concurrent Map and HNSW updated.
6.  **Response**: Concept ID returned.

## Network Protocol
The custom TCP protocol uses **MessagePack** for serialization. This reduces overhead compared to JSON over HTTP by up to 50x, providing much higher throughput for bulk ingestion.

## Threading Model
The engine utilizes a multi-threaded **Tokio** runtime. One pool of threads handles network I/O, while another handles compute-intensive tasks like vector search and graph traversals.
