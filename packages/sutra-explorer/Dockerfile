# Multi-stage build for Sutra Storage Explorer
# Stage 1: Build Rust library
FROM rust:1.75-slim as rust-builder

WORKDIR /build
COPY Cargo.toml ./
COPY src ./src

RUN cargo build --release

# Stage 2: Build Frontend
FROM node:20-alpine as frontend-builder

WORKDIR /build
COPY frontend/package*.json ./
RUN npm ci

COPY frontend/ ./
RUN npm run build

# Stage 3: Python Backend + Runtime
FROM python:3.11-slim

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    gcc \
    && rm -rf /var/lib/apt/lists/*

# Copy Rust library
COPY --from=rust-builder /build/target/release/libsutra_explorer.so /usr/local/lib/
ENV LD_LIBRARY_PATH=/usr/local/lib:$LD_LIBRARY_PATH

# Install Python dependencies
COPY backend/requirements.txt ./
RUN pip install --no-cache-dir -r requirements.txt

# Copy backend code
COPY backend/ ./backend/

# Copy frontend build
COPY --from=frontend-builder /build/dist ./frontend/dist

# Create data directory
RUN mkdir -p /data

# Environment variables
ENV PYTHONUNBUFFERED=1
ENV STORAGE_PATH=/data/storage.dat
ENV API_PORT=8100

# Expose ports
EXPOSE 8100 3000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8100/health || exit 1

# Start both backend and frontend
CMD python backend/main.py
