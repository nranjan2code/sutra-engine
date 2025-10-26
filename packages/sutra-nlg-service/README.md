# Sutra NLG Service

**Self-hosted natural language generation with strict grounding**

Version: 1.0.0 | Language: Python | License: MIT

---

## Overview

Production-grade NLG service using small, swappable LLMs (gemma-3-270m-it) for grounded text generation.

### Key Features

- **🏠 Self-Hosted**: No external API dependencies (no OpenAI, no Ollama)
- **🔄 Swappable Models**: Change model via environment variable
- **⚡ High Availability**: 3 replicas + HAProxy load balancer
- **🎯 Grounded Generation**: Constrained prompts prevent hallucinations
- **📊 Production Metrics**: Request tracking, latency monitoring
- **🔒 CPU-Optimized**: No GPU required

---

## Quick Start

### Download Models (One-Time Setup)

**NEW:** Download and cache Gemma models locally to avoid downloading during container startup:

```bash
# Install dependencies (if not already installed)
pip install transformers torch

# Download all supported models (recommended)
python download_model.py --all

# Or download individual models
python download_model.py --model google/gemma-3-270m-it
python download_model.py --model google/gemma-2-2b-it
```

**Note:** Gemma models require HuggingFace authentication. Ensure `HF_TOKEN` is set in `.env.local` at project root.

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
  -e NLG_MODEL=google/gemma-3-270m-it \
  -e INSTANCE_ID=nlg-1 \
  sutra-nlg-service:latest
```

---

## Swappable Models

Change model via `NLG_MODEL` environment variable:

```bash
# Default (recommended for speed)
NLG_MODEL=google/gemma-3-270m-it      # 270M params, fast & efficient

# Alternative (better quality, slower)
NLG_MODEL=google/gemma-2-2b-it        # 2B params, higher quality

# Other alternatives
NLG_MODEL=microsoft/phi-2              # 2.7B params
NLG_MODEL=TinyLlama/TinyLlama-1.1B-Chat-v1.0  # 1.1B params
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
  "model": "google/gemma-3-270m-it",
  "processing_time_ms": 80.5,
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
  "model_name": "google/gemma-3-270m-it",
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
  "avg_generation_time_ms": 85.3,
  "model_name": "google/gemma-3-270m-it",
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

### Model Caching

| Scenario | Startup Time | Download Size | Notes |
|----------|--------------|---------------|-------|
| **Cached (recommended)** | 15-30s | 0 MB | ✅ Models pre-downloaded |
| **Download on startup** | 2-5 min | 5 GB | ⚠️ Slow first start |

**Benefit:** Using cached models (via `download_model.py`) reduces startup time by **90%** and eliminates network dependency.

## Performance

| Model | Startup | Generation (50 tokens) | Memory |
|-------|---------|------------------------|--------|
| gemma-3-270m-it | ~20s | ~80ms | 1GB |
| gemma-2-2b-it | ~30s | ~120ms | 4GB |
| phi-2 | ~40s | ~150ms | 5GB |
| TinyLlama-1.1B | ~20s | ~80ms | 3GB |

**Recommended:** gemma-3-270m-it for production (best speed/efficiency balance)

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
| `NLG_MODEL` | google/gemma-3-270m-it | Hugging Face model ID |
| `INSTANCE_ID` | nlg-default | Instance identifier (for logging) |

---

## License

MIT License

**Built with ❤️ by the Sutra AI Team**
