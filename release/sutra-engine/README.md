# Sutra Engine
> **High-Performance Explainable Memory Engine**

[![Release](https://img.shields.io/badge/release-v1.0.0-blue.svg)]()
[![License](https://img.shields.io/badge/license-MIT-green.svg)]()

**Sutra Engine** is a self-contained, high-performance graph memory engine written in Rust. It provides a lightweight, embeddable server for storing and reasoning over domain knowledge using vectors and graphs.

Think of it as **SQLite for Knowledge Graphs** and **AI Memory**.

---

## ⚡ Quick Start

### 1. Run the Engine

The engine is distributed as a single binary. No dependencies required.

```bash
# Start with default settings (port 50051, data in ./data)
./start-engine.sh
```

### 2. Connect & Use (Python)

```python
from sutra_engine_client import SutraClient

# Connect to the local engine
client = SutraClient("localhost:50051")

# Store knowledge
concept_id = client.learn("Sutra Engine is a high-performance memory system.")

# Retrieve
print(f"Stored Concept: {concept_id}")
```

---

## 📚 Documentation

*   **[API Reference](docs/API.md)**: Full protocol and method documentation.
*   **[Python Client Guide](docs/PYTHON_CLIENT.md)**: How to use the Python SDK.
*   **[Architecture](docs/ARCHITECTURE.md)**: Internals of the storage engine.

---

## ⚙️ Configuration

The engine is configured via environment variables, making it container-friendly.

| Variable | Default | Description |
|----------|---------|-------------|
| `STORAGE_HOST` | `0.0.0.0` | Bind address. |
| `STORAGE_PORT` | `50051` | TCP listening port. |
| `STORAGE_PATH` | `./data` | Persistence directory. |
| `MEMORY_THRESHOLD` | `50000` | Operations before auto-flush. |
| `SUTRA_SECURE_MODE` | `false` | Enable TLS 1.3/HMAC Auth. |

---

## 🚀 Deployment

### Docker

```dockerfile
FROM ubuntu:22.04
COPY sutra-engine /usr/local/bin/
CMD ["sutra-engine"]
```

### Systemd (Linux)

See `examples/sutra-engine.service` for a template service file.

---

## ⚖️ License

MIT License. See [LICENSE](LICENSE) for details.
