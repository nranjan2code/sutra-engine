# Sutra Storage Service - Simple Build
# Updated to Rust 1.87+ for cxx dependency compatibility
FROM rust:1.87-slim AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev g++ && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml ./
COPY packages/sutra-storage/ ./
COPY packages/sutra-protocol/ ../sutra-protocol/

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --bin storage-server && \
    cp target/release/storage-server /tmp/

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/* /var/cache/apt/* /tmp/* /var/tmp/*

RUN useradd --create-home --shell /bin/bash --uid 1000 storage
WORKDIR /app

COPY --from=builder /tmp/storage-server /usr/local/bin/storage-server
RUN mkdir -p /data && chown -R storage:storage /app /data

USER storage
EXPOSE 7000

CMD ["storage-server"]