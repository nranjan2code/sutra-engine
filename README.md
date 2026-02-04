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

## 📚 Documentation (Exhaustive)

We've provided comprehensive guides for every aspect of the engine:

### 🏁 Foundations
- [**Getting Started**](docs/GETTING_STARTED.md): Installation and basic workflow.
- [**Project Architecture**](docs/ARCHITECTURE.md): Deep dive into the "Dual-Plane" design.

### 🛠 Developers
- [**API Reference**](docs/API_REFERENCE.md): Full specification of the binary protocol.
- [**Client Guide (Rust)**](docs/CLIENT_RUST.md): Native high-performance usage.
- [**Client Guide (TypeScript)**](docs/CLIENT_TYPESCRIPT.md): Node.js integration.

### 🛡 Production & Ops
- [**Security Manual**](docs/SECURITY.md): TLS 1.3, HMAC Auth, and Production Hardening.
- [**Operations & Tuning**](docs/OPERATIONS.md): Persistence, Sharding, and Performance Optimization.
- [**Monitoring**](docs/MONITORING.md): Interpreting engine statistics and health checks.

### 🆘 Support
- [**Troubleshooting**](docs/TROUBLESHOOTING.md): Common error codes and solutions.
- [**FAQ**](docs/FAQ.md): Frequently Asked Questions.

---

## 📁 Examples

The `crates/storage/examples` directory contains ready-to-run projects for:
- 🦀 **Rust**: Native storage client and server examples.
- 🟦 **TypeScript**: TCP client sample.

---

## ⚖️ License

Distributed under the MIT License. See `LICENSE` for more information.

---
<div align="center">
  Built with ❤️ by the Sutra Works Team.
</div>
