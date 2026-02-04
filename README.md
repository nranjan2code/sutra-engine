<div align="center">
  <img src="assets/logo.png" alt="Sutra Engine Logo" width="300"/>
  <h1>Sutra Engine</h1>
  <p><b>High-Performance Explainable Memory Engine for Reasoning Agents</b></p>

  [![Version](https://img.shields.io/badge/version-1.0.0-blue.svg)]()
  [![License](https://img.shields.io/badge/license-MIT-green.svg)]()
  [![Rust](https://img.shields.io/badge/language-Rust-orange.svg)]()
  [![Performance](https://img.shields.io/badge/latency-<1ms-brightgreen.svg)]()
</div>

---

## 🌟 Overview

**Sutra Engine** is a state-of-the-art, standalone memory engine designed for the next generation of AI agents. Unlike traditional vector databases, Sutra combines **dense vector search** with **semantic graph relationships** in a unified, high-performance reasoning core.

It is built in Rust to provide elite-level performance, supporting over **50,000 writes per second** and **sub-millisecond search latency** on standard hardware.

### Key Features
- 🧠 **Dual-Plane Memory**: Combines HNSW-based vector search with an explainable knowledge graph.
- ⚡ **Ultra-Low Latency**: Custom binary protocol replaces gRPC for 10-50x better performance.
- 🛡️ **Production Ready**: Native TLS 1.3, HMAC-SHA256 authentication, and rate limiting.
- 📈 **Horizontal Scaling**: Support for consistent-hashing sharding beyond 10M+ concepts.
- 💾 **Durability**: Write-Ahead Logging (WAL) and mmap-backed persistence for crash recovery.

---

## 🚀 Quick Start

### 1. Build from Source
Ensure you have the Rust toolchain installed.

```bash
# Build the storage server
cargo build --release --bin storage-server

# Run the engine
./target/release/storage-server
```

### 2. Connect
You can connect to the engine using any TCP client that supports the Sutra custom binary protocol.

### 3. Standalone Quickstart
For a minimal, clean setup path, see:
- `docs/STANDALONE_QUICKSTART.md`

---

## 📚 Documentation

Start at the index for the full map:
- [**Documentation Index**](docs/INDEX.md)

### 🏁 Foundations
- [**Standalone Quickstart**](docs/STANDALONE_QUICKSTART.md): Minimal run path.
- [**Getting Started**](docs/GETTING_STARTED.md): Installation and basic workflow.
- [**Project Architecture**](docs/ARCHITECTURE.md): Dual‑plane design and core layout.

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
