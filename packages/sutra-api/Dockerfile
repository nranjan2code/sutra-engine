# Minimal Sutra API - TCP Storage Client (Target: 50-80MB)
# Build from monorepo root: docker build -f packages/sutra-api/Dockerfile .

# Stage 1: Builder
FROM sutra-base-python:latest AS builder

WORKDIR /build

# Build tools already in base image

# Copy only storage-client package
COPY packages/sutra-storage-client-tcp ./packages/sutra-storage-client-tcp

# Create minimal requirements (NO sutra-core, NO sutra-hybrid!)
RUN cat > api-requirements.txt <<EOF
typing_extensions>=4.12.0
msgpack>=1.0.0
fastapi>=0.104.0
uvicorn[standard]>=0.24.0
pydantic>=2.0.0
pydantic-settings>=2.0.0
python-multipart>=0.0.6
EOF

# Install minimal packages
RUN pip install --no-cache-dir --user -r api-requirements.txt && \
    pip install --no-cache-dir --user ./packages/sutra-storage-client-tcp && \
    find /root/.local -type d -name __pycache__ -exec rm -rf {} + 2>/dev/null || true

# Stage 2: Runtime
FROM sutra-base-python:latest

# Runtime dependencies already in base image

# Python dependencies in base image

# Create non-root user
RUN adduser --disabled-password --gecos '' --uid 1000 sutra

WORKDIR /app

# Copy installed packages from builder
COPY --from=builder /root/.local /home/sutra/.local

# Copy ONLY API code (no sutra-core, no sutra-hybrid!)
COPY packages/sutra-api/sutra_api ./sutra_api

# Switch to non-root user
USER sutra

# Add local bin to PATH and ensure user site-packages is importable
ENV PATH=/home/sutra/.local/bin:$PATH \
    PYTHONPATH=/home/sutra/.local/lib/python3.11/site-packages

# Expose port
EXPOSE 8000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:8000/health || exit 1

# Environment variables
ENV PYTHONUNBUFFERED=1 \
    SUTRA_STORAGE_MODE=server \
    SUTRA_STORAGE_SERVER=storage-server:50051

# Run application
CMD ["uvicorn", "sutra_api.main:app", "--host", "0.0.0.0", "--port", "8000"]
