# Production Sutra Hybrid - Optimized Multi-Stage Build (Target: 150-200MB)
# Build from monorepo root: docker build -f packages/sutra-hybrid/Dockerfile -t sutra-hybrid:latest .

# Stage 1: Builder with full toolchain
FROM python:3.11-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    gcc g++ make \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy required packages
COPY packages/sutra-storage-client-tcp ./packages/sutra-storage-client-tcp
COPY packages/sutra-core ./packages/sutra-core
COPY packages/sutra-nlg ./packages/sutra-nlg

# Requirements (minimal versions only)
# Fixed numpy/scipy compatibility: numpy 2.0+ required for scipy 1.14+
RUN cat > hybrid-requirements.txt <<EOF
typing_extensions>=4.12.0
numpy>=2.0.0
scipy>=1.14.0
scikit-learn>=1.5.0
fastapi>=0.104.0
uvicorn[standard]>=0.24.0
pydantic>=2.0.0
pydantic-settings>=2.0.0
msgpack>=1.0.0
requests>=2.31.0
EOF

# Install with aggressive cleanup
RUN pip install --no-cache-dir --prefix=/install typing_extensions>=4.12.0 && \
    pip install --no-cache-dir --prefix=/install ./packages/sutra-storage-client-tcp && \
    pip install --no-cache-dir --prefix=/install ./packages/sutra-core && \
    pip install --no-cache-dir --prefix=/install ./packages/sutra-nlg && \
    pip install --no-cache-dir --prefix=/install --no-compile -r hybrid-requirements.txt && \
    # Aggressive cleanup
    find /install -type d -name __pycache__ -exec rm -rf {} + 2>/dev/null || true && \
    find /install -type f -name "*.pyc" -delete && \
    find /install -type f -name "*.pyo" -delete && \
    find /install -type d -name "tests" -exec rm -rf {} + 2>/dev/null || true && \
    find /install -type d -name "test" -exec rm -rf {} + 2>/dev/null || true && \
    find /install -type d -name "*.dist-info" -exec rm -rf {}/direct_url.json {} + 2>/dev/null || true && \
    find /install -type f -name "*.so" -exec strip {} + 2>/dev/null || true && \
    # Remove numpy tests and docs (huge)
    rm -rf /install/lib/python*/site-packages/numpy/tests && \
    rm -rf /install/lib/python*/site-packages/numpy/*/tests && \
    rm -rf /install/lib/python*/site-packages/sklearn/datasets/data && \
    rm -rf /install/lib/python*/site-packages/sklearn/datasets/descr

# Stage 2: Minimal runtime (NO build tools)
FROM python:3.11-slim

# PRODUCTION: Only runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    ca-certificates \
    libgomp1 \
    && rm -rf /var/lib/apt/lists/* /tmp/*

# Create non-root user
RUN useradd -m -u 1000 sutra

WORKDIR /app

# Copy ONLY installed packages from builder
COPY --from=builder /install /usr/local

# Copy hybrid code
COPY packages/sutra-hybrid/sutra_hybrid ./sutra_hybrid

# Set ownership
RUN chown -R sutra:sutra /app

# Switch to non-root user
USER sutra

# Expose port
EXPOSE 8000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:8000/ping || exit 1

# Environment
ENV PYTHONUNBUFFERED=1 \
    PYTHONDONTWRITEBYTECODE=1 \
    SUTRA_STORAGE_SERVER=storage-server:50051 \
    SUTRA_USE_SEMANTIC_EMBEDDINGS=true

# Run FastAPI app
CMD ["python3", "-m", "uvicorn", "sutra_hybrid.api.app:app", "--host", "0.0.0.0", "--port", "8000"]
