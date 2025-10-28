# OPTIMIZED API Service - Target: 80MB (down from 298MB)
# Minimal FastAPI service with only essential dependencies

ARG PYTHON_VERSION=3.11

# ============================================================================
# Stage 1: Build minimal dependencies
# ============================================================================
FROM python:${PYTHON_VERSION}-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    gcc && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy only storage-client package
COPY packages/sutra-storage-client-tcp ./packages/sutra-storage-client-tcp

# Create production-grade requirements for API service
RUN cat > api-requirements.txt <<EOF
# Core FastAPI and ASGI server
fastapi>=0.104.0,<0.105.0
uvicorn[standard]==0.24.0
gunicorn>=21.2.0
# Performance and protocols
httptools>=0.5.0,<0.7.0
uvloop>=0.15.0,<0.20.0  
websockets>=10.0,<12.0
h11>=0.14.0,<0.15.0
# Data handling
msgpack>=1.0.0,<2.0.0
pydantic>=2.0.0,<3.0.0
pydantic-settings>=2.0.0,<3.0.0
python-multipart>=0.0.6,<0.1.0
typing_extensions>=4.12.0
# Authentication and security
PyJWT>=2.8.0,<3.0.0
passlib[bcrypt]>=1.7.0,<2.0.0
python-jose[cryptography]>=3.3.0,<4.0.0
bcrypt>=4.0.0,<5.0.0
cryptography>=41.0.0,<42.0.0
argon2-cffi>=23.0.0,<24.0.0
# Additional stability
click>=8.0.0,<9.0.0
starlette>=0.27.0,<0.28.0
# HTTP client for inter-service communication
httpx>=0.25.0,<0.26.0
aiofiles>=23.0.0,<24.0.0
EOF

# Install packages with minimal footprint
RUN pip install --no-cache-dir --prefix=/install --no-compile \
    -r api-requirements.txt && \
    pip install --no-cache-dir --prefix=/install --no-compile \
    ./packages/sutra-storage-client-tcp

# Ultra-aggressive cleanup for API service
RUN find /install -type d -name "__pycache__" -exec rm -rf {} + 2>/dev/null || true && \
    find /install -type f -name "*.pyc" -delete && \
    find /install -type f -name "*.pyo" -delete && \
    find /install -type d -name "tests" -exec rm -rf {} + 2>/dev/null || true && \
    find /install -type d -name "test" -exec rm -rf {} + 2>/dev/null || true && \
    find /install -type d -name "*.dist-info" -exec rm -rf {}/direct_url.json {} + 2>/dev/null || true && \
    find /install -type f -name "*.so" -exec strip {} + 2>/dev/null || true && \
    # Only remove documentation files, preserve all code modules
    find /install -name "*.md" -delete && \
    find /install -name "LICENSE*" -delete && \
    find /install -name "README*" -delete

# ============================================================================
# Stage 2: Minimal production runtime
# ============================================================================
FROM python:${PYTHON_VERSION}-slim

# Install ONLY essential runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl ca-certificates && \
    rm -rf /var/lib/apt/lists/* /var/cache/apt/* /tmp/* /var/tmp/*

# Create non-root user
RUN useradd --create-home --shell /bin/bash --uid 1000 --gid 100 sutra

WORKDIR /app

# Copy ONLY optimized packages from builder
COPY --from=builder /install /usr/local

# Copy ONLY API code
COPY --chown=sutra:100 packages/sutra-api/sutra_api ./sutra_api

# Switch to non-root user
USER sutra

# Environment optimizations
ENV PYTHONUNBUFFERED=1 \
    PYTHONDONTWRITEBYTECODE=1 \
    PYTHONPATH=/app \
    SUTRA_STORAGE_MODE=server \
    SUTRA_STORAGE_SERVER=storage-server:50051

# Minimal health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=2 \
    CMD curl -f http://localhost:8000/health || exit 1

EXPOSE 8000

# Production command optimized for minimal resource usage
# Production-grade ASGI server with Gunicorn + Uvicorn workers
CMD ["gunicorn", "sutra_api.main:app", \
     "-w", "1", \
     "-k", "uvicorn.workers.UvicornWorker", \
     "--bind", "0.0.0.0:8000", \
     "--timeout", "120", \
     "--keep-alive", "2", \
     "--max-requests", "1000", \
     "--max-requests-jitter", "100", \
     "--preload", \
     "--access-logfile", "-", \
     "--error-logfile", "-", \
     "--log-level", "info"]