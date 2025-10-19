# Minimal Sutra Hybrid - TCP protocol orchestrator (Target: ~250-350MB)
# Build from monorepo root: docker build -f packages/sutra-hybrid/Dockerfile .

# Stage 1: Builder
FROM python:3.11-slim AS builder

WORKDIR /build

# Install minimal build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    gcc g++ protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Copy required packages
COPY packages/sutra-storage-client-tcp ./packages/sutra-storage-client-tcp
COPY packages/sutra-core ./packages/sutra-core
COPY packages/sutra-nlg ./packages/sutra-nlg

# Requirements
RUN cat > hybrid-requirements.txt <<EOF
typing_extensions>=4.12.0
numpy>=1.24.0,<2.0.0
scikit-learn>=1.3.0,<1.4.0
fastapi>=0.104.0
uvicorn[standard]>=0.24.0
pydantic>=2.0.0
pydantic-settings>=2.0.0
msgpack>=1.0.0
requests>=2.31.0
EOF

# Install
RUN pip install --no-cache-dir typing_extensions>=4.12.0 && \
    pip install --no-cache-dir --user ./packages/sutra-storage-client-tcp && \
    pip install --no-cache-dir --user ./packages/sutra-core && \
    pip install --no-cache-dir --user ./packages/sutra-nlg && \
    pip install --no-cache-dir --user --no-compile -r hybrid-requirements.txt && \
    find /root/.local -type d -name __pycache__ -exec rm -rf {} + 2>/dev/null || true && \
    find /root/.local -type f -name "*.pyc" -delete || true

# Stage 2: Runtime
FROM python:3.11-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl libgomp1 \
    && rm -rf /var/lib/apt/lists/*

# Install typing_extensions system-wide
RUN pip install --no-cache-dir typing_extensions>=4.12.0 msgpack>=1.0.0

# Create non-root user
RUN useradd -m -u 1000 sutra

WORKDIR /app

# Copy installed packages
COPY --from=builder /root/.local /home/sutra/.local

# Copy hybrid code
COPY packages/sutra-hybrid/sutra_hybrid ./sutra_hybrid

# Switch to non-root user
USER sutra

# PATH and Python path
ENV PATH=/home/sutra/.local/bin:$PATH \
    PYTHONPATH=/home/sutra/.local/lib/python3.11/site-packages

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
CMD ["uvicorn", "sutra_hybrid.api.app:app", "--host", "0.0.0.0", "--port", "8000"]
