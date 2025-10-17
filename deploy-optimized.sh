#!/bin/bash
set -e

echo "🚀 Sutra AI - Optimized Deployment Script"
echo "=========================================="

# Configuration
REGISTRY="${REGISTRY:-docker.io/sutraai}"
VERSION="${VERSION:-v2}"
DEPLOY_TARGET="${DEPLOY:-local}"

# Colors
GREEN='\033[0.32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Phase 1: Build optimized images
echo -e "\n${YELLOW}📦 Phase 1: Building optimized images...${NC}"

echo "Building storage-server (target: 24MB)..."
docker build -t sutra-storage-server:${VERSION} \
  -f packages/sutra-storage/Dockerfile \
  packages/sutra-storage

echo "Building sutra-api (target: 60MB - MINIMAL)..."
docker build -t sutra-api:${VERSION} \
  -f packages/sutra-api/Dockerfile .

echo "Building sutra-hybrid (target: 400MB)..."
docker build -t sutra-hybrid:${VERSION} \
  -f packages/sutra-hybrid/Dockerfile .

echo "Building sutra-control (target: 100MB)..."
docker build -t sutra-control:${VERSION} \
  -f packages/sutra-control/Dockerfile \
  packages/sutra-control

echo "Building sutra-client (target: 77MB)..."
docker build -t sutra-client:${VERSION} \
  -f packages/sutra-client/Dockerfile \
  packages/sutra-client

# Phase 2: Verify sizes
echo -e "\n${YELLOW}📊 Phase 2: Verifying image sizes...${NC}"
echo "Service              Size"
echo "----------------------------------------"
docker images | grep "sutra.*${VERSION}" | awk '{printf "%-20s %s\n", $1, $7}'

# Calculate total size
TOTAL_SIZE=$(docker images | grep "sutra.*${VERSION}" | awk '{print $7}' | sed 's/MB//' | awk '{sum+=$1}END{print sum}')
echo "----------------------------------------"
echo "Total: ${TOTAL_SIZE} MB"

# Phase 3: Tag for registry
if [ "$PUSH" = "true" ]; then
  echo -e "\n${YELLOW}🏷️  Phase 3: Tagging for registry...${NC}"
  for img in storage-server api hybrid control client; do
    echo "Tagging sutra-${img}..."
    docker tag sutra-${img}:${VERSION} ${REGISTRY}/sutra-${img}:${VERSION}
    docker tag sutra-${img}:${VERSION} ${REGISTRY}/sutra-${img}:latest
  done
  
  echo -e "\n${YELLOW}⬆️  Phase 4: Pushing to registry...${NC}"
  for img in storage-server api hybrid control client; do
    echo "Pushing ${REGISTRY}/sutra-${img}:${VERSION}..."
    docker push ${REGISTRY}/sutra-${img}:${VERSION}
    docker push ${REGISTRY}/sutra-${img}:latest
  done
fi

# Phase 5: Deploy
if [ "$DEPLOY_TARGET" = "k8s" ]; then
  echo -e "\n${YELLOW}☸️  Phase 5: Deploying to Kubernetes...${NC}"
  
  # Apply manifests
  kubectl apply -f k8s/00-namespace.yaml
  kubectl apply -f k8s/sutra-ai-deployment-v2.yaml
  kubectl apply -f k8s/hpa.yaml
  
  echo "Waiting for deployments to be ready..."
  kubectl wait --for=condition=available --timeout=300s \
    deployment/storage-server \
    deployment/sutra-api \
    deployment/sutra-hybrid \
    deployment/sutra-control \
    deployment/sutra-client \
    -n sutra-ai || echo "Some deployments not ready yet"
  
  echo -e "\n${GREEN}✅ Kubernetes deployment complete!${NC}"
  kubectl get pods -n sutra-ai
  kubectl get hpa -n sutra-ai
  
elif [ "$DEPLOY_TARGET" = "local" ]; then
  echo -e "\n${YELLOW}🐳 Phase 5: Starting local stack...${NC}"
  
  # Stop existing containers
  docker compose down 2>/dev/null || true
  
  # Update docker-compose to use v2 images
  sed "s/:latest/:${VERSION}/g" docker-compose.yml > docker-compose-v2.yml
  
  # Start services
  docker compose -f docker-compose-v2.yml up -d
  
  echo "Waiting for services to be healthy..."
  sleep 15
  
  echo -e "\n${YELLOW}🧪 Testing services...${NC}"
  curl -f http://localhost:8000/health && echo " ✅ API healthy" || echo " ❌ API not ready"
  curl -f http://localhost:8080/ && echo " ✅ Client healthy" || echo " ❌ Client not ready"
  curl -f http://localhost:5001/ && echo " ✅ Control healthy" || echo " ❌ Control not ready"
  
  echo -e "\n${GREEN}✅ Local stack running!${NC}"
  docker compose -f docker-compose-v2.yml ps
  
else
  echo -e "\n${YELLOW}ℹ️  Skipping deployment (set DEPLOY=local or DEPLOY=k8s)${NC}"
fi

# Summary
echo -e "\n${GREEN}=========================================="
echo "📊 Deployment Summary"
echo "==========================================${NC}"
echo "Version: ${VERSION}"
echo "Total size: ${TOTAL_SIZE} MB"
echo "Services: 5 containers"
if [ "$PUSH" = "true" ]; then
  echo "Registry: ${REGISTRY}"
fi
if [ "$DEPLOY_TARGET" != "none" ]; then
  echo "Deployment: ${DEPLOY_TARGET}"
fi
echo -e "${GREEN}✅ Done!${NC}"
