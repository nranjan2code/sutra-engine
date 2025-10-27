#!/bin/bash
# Sutra Docker Image Optimization Script
# Build size-optimized images based on edition requirements

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
BUILD_DATE=$(date -u +'%Y-%m-%dT%H:%M:%SZ')
VERSION=${SUTRA_VERSION:-latest}
EDITION=${SUTRA_EDITION:-simple}
NO_CACHE=${NO_CACHE:-false}
PARALLEL=${PARALLEL:-false}
PUSH_IMAGES=${PUSH_IMAGES:-false}

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Edition-specific size targets (bash 3.2 compatible)
get_size_targets() {
    local edition="$1"
    case "$edition" in
        "simple")
            echo "embedding:450MB nlg:550MB hybrid:180MB control:120MB api:80MB bulk-ingester:250MB storage:160MB client:80MB"
            ;;
        "community") 
            echo "embedding:500MB nlg:600MB hybrid:250MB control:140MB api:100MB bulk-ingester:270MB storage:180MB client:90MB"
            ;;
        "enterprise")
            echo "embedding:600MB nlg:700MB hybrid:350MB control:160MB api:120MB bulk-ingester:300MB storage:200MB client:100MB"
            ;;
        *)
            echo "embedding:450MB nlg:550MB hybrid:180MB control:120MB api:80MB bulk-ingester:250MB storage:160MB client:80MB"
            ;;
    esac
}

print_usage() {
    cat << EOF
Usage: $0 [COMMAND] [OPTIONS]

COMMANDS:
    build-all           Build all optimized images for current edition
    build SERVICE       Build specific optimized service
    compare             Compare original vs optimized image sizes
    analyze SERVICE     Analyze layers of specific service
    clean               Remove all optimized images
    test                Build and test all optimized images

OPTIONS:
    --edition EDITION   Target edition (simple|community|enterprise) [default: simple]
    --parallel          Build images in parallel (faster but more resource intensive)
    --no-cache          Force rebuild without Docker cache
    --push              Push images to registry after successful build

SERVICES:
    embedding, nlg, hybrid, control, api, bulk-ingester, storage, client

EXAMPLES:
    $0 build-all --edition simple
    $0 build embedding --no-cache
    $0 compare
    $0 analyze embedding

EOF
}

# Service build configurations (bash 3.2 compatible)
get_service_path() {
    local service="$1"
    case "$service" in
        "embedding") echo "packages/sutra-embedding-service" ;;
        "nlg") echo "packages/sutra-nlg-service" ;;
        "hybrid") echo "packages/sutra-hybrid" ;;
        "control") echo "packages/sutra-control" ;;
        "api") echo "packages/sutra-api" ;;
        "bulk-ingester") echo "packages/sutra-bulk-ingester" ;;
        "storage") echo "packages/sutra-storage" ;;
        "client") echo "packages/sutra-client" ;;
        *) echo "" ;;
    esac
}

# List of all services
ALL_SERVICES="embedding nlg hybrid control api bulk-ingester storage client"

build_service() {
    local service="$1"
    local service_path
    local dockerfile_path
    local image_name="sutra-${service}:${VERSION}-optimized"
    local build_args=""
    
    service_path=$(get_service_path "${service}")
    if [[ -z "${service_path}" ]]; then
        log_error "Unknown service: ${service}"
        return 1
    fi
    
    # Use ultra-optimized Dockerfiles for ML services, simple for others
    if [[ "$service" == "embedding" ]] || [[ "$service" == "nlg" ]]; then
        dockerfile_path="${service_path}/Dockerfile.ultra"
    else
        dockerfile_path="${service_path}/Dockerfile.simple"
    fi
    
    # Fallback to optimized, then original
    if [[ ! -f "${REPO_ROOT}/${dockerfile_path}" ]]; then
        dockerfile_path="${service_path}/Dockerfile.optimized"
    fi
    
    # Add edition-specific build args
    build_args="--build-arg SUTRA_EDITION=${EDITION}"
    build_args="${build_args} --build-arg BUILD_DATE=${BUILD_DATE}"
    build_args="${build_args} --build-arg VERSION=${VERSION}"
    
    # Add no-cache flag if requested
    if [[ "${NO_CACHE:-false}" == "true" ]]; then
        build_args="${build_args} --no-cache"
    fi
    
    log_info "Building optimized ${service} service (edition: ${EDITION})"
    
    if [[ ! -f "${REPO_ROOT}/${dockerfile_path}" ]]; then
        log_error "Optimized Dockerfile not found: ${dockerfile_path}"
        return 1
    fi
    
    # Build the image
    if docker build ${build_args} \
        -f "${REPO_ROOT}/${dockerfile_path}" \
        -t "${image_name}" \
        "${REPO_ROOT}"; then
        log_success "Built ${image_name}"
        
        # Check size against target
        check_size_target "${service}" "${image_name}"
        return 0
    else
        log_error "Failed to build ${service}"
        return 1
    fi
}

check_size_target() {
    local service="$1"
    local image_name="$2"
    local actual_size
    local target_sizes
    local target_size
    
    actual_size=$(docker images --format "{{.Size}}" "${image_name}")
    target_sizes=$(get_size_targets "${EDITION}")
    
    # Extract target size for this service from the targets string
    target_size=$(echo "${target_sizes}" | tr ' ' '\n' | grep "^${service}:" | cut -d: -f2 || echo "unknown")
    
    if [[ "${target_size}" != "unknown" ]]; then
        log_info "Size check: ${service} = ${actual_size} (target: ${target_size})"
        
        # Convert to numeric comparison (rough)
        actual_numeric=$(echo "${actual_size}" | sed 's/MB//' | sed 's/GB/*1000/' | bc 2>/dev/null || echo "0")
        target_numeric=$(echo "${target_size}" | sed 's/MB//' | sed 's/GB/*1000/' | bc 2>/dev/null || echo "999999")
        
        if (( actual_numeric <= target_numeric )); then
            log_success "✓ Size target met for ${service}"
        else
            log_warning "⚠ Size target exceeded for ${service} (${actual_size} > ${target_size})"
        fi
    fi
}

build_all() {
    local failed_services=()
    local build_start
    local build_end 
    local build_duration
    
    build_start=$(date +%s)
    
    log_info "Building all optimized images for edition: ${EDITION}"
    
    if [[ "${PARALLEL:-false}" == "true" ]]; then
        log_info "Building in parallel mode..."
        # Build in parallel with job control
        local pids=()
        for service in embedding nlg hybrid control api bulk-ingester storage client; do
            build_service "${service}" &
            pids+=($!)
        done
        
        # Wait for all builds to complete
        for pid in "${pids[@]}"; do
            if ! wait $pid; then
                failed_services+=("$pid")
            fi
        done
    else
        # Sequential builds
        for service in embedding nlg hybrid control api bulk-ingester storage client; do
            if ! build_service "${service}"; then
                failed_services+=("${service}")
            fi
        done
    fi
    
    build_end=$(date +%s)
    build_duration=$((build_end - build_start))
    
    log_info "Build completed in ${build_duration}s"
    
    if [[ ${#failed_services[@]} -eq 0 ]]; then
        log_success "All services built successfully!"
        compare_sizes
    else
        log_error "Failed services: ${failed_services[*]}"
        return 1
    fi
}

compare_sizes() {
    log_info "Image size comparison:"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    printf "%-20s %-15s %-15s %-15s %-10s\n" "SERVICE" "ORIGINAL" "OPTIMIZED" "TARGET" "SAVINGS"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    local total_original=0
    local total_optimized=0
    
    for service in embedding nlg hybrid control api bulk-ingester storage client; do
        local original_image="sutra-${service}:latest"
        local optimized_image="sutra-${service}:${VERSION}-optimized"
        local target_sizes
        local target_size
        
        target_sizes=$(get_size_targets "${EDITION}")
        target_size=$(echo "${target_sizes}" | tr ' ' '\n' | grep "^${service}:" | cut -d: -f2 || echo "N/A")
        
        if docker images --format "{{.Repository}}:{{.Tag}}" | grep -q "^${original_image}$"; then
            original_size=$(docker images --format "{{.Size}}" "${original_image}")
        else
            original_size="N/A"
        fi
        
        if docker images --format "{{.Repository}}:{{.Tag}}" | grep -q "^${optimized_image}$"; then
            optimized_size=$(docker images --format "{{.Size}}" "${optimized_image}")
        else
            optimized_size="N/A"
        fi
        
        # Calculate savings
        if [[ "${original_size}" != "N/A" ]] && [[ "${optimized_size}" != "N/A" ]]; then
            local original_mb
            local optimized_mb 
            local savings_mb
            local savings_pct
            original_mb=$(echo "${original_size}" | sed 's/[GM]B//' | awk '{if($0 ~ /GB/) print $0*1000; else print $0}')
            optimized_mb=$(echo "${optimized_size}" | sed 's/[GM]B//' | awk '{if($0 ~ /GB/) print $0*1000; else print $0}')
            savings_mb=$((original_mb - optimized_mb))
            savings_pct=$(awk "BEGIN {printf \"%.1f%%\", ($savings_mb / $original_mb) * 100}")
            total_original=$((total_original + original_mb))
            total_optimized=$((total_optimized + optimized_mb))
        else
            savings_pct="N/A"
        fi
        
        printf "%-20s %-15s %-15s %-15s %-10s\n" \
            "${service}" "${original_size}" "${optimized_size}" "${target_size}" "${savings_pct}"
    done
    
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    if [[ $total_original -gt 0 ]] && [[ $total_optimized -gt 0 ]]; then
        local total_savings_mb=$((total_original - total_optimized))
        local total_savings_pct
        total_savings_pct=$(awk "BEGIN {printf \"%.1f%%\", ($total_savings_mb / $total_original) * 100}")
        printf "%-20s %-15s %-15s %-15s %-10s\n" \
            "TOTAL" "${total_original}MB" "${total_optimized}MB" "-" "${total_savings_pct}"
        log_success "Total space saved: ${total_savings_mb}MB (${total_savings_pct})"
    fi
}

analyze_service() {
    local service="$1"
    local original_image="sutra-${service}:latest"
    local optimized_image="sutra-${service}:${VERSION}-optimized"
    
    log_info "Analyzing layers for ${service}"
    
    if docker images --format "{{.Repository}}:{{.Tag}}" | grep -q "^${original_image}$"; then
        echo "Original image layers:"
        docker history "${original_image}" --human --format "table {{.CreatedBy}}\t{{.Size}}"
    fi
    
    echo
    
    if docker images --format "{{.Repository}}:{{.Tag}}" | grep -q "^${optimized_image}$"; then
        echo "Optimized image layers:"
        docker history "${optimized_image}" --human --format "table {{.CreatedBy}}\t{{.Size}}"
    else
        log_warning "Optimized image not found. Build it first with: $0 build ${service}"
    fi
}

clean_optimized() {
    log_info "Cleaning optimized images..."
    local removed=0
    
    for service in embedding nlg hybrid control api bulk-ingester storage client; do
        local image_name="sutra-${service}:${VERSION}-optimized"
        if docker images --format "{{.Repository}}:{{.Tag}}" | grep -q "^${image_name}$"; then
            docker rmi "${image_name}" >/dev/null 2>&1 || true
            removed=$((removed + 1))
        fi
    done
    
    log_success "Removed ${removed} optimized images"
}

test_optimized() {
    log_info "Testing optimized images..."
    
    # Build all first
    if ! build_all; then
        log_error "Build failed, cannot test"
        return 1
    fi
    
    # Basic smoke tests
    local failed_tests=()
    
    for service in embedding nlg hybrid control api bulk-ingester storage client; do
        local image_name="sutra-${service}:${VERSION}-optimized"
        log_info "Testing ${service}..."
        
        # Test that image can start (quick check)
        if docker run --rm "${image_name}" python --version >/dev/null 2>&1; then
            log_success "✓ ${service} basic test passed"
        else
            log_error "✗ ${service} basic test failed"
            failed_tests+=("${service}")
        fi
    done
    
    if [[ ${#failed_tests[@]} -eq 0 ]]; then
        log_success "All image tests passed!"
    else
        log_error "Failed tests: ${failed_tests[*]}"
        return 1
    fi
}

# Main command handling
case "${1:-}" in
    "build-all")
        shift
        while [[ $# -gt 0 ]]; do
            case $1 in
                --edition) EDITION="$2"; shift 2 ;;
                --parallel) PARALLEL="true"; shift ;;
                --no-cache) NO_CACHE="true"; shift ;;
                --push) PUSH_IMAGES="true"; shift ;;
                *) log_error "Unknown option: $1"; exit 1 ;;
            esac
        done
        build_all
        ;;
    "build")
        if [[ -z "${2:-}" ]]; then
            log_error "Service name required"
            exit 1
        fi
        service="$2"
        service_path=$(get_service_path "${service}")
        if [[ -z "${service_path}" ]]; then
            log_error "Unknown service: $service"
            log_info "Available services: embedding nlg hybrid control api bulk-ingester storage client"
            exit 1
        fi
        shift 2
        while [[ $# -gt 0 ]]; do
            case $1 in
                --edition) EDITION="$2"; shift 2 ;;
                --no-cache) NO_CACHE="true"; shift ;;
                *) log_error "Unknown option: $1"; exit 1 ;;
            esac
        done
        build_service "$service"
        ;;
    "compare")
        compare_sizes
        ;;
    "analyze")
        if [[ -z "${2:-}" ]]; then
            log_error "Service name required"
            exit 1
        fi
        analyze_service "$2"
        ;;
    "clean")
        clean_optimized
        ;;
    "test")
        shift
        while [[ $# -gt 0 ]]; do
            case $1 in
                --edition) EDITION="$2"; shift 2 ;;
                *) log_error "Unknown option: $1"; exit 1 ;;
            esac
        done
        test_optimized
        ;;
    "")
        print_usage
        ;;
    *)
        log_error "Unknown command: $1"
        print_usage
        exit 1
        ;;
esac