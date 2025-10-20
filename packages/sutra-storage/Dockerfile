# Multi-stage build for Sutra Storage Server
# Stage 1: Builder
FROM sutra-base-rust:latest AS builder

# Build dependencies already in base image

WORKDIR /build

# Copy manifests
COPY packages/sutra-storage/Cargo.toml packages/sutra-storage/Cargo.lock packages/sutra-storage/build.rs ./
COPY packages/sutra-storage/proto ./proto
COPY packages/sutra-storage/src ./src

# Build for release with static linking
RUN cargo build --release --bin storage_server_simple

# Stage 2: Runtime
FROM sutra-base-runtime:latest

# Runtime dependencies already in base image

# Create non-root user
RUN addgroup -g 1000 sutra && \
    adduser -D -u 1000 -G sutra sutra

# Create data directory
RUN mkdir -p /data && chown sutra:sutra /data

WORKDIR /app

# Copy binary from builder
COPY --from=builder /build/target/release/storage_server_simple /app/storage_server

# Set ownership
RUN chown -R sutra:sutra /app

# Switch to non-root user
USER sutra

# Expose gRPC port
EXPOSE 50051

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD grpc_health_probe -addr=:50051 || exit 1

# Environment variables
ENV RUST_LOG=info \
    STORAGE_PATH=/data \
    STORAGE_HOST=0.0.0.0 \
    STORAGE_PORT=50051

# Run server
ENTRYPOINT ["/app/storage_server"]
