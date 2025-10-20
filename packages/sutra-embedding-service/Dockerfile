# Use Ubuntu-based Python for PyTorch compatibility
FROM python:3.11-slim as builder

# Install system dependencies for PyTorch
RUN apt-get update && apt-get install -y \
    gcc \
    g++ \
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

# Copy virtual environment
COPY --from=builder /opt/venv /opt/venv
ENV PATH="/opt/venv/bin:$PATH"

# Create non-root user with home directory (Ubuntu syntax)
RUN groupadd -g 1000 embedding && \
    useradd -u 1000 -g embedding -m -d /home/embedding embedding

# Set up cache directory
RUN mkdir -p /home/embedding/.cache/huggingface && \
    chown -R embedding:embedding /home/embedding

# Copy application and model
WORKDIR /app
COPY main.py .
# Copy models directory if it exists, otherwise create empty one
RUN mkdir -p /app/models
RUN chown -R embedding:embedding /app

# Switch to non-root user
USER embedding

# Environment defaults
ENV PORT=8888
ENV WORKERS=1
ENV EMBEDDING_MODEL=nomic-ai/nomic-embed-text-v1.5
ENV EMBEDDING_BATCH_SIZE=64
ENV EMBEDDING_MAX_WAIT_MS=50

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD python -c "import requests; requests.get('http://localhost:8888/health').raise_for_status()"

# Expose port
EXPOSE 8888

# Run application
CMD ["python", "-m", "uvicorn", "main:app", "--host", "0.0.0.0", "--port", "8888", "--workers", "1"]