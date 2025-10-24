# Sutra API

**Production REST API for Sutra AI knowledge graph.**

Version: 1.0.0 | Framework: FastAPI | License: MIT

---

## Overview

Lightweight FastAPI service providing REST endpoints to the Sutra AI system via TCP storage client.

### Key Features

- **⚡ Fast**: Async FastAPI with rate limiting
- **🔒 Secure**: CORS, input validation
- **📊 Observable**: Health checks, metrics
- **🌐 Distributed**: TCP client to storage server

---

## Quick Start

```bash
# Environment
export SUTRA_STORAGE_SERVER=storage-server:50051

# Run
uvicorn sutra_api.main:app --host 0.0.0.0 --port 8000
```

---

## Endpoints

### Health

```bash
GET /health
```

### Learn

```bash
POST /learn
Content-Type: application/json

{
  "content": "The Eiffel Tower is in Paris",
  "source": "Wikipedia"
}
```

### Stats

```bash
GET /stats
```

---

## Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `SUTRA_STORAGE_SERVER` | `storage-server:50051` | Storage address |
| `SUTRA_RATE_LIMIT_LEARN` | `100` | Learn requests/min |
| `SUTRA_RATE_LIMIT_SEARCH` | `200` | Search requests/min |

---

## Docker

```yaml
sutra-api:
  image: sutra-api:latest
  ports:
    - "8000:8000"
  environment:
    - SUTRA_STORAGE_SERVER=storage-server:50051
```

---

## License

MIT License

**Built with ❤️ by the Sutra AI Team**
