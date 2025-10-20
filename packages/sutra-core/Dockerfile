# Dockerfile for Sutra Core Python Package
FROM sutra-base-python:latest

# System dependencies already in base image
RUN adduser --disabled-password --gecos '' --uid 1000 sutra

WORKDIR /app

# Copy core package files
COPY packages/sutra-core/ /app/sutra-core/

# Install sutra-core in development mode
RUN pip install -e /app/sutra-core/

# Copy any additional dependencies that might be needed
COPY requirements-dev.txt* /app/

# Install additional dependencies if they exist
RUN if [ -f /app/requirements-dev.txt ]; then pip install --no-cache-dir -r /app/requirements-dev.txt; fi

# Switch to non-root user
RUN chown -R sutra:sutra /app
USER sutra

# Set Python path
ENV PYTHONPATH=/app:$PYTHONPATH \
    PYTHONUNBUFFERED=1

# Health check (simple Python import test)
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD python -c "import sutra_core; print('Sutra Core OK')" || exit 1

# Default command (keep container running for development)
CMD ["python", "-c", "import sutra_core; print('Sutra Core loaded successfully'); import time; time.sleep(86400)"]