# Sutra NLG Service

**Self-hosted natural language generation with strict grounding**

Version: 1.0.0 | Language: Python | License: MIT

---

## Overview

Production-grade NLG service using small, swappable LLMs (gemma-2-2b-it) for grounded text generation.

### Key Features

- **🏠 Self-Hosted**: No external API dependencies (no OpenAI, no Ollama)
- **🔄 Swappable Models**: Change model via environment variable
- **⚡ High Availability**: 3 replicas + HAProxy load balancer
- **🎯 Grounded Generation**: Constrained prompts prevent hallucinations
- **📊 Production Metrics**: Request tracking, latency monitoring
- **🔒 CPU-Optimized**: No GPU required

---

## Quick Start

### Local Development

```bash
# Install dependencies
pip install -r requirements.txt

# Run service
python main.py

# Test health
curl http://localhost:8889/health

# Test generation
curl -X POST http://localhost:8889/generate \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "FACTS:\n- Paris is the capital of France\n\nQUESTION: What is the capital of France?\n\nANSWER:",
    "max_tokens": 50,
    "temperature": 0.3
  }'
```

### Docker Deployment

```bash
# Build image
docker build -t sutra-nlg-service:latest .

# Run container
docker run -p 8889:8889 \
  -e NLG_MODEL=google/gemma-2-2b-it \
  -e INSTANCE_ID=nlg-1 \
  sutra-nlg-service:latest
```

---

## Swappable Models

Change model via `NLG_MODEL` environment variable:

```bash
# Default (recommended)
NLG_MODEL=google/gemma-2-2b-it        # 2B params, best quality

# Alternatives
NLG_MODEL=microsoft/phi-2              # 2.7B params
NLG_MODEL=TinyLlama/TinyLlama-1.1B-Chat-v1.0  # 1.1B params (faster)
NLG_MODEL=stabilityai/stablelm-2-1_6b  # 1.6B params
```

**No code changes required** - restart service after env change.

---

## API Endpoints

### `POST /generate`

Generate natural language from constrained prompt.

**Request:**
```json
{
  "prompt": "FACTS:\n- [verified facts]\n\nQUESTION: [user question]\n\nANSWER:",
  "max_tokens": 150,
  "temperature": 0.3,
  "stop_sequences": ["FACTS:", "QUESTION:"]
}
```

**Response:**
```json
{
  "text": "Generated answer using only the facts",
  "model": "google/gemma-2-2b-it",
  "processing_time_ms": 120.5,
  "tokens_generated": 45
}
```

### `GET /health`

Health check endpoint.

**Response:**
```json
{
  "status": "healthy",
  "model_loaded": true,
  "model_name": "google/gemma-2-2b-it",
  "device": "cpu",
  "instance_id": "nlg-1"
}
```

### `GET /metrics`

Service metrics.

**Response:**
```json
{
  "total_requests": 1523,
  "total_tokens_generated": 45678,
  "avg_generation_time_ms": 125.3,
  "model_name": "google/gemma-2-2b-it",
  "uptime_seconds": 3600.5
}
```

---

## Grounding Strategy

**CRITICAL:** Prompts must include verified facts to prevent hallucinations.

### Prompt Template

```
FACTS:
- [Verified fact 1 from graph reasoning]
- [Verified fact 2 from graph reasoning]
- [Verified fact 3 from graph reasoning]

QUESTION: [User's original question]

ANSWER (using ONLY the facts above):
```

### Post-Generation Validation

**Always validate** that generated text:
1. Uses only words/concepts from fact pool
2. Doesn't introduce new claims
3. Stops at appropriate boundaries

If validation fails, **fall back to template-based NLG**.

---

## Performance

| Model | Startup | Generation (50 tokens) | Memory |
|-------|---------|------------------------|--------|
| gemma-2-2b-it | ~30s | ~120ms | 4GB |
| phi-2 | ~40s | ~150ms | 5GB |
| TinyLlama-1.1B | ~20s | ~80ms | 3GB |

**Recommended:** gemma-2-2b-it for production (best quality/speed balance)

---

## High Availability

Deployed with 3 replicas + HAProxy load balancer:

```
User Request → HAProxy (8889) → Least-connection routing
                ├─ nlg-1:8889 (healthy)
                ├─ nlg-2:8889 (healthy)
                └─ nlg-3:8889 (healthy)
```

**Benefits:**
- Zero downtime during updates
- 3× capacity for load spikes
- Automatic failover (<10s)

---

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `PORT` | 8889 | Service port |
| `NLG_MODEL` | google/gemma-2-2b-it | Hugging Face model ID |
| `INSTANCE_ID` | nlg-default | Instance identifier (for logging) |

---

## License

MIT License

**Built with ❤️ by the Sutra AI Team**
