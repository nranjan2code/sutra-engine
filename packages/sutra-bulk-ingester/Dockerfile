# OPTIMIZED Bulk Ingester - Target: 250MB (down from 265MB) 
# Rust binary + minimal Python runtime for plugins

ARG RUST_VERSION=1.82
ARG PYTHON_VERSION=3.11

# ============================================================================
# Stage 1: Build Rust binary with aggressive optimization
# ============================================================================
FROM rust:${RUST_VERSION}-slim AS rust-builder

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy Rust project files
COPY packages/sutra-bulk-ingester/Cargo.toml ./
COPY packages/sutra-bulk-ingester/src ./src/
COPY packages/sutra-protocol ../sutra-protocol/

# Build with maximum optimization
ENV CARGO_PROFILE_RELEASE_LTO=true
ENV CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1
ENV CARGO_PROFILE_RELEASE_STRIP=true

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release && \
    # Strip debug symbols and copy
    strip /app/target/release/sutra-bulk-ingester && \
    cp /app/target/release/sutra-bulk-ingester /tmp/sutra-bulk-ingester

# ============================================================================
# Stage 2: Minimal Python runtime for plugins
# ============================================================================
FROM python:${PYTHON_VERSION}-slim AS python-builder

# Install minimal Python dependencies for plugins
RUN pip install --no-cache-dir --prefix=/install --no-compile \
    asyncio aiofiles requests psutil && \
    # Cleanup Python installation
    find /install -type d -name "__pycache__" -exec rm -rf {} + 2>/dev/null || true && \
    find /install -type f -name "*.pyc" -delete && \
    find /install -type f -name "*.pyo" -delete && \
    find /install -type d -name "tests" -exec rm -rf {} + 2>/dev/null || true && \
    find /install -type f -name "*.so" -exec strip {} + 2>/dev/null || true

# ============================================================================
# Stage 3: Ultra-minimal production runtime
# ============================================================================
FROM python:${PYTHON_VERSION}-slim

# Install ONLY essential runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl ca-certificates && \
    rm -rf /var/lib/apt/lists/* /var/cache/apt/* /tmp/* /var/tmp/*

# Create non-root user
RUN useradd --create-home --shell /bin/bash --uid 1000 --gid 100 app

WORKDIR /app

# Copy optimized Python environment
COPY --from=python-builder /install /usr/local

# Copy optimized Rust binary
COPY --from=rust-builder /tmp/sutra-bulk-ingester /usr/local/bin/sutra-bulk-ingester

# Copy ONLY essential Python plugins
COPY --chown=app:100 packages/sutra-bulk-ingester/plugins ./plugins/

# Create minimal directories
RUN mkdir -p /datasets /jobs && chown -R app:100 /app /datasets /jobs

# Switch to non-root user
USER app

# Production environment
ENV PYTHONUNBUFFERED=1 \
    PYTHONDONTWRITEBYTECODE=1

# Minimal health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=2 \
    CMD curl -f http://localhost:8005/health || exit 1

EXPOSE 8005

# Production command
CMD ["/usr/local/bin/sutra-bulk-ingester", "--port", "8005", "--plugin-dir", "./plugins"]