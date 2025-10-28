# Sutra ML Base - Shared ML Foundation Image
# This base image contains all heavy ML dependencies to reduce duplicate installations

FROM python:3.11-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    gcc \
    g++ \
    git \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create virtual environment
RUN python -m venv /opt/venv
ENV PATH="/opt/venv/bin:$PATH"

# Set up workspace
WORKDIR /workspace

# Copy ML base requirements and install heavy dependencies
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy and install sutra-ml-base package
COPY . .
RUN pip install --no-deps .

# Production stage - Minimal runtime image with ML dependencies
FROM python:3.11-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

# Copy virtual environment with all ML dependencies
COPY --from=builder /opt/venv /opt/venv
ENV PATH="/opt/venv/bin:$PATH"

# Copy sutra-ml-base source code for services to use
WORKDIR /app
COPY --from=builder /workspace/sutra_ml_base ./sutra_ml_base

# Create base user (services can switch to their own users)
RUN groupadd -g 1000 sutra && \
    useradd -u 1000 -g sutra -m -d /home/sutra sutra

# Set up cache directories
RUN mkdir -p /home/sutra/.cache/huggingface && \
    chown -R sutra:sutra /home/sutra /app

# Environment defaults
ENV SUTRA_ML_BASE_VERSION=1.0.0
ENV LOG_LEVEL=INFO
ENV PYTHONPATH=/app

# Health check endpoint (basic Python/ML stack check)
HEALTHCHECK --interval=60s --timeout=10s --start-period=30s --retries=2 \
    CMD python -c "import torch, transformers, sutra_ml_base; print('ML Base OK')"

USER sutra