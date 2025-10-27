# ML Services Deployment Guide

## Overview

This guide covers deploying the next-generation ML services built on the `sutra-ml-base` foundation. These services provide edition-aware scaling, advanced caching, and consistent APIs across all Sutra editions.

## Prerequisites

### System Requirements

| Edition | RAM | CPU | Storage | GPU |
|---------|-----|-----|---------|-----|
| **Simple** | 4GB | 2 cores | 10GB | Not required |
| **Community** | 8GB | 4 cores | 20GB | Not required |
| **Enterprise** | 16GB | 8 cores | 50GB | Optional |

### Software Dependencies

```bash
# Required
Docker >= 20.10
Docker Compose >= 2.0
Python >= 3.11

# Optional (for development)
pip >= 22.0
git >= 2.30
```

## Quick Deployment

### 1. Install ML Foundation

```bash
# Install the ML foundation locally
cd packages/sutra-ml-base
pip install -e .

# Verify installation
python -c "from sutra_ml_base import BaseMlService; print('✅ ML Foundation installed')"
```

### 2. Build ML Services

```bash
# Build both embedding and NLG services
./sutra-optimize.sh build-ml

# Or build individually
./sutra-optimize.sh build embedding
./sutra-optimize.sh build nlg
```

### 3. Deploy Services

```bash
# Deploy with specific edition
SUTRA_EDITION=community ./sutra-deploy.sh install

# Or using Docker Compose directly
SUTRA_EDITION=enterprise docker-compose -f docker-compose-grid.yml up -d
```

### 4. Verify Deployment

```bash
# Check embedding service
curl -s http://localhost:8888/health | jq

# Check NLG service  
curl -s http://localhost:8889/health | jq

# Test embedding generation
curl -X POST http://localhost:8888/embed \
  -H "Content-Type: application/json" \
  -d '{"texts": ["Hello world"], "normalize": true}' | jq

# Test text generation
curl -X POST http://localhost:8889/generate \
  -H "Content-Type: application/json" \
  -d '{"prompt": "FACTS: Paris is in France. QUESTION: Where is Paris? ANSWER:", "max_tokens": 50}' | jq
```

## Edition Configuration

### Environment Variables

```bash
# Edition selection (affects all ML services)
SUTRA_EDITION=simple|community|enterprise

# Service-specific configuration
SUTRA_EMBEDDING_PORT=8888
SUTRA_NLG_PORT=8889
LOG_LEVEL=INFO

# Model configuration (optional overrides)
EMBEDDING_MODEL_OVERRIDE=custom-model-name
NLG_MODEL_OVERRIDE=custom-nlg-model
```

### Edition Comparison

| Feature | Simple | Community | Enterprise |
|---------|--------|-----------|------------|
| **Embedding Batch Size** | 32 | 64 | 128 |
| **NLG Max Tokens** | 128 | 256 | 512 |
| **Cache Size** | 128MB | 256MB | 512MB |
| **Advanced Caching** | ❌ | ✅ | ✅ |
| **Custom Models** | ❌ | ❌ | ✅ |
| **Priority Support** | ❌ | ❌ | ✅ |

## Docker Deployment

### Using Docker Compose

```yaml
# docker-compose.ml-services.yml
version: '3.8'
services:
  sutra-embedding-service:
    build:
      context: .
      dockerfile: packages/sutra-embedding-service/Dockerfile
    image: sutra-embedding-service:${SUTRA_VERSION:-latest}
    ports:
      - "8888:8888"
    environment:
      - SUTRA_EDITION=${SUTRA_EDITION:-simple}
      - LOG_LEVEL=${LOG_LEVEL:-INFO}
    healthcheck:
      test: ["CMD", "python", "-c", "import requests; requests.get('http://localhost:8888/health').raise_for_status()"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 60s
    networks:
      - sutra-network

  sutra-nlg-service:
    build:
      context: .
      dockerfile: packages/sutra-nlg-service/Dockerfile  
    image: sutra-nlg-service:${SUTRA_VERSION:-latest}
    ports:
      - "8889:8889"
    environment:
      - SUTRA_EDITION=${SUTRA_EDITION:-simple}
      - LOG_LEVEL=${LOG_LEVEL:-INFO}
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8889/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 120s
    networks:
      - sutra-network

networks:
  sutra-network:
    driver: bridge
```

### Deploy ML Services

```bash
# Deploy with edition
SUTRA_EDITION=community docker-compose -f docker-compose.ml-services.yml up -d

# Scale embedding service for high throughput
docker-compose -f docker-compose.ml-services.yml up -d --scale sutra-embedding-service=3

# Check status
docker-compose -f docker-compose.ml-services.yml ps
```

## Kubernetes Deployment

### ML Foundation ConfigMap

```yaml
# k8s/ml-foundation-config.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: ml-foundation-config
data:
  SUTRA_EDITION: "community"
  LOG_LEVEL: "INFO"
  SUTRA_ML_CACHE_DIR: "/tmp/cache"
  SUTRA_ML_MODEL_VERIFICATION: "true"
```

### Embedding Service Deployment

```yaml
# k8s/embedding-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: sutra-embedding-service
spec:
  replicas: 2
  selector:
    matchLabels:
      app: sutra-embedding-service
  template:
    metadata:
      labels:
        app: sutra-embedding-service
    spec:
      containers:
      - name: embedding
        image: sutra-embedding-service:latest
        ports:
        - containerPort: 8888
        env:
        - name: SUTRA_EDITION
          valueFrom:
            configMapKeyRef:
              name: ml-foundation-config
              key: SUTRA_EDITION
        - name: LOG_LEVEL
          valueFrom:
            configMapKeyRef:
              name: ml-foundation-config
              key: LOG_LEVEL
        resources:
          requests:
            memory: "2Gi"
            cpu: "1"
          limits:
            memory: "4Gi" 
            cpu: "2"
        livenessProbe:
          httpGet:
            path: /health
            port: 8888
          initialDelaySeconds: 60
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /health
            port: 8888
          initialDelaySeconds: 30
          periodSeconds: 10
---
apiVersion: v1
kind: Service
metadata:
  name: sutra-embedding-service
spec:
  selector:
    app: sutra-embedding-service
  ports:
  - port: 8888
    targetPort: 8888
  type: ClusterIP
```

### Deploy to Kubernetes

```bash
# Apply ML foundation config
kubectl apply -f k8s/ml-foundation-config.yaml

# Deploy embedding service
kubectl apply -f k8s/embedding-deployment.yaml

# Deploy NLG service
kubectl apply -f k8s/nlg-deployment.yaml

# Check deployment status
kubectl get deployments
kubectl get pods -l app=sutra-embedding-service
kubectl get services

# Test services
kubectl port-forward service/sutra-embedding-service 8888:8888 &
curl -s http://localhost:8888/health
```

## Production Configuration

### Resource Limits

```yaml
# Production resource configuration
services:
  sutra-embedding-service:
    deploy:
      resources:
        limits:
          memory: 4G
          cpus: '2.0'
        reservations:
          memory: 2G
          cpus: '1.0'
      replicas: 3
      restart_policy:
        condition: on-failure
        delay: 10s
        max_attempts: 5

  sutra-nlg-service:
    deploy:
      resources:
        limits:
          memory: 6G
          cpus: '2.0'
        reservations:
          memory: 3G
          cpus: '1.0'
      replicas: 2
```

### High Availability Setup

```bash
# Deploy with HA configuration
docker-compose -f docker-compose-grid.yml --profile production up -d

# Load balancer for embedding service (if multiple replicas)
# HAProxy configuration
cat > haproxy-ml.cfg << 'EOF'
global
    daemon

defaults
    mode http
    timeout connect 5000ms
    timeout client 50000ms
    timeout server 50000ms

frontend embedding-frontend
    bind *:8888
    default_backend embedding-backend

backend embedding-backend
    balance leastconn
    option httpchk GET /health
    server embedding-1 sutra-embedding-service-1:8888 check
    server embedding-2 sutra-embedding-service-2:8888 check
    server embedding-3 sutra-embedding-service-3:8888 check

frontend nlg-frontend
    bind *:8889
    default_backend nlg-backend

backend nlg-backend
    balance leastconn
    option httpchk GET /health
    server nlg-1 sutra-nlg-service-1:8889 check
    server nlg-2 sutra-nlg-service-2:8889 check
EOF

# Add HAProxy to docker-compose
```

### Monitoring & Observability

```yaml
# Monitoring setup
services:
  sutra-embedding-service:
    environment:
      - ENABLE_METRICS=true
      - METRICS_PORT=9090
    ports:
      - "9090:9090"  # Metrics port
    
  prometheus:
    image: prom/prometheus
    ports:
      - "9091:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
    
  grafana:
    image: grafana/grafana
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
```

### Prometheus Configuration

```yaml
# prometheus.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'sutra-ml-services'
    static_configs:
      - targets: ['sutra-embedding-service:9090', 'sutra-nlg-service:9090']
    scrape_interval: 10s
    metrics_path: /metrics

  - job_name: 'sutra-foundation'
    static_configs:
      - targets: ['sutra-embedding-service:8888', 'sutra-nlg-service:8889']
    scrape_interval: 30s
    metrics_path: /metrics
```

## Security Configuration

### Authentication Setup (Enterprise)

```bash
# Enable security features
export SUTRA_SECURE_MODE=true
export SUTRA_API_KEY=your-secure-api-key
export SUTRA_JWT_SECRET=your-jwt-secret

# Deploy with security
SUTRA_EDITION=enterprise SUTRA_SECURE_MODE=true docker-compose up -d
```

### Network Security

```yaml
# docker-compose with network isolation
networks:
  sutra-internal:
    driver: bridge
    internal: true
  sutra-external:
    driver: bridge

services:
  sutra-embedding-service:
    networks:
      - sutra-internal
      - sutra-external
    ports:
      - "127.0.0.1:8888:8888"  # Bind to localhost only
      
  sutra-nlg-service:
    networks:
      - sutra-internal
    # No external ports - internal only
```

## Testing & Validation

### Smoke Tests

```bash
#!/bin/bash
# smoke-test-ml-services.sh

echo "🧪 Testing ML Services..."

# Test embedding service
echo "Testing embedding service..."
EMBED_RESPONSE=$(curl -s -X POST http://localhost:8888/embed \
  -H "Content-Type: application/json" \
  -d '{"texts": ["test"], "normalize": true}')

if echo "$EMBED_RESPONSE" | jq -e '.embeddings[0] | length' > /dev/null; then
    echo "✅ Embedding service working"
else
    echo "❌ Embedding service failed"
    exit 1
fi

# Test NLG service
echo "Testing NLG service..."
NLG_RESPONSE=$(curl -s -X POST http://localhost:8889/generate \
  -H "Content-Type: application/json" \
  -d '{"prompt": "Test prompt", "max_tokens": 10}')

if echo "$NLG_RESPONSE" | jq -e '.text' > /dev/null; then
    echo "✅ NLG service working"
else
    echo "❌ NLG service failed"
    exit 1
fi

# Test edition awareness
echo "Testing edition features..."
INFO_RESPONSE=$(curl -s http://localhost:8888/info)
EDITION=$(echo "$INFO_RESPONSE" | jq -r '.edition // "unknown"')

if [[ "$EDITION" != "unknown" ]]; then
    echo "✅ Edition-aware deployment: $EDITION"
else
    echo "⚠️  Edition not detected in response"
fi

echo "🎉 All tests passed!"
```

### Load Testing

```bash
#!/bin/bash
# load-test-ml-services.sh

# Install dependencies
pip install locust

# Create locust test file
cat > locustfile.py << 'EOF'
from locust import HttpUser, task, between
import json

class MLServiceUser(HttpUser):
    wait_time = between(1, 3)
    
    @task(3)
    def test_embedding(self):
        self.client.post("/embed", 
            json={"texts": ["test text"], "normalize": True})
    
    @task(1) 
    def test_nlg(self):
        self.client.post("/generate",
            json={"prompt": "Test", "max_tokens": 20})
    
    @task(1)
    def test_health(self):
        self.client.get("/health")
EOF

# Run load test
locust --host=http://localhost:8888 --users=10 --spawn-rate=2 --run-time=60s --headless
```

## Troubleshooting

### Common Issues

#### 1. ML Foundation Import Errors

**Symptoms:**
```
ImportError: No module named 'sutra_ml_base'
```

**Solutions:**
```bash
# Install ML foundation
cd packages/sutra-ml-base
pip install -e .

# Verify installation
python -c "import sutra_ml_base; print('OK')"

# Check Docker build context
# Ensure Dockerfile copies sutra-ml-base correctly
```

#### 2. Model Loading Failures

**Symptoms:**
```
Failed to load model: Model not found or insufficient memory
```

**Solutions:**
```bash
# Check edition limits
curl -s http://localhost:8888/info | jq '.limits'

# Increase memory limits
# Adjust docker-compose memory limits

# Use smaller model for development
SUTRA_EDITION=simple docker-compose up -d

# Check disk space for model cache
df -h /tmp/.cache
```

#### 3. Edition Features Not Working

**Symptoms:**
```
Advanced caching not available
Custom models not supported
```

**Solutions:**
```bash
# Verify edition environment
echo $SUTRA_EDITION

# Check service configuration
curl -s http://localhost:8888/info | jq '.edition'

# Restart with correct edition
SUTRA_EDITION=enterprise docker-compose restart
```

#### 4. Service Startup Issues

**Symptoms:**
```
Service health check failing
Container exits immediately
```

**Solutions:**
```bash
# Check logs
docker logs sutra-embedding-service
docker logs sutra-nlg-service

# Verify dependencies
docker exec -it sutra-embedding-service pip list | grep sutra-ml-base

# Check resource usage
docker stats sutra-embedding-service

# Restart services
docker-compose restart sutra-embedding-service sutra-nlg-service
```

### Debug Mode

```bash
# Enable debug logging
LOG_LEVEL=DEBUG docker-compose up -d

# Check detailed logs
docker logs sutra-embedding-service 2>&1 | grep -i debug

# Exec into container for inspection
docker exec -it sutra-embedding-service bash
python -c "from sutra_ml_base import EditionManager; print(EditionManager().edition)"
```

### Performance Issues

#### High Memory Usage

```bash
# Check memory usage by service
docker stats --no-stream

# Adjust cache sizes
# Set smaller cache limits for testing
SUTRA_ML_CACHE_SIZE_MB=64 docker-compose up -d

# Use simple edition for development
SUTRA_EDITION=simple docker-compose up -d
```

#### Slow Response Times

```bash
# Check service metrics
curl -s http://localhost:8888/metrics | grep -E "(latency|duration)"

# Monitor request patterns
docker logs sutra-embedding-service | grep -E "(processing_time|batch_size)"

# Scale services for higher throughput
docker-compose up -d --scale sutra-embedding-service=3
```

## Maintenance

### Updates & Upgrades

```bash
# Update to new version
git pull origin main

# Rebuild with new foundation
./sutra-optimize.sh build-ml

# Rolling update (zero downtime)
docker-compose -f docker-compose-grid.yml up -d --no-deps sutra-embedding-service
docker-compose -f docker-compose-grid.yml up -d --no-deps sutra-nlg-service
```

### Backup & Recovery

```bash
# Backup model cache (optional)
tar -czf ml-models-backup.tar.gz /tmp/.cache/huggingface/

# Backup configuration
cp docker-compose-grid.yml docker-compose-backup.yml

# Recovery
# Models will be re-downloaded automatically on startup
# Configuration can be restored from backup
```

### Health Monitoring

```bash
#!/bin/bash
# health-monitor.sh

while true; do
    echo "$(date): Checking ML services..."
    
    # Check embedding service
    if curl -sf http://localhost:8888/health > /dev/null; then
        echo "✅ Embedding service healthy"
    else
        echo "❌ Embedding service unhealthy"
        # Alert or restart logic here
    fi
    
    # Check NLG service
    if curl -sf http://localhost:8889/health > /dev/null; then
        echo "✅ NLG service healthy"
    else
        echo "❌ NLG service unhealthy"
        # Alert or restart logic here
    fi
    
    sleep 60
done
```

## Best Practices

### Development
- Use `SUTRA_EDITION=simple` for development and testing
- Enable `LOG_LEVEL=DEBUG` for troubleshooting
- Install ML foundation in development mode: `pip install -e packages/sutra-ml-base`

### Production
- Use `SUTRA_EDITION=enterprise` for production features
- Set appropriate resource limits based on usage
- Monitor memory usage and model loading times
- Implement proper logging and alerting

### Security
- Enable `SUTRA_SECURE_MODE=true` for production
- Use network isolation and firewalls
- Regularly update base images and dependencies
- Monitor for security vulnerabilities

---

*ML Services Deployment Guide v2.0.0*  
*Built on Sutra ML Foundation*