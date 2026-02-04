# Multi-stage build: Use same Debian version for builder and runtime
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

# Copy manifests
COPY Cargo.toml ./

# Build dependencies first (caching layer)
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    cargo build --release --bin sutra-embedder && \
    rm -rf src target/release/deps/sutra_embedder*

# Copy source and build
COPY src ./src
RUN cargo build --release --bin sutra-embedder

# Runtime stage - Use same Debian version (bookworm)
FROM debian:bookworm-slim

# Install only runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    libgcc-s1 \
    libstdc++6 \
    libgomp1 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -s /bin/bash sutra && \
    mkdir -p /app/models && \
    chown -R sutra:sutra /app

# Copy binary from builder
COPY --from=builder /app/target/release/sutra-embedder /app/sutra-embedder

WORKDIR /app
USER sutra

EXPOSE 8888
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8888/health || exit 1

CMD ["/app/sutra-embedder", "serve"]
