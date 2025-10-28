#!/bin/bash
# Image Validation Script
# Validates that all required deployment images exist with correct tags

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${BLUE}ℹ ${NC}$1"
}

log_success() {
    echo -e "${GREEN}✓${NC} $1"
}

log_error() {
    echo -e "${RED}✗${NC} $1"
}

validate_deployment_images() {
    local required_images=(
        "sutra-embedding-service"
        "sutra-nlg-service"
        "sutra-hybrid"
        "sutra-control"
        "sutra-api"
        "sutra-bulk-ingester"
        "sutra-storage-server"
        "sutra-client"
    )
    
    local tag="${SUTRA_IMAGE_TAG:-latest}"
    local missing=()
    local found=()
    
    log_info "Validating deployment images with tag: $tag"
    echo ""
    
    for img in "${required_images[@]}"; do
        if docker image inspect "$img:$tag" >/dev/null 2>&1; then
            found+=("$img:$tag")
            log_success "$img:$tag"
        else
            missing+=("$img:$tag")
            log_error "$img:$tag (MISSING)"
        fi
    done
    
    echo ""
    
    if [ ${#missing[@]} -gt 0 ]; then
        log_error "Validation failed: ${#missing[@]} image(s) missing"
        echo ""
        echo "Missing images:"
        for img in "${missing[@]}"; do
            echo "  ✗ $img"
        done
        echo ""
        echo "Build these images first:"
        echo "  ./sutra-optimize.sh build-all"
        return 1
    fi
    
    log_success "All ${#found[@]} deployment images verified ✓"
    return 0
}

# Run validation
validate_deployment_images
