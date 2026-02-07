# Getting Started with Sutra Engine

This guide walks you through setting up Sutra Engine and making your first vector search.
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
   # Defaults to port 50051. Use 9000 for NL examples.
   STORAGE_PORT=9000 cargo run --release --bin storage-server
   ```

By default, the engine will create a `./data` directory to store its graph index and HNSW vector indices.

---

## 💡 Core Concepts

### 1. Records
Every piece of information in Sutra is stored as a **Record**. A record has content (text), an optional vector embedding, and metadata.

### 2. Dual-Plane Storage
Sutra operates on two planes:
- **Vector Plane**: For fast, fuzzy semantic similarity search.
- **Graph Plane**: For exact, traversable relationships (edges between records).

---

## 🛠 Your First Operation

### Option A: Natural Language (Simplest)
1. Ensure server is running on port 9000 (`STORAGE_PORT=9000`).
2. Use `netcat` to talk to it:
   ```bash
   echo "Insert: the sky is blue" | nc localhost 9000
   echo "Search for: sky" | nc localhost 9000
   ```

### Option B: Binary Protocol (High Performance)
1. Open a TCP connection to `localhost:50051`.
2. Send a 4-byte big-endian length prefix.
3. Send the MessagePack-encoded request body.
4. Receive highly compact binary response.

See the [**API Reference**](API_REFERENCE.md) for full details.


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

## 🔧 Background Maintenance

Sutra includes configurable background jobs (enabled by default). Try these NL commands:

```bash
echo "status" | nc localhost 9000              # View engine stats
echo "set goal: track new records" | nc localhost 9000  # Create a trigger
echo "list goals" | nc localhost 9000           # List all triggers
echo "subscribe to Rust" | nc localhost 9000    # Subscribe to matching records
```

To disable background jobs (e.g., for benchmarking):
```bash
export SUTRA_AUTONOMY=false
```

---

## 🚀 Next Steps
- Explore the [**API Reference**](API_REFERENCE.md) for advanced queries.
- Read the [**Operations Guide**](OPERATIONS.md) for scaling, tuning, and backups.
