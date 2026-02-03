# Getting Started with Sutra Engine

This guide will walk you through setting up Sutra Engine and making your first semantic search.

---

## 🏗 Installation

Sutra Engine is distributed as a single static binary. 

1. **Download**: Obtain the binary for your architecture (Linux/macOS/Windows).
2. **Setup**: Place it in a directory where you have write permissions (for data storage).
3. **Run**:
   ```bash
   chmod +x sutra-engine
   ./start-engine.sh
   ```

By default, the engine will create a `./data` directory to store its Persistent Knowledge Graph and HNSW indices.

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

### Using the Python Client

Install the requirements:
```bash
pip install msgpack
```

Create a script `hello_sutra.py`:

```python
from sutra_engine_client import SutraClient

client = SutraClient()

# 1. Ingest
print("Ingesting knowledge...")
cid = client.learn("The Burj Khalifa is the tallest building in the world.")

# 2. Search
print("Searching...")
results = client.search("tallest skyscraper")

for res in results:
    print(f"Result: {res['id']} (Score: {res['score']})")
```

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
