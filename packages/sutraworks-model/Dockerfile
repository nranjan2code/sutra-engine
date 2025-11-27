# Multi-stage Docker build for SutraWorks HTTP API server
# Use Rust nightly for edition2024 support + Debian bookworm runtime

FROM rustlang/rust:nightly-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    build-essential \
    g++ \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy workspace configuration
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY examples/ examples/

# Build dependencies first (caching layer)
RUN cargo build --release --bin sutra-server

# Runtime stage - Use same Debian version (bookworm)
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    libgcc-s1 \
    libstdc++6 \
    libgomp1 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -s /bin/bash sutraworks && \
    mkdir -p /app && \
    chown -R sutraworks:sutraworks /app

# Copy binary from builder
COPY --from=builder /app/target/release/sutra-server /app/sutra-server

WORKDIR /app
USER sutraworks

EXPOSE 8003
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8003/health || exit 1

CMD ["/app/sutra-server", "serve", "--port", "8003", "--model", "rwkv", "--warmup"]

# Labels for container metadata
LABEL org.opencontainers.image.title="SutraWorks HTTP Server"
LABEL org.opencontainers.image.description="Production HTTP API for SutraWorks AI models"
LABEL org.opencontainers.image.vendor="SutraWorks"
LABEL org.opencontainers.image.version="1.0.0"
LABEL org.opencontainers.image.source="https://github.com/nranjan2code/sutraworks-model"
