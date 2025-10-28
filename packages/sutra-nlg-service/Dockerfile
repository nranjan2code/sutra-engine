# Lightweight NLG Service v2 - Uses ML-Base Service
FROM python:3.11-slim

# Install system dependencies
RUN apt-get update && apt-get install -y \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy and install requirements
COPY packages/sutra-nlg-service/requirements.txt requirements.txt
RUN pip install --no-cache-dir -r requirements.txt

# Copy lightweight client code
COPY packages/sutra-nlg-service/main.py main.py

# Copy shared ML-Base client libraries
COPY packages/sutra-ml-base-service/client.py ./
COPY packages/sutra-ml-base-service/config.py ./
COPY packages/sutra-ml-base-service/monitoring.py ./

# Create necessary directories
RUN mkdir -p /var/log/sutra

# Set environment variables
ENV PYTHONUNBUFFERED=1
ENV PYTHONDONTWRITEBYTECODE=1
ENV NLG_PORT=8003
ENV NLG_HOST=0.0.0.0

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=5 \
    CMD curl -f http://localhost:8003/health || exit 1

# Expose port
EXPOSE 8003

# Run lightweight NLG service
CMD ["python", "-m", "uvicorn", "main:app", "--host", "0.0.0.0", "--port", "8003"]