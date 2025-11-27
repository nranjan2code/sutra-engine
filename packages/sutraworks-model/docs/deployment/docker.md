# Docker Deployment Guide

Deploy SutraWorks Model using Docker containers for scalable, reproducible deployments.

## Quick Start

```bash
# Build and run with Docker Compose
docker-compose up --build

# Or build manually
docker build -t sutraworks:latest .
docker run -p 8080:8080 sutraworks:latest
```

## Dockerfile

### Production Dockerfile

```dockerfile
# Multi-stage build for optimized production image
FROM rust:1.70-alpine as builder

# Install build dependencies
RUN apk add --no-cache musl-dev

WORKDIR /app
COPY . .

# Build optimized release binary
RUN cargo build --release --target x86_64-unknown-linux-musl

# Production stage - minimal Alpine image
FROM alpine:3.18

# Install runtime dependencies
RUN apk add --no-cache ca-certificates libgcc

# Create non-root user
RUN adduser -D -s /bin/sh sutra

WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/sutra-server ./
COPY --from=builder /app/examples ./examples

# Set permissions
RUN chown -R sutra:sutra /app
USER sutra

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8080/health || exit 1

EXPOSE 8080

CMD ["./sutra-server"]
```

### Development Dockerfile

```dockerfile
FROM rust:1.70

WORKDIR /app

# Install development tools
RUN cargo install cargo-watch

# Copy source
COPY . .

# Build in development mode
RUN cargo build

# Development server with hot reload
CMD ["cargo", "watch", "-x", "run"]
```

## Docker Compose

### Production Compose

```yaml
# docker-compose.prod.yml
version: '3.8'

services:
  # Main inference service
  inference:
    build: 
      context: .
      dockerfile: Dockerfile.prod
    ports:
      - "8080:8080"
    environment:
      - SUTRA_LOG_LEVEL=info
      - SUTRA_MAX_MEMORY=8GB
      - SUTRA_QUANTIZATION_ENABLED=true
    volumes:
      - ./models:/app/models:ro
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
    restart: unless-stopped

  # Quantization service
  quantization:
    build: .
    command: ["./sutra-quantizer"]
    ports:
      - "8081:8081"
    environment:
      - SUTRA_QUANTIZATION_BITS=4
      - SUTRA_COMPRESSION_TARGET=7.4
    volumes:
      - ./models:/app/models
    depends_on:
      - inference

  # Trading terminal service
  trading:
    build: .
    command: ["cargo", "run", "--example", "trading_terminal_demo", "--release"]
    environment:
      - SUTRA_TERMINAL_MODE=headless
      - SUTRA_API_ENDPOINT=http://inference:8080
    depends_on:
      - inference

  # Monitoring
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    volumes:
      - grafana-storage:/var/lib/grafana

volumes:
  grafana-storage:
```

### Development Compose

```yaml
# docker-compose.dev.yml
version: '3.8'

services:
  sutra-dev:
    build:
      context: .
      dockerfile: Dockerfile.dev
    ports:
      - "8080:8080"
    volumes:
      - .:/app
      - target:/app/target  # Cache build artifacts
    environment:
      - RUST_LOG=debug
      - SUTRA_DEV_MODE=true
    command: ["cargo", "watch", "-x", "run", "--", "--examples"]

volumes:
  target:
```

## Kubernetes Deployment

### Deployment Manifest

```yaml
# k8s/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: sutraworks-inference
  labels:
    app: sutraworks
    component: inference
spec:
  replicas: 3
  selector:
    matchLabels:
      app: sutraworks
      component: inference
  template:
    metadata:
      labels:
        app: sutraworks
        component: inference
    spec:
      containers:
      - name: inference
        image: sutraworks:latest
        ports:
        - containerPort: 8080
        env:
        - name: SUTRA_LOG_LEVEL
          value: "info"
        - name: SUTRA_MAX_MEMORY
          value: "4GB"
        - name: SUTRA_QUANTIZATION_ENABLED
          value: "true"
        resources:
          requests:
            memory: "2Gi"
            cpu: "500m"
          limits:
            memory: "8Gi"
            cpu: "2"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
        volumeMounts:
        - name: models
          mountPath: /app/models
          readOnly: true
      volumes:
      - name: models
        persistentVolumeClaim:
          claimName: models-pvc
```

### Service Manifest

```yaml
# k8s/service.yaml
apiVersion: v1
kind: Service
metadata:
  name: sutraworks-inference
  labels:
    app: sutraworks
    component: inference
spec:
  selector:
    app: sutraworks
    component: inference
  ports:
  - port: 80
    targetPort: 8080
    protocol: TCP
  type: ClusterIP

---
apiVersion: v1
kind: Service
metadata:
  name: sutraworks-inference-lb
  labels:
    app: sutraworks
spec:
  selector:
    app: sutraworks
    component: inference
  ports:
  - port: 8080
    targetPort: 8080
    protocol: TCP
  type: LoadBalancer
```

### Ingress Configuration

```yaml
# k8s/ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: sutraworks-ingress
  annotations:
    nginx.ingress.kubernetes.io/rewrite-target: /
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
    cert-manager.io/cluster-issuer: letsencrypt-prod
spec:
  tls:
  - hosts:
    - api.sutraworks.com
    secretName: sutraworks-tls
  rules:
  - host: api.sutraworks.com
    http:
      paths:
      - path: /inference
        pathType: Prefix
        backend:
          service:
            name: sutraworks-inference
            port:
              number: 80
      - path: /quantization
        pathType: Prefix
        backend:
          service:
            name: sutraworks-quantization
            port:
              number: 80
```

## Helm Chart

### Chart.yaml

```yaml
# helm/Chart.yaml
apiVersion: v2
name: sutraworks
description: SutraWorks Model - Enterprise AI Framework
type: application
version: 1.0.0
appVersion: "1.0.0"
keywords:
  - ai
  - machine-learning
  - quantization
  - inference
home: https://github.com/nranjan2code/sutraworks-model
maintainers:
  - name: SutraWorks Team
    email: support@sutraworks.com
```

### Values.yaml

```yaml
# helm/values.yaml
replicaCount: 3

image:
  repository: sutraworks
  pullPolicy: IfNotPresent
  tag: "latest"

service:
  type: ClusterIP
  port: 80
  targetPort: 8080

ingress:
  enabled: true
  annotations:
    nginx.ingress.kubernetes.io/rewrite-target: /
  hosts:
    - host: api.sutraworks.local
      paths:
        - path: /
          pathType: ImplementationSpecific
  tls: []

resources:
  requests:
    cpu: 500m
    memory: 2Gi
  limits:
    cpu: 2
    memory: 8Gi

autoscaling:
  enabled: true
  minReplicas: 2
  maxReplicas: 10
  targetCPUUtilizationPercentage: 70
  targetMemoryUtilizationPercentage: 80

config:
  logLevel: info
  maxMemory: "4GB"
  quantizationEnabled: true
  numThreads: 4

models:
  persistence:
    enabled: true
    size: 10Gi
    storageClass: ""
```

### Deployment Template

```yaml
# helm/templates/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: {{ include "sutraworks.fullname" . }}
  labels:
    {{- include "sutraworks.labels" . | nindent 4 }}
spec:
  replicas: {{ .Values.replicaCount }}
  selector:
    matchLabels:
      {{- include "sutraworks.selectorLabels" . | nindent 6 }}
  template:
    metadata:
      labels:
        {{- include "sutraworks.selectorLabels" . | nindent 8 }}
    spec:
      containers:
      - name: {{ .Chart.Name }}
        image: "{{ .Values.image.repository }}:{{ .Values.image.tag }}"
        imagePullPolicy: {{ .Values.image.pullPolicy }}
        ports:
        - name: http
          containerPort: {{ .Values.service.targetPort }}
          protocol: TCP
        env:
        - name: SUTRA_LOG_LEVEL
          value: {{ .Values.config.logLevel | quote }}
        - name: SUTRA_MAX_MEMORY
          value: {{ .Values.config.maxMemory | quote }}
        - name: SUTRA_QUANTIZATION_ENABLED
          value: {{ .Values.config.quantizationEnabled | quote }}
        resources:
          {{- toYaml .Values.resources | nindent 12 }}
        livenessProbe:
          httpGet:
            path: /health
            port: http
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: http
          initialDelaySeconds: 5
          periodSeconds: 5
```

## CI/CD Pipeline

### GitHub Actions

```yaml
# .github/workflows/docker.yml
name: Docker Build and Deploy

on:
  push:
    branches: [ main ]
    tags: [ 'v*' ]
  pull_request:
    branches: [ main ]

env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}

jobs:
  build:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      packages: write

    steps:
    - name: Checkout
      uses: actions/checkout@v3

    - name: Setup Docker Buildx
      uses: docker/setup-buildx-action@v2

    - name: Log in to Container Registry
      uses: docker/login-action@v2
      with:
        registry: ${{ env.REGISTRY }}
        username: ${{ github.actor }}
        password: ${{ secrets.GITHUB_TOKEN }}

    - name: Extract metadata
      id: meta
      uses: docker/metadata-action@v4
      with:
        images: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}
        tags: |
          type=ref,event=branch
          type=ref,event=pr
          type=semver,pattern={{version}}
          type=semver,pattern={{major}}.{{minor}}

    - name: Build and push
      uses: docker/build-push-action@v4
      with:
        context: .
        platforms: linux/amd64,linux/arm64
        push: true
        tags: ${{ steps.meta.outputs.tags }}
        labels: ${{ steps.meta.outputs.labels }}
        cache-from: type=gha
        cache-to: type=gha,mode=max

  deploy:
    needs: build
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    
    steps:
    - name: Deploy to staging
      run: |
        echo "Deploying to staging environment"
        # Add your deployment commands here
```

### GitLab CI

```yaml
# .gitlab-ci.yml
stages:
  - test
  - build
  - deploy

variables:
  DOCKER_DRIVER: overlay2
  DOCKER_TLS_CERTDIR: "/certs"

test:
  stage: test
  image: rust:1.70
  script:
    - cargo test --all --release
  only:
    - merge_requests
    - main

build:
  stage: build
  image: docker:20.10.16
  services:
    - docker:20.10.16-dind
  script:
    - docker build -t $CI_REGISTRY_IMAGE:$CI_COMMIT_SHA .
    - docker push $CI_REGISTRY_IMAGE:$CI_COMMIT_SHA
  only:
    - main

deploy:
  stage: deploy
  image: alpine/helm:latest
  script:
    - helm upgrade --install sutraworks ./helm
      --set image.tag=$CI_COMMIT_SHA
      --namespace production
      --create-namespace
  only:
    - main
```

## Environment-Specific Configurations

### Development

```yaml
# docker-compose.dev.yml
version: '3.8'
services:
  sutra-dev:
    build:
      dockerfile: Dockerfile.dev
    environment:
      - RUST_LOG=debug
      - SUTRA_DEV_MODE=true
      - SUTRA_QUANTIZATION_ENABLED=false  # Faster development
    volumes:
      - .:/app
      - cargo-cache:/usr/local/cargo/registry
    ports:
      - "8080:8080"
      - "9229:9229"  # Debug port

volumes:
  cargo-cache:
```

### Staging

```yaml
# docker-compose.staging.yml
version: '3.8'
services:
  sutra-staging:
    image: sutraworks:staging
    environment:
      - SUTRA_LOG_LEVEL=info
      - SUTRA_QUANTIZATION_ENABLED=true
      - SUTRA_METRICS_ENABLED=true
    deploy:
      replicas: 2
      resources:
        limits:
          memory: 4G
        reservations:
          memory: 2G
```

### Production

```yaml
# docker-compose.prod.yml
version: '3.8'
services:
  sutra-prod:
    image: sutraworks:latest
    environment:
      - SUTRA_LOG_LEVEL=warn
      - SUTRA_QUANTIZATION_ENABLED=true
      - SUTRA_METRICS_ENABLED=true
      - SUTRA_SECURITY_ENABLED=true
    deploy:
      replicas: 5
      resources:
        limits:
          memory: 8G
        reservations:
          memory: 4G
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
```

## Security Configuration

### Dockerfile Security

```dockerfile
# Security-hardened Dockerfile
FROM rust:1.70-alpine as builder

# Security: Update packages and remove package manager
RUN apk update && apk upgrade && apk add --no-cache musl-dev
RUN rm -rf /var/cache/apk/*

# Security: Non-root build user
RUN adduser -D -s /bin/sh builder
USER builder

WORKDIR /app
COPY --chown=builder:builder . .

# Security: Build with security flags
ENV RUSTFLAGS="-C target-feature=+crt-static"
RUN cargo build --release

# Production stage
FROM scratch

# Security: Minimal image with no shell
COPY --from=builder /app/target/release/sutra-server /sutra-server
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

# Security: Non-root user
USER 1000:1000

EXPOSE 8080
ENTRYPOINT ["/sutra-server"]
```

### Secret Management

```yaml
# k8s/secrets.yaml
apiVersion: v1
kind: Secret
metadata:
  name: sutraworks-secrets
type: Opaque
data:
  api-key: <base64-encoded-api-key>
  model-encryption-key: <base64-encoded-encryption-key>

---
apiVersion: v1
kind: ConfigMap
metadata:
  name: sutraworks-config
data:
  config.toml: |
    [deployment]
    log_level = "info"
    max_memory = "8GB"
    
    [security]
    encryption_enabled = true
    tls_enabled = true
```

## Best Practices

### Image Optimization

1. **Multi-stage builds**: Reduce final image size
2. **Alpine base**: Minimal attack surface
3. **Non-root user**: Security hardening
4. **Dependency caching**: Faster builds

### Resource Management

```yaml
resources:
  requests:
    memory: "2Gi"      # Guaranteed memory
    cpu: "500m"        # Guaranteed CPU
  limits:
    memory: "8Gi"      # Maximum memory
    cpu: "2"           # Maximum CPU
```

### Health Checks

```yaml
livenessProbe:
  httpGet:
    path: /health
    port: 8080
  initialDelaySeconds: 30
  periodSeconds: 10
  timeoutSeconds: 5
  failureThreshold: 3

readinessProbe:
  httpGet:
    path: /ready
    port: 8080
  initialDelaySeconds: 5
  periodSeconds: 5
  timeoutSeconds: 3
  failureThreshold: 3
```

## Monitoring and Logging

### Prometheus Metrics

```rust
// Add to your application
use prometheus::{Counter, Histogram, register_counter, register_histogram};

lazy_static! {
    static ref INFERENCE_COUNTER: Counter = register_counter!(
        "sutra_inference_total", "Total number of inference requests"
    ).unwrap();
    
    static ref INFERENCE_DURATION: Histogram = register_histogram!(
        "sutra_inference_duration_seconds", "Inference duration"
    ).unwrap();
}
```

### Structured Logging

```rust
use tracing::{info, warn, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Initialize structured logging
tracing_subscriber::registry()
    .with(tracing_subscriber::fmt::layer().json())
    .init();

// Log with structured data
info!(
    model = "mamba-130m",
    latency_ms = 0.8,
    "Model inference completed"
);
```

## Troubleshooting

### Common Issues

1. **Out of Memory**
   ```bash
   # Check memory limits
   docker stats
   
   # Increase memory limit
   docker run -m 8g sutraworks:latest
   ```

2. **Permission Denied**
   ```bash
   # Fix permissions
   docker run --user 1000:1000 sutraworks:latest
   ```

3. **Health Check Failures**
   ```bash
   # Check health endpoint
   docker exec -it container_name curl localhost:8080/health
   ```

### Debug Mode

```bash
# Run with debug logging
docker run -e RUST_LOG=debug sutraworks:latest

# Run with profiling
docker run -e SUTRA_PROFILING=true sutraworks:latest
```

---

This guide provides comprehensive Docker deployment options from development to production. Choose the configuration that best fits your needs!