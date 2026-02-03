# Standalone Engine Releases

Sutra Memory allows you to release the core storage engine as a single, standalone binary. This is ideal for embedded use cases, similar to how SQLite or DuckDB are distributed, where you simply need the high-performance storage and reasoning engine without the full web stack or distributed architecture.

## 📦 What is the Standalone Engine?

The Standalone Engine is a single binary executable (`sutra-engine`) that contains:

*   **Core Storage Engine**: The Rust-based, lock-free concurrent memory system.
*   **TCP Server**: High-performance binary protocol server (10-50x faster than REST).
*   **HNSW Vector Index**: Embedded vector search capabilities.
*   **Adaptive Reconciler**: Background maintenance and optimization.
*   **WAL (Write-Ahead Log)**: Crash recovery and data persistence.

It does **NOT** contain:
*   Python/Javascript bindings (though you can connect to it using them).
*   Web UI or Dashboard.
*   Distributed consensus or sharding logic (unless configured for sharded mode).

## 🚀 Building a Release

We provide a script to automatically build and package the standalone engine:

```bash
./scripts/release_storage_engine.sh
```

This script will:
1.  Compile the `storage-server` binary in release mode (optimizing for speed).
2.  Create a `release/sutra-engine` directory.
3.  Package the binary, a README, and a convenience start script.

### Output Structure

The release directory (`release/sutra-engine/`) will contain:

```text
sutra-engine/
├── sutra-engine       # The standalone binary
├── start-engine.sh    # Convenience launch script
└── README.md          # Usage instructions
```

## 🛠️ Usage

You can run the engine directly:

```bash
./sutra-engine
```

### Configuration

The engine is configured entirely via environment variables, making it easy to embed in Docker containers or other management systems.

| Variable | Default | Description |
|----------|---------|-------------|
| `STORAGE_HOST` | `0.0.0.0` | IP address to bind to. |
| `STORAGE_PORT` | `50051` | TCP port to listen on. |
| `STORAGE_PATH` | `./data` | Directory where data/WAL will be stored. |
| `MEMORY_THRESHOLD` | `50000` | Number of operations before flushing to disk. |
| `SUTRA_SECURE_MODE` | `false` | Set to `true` to enable TLS and Auth. |

### Example: Embedded in a Python Project

If you are building a Python application that needs a high-performance memory engine, you can spawn the `sutra-engine` binary as a subprocess:

```python
import subprocess
import os

# Start the engine
env = os.environ.copy()
env["STORAGE_PORT"] = "5555"
env["STORAGE_PATH"] = "./my_app_memory"

process = subprocess.Popen(["./sutra-engine"], env=env)

# Connecting using the Python client
from sutra_core import SutraClient
client = SutraClient(port=5555)

# ... use the client ...

# Cleanup
process.terminate()
```

## 🔄 Cross-Platform Builds

To build for other platforms (e.g., Linux from macOS), you will need the appropriate Rust targets installed:

```bash
# Add Linux target
rustup target add x86_64-unknown-linux-gnu

# Build (may require cross-linker)
cargo build --release --target x86_64-unknown-linux-gnu --bin storage-server -p sutra-storage
```

## 📦 Distribution Strategy

For distributing this engine to users, we recommend:

1.  **GitHub Releases**: Attach the compiled binaries for Windows, macOS, and Linux to your GitHub Releases.
2.  **Versioning**: Ensure the binary version matches the semantic version of the core library.
3.  **Checksums**: Always provide SHA256 checksums for the binaries so users can verify integrity.

### Sub-Repo for Releases

For managing regular binary releases, consider creating a dedicated repository (e.g., `sutraworks/sutra-engine-releases`) that contains:
-   CI/CD pipelines to build binaries for all platforms.
-   Tagged releases with artifacts.
-   Installation scripts (e.g., `install.sh`) that fetch the correct binary for the user's OS.
