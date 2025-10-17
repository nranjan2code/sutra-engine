# Multi-stage build for Sutra Control (OPTIMIZED - Target: 100MB)
# Build from control directory: docker build -f Dockerfile.optimized .

# Stage 1: Builder
FROM python:3.11-alpine AS builder

WORKDIR /build

# Install build dependencies
RUN apk add --no-cache gcc musl-dev libffi-dev

# Create minimal requirements
RUN cat > control-requirements.txt <<EOF
fastapi>=0.104.0
uvicorn[standard]>=0.24.0
psutil>=5.9.0
websockets>=12.0
pydantic>=2.0.0
pydantic-settings>=2.0.0
httpx>=0.25.0
EOF

# Install with cleanup
RUN pip install --no-cache-dir --user -r control-requirements.txt && \
    find /root/.local -type d -name __pycache__ -exec rm -rf {} + 2>/dev/null || true

# Stage 2: Runtime
FROM python:3.11-alpine

# Install runtime dependencies
RUN apk add --no-cache curl

# Create non-root user
RUN adduser --disabled-password --gecos '' --uid 1000 sutra

WORKDIR /app

# Copy installed packages
COPY --from=builder /root/.local /home/sutra/.local

# Copy application code
COPY --chown=sutra:sutra sutra_control ./sutra_control
COPY --chown=sutra:sutra templates ./templates
COPY --chown=sutra:sutra static ./static

# Switch to non-root user
USER sutra

# Add local bin to PATH
ENV PATH=/home/sutra/.local/bin:$PATH

# Expose port
EXPOSE 5000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:5000/ || exit 1

# Environment variables
ENV PYTHONUNBUFFERED=1 \
    PYTHONDONTWRITEBYTECODE=1 \
    SUTRA_STORAGE_MODE=server \
    SUTRA_STORAGE_SERVER=storage-server:50051

# Run application
CMD ["python", "-m", "sutra_control.main"]
