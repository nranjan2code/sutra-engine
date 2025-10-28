# Sutra ML-Base Service - Centralized ML Inference Platform
# Provides horizontally scalable ML inference for all Sutra services

ARG SUTRA_VERSION=latest
FROM sutra-ml-base:${SUTRA_VERSION}

ARG SUTRA_EDITION=simple

# Install service-specific dependencies
USER root
COPY packages/sutra-ml-base-service/requirements.txt /tmp/
RUN pip install --no-cache-dir -r /tmp/requirements.txt

# Copy application code
WORKDIR /app
COPY packages/sutra-ml-base-service/main.py ./
COPY packages/sutra-ml-base-service/client.py ./
COPY packages/sutra-ml-base-service/config.py ./
COPY packages/sutra-ml-base-service/monitoring.py ./

# Service configuration
ENV SUTRA_SERVICE_TYPE=ml-base
ENV SUTRA_EDITION=${SUTRA_EDITION}
ENV PORT=8887
ENV LOG_LEVEL=INFO

# ML-Base service runs with more memory and CPU
ENV PYTHONUNBUFFERED=1
ENV TORCH_NUM_THREADS=4

# Health check for ML model readiness
HEALTHCHECK --interval=30s --timeout=15s --start-period=120s --retries=3 \
    CMD python -c "import requests; requests.get('http://localhost:8887/health', timeout=10).raise_for_status()"

# Expose ML-Base port
EXPOSE 8887

# Switch to service user
USER sutra

# Run ML-Base service
CMD ["python", "main.py"]