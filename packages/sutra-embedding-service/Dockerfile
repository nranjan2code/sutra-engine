# Sutra Embedding Service - Next Generation
# Uses sutra-ml-base foundation for consistency and performance

# Build stage - Install ML foundation and dependencies  
FROM python:3.11-slim AS builder

RUN apt-get update && apt-get install -y \
    gcc \
    g++ \
    git \
    && rm -rf /var/lib/apt/lists/*

# Create virtual environment
RUN python -m venv /opt/venv
ENV PATH="/opt/venv/bin:$PATH"

# Set up workspace structure
WORKDIR /workspace

# Copy ML foundation first (this is the key change!)
COPY packages/sutra-ml-base/ ./sutra-ml-base/
COPY packages/sutra-embedding-service/requirements.txt ./embedding-service/

# Install foundation and service dependencies
RUN cd sutra-ml-base && pip install --force-reinstall --no-deps . && \
    cd ../embedding-service && pip install -r requirements.txt

# Production stage
FROM python:3.11-slim

# Copy virtual environment with dependencies
COPY --from=builder /opt/venv /opt/venv
ENV PATH="/opt/venv/bin:$PATH"

# Create service user
RUN groupadd -g 1000 embedding && \
    useradd -u 1000 -g embedding -m -d /home/embedding embedding

# Set up directories
RUN mkdir -p /home/embedding/.cache/huggingface /app && \
    chown -R embedding:embedding /home/embedding /app

# Copy ML Foundation source code directly 
WORKDIR /app
COPY packages/sutra-ml-base/sutra_ml_base ./sutra_ml_base

# Copy application code
COPY packages/sutra-embedding-service/main.py .

# Switch to service user
USER embedding

# Environment configuration for new architecture
ENV SUTRA_SERVICE_TYPE=embedding
ENV SUTRA_EDITION=${SUTRA_EDITION:-simple}
ENV PORT=8888
ENV LOG_LEVEL=INFO

# Health check using new foundation endpoints
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD python -c "import requests; requests.get('http://localhost:8888/health').raise_for_status()"

# Expose port  
EXPOSE 8888

# Run using new ML foundation
CMD ["python", "main.py"]