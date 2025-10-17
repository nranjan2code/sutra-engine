# Sutra Storage Client

Production-ready gRPC client for connecting to Sutra Storage Server.

## Installation

```bash
pip install -e packages/sutra-storage-client/
```

## Usage

```python
from sutra_storage_client import StorageClient

# Connect to storage server
client = StorageClient("localhost:50051")

# Learn concepts
client.learn_concept("concept1", "content", strength=1.0)

# Query concepts
concept = client.query_concept("concept1")

# Get stats
stats = client.stats()
print(f"Concepts: {stats['concepts']}, Edges: {stats['edges']}")

# Clean up
client.close()
```

## Environment Variables

- `SUTRA_STORAGE_SERVER`: Storage server address (default: "localhost:50051")
