# Sutra NLG Service - Next Generation
# Uses sutra-ml-base foundation for grounded text generation

# Build stage - Install ML foundation and dependencies
FROM python:3.11-slim AS builder

RUN apt-get update && apt-get install -y \
    gcc \
    g++ \
    git \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create virtual environment
RUN python -m venv /opt/venv
ENV PATH="/opt/venv/bin:$PATH"

# Set up workspace structure
WORKDIR /workspace

# Copy ML foundation first (consistent with embedding service)
COPY packages/sutra-ml-base/ ./sutra-ml-base/
COPY packages/sutra-nlg-service/requirements.txt ./nlg-service/

# Install service dependencies  
RUN cd nlg-service && pip install -r requirements.txt

# Production stage
FROM python:3.11-slim

# Install curl for health checks
RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

# Copy virtual environment with dependencies
COPY --from=builder /opt/venv /opt/venv
ENV PATH="/opt/venv/bin:$PATH"

# Create service user
RUN groupadd -g 1000 nlg && \
    useradd -u 1000 -g nlg -m -d /home/nlg nlg

# Set up directories
RUN mkdir -p /home/nlg/.cache/huggingface /app && \
    chown -R nlg:nlg /home/nlg /app

# Copy ML Foundation source code directly 
WORKDIR /app
COPY packages/sutra-ml-base/sutra_ml_base ./sutra_ml_base

# Copy application code
COPY packages/sutra-nlg-service/main.py .

# Switch to service user
USER nlg

# Environment configuration for new architecture
ENV SUTRA_SERVICE_TYPE=nlg
ENV SUTRA_EDITION=${SUTRA_EDITION:-simple}
ENV PORT=8889
ENV LOG_LEVEL=INFO

# Health check using new foundation endpoints (longer start period for model loading)
HEALTHCHECK --interval=30s --timeout=10s --start-period=120s --retries=3 \
    CMD curl -f http://localhost:8889/health || exit 1

# Expose port
EXPOSE 8889

# Run using new ML foundation
CMD ["python", "main.py"]
