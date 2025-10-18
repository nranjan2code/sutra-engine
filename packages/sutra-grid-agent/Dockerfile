# Multi-stage build for Sutra Grid Agent
# Stage 1: Builder
FROM rustlang/rust:nightly-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev

WORKDIR /build

# Create workspace structure to match dependency paths
RUN mkdir -p sutra-grid-agent sutra-grid-events sutra-storage sutra-protocol

# Copy protocol dependency
COPY packages/sutra-protocol/ ./sutra-protocol/

# Copy storage proto dependency (needed by grid-events)
COPY packages/sutra-storage/proto ./sutra-storage/proto

# Copy grid-events dependency
COPY packages/sutra-grid-events/ ./sutra-grid-events/

# Copy agent files to subdirectory to maintain relative paths
COPY packages/sutra-grid-agent/Cargo.toml packages/sutra-grid-agent/Cargo.lock ./sutra-grid-agent/
COPY packages/sutra-grid-agent/src ./sutra-grid-agent/src

# Change to agent directory for build
WORKDIR /build/sutra-grid-agent

# Build for release with static linking
RUN cargo build --release --bin sutra-grid-agent

# Stage 2: Runtime
FROM alpine:3.19

# Install runtime dependencies
RUN apk add --no-cache ca-certificates libgcc curl procps netcat-openbsd

# Create non-root user
RUN addgroup -g 1000 sutra && \
    adduser -D -u 1000 -G sutra sutra

# Create directories for storage nodes and config
RUN mkdir -p /data /config /storage-nodes && \
    chown -R sutra:sutra /data /config /storage-nodes

WORKDIR /app

# Copy binary from builder
COPY --from=builder /build/sutra-grid-agent/target/release/sutra-grid-agent /app/grid-agent

# Create default agent configuration
RUN cat > /config/agent-config.toml << 'EOF'
[agent]
agent_id = "docker-agent"
master_host = "grid-master:7002"
platform = "docker"
max_storage_nodes = 5
agent_port = 8001

[storage]
binary_path = "/app/dummy-storage-server.sh"
data_path = "/storage-nodes"
default_memory_mb = 512
default_port_range_start = 50053

[monitoring]
heartbeat_interval_secs = 5
health_check_interval_secs = 10
restart_failed_nodes = true
EOF

# Create dummy storage script for containerized deployment
RUN cat > /app/dummy-storage-server.sh << 'EOF'
#!/bin/sh
# Dummy storage server for containerized Grid Agent
echo "Starting dummy storage server on port $1 with storage path $2"
echo "Memory limit: $3 MB"

# Create storage directory
mkdir -p "$2"

# Simple server that stays alive and responds to health checks
while true; do
    sleep 30
    echo "Storage server heartbeat: $(date)"
done
EOF

RUN chmod +x /app/dummy-storage-server.sh

# Set ownership
RUN chown -R sutra:sutra /app

# Switch to non-root user  
USER sutra

# Expose optional agent metrics port (unused)
EXPOSE 8001

# Health check (connectivity to master TCP)
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD nc -z grid-master 7002 || exit 1

# Environment variables
ENV RUST_LOG=info \
    GRID_AGENT_HOST=0.0.0.0 \
    GRID_AGENT_PORT=8001 \
    GRID_MASTER_ADDRESS=grid-master:7002 \
    CONFIG_PATH=/config/agent-config.toml \
    STORAGE_SERVER_SCRIPT=/app/dummy-storage-server.sh

# Run Grid Agent with config file
ENTRYPOINT ["/app/grid-agent", "/config/agent-config.toml"]
