# Multi-stage build for Sutra Storage Server
# Stage 1: Builder
FROM rustlang/rust:nightly-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev protobuf-dev

WORKDIR /build

# Copy manifests
COPY Cargo.toml Cargo.lock build.rs ./
COPY proto ./proto
COPY src ./src

# Build for release with static linking
RUN cargo build --release --bin storage_server_simple

# Stage 2: Runtime
FROM alpine:3.19

# Install runtime dependencies
RUN apk add --no-cache ca-certificates libgcc

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
