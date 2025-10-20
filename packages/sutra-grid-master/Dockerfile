# Multi-stage build for Sutra Grid Master
# Stage 1: Builder
FROM sutra-base-rust:latest AS builder

# Build dependencies already in base image

WORKDIR /build

# Create workspace structure to match dependency paths
RUN mkdir -p sutra-grid-master sutra-grid-events sutra-storage sutra-protocol

# Copy protocol dependency
COPY packages/sutra-protocol/ ./sutra-protocol/

# Copy storage proto dependency (needed by grid-events)
COPY packages/sutra-storage/proto ./sutra-storage/proto

# Copy grid-events dependency
COPY packages/sutra-grid-events/ ./sutra-grid-events/

# Copy master files to subdirectory to maintain relative paths
COPY packages/sutra-grid-master/Cargo.toml packages/sutra-grid-master/Cargo.lock ./sutra-grid-master/
COPY packages/sutra-grid-master/src ./sutra-grid-master/src

# Change to master directory for build
WORKDIR /build/sutra-grid-master

# Build for release with static linking
RUN cargo build --release --bin sutra-grid-master
# TODO: Migrate CLI to TCP protocol
# RUN cargo build --release --bin sutra-grid-cli

# Stage 2: Runtime
FROM sutra-base-runtime:latest

# Runtime dependencies already in base image

# Create non-root user
RUN addgroup -g 1000 sutra && \
    adduser -D -u 1000 -G sutra sutra

# Create data and config directories
RUN mkdir -p /data /config && chown -R sutra:sutra /data /config

WORKDIR /app

# Copy binaries from builder
COPY --from=builder /build/sutra-grid-master/target/release/sutra-grid-master /app/grid-master
# CLI not yet migrated to TCP
# COPY --from=builder /build/sutra-grid-master/target/release/sutra-grid-cli /app/grid-cli

# Create default master configuration
RUN mkdir -p /config && cat > /config/master-config.toml << 'EOF'
[master]
port = 7000
host = "0.0.0.0"

[events]
storage_url = "http://grid-event-storage:50051"
EOF

# Set ownership
RUN chown -R sutra:sutra /app

# Switch to non-root user
USER sutra

# Expose ports (HTTP binary server + TCP protocol)
EXPOSE 7001 7002

# Health check (TCP protocol port)
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD nc -z localhost 7002 || exit 1

# Environment variables
ENV RUST_LOG=info \
    GRID_MASTER_HOST=0.0.0.0 \
    GRID_MASTER_TCP_PORT=7002 \
    GRID_MASTER_HTTP_PORT=7001 \
    CONFIG_PATH=/config

# Run Grid Master
ENTRYPOINT ["/app/grid-master"]
