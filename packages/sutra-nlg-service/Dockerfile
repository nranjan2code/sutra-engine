# Multi-stage build for production-grade NLG service
# Optimized for CPU deployment with gemma-2-2b

FROM python:3.11-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    gcc \
    g++ \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create virtual environment
RUN python -m venv /opt/venv
ENV PATH="/opt/venv/bin:$PATH"

# Copy and install requirements
COPY requirements.txt .
RUN pip install --no-cache-dir --upgrade pip && \
    pip install --no-cache-dir torch>=2.0.0 --index-url https://download.pytorch.org/whl/cpu && \
    pip install --no-cache-dir -r requirements.txt

# Production stage
FROM python:3.11-slim

# Install curl for health checks
RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

# Copy virtual environment from builder
COPY --from=builder /opt/venv /opt/venv
ENV PATH="/opt/venv/bin:$PATH"

# Create non-root user
RUN groupadd -g 1000 nlg && \
    useradd -u 1000 -g nlg -m -d /home/nlg nlg

# Set up cache directories
RUN mkdir -p /tmp/.cache/huggingface && \
    mkdir -p /tmp/.cache/torch && \
    chown -R nlg:nlg /tmp/.cache

# Copy application
WORKDIR /app
COPY main.py .

# Create models directory for volume mount
RUN mkdir -p /app/models && chown -R nlg:nlg /app

# Switch to non-root user
USER nlg

# Environment defaults (swappable via docker-compose)
ENV PORT=8889
ENV NLG_MODEL=google/gemma-3-270m-it
ENV INSTANCE_ID=nlg-default

# Health check (longer start period for model loading)
HEALTHCHECK --interval=30s --timeout=10s --start-period=120s --retries=3 \
    CMD curl -f http://localhost:8889/health || exit 1

# Expose port
EXPOSE 8889

# Run application
CMD ["python", "-m", "uvicorn", "main:app", "--host", "0.0.0.0", "--port", "8889", "--workers", "1"]
