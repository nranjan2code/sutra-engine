# NLG Service API

**Service:** `sutra-nlg-service`  
**Port:** 8890  
**Version:** 2.0.0  
**Last Updated:** October 27, 2025

## Overview

The Sutra Natural Language Generation (NLG) Service provides grounded text generation using state-of-the-art language models. Built on the ML Foundation (`sutra-ml-base`), it generates contextually relevant responses grounded in your knowledge graph with configurable creativity levels.

**Key Features:**
- **Grounded generation** from knowledge graph concepts
- **Multi-mode generation** (strict, balanced, creative)
- **Edition-aware model selection** and resource limits
- **Advanced prompt engineering** with context validation
- **Streaming responses** for real-time applications
- **Safety filters** and content validation

## Base URL

```
http://localhost:8890  # Development
https://nlg.yourdomain.com  # Production
```

## Authentication

### Development Mode (Default)
No authentication required - service runs on localhost only.

### Production Mode
JWT authentication required when `SUTRA_SECURE_MODE=true`:

```bash
# Include JWT token in requests
curl -H "Authorization: Bearer ${JWT_TOKEN}" \
     http://localhost:8890/generate
```

## Endpoints

### Core Generation Endpoints

#### `POST /generate`
Generate grounded text based on knowledge graph context and user query.

**Request:**
```json
{
  "query": "Explain the relationship between machine learning and artificial intelligence",
  "context_concepts": ["machine_learning", "artificial_intelligence", "neural_networks"],
  "grounding_mode": "balanced",
  "max_length": 500,
  "temperature": 0.7,
  "stream": false
}
```

**Parameters:**
- `query` (string, required): User query or prompt for generation
- `context_concepts` (array[string], optional): Concept IDs from knowledge graph to ground generation
- `grounding_mode` (string, optional): Generation mode - "strict", "balanced", or "creative" (default: "balanced")
- `max_length` (integer, optional): Maximum response length in tokens
- `temperature` (float, optional): Creativity level (0.0-2.0, default varies by mode)
- `stream` (boolean, optional): Enable streaming response (default: false)

**Response:**
```json
{
  "generated_text": "Machine learning is a subset of artificial intelligence that focuses on...",
  "grounding_sources": [
    {
      "concept_id": "machine_learning",
      "confidence": 0.95,
      "content": "Machine learning involves algorithms that learn from data..."
    }
  ],
  "generation_metadata": {
    "model": "microsoft/DialoGPT-medium",
    "grounding_mode": "balanced",
    "temperature": 0.7,
    "tokens_generated": 87,
    "processing_time_ms": 234.5,
    "safety_score": 0.98
  },
  "confidence_score": 0.89
}
```

#### `POST /generate/stream`
Generate grounded text with streaming response for real-time applications.

**Request:**
Same as `/generate` with `stream: true` implied.

**Response:**
Server-Sent Events (SSE) stream:

```
data: {"type": "start", "generation_id": "gen_123456"}

data: {"type": "token", "text": "Machine", "confidence": 0.92}

data: {"type": "token", "text": " learning", "confidence": 0.94}

data: {"type": "grounding", "concept": {"id": "ml", "confidence": 0.95}}

data: {"type": "complete", "total_tokens": 87, "final_confidence": 0.89}
```

#### `POST /validate`
Validate and analyze prompts before generation.

**Request:**
```json
{
  "query": "Tell me about quantum computing applications",
  "context_concepts": ["quantum_computing", "applications"]
}
```

**Response:**
```json
{
  "validation_result": "valid",
  "safety_score": 0.98,
  "grounding_coverage": 0.85,
  "suggested_concepts": ["quantum_algorithms", "quantum_supremacy"],
  "estimated_response_length": 450,
  "recommendations": {
    "grounding_mode": "balanced",
    "temperature": 0.6
  }
}
```

### Standard ML Foundation Endpoints

The NLG service inherits these endpoints from `BaseMlService`:

#### `GET /health`
Service health check.

#### `GET /health/detailed`
Detailed health including model status and knowledge graph connectivity.

#### `GET /metrics`
Prometheus metrics for monitoring.

#### `GET /info`
Service information and edition limits.

**See [ML Foundation API](./ML_FOUNDATION_API.md) for complete documentation of standard endpoints.**

## Grounding Modes

The NLG service supports three grounding modes that control how strictly responses adhere to knowledge graph content:

### Strict Mode
- **Use case:** Regulatory compliance, medical/legal domains
- **Behavior:** Only generate content directly supported by grounded concepts
- **Temperature:** 0.1-0.3 (low creativity)
- **Hallucination risk:** Minimal
- **Response time:** Fast (smaller search space)

```json
{
  "grounding_mode": "strict",
  "temperature": 0.2
}
```

### Balanced Mode (Default)
- **Use case:** General knowledge applications, customer support
- **Behavior:** Blend grounded facts with reasonable inferences
- **Temperature:** 0.5-0.8 (moderate creativity)
- **Hallucination risk:** Low
- **Response time:** Medium

```json
{
  "grounding_mode": "balanced", 
  "temperature": 0.7
}
```

### Creative Mode
- **Use case:** Content creation, brainstorming, education
- **Behavior:** Use grounded concepts as inspiration for creative expansion
- **Temperature:** 0.8-1.2 (high creativity)
- **Hallucination risk:** Moderate (monitored)
- **Response time:** Slower (larger search space)

```json
{
  "grounding_mode": "creative",
  "temperature": 1.0
}
```

## Edition-Specific Features

### Simple Edition
- **Model:** DialoGPT-small (117M parameters)
- **Max response length:** 200 tokens
- **Concurrent requests:** 2
- **Grounding modes:** Strict only
- **Rate limiting:** 50 requests/minute

```bash
export SUTRA_EDITION=simple
python -m sutra_nlg_service.main
```

### Community Edition
- **Model:** DialoGPT-medium (345M parameters)
- **Max response length:** 500 tokens
- **Concurrent requests:** 10
- **Grounding modes:** Strict, Balanced
- **Streaming:** Supported
- **Rate limiting:** 200 requests/minute

```bash
export SUTRA_EDITION=community
python -m sutra_nlg_service.main
```

### Enterprise Edition
- **Model:** DialoGPT-large (762M parameters) + custom models
- **Max response length:** 1000 tokens
- **Concurrent requests:** 50
- **Grounding modes:** All modes (Strict, Balanced, Creative)
- **Streaming:** Advanced with confidence scoring
- **Custom prompts:** Template customization
- **Rate limiting:** 1000 requests/minute

```bash
export SUTRA_EDITION=enterprise
export SUTRA_LICENSE_KEY=your-enterprise-key
python -m sutra_nlg_service.main
```

## Error Handling

### Error Response Format

```json
{
  "error": {
    "code": "INSUFFICIENT_GROUNDING",
    "message": "Not enough grounded concepts found for reliable generation",
    "details": {
      "query": "Tell me about quantum teleportation",
      "concepts_found": 0,
      "minimum_required": 1,
      "suggested_action": "Add quantum physics concepts to knowledge graph"
    }
  },
  "timestamp": "2025-10-27T10:30:00Z",
  "request_id": "nlg_123456789"
}
```

### Error Codes

| Code | Description | HTTP Status | Solution |
|------|-------------|-------------|----------|
| `INVALID_QUERY` | Empty or malformed query | 400 | Provide valid query text |
| `QUERY_TOO_LONG` | Query exceeds length limit | 400 | Shorten query or upgrade edition |
| `INSUFFICIENT_GROUNDING` | No relevant concepts found | 400 | Add concepts to knowledge graph |
| `UNSAFE_CONTENT` | Content failed safety filters | 400 | Modify query to comply with policies |
| `RATE_LIMIT_EXCEEDED` | Too many requests | 429 | Wait or upgrade edition |
| `MODEL_NOT_LOADED` | NLG model unavailable | 503 | Check service logs |
| `GENERATION_TIMEOUT` | Response generation timed out | 504 | Reduce complexity or retry |

## Integration Examples

### Python Client

```python
import asyncio
import aiohttp
from typing import List, Dict, Optional

class NLGClient:
    def __init__(self, base_url: str = "http://localhost:8890"):
        self.base_url = base_url
        
    async def generate_text(
        self, 
        query: str,
        context_concepts: Optional[List[str]] = None,
        grounding_mode: str = "balanced",
        max_length: int = 500
    ) -> Dict:
        """Generate grounded text response"""
        async with aiohttp.ClientSession() as session:
            async with session.post(
                f"{self.base_url}/generate",
                json={
                    "query": query,
                    "context_concepts": context_concepts or [],
                    "grounding_mode": grounding_mode,
                    "max_length": max_length
                }
            ) as response:
                return await response.json()
    
    async def stream_generation(
        self,
        query: str,
        context_concepts: Optional[List[str]] = None
    ):
        """Stream text generation in real-time"""
        async with aiohttp.ClientSession() as session:
            async with session.post(
                f"{self.base_url}/generate/stream",
                json={
                    "query": query,
                    "context_concepts": context_concepts or [],
                    "stream": True
                }
            ) as response:
                async for line in response.content:
                    if line.startswith(b"data: "):
                        data = json.loads(line[6:])
                        yield data

# Usage example
async def main():
    client = NLGClient()
    
    # Simple generation
    result = await client.generate_text(
        query="Explain machine learning basics",
        context_concepts=["machine_learning", "algorithms"],
        grounding_mode="balanced"
    )
    print(f"Generated: {result['generated_text']}")
    
    # Streaming generation
    async for token_data in client.stream_generation(
        query="What are the applications of AI?",
        context_concepts=["artificial_intelligence", "applications"]
    ):
        if token_data["type"] == "token":
            print(token_data["text"], end="", flush=True)

asyncio.run(main())
```

### cURL Examples

```bash
# Basic text generation
curl -X POST http://localhost:8890/generate \
  -H "Content-Type: application/json" \
  -d '{
    "query": "What is machine learning?",
    "context_concepts": ["machine_learning", "algorithms"],
    "grounding_mode": "balanced",
    "max_length": 300
  }'

# Streaming generation
curl -X POST http://localhost:8890/generate/stream \
  -H "Content-Type: application/json" \
  -d '{
    "query": "Explain neural networks",
    "context_concepts": ["neural_networks", "deep_learning"]
  }'

# Validate prompt
curl -X POST http://localhost:8890/validate \
  -H "Content-Type: application/json" \
  -d '{
    "query": "Tell me about quantum computing",
    "context_concepts": ["quantum_computing"]
  }'

# Service information
curl http://localhost:8890/info
```

### JavaScript/React Integration

```javascript
import React, { useState, useEffect } from 'react';

function NLGGenerator() {
    const [query, setQuery] = useState('');
    const [response, setResponse] = useState('');
    const [streaming, setStreaming] = useState(false);
    
    const generateText = async () => {
        try {
            const response = await fetch('http://localhost:8890/generate', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    query: query,
                    grounding_mode: 'balanced',
                    max_length: 500
                })
            });
            
            const result = await response.json();
            setResponse(result.generated_text);
        } catch (error) {
            console.error('Generation failed:', error);
        }
    };
    
    const streamGeneration = async () => {
        setStreaming(true);
        setResponse('');
        
        try {
            const response = await fetch('http://localhost:8890/generate/stream', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    query: query,
                    grounding_mode: 'balanced'
                })
            });
            
            const reader = response.body.getReader();
            const decoder = new TextDecoder();
            
            while (true) {
                const { done, value } = await reader.read();
                if (done) break;
                
                const chunk = decoder.decode(value);
                const lines = chunk.split('\n');
                
                for (const line of lines) {
                    if (line.startsWith('data: ')) {
                        const data = JSON.parse(line.substring(6));
                        if (data.type === 'token') {
                            setResponse(prev => prev + data.text);
                        }
                    }
                }
            }
        } catch (error) {
            console.error('Streaming failed:', error);
        } finally {
            setStreaming(false);
        }
    };
    
    return (
        <div>
            <textarea
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                placeholder="Enter your query..."
                rows={4}
                cols={50}
            />
            <br />
            <button onClick={generateText} disabled={streaming}>
                Generate
            </button>
            <button onClick={streamGeneration} disabled={streaming}>
                Stream Generate
            </button>
            
            <div style={{ marginTop: '20px', minHeight: '200px', border: '1px solid #ccc', padding: '10px' }}>
                <strong>Response:</strong>
                <p>{response}</p>
            </div>
        </div>
    );
}

export default NLGGenerator;
```

## Advanced Features

### Custom Prompt Templates (Enterprise)

Customize generation with domain-specific templates:

```python
# Register custom template
await client.post("/templates", json={
    "template_id": "medical_diagnosis",
    "template": """
    Based on the following medical knowledge: {grounded_concepts}
    
    Patient Query: {user_query}
    
    Provide a careful, evidence-based response that:
    1. References specific medical concepts
    2. Includes appropriate disclaimers
    3. Suggests consulting healthcare professionals
    
    Response:
    """,
    "safety_level": "strict"
})

# Use custom template
result = await client.generate_text(
    query="I have chest pain, what could it be?",
    context_concepts=["chest_pain", "cardiology", "emergency_symptoms"],
    template_id="medical_diagnosis"
)
```

### Confidence-Based Filtering

Filter responses based on confidence thresholds:

```json
{
  "query": "Explain quantum entanglement",
  "context_concepts": ["quantum_physics", "entanglement"],
  "min_confidence": 0.8,
  "fallback_response": "I don't have enough reliable information about quantum entanglement to provide a confident answer."
}
```

### Multi-Language Support (Enterprise)

Generate responses in multiple languages:

```json
{
  "query": "Explain artificial intelligence",
  "context_concepts": ["artificial_intelligence"],
  "target_language": "es",  // Spanish
  "preserve_technical_terms": true
}
```

## Monitoring & Observability

### Key Metrics

Monitor these Prometheus metrics for optimal performance:

```text
# Generation metrics
sutra_nlg_requests_total{grounding_mode="strict|balanced|creative"}
sutra_nlg_generation_duration_seconds
sutra_nlg_token_count_histogram

# Quality metrics
sutra_nlg_confidence_score_histogram
sutra_nlg_safety_score_histogram
sutra_nlg_grounding_coverage_histogram

# Resource metrics
sutra_nlg_model_memory_usage_bytes
sutra_nlg_gpu_utilization_percent
sutra_nlg_concurrent_generations
```

### Grafana Dashboard

Example dashboard queries:

```promql
# Generation rate by mode
sum(rate(sutra_nlg_requests_total[5m])) by (grounding_mode)

# Average confidence score
avg(sutra_nlg_confidence_score_histogram)

# 95th percentile generation time
histogram_quantile(0.95, sutra_nlg_generation_duration_seconds)

# Safety score distribution
histogram_quantile(0.5, sutra_nlg_safety_score_histogram)
```

### Alerts

```yaml
# Low confidence generation
- alert: NLGLowConfidenceGeneration
  expr: avg(sutra_nlg_confidence_score_histogram) < 0.7
  for: 5m
  
# High generation latency
- alert: NLGHighLatency
  expr: histogram_quantile(0.95, sutra_nlg_generation_duration_seconds) > 5
  for: 2m
  
# Safety filter violations
- alert: NLGSafetyViolations
  expr: rate(sutra_nlg_requests_total{status="unsafe"}[5m]) > 0.01
  for: 1m
```

## Deployment

### Docker

```dockerfile
# Use official ML Foundation image
FROM sutra/ml-base:2.0.0

# Copy NLG service code
COPY packages/sutra-nlg-service/ /app/
WORKDIR /app

# Install dependencies
RUN pip install -e .

# Download models
RUN python -c "from transformers import AutoModel; AutoModel.from_pretrained('microsoft/DialoGPT-medium')"

# Expose port
EXPOSE 8890

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=120s \
  CMD curl -f http://localhost:8890/health || exit 1

# Start service
CMD ["python", "-m", "sutra_nlg_service.main"]
```

### Docker Compose

```yaml
version: '3.8'
services:
  nlg-service:
    image: sutra/nlg-service:2.0.0
    ports:
      - "8890:8890"
    environment:
      - SUTRA_EDITION=${SUTRA_EDITION:-simple}
      - SUTRA_LICENSE_KEY=${SUTRA_LICENSE_KEY}
      - SUTRA_STORAGE_URL=tcp://storage-server:7000
    volumes:
      - ./models:/app/models
      - ./templates:/app/templates
    depends_on:
      - storage-server
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8890/health"]
      interval: 30s
      timeout: 10s
      retries: 3
    deploy:
      resources:
        limits:
          memory: 4G
        reservations:
          memory: 2G
```

## Safety & Content Filtering

### Built-in Safety Filters

The NLG service includes comprehensive safety mechanisms:

1. **Content Classification**
   - Harmful/toxic content detection
   - PII (Personally Identifiable Information) filtering
   - Bias detection and mitigation

2. **Medical/Legal Disclaimers**
   - Automatic disclaimer injection for health/legal queries
   - Professional consultation recommendations
   - Liability protection statements

3. **Hallucination Detection**
   - Cross-reference with grounded concepts
   - Confidence scoring for factual claims
   - "I don't know" responses for uncertain content

### Configuration

```json
{
  "safety_config": {
    "enable_content_filter": true,
    "enable_pii_filter": true,
    "enable_bias_detection": true,
    "min_safety_score": 0.8,
    "auto_disclaimers": {
      "medical": true,
      "legal": true,
      "financial": true
    }
  }
}
```

## Troubleshooting

### Common Issues

#### Model Loading Errors
```bash
# Check model files and cache
ls -la ~/.cache/huggingface/transformers/
python -c "from transformers import AutoTokenizer; AutoTokenizer.from_pretrained('microsoft/DialoGPT-medium')"

# Clear corrupted cache
rm -rf ~/.cache/huggingface/transformers/
```

#### Generation Quality Issues
```bash
# Check grounding coverage
curl -X POST http://localhost:8890/validate \
  -d '{"query": "your query", "context_concepts": ["concept1", "concept2"]}'

# Monitor confidence scores
curl http://localhost:8890/metrics | grep confidence

# Test different grounding modes
curl -X POST http://localhost:8890/generate \
  -d '{"query": "test", "grounding_mode": "strict"}'
```

#### Performance Issues
```bash
# Check concurrent generation count
curl http://localhost:8890/metrics | grep concurrent

# Monitor memory usage
docker stats sutra-nlg-service

# Profile generation time
time curl -X POST http://localhost:8890/generate \
  -d '{"query": "test query", "max_length": 100}'
```

---

## Changelog

### 2.0.0 (2025-10-27)
- ✅ Complete rewrite using ML Foundation
- ✅ Multi-mode grounded generation (strict/balanced/creative)
- ✅ Edition-aware model selection and limits
- ✅ Advanced streaming with confidence scoring
- ✅ Comprehensive safety filtering and content validation
- ✅ Production-ready monitoring and deployment guides