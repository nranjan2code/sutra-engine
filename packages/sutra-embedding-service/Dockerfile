# Lightweight Embedding Service v2 - Uses ML-Base Service
FROM python:3.11-slim

# Install system dependencies
RUN apt-get update && apt-get install -y \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy and install requirements
COPY packages/sutra-embedding-service/requirements.txt requirements.txt
RUN pip install --no-cache-dir -r requirements.txt

# Copy lightweight client code
COPY packages/sutra-embedding-service/main.py main.py

# Copy shared ML-Base client libraries
COPY packages/sutra-ml-base-service/client.py ./
COPY packages/sutra-ml-base-service/config.py ./
COPY packages/sutra-ml-base-service/monitoring.py ./

# Create necessary directories
RUN mkdir -p /var/log/sutra

# Set environment variables
ENV PYTHONUNBUFFERED=1
ENV PYTHONDONTWRITEBYTECODE=1
ENV EMBEDDING_PORT=8888
ENV EMBEDDING_HOST=0.0.0.0

# Health check
HEALTHCHECK --interval=10s --timeout=5s --start-period=30s --retries=5 \
    CMD curl -f http://localhost:8888/health || exit 1

# Expose port
EXPOSE 8888

# Run lightweight embedding service
CMD ["python", "-m", "uvicorn", "main:app", "--host", "0.0.0.0", "--port", "8888"]