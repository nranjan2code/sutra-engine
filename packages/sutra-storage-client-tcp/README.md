# Sutra Storage Client (TCP)

**Python client for Sutra Storage Server via TCP binary protocol.**

Version: 1.0.0 | Language: Python | License: MIT

---

## Overview

High-performance TCP client for connecting to Sutra Storage Server using custom binary protocol.

### Key Features

- **⚡ Fast**: 10-50× faster than gRPC
- **🔌 Simple**: Single connection, automatic reconnect
- **🔒 Reliable**: Exponential backoff, error handling
- **📦 Binary**: bincode serialization

---

## Quick Start

```python
from sutra_storage_client_tcp import TcpStorageAdapter

client = TcpStorageAdapter(
    server_address="storage-server:50051",
    vector_dimension=768,
)

# Learn
concept_id = client.learn_concept_v2(
    content="The Eiffel Tower is in Paris",
    options={"generate_embedding": True},
)

# Query
concept = client.get_concept(concept_id)
print(concept)
```

---

## API Reference

```python
# Initialize
client = TcpStorageAdapter(
    server_address="localhost:50051",
    vector_dimension=768,
)

# Learn
concept_id = client.learn_concept_v2(
    content="text",
    options={
        "generate_embedding": True,
        "extract_associations": True,
    },
)

# Query
concept = client.get_concept(concept_id)
neighbors = client.get_neighbors(concept_id)
stats = client.stats()
```

---

## License

MIT License

**Built with ❤️ by the Sutra AI Team**
