#!/bin/bash
set -euo pipefail

echo "🔧 Building Self-Contained Sutra AI System"
echo "=========================================="

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

print_status() { echo -e "${GREEN}[INFO]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[WARN]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Download Alpine rootfs if not present
ALPINE_ROOTFS="base-images/alpine-minirootfs-3.18.4-x86_64.tar.gz"
if [[ ! -f "$ALPINE_ROOTFS" ]]; then
    print_status "Downloading Alpine rootfs..."
    curl -fsSL "https://dl-cdn.alpinelinux.org/alpine/v3.18/releases/x86_64/alpine-minirootfs-3.18.4-x86_64.tar.gz" -o "$ALPINE_ROOTFS"
fi

# Build base images
print_status "Building self-contained base images..."

cd base-images

docker build -f python.Dockerfile -t sutra-base-python:latest .
docker build -f rust.Dockerfile -t sutra-base-rust:latest .  
docker build -f runtime.Dockerfile -t sutra-base-runtime:latest .
docker build -f node.Dockerfile -t sutra-base-node:latest .
docker build -f nginx.Dockerfile -t sutra-base-nginx:latest .

cd ..

# Build all services using self-contained base images
print_status "Building Sutra services..."

# Storage server
docker build -f Dockerfile.test-storage -t sutra-storage-server:latest .

# API services
docker build -f packages/sutra-api/Dockerfile -t sutra-api:latest .
docker build -f packages/sutra-hybrid/Dockerfile -t sutra-hybrid:latest .
# TODO: Fix PyTorch Alpine compatibility
# docker build -t sutra-embedding-service:latest packages/sutra-embedding-service

# Web services
docker build -t sutra-client:latest packages/sutra-client
docker build -f packages/sutra-control/Dockerfile -t sutra-control:grid-integrated .

# Grid services
docker build -f packages/sutra-grid-master/Dockerfile -t sutra-grid-master:latest .
docker build -f packages/sutra-grid-agent/Dockerfile -t sutra-grid-agent:latest .

# Bulk ingester
docker build -f packages/sutra-bulk-ingester/Dockerfile -t sutra-bulk-ingester:latest .

# Development services (not in main deployment) - commented out
# docker build -f packages/sutra-core/Dockerfile -t sutra-core:latest .
# docker build -f packages/sutra-storage/Dockerfile -t sutra-storage:latest .

print_status "✅ All images built successfully!"
print_status "System ready for deployment with docker-compose-grid.yml"

# Show built images
echo -e "\n${GREEN}Built Images:${NC}"
docker images --format "table {{.Repository}}\t{{.Tag}}\t{{.Size}}" | grep -E "(sutra-|sutra-base)"