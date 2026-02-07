<div align="center">
  <img src="assets/logo.png" alt="Sutra Engine Logo" width="300"/>
  <h1>Sutra Engine</h1>
  <p><b>Embedded Vector-Graph Database with Built-in Embeddings</b></p>

  [![Version](https://img.shields.io/badge/version-1.0.0-blue.svg)]()
  [![License](https://img.shields.io/badge/license-MIT-green.svg)]()
  [![Rust](https://img.shields.io/badge/language-Rust-orange.svg)]()
  [![Performance](https://img.shields.io/badge/latency-<1ms-brightgreen.svg)]()
</div>

---

## 🌟 Overview

**Sutra Engine** is a high-performance hybrid storage engine that combines **dense vector search** with **graph relationships** in a unified architecture. Built in Rust, it delivers production-grade performance with embedded ML inference—no external API calls required.

Designed for applications requiring both semantic similarity and relational knowledge, Sutra handles **50,000+ writes per second** with **sub-millisecond search latency**.

### Key Features
- 🔌 **Zero Dependencies**: Embedded inference (Candle/BERT) runs locally on Metal/CPU—no API keys needed.
- 🗣️ **Natural Language Interface**: Native TCP interface for text commands ("Insert...", "Search for...").
- ⚡ **Ultra-Low Latency**: Custom binary protocol for 10-50x better performance than gRPC.
- 🛡️ **Production Ready**: Native TLS 1.3, HMAC-SHA256 authentication, and rate limiting.
- 📈 **Horizontal Scaling**: Consistent-hashing sharding for 10M+ records.
- 💾 **Durability**: Write-Ahead Logging (WAL) and mmap-backed persistence for crash recovery.
- 🔧 **Background Maintenance**: Configurable jobs for strength decay, auto-association, health metrics, triggers, and subscriptions.

---

## 🚀 Quick Start

### 1. Build and Run
Ensure you have the Rust toolchain installed.

```bash
# Build and run the storage server (defaults to port 50051 for binary, or 9000 for NL)
STORAGE_PORT=9000 cargo run --release --bin storage-server
```

### 2. Connect via Natural Language
You can talk to the engine directly using `netcat`:

```bash
echo "Insert: Sutra is fast" | nc localhost 9000
echo "Search for: fast database" | nc localhost 9000
```

### 3. Connect via Binary Protocol
For high-performance applications, use the binary protocol (SDKs available).

### 4. Standalone Quickstart
For a minimal, clean setup path, see:
- `docs/STANDALONE_QUICKSTART.md`

---

## 📚 Documentation

Start at the index for the full map:
- [**Documentation Index**](docs/INDEX.md)

### 🏁 Foundations
- [**Standalone Quickstart**](docs/STANDALONE_QUICKSTART.md): Minimal run path.
- [**Getting Started**](docs/GETTING_STARTED.md): Installation and basic workflow.
- [**Project Architecture**](docs/ARCHITECTURE.md): Dual-plane design and core layout.

### 🔧 Background Maintenance
- [**Background Jobs**](docs/ARCHITECTURE.md#background-maintenance): Strength decay, auto-association, triggers, subscriptions, graph analysis, and health metrics.

### 🛠 Developers
- [**API Reference**](docs/API_REFERENCE.md): Full TCP + MessagePack protocol spec.

### 🛡 Production & Ops
- [**Security Manual**](docs/SECURITY.md): TLS 1.3, HMAC Auth, and hardening.
- [**Operations & Tuning**](docs/OPERATIONS.md): Persistence, sharding, performance.
- [**Test Results**](docs/TEST_RESULTS.md): Latest validation run.
- [**Final Report**](docs/FINAL_REPORT.md): Standalone cleanup and verification summary.

### 🆘 Support
- [**Troubleshooting**](docs/TROUBLESHOOTING.md): Common error codes and fixes.

---

## 📁 Examples

- `crates/protocol/examples/minimal_roundtrip.rs`: minimal protocol roundtrip.
- `crates/storage/examples/concurrent_burst_demo.rs`: storage stress demo.

---

## ⚖️ License

Distributed under the MIT License. See `LICENSE` for more information.

---
<div align="center">
  Built with ❤️ by the Sutra Works Team.
</div>
