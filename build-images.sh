#!/bin/bash
set -e

echo "🚀 Building all Sutra AI Docker images..."
cd "$(dirname "$0")"

# Build storage server (Rust/Alpine)
echo "📦 Building sutra-storage-server..."
docker build -t sutra-storage-server:latest -f packages/sutra-storage/Dockerfile packages/sutra-storage

# Build Python services (from monorepo root for dependencies)
echo "📦 Building sutra-api..."
docker build -t sutra-api:latest -f packages/sutra-api/Dockerfile .

echo "📦 Building sutra-hybrid..."
docker build -t sutra-hybrid:latest -f packages/sutra-hybrid/Dockerfile .

echo "📦 Building sutra-control..."
docker build -t sutra-control:latest -f packages/sutra-control/Dockerfile .

# Build React client
echo "📦 Building sutra-client..."
docker build -t sutra-client:latest -f packages/sutra-client/Dockerfile packages/sutra-client

echo "✅ All images built successfully!"
docker images | grep sutra-
