# SIMPLIFIED Hybrid Service - No scientific computing for simple edition
FROM python:3.11-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    gcc g++ && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy core packages
COPY packages/sutra-storage-client-tcp ./packages/sutra-storage-client-tcp
COPY packages/sutra-core ./packages/sutra-core
COPY packages/sutra-nlg ./packages/sutra-nlg

# Install production-grade dependencies for hybrid reasoning
RUN pip install --no-cache-dir --prefix=/install \
    ./packages/sutra-storage-client-tcp \
    ./packages/sutra-core \
    ./packages/sutra-nlg && \
    pip install --no-cache-dir --prefix=/install \
    "fastapi>=0.104.0,<0.105.0" \
    "gunicorn>=21.2.0" \
    "uvicorn[standard]==0.24.0" \
    "pydantic>=2.0.0,<3.0.0" \
    "pydantic-settings>=2.0.0,<3.0.0" \
    "msgpack>=1.0.0,<2.0.0" \
    "requests>=2.31.0,<3.0.0" \
    "numpy==1.24.4" \
    "scipy==1.11.4" \
    "scikit-learn==1.3.2"

# Cleanup
RUN find /install -name "*.pyc" -delete && \
    find /install -name "__pycache__" -type d -exec rm -rf {} + || true

# Production stage
FROM python:3.11-slim

# Install runtime essentials
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd --create-home --uid 1000 sutra

WORKDIR /app

# Copy optimized packages
COPY --from=builder /install /usr/local

# Copy hybrid code
COPY packages/sutra-hybrid/sutra_hybrid ./sutra_hybrid

RUN chown -R sutra:sutra /app

USER sutra

# Environment
ENV PYTHONUNBUFFERED=1 \
    PYTHONDONTWRITEBYTECODE=1 \
    SUTRA_STORAGE_SERVER=storage-server:50051

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:8000/ping || exit 1

EXPOSE 8000

CMD ["gunicorn", "sutra_hybrid.api.app:app", \
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