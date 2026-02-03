# Sutra Engine: Multi-Language Examples

This directory contains production-ready examples demonstrating how to integrate Sutra Engine into your application stack.

---

## 🏗 Prerequisites

Before running any examples, ensure the Sutra Engine is running locally:
```bash
./start-engine.sh
```

---

## 🐍 Python (Elite SDK)
The Python client is the most mature and recommended for most users. It includes **Connection Pooling**, **Adaptive Backpressure**, and **Automatic Serialization**.

**Quick Start:**
```bash
cd examples/python
pip install -r requirements.txt
export PYTHONPATH=$PYTHONPATH:../../
python3 demo.py
```

**What it demonstrates:**
- Asynchronous ingestion.
- Semantic vector search.
- Connection resilience.

---

## 🟦 TypeScript / Node.js
Ideal for web backends and integration with modern JavaScript frameworks.

**Quick Start:**
```bash
cd examples/typescript
npm install
npm start
```

**What it demonstrates:**
- Binary protocol implementation using `msgpack5`.
- TCP socket management in Node.js.
- Clean Request/Response cycling.

---

## 🦀 Rust (Native High-Performance)
Use the Rust client for systems-level integration or when you need the absolute maximum throughput.

**Quick Start:**
```bash
cd examples/rust
cargo run
```

**What it demonstrates:**
- Zero-copy deserialization with `rmp-serde`.
- Async/Await I/O with `tokio`.
- High-performance binary framing.

---

## 🛠 Which one should I choose?

| Stack | Best For | Complexity |
|-------|----------|------------|
| **Python** | AI Agents, RAG, Data Science | Low |
| **Node.js** | Web APIs, Microservices | Medium |
| **Rust** | Ingestion Pipelines, High-Load Systems | High |

---

## 🛰 Cross-Language Compatibility
All clients communicate via the same **MessagePack binary protocol**. You can ingest data with Rust and search for it using Python without any translation layer.
