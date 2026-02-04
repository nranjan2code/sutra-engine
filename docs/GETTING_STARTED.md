# Getting Started with Sutra Engine

This guide walks you through setting up Sutra Engine and making your first semantic search.
If you want the shortest path, use `docs/STANDALONE_QUICKSTART.md`.

---

## 🏗 Installation

Sutra Engine is built using Rust.

1. **Clone the repository**:
   ```bash
   git clone https://github.com/nranjan2code/sutra-engine.git
   cd sutra-engine
   ```

2. **Build the binary**:
   ```bash
   cargo build --release --bin storage-server
   ```

3. **Run the engine**:
   ```bash
   ./target/release/storage-server
   ```

By default, the engine will create a `./data` directory (as configured in your environment or defaults) to store its Persistent Knowledge Graph and HNSW indices.

---

## 💡 Core Concepts

### 1. Concepts
Every piece of information in Sutra is stored as a **Concept**. A concept has content (text), an optional vector embedding, and metadata.

### 2. Dual-Plane Memory
Sutra operates on two planes:
- **Vector Plane**: For fast, fuzzy semantic similarity.
- **Graph Plane**: For exact, explainable relationships (links between concepts).

---

## 🛠 Your First Operation

The engine uses a custom binary protocol. You can interact with it using any TCP client that supports `bincode` serialization.

### Basic Request Flow:
1. Open a TCP connection to `localhost:50051`.
2. Send a 4-byte big-endian length prefix.
3. Send the `bincode`-encoded request body.

See the [**API Reference**](API_REFERENCE.md) for the full list of available commands and response formats.


---

## 🛡 Enabling Security

For production, you **must** enable security mode.

1. **Generate a Secret**:
   ```bash
   export SUTRA_AUTH_SECRET="your-very-long-and-secure-secret-32-chars"
   ```
2. **Enable Secure Mode**:
   ```bash
   export SUTRA_SECURE_MODE=true
   ./start-engine.sh
   ```

Now, all clients must provide a valid HMAC signature to interact with the engine.

---

## 🚀 Next Steps
- Explore the [**API Reference**](API_REFERENCE.md) for advanced queries.
- Read the [**Operations Guide**](OPERATIONS.md) for scaling and backups.
