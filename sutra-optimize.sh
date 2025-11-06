#!/bin/bash
# Sutra Docker Build System v2.0
# Production-grade Docker image builder for all Sutra services
# 
# Principles:
# - Single optimized build per service (no dual-build comparison)
# - Menu-driven interface for ease of use
# - Edition-aware builds (simple, community, enterprise)
# - Comprehensive size analysis
# - Production deployment integration

set -euo pipefail  # Fail on undefined vars and pipe failures

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EDITION="${SUTRA_EDITION:-simple}"
VERSION="${SUTRA_VERSION:-latest}"
NO_CACHE="${NO_CACHE:-false}"
PARALLEL="${PARALLEL:-false}"
PUSH_IMAGES="${PUSH_IMAGES:-false}"

# Configuration functions (bash 3.2 compatible)
get_edition_targets() {
    local edition="$1"
    case "$edition" in
        simple) echo "ml-base-service:1500MB embedding:50MB nlg:50MB hybrid:180MB control:120MB api:80MB bulk-ingester:250MB storage:160MB client:80MB" ;;
        community) echo "ml-base-service:1600MB embedding:60MB nlg:60MB hybrid:250MB control:140MB api:100MB bulk-ingester:270MB storage:180MB client:90MB" ;;
        enterprise) echo "ml-base-service:1800MB embedding:70MB nlg:70MB hybrid:350MB control:160MB api:120MB bulk-ingester:300MB storage:200MB client:100MB" ;;
        *) echo "ml-base-service:1500MB embedding:50MB nlg:50MB hybrid:180MB control:120MB api:80MB bulk-ingester:250MB storage:160MB client:80MB" ;;
    esac
}

get_service_strategy() {
    local service="$1"
    case "$service" in
        ml-base-service) echo "ultra" ;;
        embedding) echo "simple" ;;
        nlg) echo "simple" ;;
        hybrid) echo "simple" ;;
        control) echo "simple" ;;
        api) echo "optimized" ;;
        bulk-ingester) echo "optimized" ;;
        storage) echo "simple" ;;
        client) echo "simple" ;;
        *) echo "simple" ;;
    esac
}

# Helper functions
log_info() {
    echo -e "${BLUE}ℹ ${NC}$1"
}

log_success() {
    echo -e "${GREEN}✓${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

log_error() {
    echo -e "${RED}✗${NC} $1"
}

log_header() {
    echo -e "${BOLD}${CYAN}$1${NC}"
}

print_banner() {
    echo -e "${CYAN}"
    echo "╔══════════════════════════════════════════════════════════════╗"
    echo "║                 🐳 SUTRA BUILD SYSTEM                        ║"
    echo "║              Production Docker Image Builder                 ║"
    echo "╠══════════════════════════════════════════════════════════════╣"
    echo "║ Default: Build all services + deploy                        ║"
    echo "║ • 8 Services: embedding, nlg, hybrid, control, api,        ║"
    echo "║              bulk-ingester, storage, client                 ║"
    echo "║ • Run './sutra-optimize.sh menu' for interactive options    ║"
    echo "╚══════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
    echo -e "${YELLOW}Edition: ${EDITION} | Version: ${VERSION}${NC}"
    echo ""
}

get_image_size() {
    local image="$1"
    docker images --format "{{.Size}}" "$image" 2>/dev/null | head -1 || echo "N/A"
}

convert_to_mb() {
    local size="$1"
    if [[ "$size" == *"GB" ]]; then
        echo "$size" | sed 's/GB//' | awk '{print int($1 * 1024)}'
    elif [[ "$size" == *"MB" ]]; then
        echo "$size" | sed 's/MB//' | awk '{print int($1)}'
    else
        echo "0"
    fi
}

show_size_comparison() {
    log_header "📊 Built Image Sizes"
    echo ""
    printf "%-15s %-15s %-12s\n" "SERVICE" "SIZE" "TARGET"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    local deploy_tag="${SUTRA_IMAGE_TAG:-latest}"
    local total_size_mb=0
    
    # Show ML base first (foundation for ML services)
    local ml_base_size=$(get_image_size "sutra-works-ml-base:$deploy_tag")
    printf "%-15s %-15s %-12s\n" "ml-base" "$ml_base_size" "~800MB"
    
    # Extract size in MB for total calculation
    if [[ "$ml_base_size" =~ ([0-9]+(\.[0-9]+)?)([GMK]B) ]]; then
        local size_num="${BASH_REMATCH[1]}"
        local size_unit="${BASH_REMATCH[3]}"
        case "$size_unit" in
            GB) total_size_mb=$(awk "BEGIN {print $total_size_mb + ($size_num * 1024)}") ;;
            MB) total_size_mb=$(awk "BEGIN {print $total_size_mb + $size_num}") ;;
        esac
    fi
    
    # Core services
    local services=(ml-base-service embedding nlg hybrid control api bulk-ingester storage client)
    
    # Add grid services for enterprise
    if [ "$EDITION" = "enterprise" ]; then
        services+=(grid-master grid-agent)
    fi
    
    for service in "${services[@]}"; do
        # Map service names to actual image names
        case "$service" in
            ml-base-service) local image_name="sutra-works-ml-base-service" ;;
            embedding) local image_name="sutra-works-embedding-service" ;;
            nlg) local image_name="sutra-works-nlg-service" ;;
            storage) local image_name="sutra-works-storage-server" ;;
            *) local image_name="sutra-works-$service" ;;
        esac
        
        local size=$(get_image_size "$image_name:$deploy_tag")
        
        # Get target from edition
        local targets
        targets=$(get_edition_targets "$EDITION")
        local target
        target=$(echo "$targets" | grep -o "$service:[0-9]*MB" | cut -d: -f2 || echo "N/A")
        
        printf "%-15s %-15s %-12s\n" "$service" "$size" "$target"
        
        # Add to total if size is valid
        if [ "$size" != "N/A" ]; then
            local size_mb
            size_mb=$(convert_to_mb "$size")
            if [ "$size_mb" -gt 0 ]; then
                total_size_mb=$((total_size_mb + size_mb))
            fi
        fi
    done
    
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    if [ $total_size_mb -gt 0 ]; then
        printf "%-15s %-15s %-12s\n" "TOTAL" "${total_size_mb}MB" "-"
        echo ""
        log_success "Total images size: ${total_size_mb}MB"
    fi
    echo ""
}

build_service() {
    local service="$1"
    local strategy
    strategy=$(get_service_strategy "$service")
    
    log_info "Building $service service (strategy: $strategy)"
    
    # NEW ML SERVICES: Use shared ML foundation
    if [ "$service" = "embedding" ] || [ "$service" = "nlg" ]; then
        build_ml_service "$service" "$strategy"
        return $?
    fi
    
    # STORAGE SERVICE: Rust-based with special image naming
    if [ "$service" = "storage" ]; then
        build_storage_service "$strategy"
        return $?
    fi
    
    # GRID SERVICES: Rust-based (enterprise only)
    if [ "$service" = "grid-master" ] || [ "$service" = "grid-agent" ]; then
        build_grid_service "$service" "$strategy"
        return $?
    fi
    
    # Standard Dockerfile (now consolidated - single Dockerfile per service)
    local dockerfile="packages/sutra-$service/Dockerfile"
    
    # Check if Dockerfile exists
    if [ ! -f "$dockerfile" ]; then
        log_error "Dockerfile not found: $dockerfile"
        return 1
    fi
    
    # Build command
    local build_cmd="docker build"
    if [ "$NO_CACHE" = "true" ]; then
        build_cmd="$build_cmd --no-cache"
    fi
    
    # Map service name to image name
    local image_name="sutra-works-$service"
    local deploy_tag="${SUTRA_IMAGE_TAG:-latest}"

    # Build directly to deployment tag (no intermediate tag needed)
    build_cmd="$build_cmd -f $dockerfile -t $image_name:$deploy_tag ."
    
    log_info "Running: $build_cmd"
    
    # Execute build
    if eval "$build_cmd"; then
        log_success "Built $image_name:$deploy_tag"
        
        # Show size
        local size=$(get_image_size "$image_name:$deploy_tag")
        log_info "Size: $size"
        
        return 0
    else
        log_error "Failed to build $service"
        return 1
    fi
}

# NEW: Build ML services using shared foundation
build_ml_service() {
    local service="$1"
    local strategy="$2"
    
    log_info "Building ML service: $service using sutra-ml-base foundation"
    
    # Use individual service Dockerfile
    local dockerfile="packages/sutra-$service-service/Dockerfile"
    
    # Check if service Dockerfile exists
    if [ ! -f "$dockerfile" ]; then
        log_error "ML service Dockerfile not found: $dockerfile"
        return 1
    fi
    
    # Build command with service-specific arguments
    local build_cmd="docker build"
    if [ "$NO_CACHE" = "true" ]; then
        build_cmd="$build_cmd --no-cache"
    fi
    
    # Set build arguments for ML services
    local service_port=""
    if [ "$service" = "embedding" ]; then
        service_port="8889"  # Updated to match our ML Foundation architecture
    elif [ "$service" = "nlg" ]; then
        service_port="8890"  # Updated to match our ML Foundation architecture  
    fi
    
    build_cmd="$build_cmd --build-arg SUTRA_EDITION=$EDITION"
    build_cmd="$build_cmd --build-arg SUTRA_VERSION=$VERSION"  
    build_cmd="$build_cmd --build-arg SERVICE_PORT=$service_port"
    build_cmd="$build_cmd -f $dockerfile"
    
    # Build directly to deployment tag (no intermediate tag needed)
    local deploy_tag="${SUTRA_IMAGE_TAG:-latest}"
    build_cmd="$build_cmd -t sutra-works-$service-service:$deploy_tag ."
    
    log_info "Running: $build_cmd"
    
    # Execute build
    if eval "$build_cmd"; then
        log_success "Built sutra-$service-service:$deploy_tag"
        
        # Show size
        local size
        size=$(get_image_size "sutra-works-$service-service:$deploy_tag")
        log_info "Size: $size"
        
        return 0
    else
        log_error "Failed to build ML service: $service"
        return 1
    fi
}

# NEW: Build ML base foundation image (prerequisite for ML services)
build_ml_base() {
    log_info "Building ML base foundation image"
    
    local dockerfile="packages/sutra-ml-base/Dockerfile"
    
    # Check if Dockerfile exists
    if [ ! -f "$dockerfile" ]; then
        log_error "ML base Dockerfile not found: $dockerfile"
        return 1
    fi
    
    # Build command
    local build_cmd="docker build"
    if [ "$NO_CACHE" = "true" ]; then
        build_cmd="$build_cmd --no-cache"
    fi
    
    # Build arguments
    build_cmd="$build_cmd --build-arg SUTRA_VERSION=$VERSION"
    build_cmd="$build_cmd -f $dockerfile"
    
    # Build to deployment tag (foundation image, not the service)
    local deploy_tag="${SUTRA_IMAGE_TAG:-latest}"
    build_cmd="$build_cmd -t sutra-works-ml-base:$deploy_tag"
    build_cmd="$build_cmd packages/sutra-ml-base/"
    
    log_info "Running: $build_cmd"
    
    # Execute build
    if eval "$build_cmd"; then
        log_success "Built sutra-ml-base:$deploy_tag (foundation)"

        # Show size
        local size
        size=$(get_image_size "sutra-works-ml-base:$deploy_tag")
        log_info "Size: $size"
        
        return 0
    else
        log_error "Failed to build ML base foundation"
        return 1
    fi
}

# Build ML-Base Service (runs as a service on port 8887)
build_ml_base_service() {
    log_info "Building ML-Base Service (inference platform)"

    local dockerfile="packages/sutra-ml-base-service/Dockerfile"

    # Check if Dockerfile exists
    if [ ! -f "$dockerfile" ]; then
        log_error "ML-Base Service Dockerfile not found: $dockerfile"
        return 1
    fi

    # Build command
    local build_cmd="docker build"
    if [ "$NO_CACHE" = "true" ]; then
        build_cmd="$build_cmd --no-cache"
    fi

    # Build arguments
    build_cmd="$build_cmd --build-arg SUTRA_VERSION=$VERSION"
    build_cmd="$build_cmd --build-arg SUTRA_EDITION=$EDITION"
    build_cmd="$build_cmd -f $dockerfile"

    # Build to deployment tag (same as compose expects)
    local deploy_tag="${SUTRA_IMAGE_TAG:-latest}"
    build_cmd="$build_cmd -t sutra-works-ml-base-service:$deploy_tag"
    build_cmd="$build_cmd ."

    log_info "Running: $build_cmd"

    # Execute build
    if eval "$build_cmd"; then
        log_success "Built sutra-ml-base-service:$deploy_tag (service)"

        # Show size
        local size
        size=$(get_image_size "sutra-works-ml-base-service:$deploy_tag")
        log_info "Size: $size"

        return 0
    else
        log_error "Failed to build ML-Base Service"
        return 1
    fi
}

# NEW: Build Rust-based storage service with proper image naming
build_storage_service() {
    local strategy="$1"
    
    log_info "Building Rust-based storage service using sutra-storage foundation"
    
    # Use standard Dockerfile (now consolidated)
    local dockerfile="packages/sutra-storage/Dockerfile"
    
    # Check if Dockerfile exists
    if [ ! -f "$dockerfile" ]; then
        log_error "Storage Dockerfile not found: $dockerfile"
        return 1
    fi
    
    # Build command
    local build_cmd="docker build"
    if [ "$NO_CACHE" = "true" ]; then
        build_cmd="$build_cmd --no-cache"
    fi
    
    # CRITICAL: Use image name that matches docker-compose-grid.yml
    # Compose file expects: sutra-storage-server:latest
    build_cmd="$build_cmd --build-arg SUTRA_EDITION=$EDITION"
    build_cmd="$build_cmd --build-arg SUTRA_VERSION=$VERSION"
    build_cmd="$build_cmd -f $dockerfile"
    
    # Build directly to deployment tag (no intermediate tag needed)
    local deploy_tag="${SUTRA_IMAGE_TAG:-latest}"
    build_cmd="$build_cmd -t sutra-works-storage-server:$deploy_tag ."
    
    log_info "Running: $build_cmd"
    
    # Execute build
    if eval "$build_cmd"; then
        log_success "Built sutra-storage-server:$deploy_tag"
        
        # Show size
        local size
        size=$(get_image_size "sutra-works-storage-server:$deploy_tag")
        log_info "Size: $size"
        
        return 0
    else
        log_error "Failed to build storage service"
        return 1
    fi
}

# NEW: Build Rust-based grid services (enterprise only)
build_grid_service() {
    local service="$1"
    local strategy="$2"
    
    log_info "Building Rust-based $service service"
    
    # Use standard Dockerfile
    local dockerfile="packages/sutra-$service/Dockerfile"
    
    # Check if Dockerfile exists
    if [ ! -f "$dockerfile" ]; then
        log_error "$service Dockerfile not found: $dockerfile"
        return 1
    fi
    
    # Build command
    local build_cmd="docker build"
    if [ "$NO_CACHE" = "true" ]; then
        build_cmd="$build_cmd --no-cache"
    fi
    
    # Use image name that matches docker-compose-grid.yml
    build_cmd="$build_cmd --build-arg SUTRA_EDITION=$EDITION"
    build_cmd="$build_cmd --build-arg SUTRA_VERSION=$VERSION"
    build_cmd="$build_cmd -f $dockerfile"
    
    # Build directly to deployment tag
    local deploy_tag="${SUTRA_IMAGE_TAG:-latest}"
    build_cmd="$build_cmd -t sutra-works-$service:$deploy_tag ."
    
    log_info "Running: $build_cmd"
    
    # Execute build
    if eval "$build_cmd"; then
        log_success "Built sutra-$service:$deploy_tag"
        
        # Show size
        local size
        size=$(get_image_size "sutra-works-$service:$deploy_tag")
        log_info "Size: $size"
        
        return 0
    else
        log_error "Failed to build $service"
        return 1
    fi
}

build_all_services() {
    log_header "🔨 Building All Services"
    echo ""
    
    # Step 1: Build ML base foundation first (required by ML services)
    log_info "Step 1: Building ML foundation"
    if ! build_ml_base; then
        log_error "Failed to build ML base - cannot continue with ML services"
        return 1
    fi
    echo ""

    # Step 1.5: Build ML-Base Service (depends on ML base foundation)
    log_info "Step 1.5: Building ML-Base Service"
    if ! build_ml_base_service; then
        log_error "Failed to build ML-Base Service"
        return 1
    fi
    echo ""

    # Step 2: Core services for all editions
    local services=(embedding nlg hybrid control api bulk-ingester storage client)
    
    # Add grid services for enterprise edition
    if [ "$EDITION" = "enterprise" ]; then
        log_info "Enterprise edition detected - adding grid services"
        services+=(grid-master grid-agent)
        echo ""
    fi
    
    local failed_services=()
    local success_count=1  # ML base already built successfully
    
    log_info "Step 2: Building service images"
    for service in "${services[@]}"; do
        echo ""
        if build_service "$service"; then
            success_count=$((success_count + 1))
        else
            failed_services+=("$service")
        fi
        echo ""
    done
    
    echo ""
    log_header "📋 Build Summary"
    local total_services=$((${#services[@]} + 1))  # +1 for ML base
    log_success "Successfully built: $success_count/${total_services} images (ML base + services)"
    
    if [ ${#failed_services[@]} -gt 0 ]; then
        log_warning "Failed services: ${failed_services[*]}"
        echo ""
        log_info "Check the build logs above for details"
        return 1
    else
        log_success "All services built successfully!"
        echo ""
        show_size_comparison
        return 0
    fi
}

test_optimized_images() {
    log_header "🧪 Testing Optimized Images"
    echo ""
    
    local services=(embedding nlg hybrid control api bulk-ingester client)
    local test_count=0
    local pass_count=0
    
    for service in "${services[@]}"; do
        local image_name="sutra-works-$service"
        if [ "$service" = "embedding" ]; then
            image_name="sutra-works-embedding-service"
        elif [ "$service" = "nlg" ]; then
            image_name="sutra-works-nlg-service"
        else
            image_name="sutra-works-$service"
        fi
        
        log_info "Testing $image_name:latest"
        test_count=$((test_count + 1))
        
        # Basic image inspection test
        if docker images --format "{{.Repository}}:{{.Tag}}" | grep -q "^$image_name:latest$"; then
            log_success "$service image exists and is valid"
            pass_count=$((pass_count + 1))
        else
            log_error "$service image not found or invalid"
        fi
    done
    
    echo ""
    log_info "Test Results: $pass_count/$test_count images passed basic validation"
    
    if [ $pass_count -eq $test_count ]; then
        log_success "All optimized images are valid!"
        return 0
    else
        log_warning "Some images failed validation"
        return 1
    fi
}

deploy_optimized() {
    log_header "🚀 Deploying Optimized Images"
    echo ""
    
    log_info "Setting up production-grade environment for optimized deployment..."
    export SUTRA_VERSION="${SUTRA_VERSION:-latest}"  # Use latest tag by default
    export SUTRA_EDITION="${SUTRA_EDITION:-simple}"
    export SUTRA_SECURE_MODE="${SUTRA_SECURE_MODE:-false}"
    export HF_TOKEN="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"
    
    # Production environment variables
    export SUTRA_JWT_SECRET_KEY="${SUTRA_JWT_SECRET_KEY:-PRODUCTION_SECRET_$(openssl rand -hex 32)}"
    export SUTRA_LOG_LEVEL="${SUTRA_LOG_LEVEL:-info}"
    export PYTHONUNBUFFERED=1
    export PYTHONDONTWRITEBYTECODE=1
    
    # Determine compose file
    if [ "$SUTRA_SECURE_MODE" = "true" ]; then
        COMPOSE_FILE="sutrabuild/compose/docker-compose-secure.yml"
    else
        COMPOSE_FILE=".sutra/compose/production.yml"
    fi
    
    # Determine profile based on edition
    case "$SUTRA_EDITION" in
        simple)
            PROFILE="simple"
            ;;
        community)
            PROFILE="community" 
            ;;
        enterprise)
            PROFILE="enterprise"
            ;;
        *)
            log_error "Unknown edition: $SUTRA_EDITION"
            return 1
            ;;
    esac
    
    log_info "Edition: $SUTRA_EDITION ($PROFILE profile)"
    log_info "Compose file: $COMPOSE_FILE"
    
    # Set deployment tag (compose expects SUTRA_VERSION)
    export SUTRA_VERSION="${SUTRA_VERSION:-latest}"
    export SUTRA_IMAGE_TAG="${SUTRA_VERSION}"  # For compatibility
    log_info "Using image tag: $SUTRA_VERSION"
    
    # Check that compose file exists
    if [ ! -f "$COMPOSE_FILE" ]; then
        log_error "Compose file not found: $COMPOSE_FILE"
        return 1
    fi
    
    # Note: Skipping image verification - docker-compose will fail if images don't exist
    # The verification was having issues with image detection despite images being present
    log_success "Compose file verified ✓"
    
    # Deploy using docker-compose with pre-built images (skip building)
    log_info "Deploying with docker-compose using pre-built images..."
    log_info "Compose file: $COMPOSE_FILE"
    log_info "Project name: sutra-works"
    log_info "Command: docker-compose -p sutra-works -f $COMPOSE_FILE --profile $PROFILE up -d --no-build"

    if docker-compose -p sutra-works -f "$COMPOSE_FILE" --profile "$PROFILE" up -d --no-build; then
        echo ""
        log_success "🎉 Deployment completed successfully!"
        echo ""
        log_info "Services are starting up. Check status with:"
        echo "  docker-compose -p sutra-works -f $COMPOSE_FILE --profile $PROFILE ps"
        echo ""
        log_info "ML Foundation Services:"
        echo "  • Embedding Service: http://localhost:8889"
        echo "  • NLG Service: http://localhost:8890"
        echo ""
        log_info "Other Services:"
        echo "  • API: http://localhost:8000"
        echo "  • Control Panel: http://localhost:3000" 
        echo "  • Client: http://localhost:3001"
        return 0
    else
        log_error "Deployment failed"
        return 1
    fi
}

clean_images() {
    log_header "🧹 Cleaning Docker Images"
    echo ""
    
    log_warning "This will remove:"
    echo "  • All untagged Docker images"
    echo "  • Docker build cache"
    echo "  • Stopped containers"
    echo ""
    
    read -p "Continue? [y/N] " -n 1 -r
    echo ""
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        log_info "Cleaning Docker system..."
        docker system prune -f
        docker image prune -f
        log_success "Docker cleanup completed"
    else
        log_info "Cleanup cancelled"
    fi
}

show_optimization_status() {
    log_header "📊 Current Optimization Status"
    echo ""
    
    local services=(embedding nlg hybrid control api bulk-ingester storage client)
    local optimized_count=0
    
    for service in "${services[@]}"; do
        local image_name="sutra-works-$service"
        if [ "$service" = "embedding" ]; then
            image_name="sutra-works-embedding-service"
        elif [ "$service" = "nlg" ]; then
            image_name="sutra-works-nlg-service"
        elif [ "$service" = "storage" ]; then
            image_name="sutra-works-storage-server"
        fi
        
        local status="❌ Not built"
        if docker images --format "{{.Repository}}:{{.Tag}}" | grep -q "^$image_name:latest$"; then
            status="✅ Built"
            optimized_count=$((optimized_count + 1))
        fi
        
        local strategy
        strategy=$(get_service_strategy "$service")
        printf "%-15s %-15s %s\n" "$service" "($strategy)" "$status"
    done
    
    echo ""
    log_info "Build Progress: $optimized_count/${#services[@]} services built"
    echo ""
    
    if [ $optimized_count -gt 0 ]; then
        show_size_comparison
    fi
}

interactive_menu() {
    while true; do
        clear
        print_banner
        
        echo -e "${BOLD}Main Menu:${NC}"
        echo ""
        echo "🔨 Build Options:"
        echo "  1) Build all optimized services"
        echo "  2) Build specific service"
        echo "  3) Build ML services only (embedding + nlg)"
        echo ""
        echo "📊 Analysis & Testing:"
        echo "  4) Show size comparison"
        echo "  5) Show optimization status"
        echo "  6) Test optimized images"
        echo ""
        echo "🚀 Deployment:"
        echo "  7) Deploy with optimized images"
        echo "  8) Create Docker Compose override"
        echo ""
        echo "⚙️  Configuration:"
        echo "  9) Change edition (current: $EDITION)"
        echo "  10) Toggle build options"
        echo ""
        echo "🧹 Maintenance:"
        echo "  11) Clean Docker images/cache"
        echo "  12) View documentation"
        echo ""
        echo "  0) Exit"
        echo ""
        
        read -p "Select an option [0-12]: " choice
        echo ""
        
        case $choice in
            1)
                build_all_services
                read -p "Press Enter to continue..." -r
                ;;
            2)
                echo "Available services: embedding nlg hybrid control api bulk-ingester storage client"
                read -p "Enter service name: " service
                if [[ " embedding nlg hybrid control api bulk-ingester storage client " =~ " $service " ]]; then
                    build_service "$service"
                else
                    log_error "Invalid service: $service"
                fi
                read -p "Press Enter to continue..." -r
                ;;
            3)
                log_header "🤖 Building ML Services"
                build_service "embedding"
                echo ""
                build_service "nlg"
                read -p "Press Enter to continue..." -r
                ;;
            4)
                show_size_comparison
                read -p "Press Enter to continue..." -r
                ;;
            5)
                show_optimization_status
                read -p "Press Enter to continue..." -r
                ;;
            6)
                test_optimized_images
                read -p "Press Enter to continue..." -r
                ;;
            7)
                deploy_optimized
                read -p "Press Enter to continue..." -r
                ;;
            8)
                create_docker_compose_override
                read -p "Press Enter to continue..." -r
                ;;
            9)
                echo "Available editions: simple community enterprise"
                read -p "Enter new edition [$EDITION]: " new_edition
                if [[ " simple community enterprise " =~ " $new_edition " ]]; then
                    EDITION="$new_edition"
                    export SUTRA_EDITION="$new_edition"
                    log_success "Edition changed to: $new_edition"
                elif [ -n "$new_edition" ]; then
                    log_error "Invalid edition: $new_edition"
                fi
                read -p "Press Enter to continue..." -r
                ;;
            10)
                configure_build_options
                ;;
            11)
                clean_images
                read -p "Press Enter to continue..." -r
                ;;
            12)
                show_documentation
                read -p "Press Enter to continue..." -r
                ;;
            0)
                log_info "Goodbye!"
                exit 0
                ;;
            *)
                log_error "Invalid option: $choice"
                read -p "Press Enter to continue..." -r
                ;;
        esac
    done
}

create_docker_compose_override() {
    log_header "📝 Creating Docker Compose Override"
    echo ""
    
    local override_file="docker-compose.override.yml"
    
    if [ -f "$override_file" ]; then
        log_warning "Override file already exists: $override_file"
        read -p "Overwrite? [y/N] " -n 1 -r
        echo ""
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_info "Cancelled"
            return
        fi
    fi
    
    cat > "$override_file" << 'EOF'
version: '3.8'

services:
  embedding-service:
    image: sutra-embedding-service:${SUTRA_VERSION:-latest}
    
  nlg-service:
    image: sutra-nlg-service:${SUTRA_VERSION:-latest}
    
  hybrid-service:
    image: sutra-hybrid:${SUTRA_VERSION:-latest}
    
  control-service:
    image: sutra-control:${SUTRA_VERSION:-latest}
    
  api-service:
    image: sutra-api:${SUTRA_VERSION:-latest}
    
  bulk-ingester:
    image: sutra-bulk-ingester:${SUTRA_VERSION:-latest}
    
  client-service:
    image: sutra-client:${SUTRA_VERSION:-latest}
    
  storage-service:
    image: sutra-storage-server:${SUTRA_VERSION:-latest}
EOF
    
    log_success "Created $override_file"
    log_info "Use with: docker-compose -f docker-compose.yml -f $override_file up"
}

configure_build_options() {
    clear
    log_header "⚙️ Build Configuration"
    echo ""
    echo "Current settings:"
    echo "  Edition: $EDITION"
    echo "  No Cache: $NO_CACHE"
    echo "  Parallel: $PARALLEL"
    echo "  Push Images: $PUSH_IMAGES"
    echo ""
    
    echo "1) Toggle no-cache (force rebuild): $NO_CACHE"
    echo "2) Toggle parallel builds: $PARALLEL"  
    echo "3) Toggle push to registry: $PUSH_IMAGES"
    echo "0) Back to main menu"
    echo ""
    
    read -p "Select option [0-3]: " choice
    
    case $choice in
        1)
            if [ "$NO_CACHE" = "true" ]; then
                NO_CACHE="false"
                export NO_CACHE="false"
            else
                NO_CACHE="true"
                export NO_CACHE="true"
            fi
            log_success "No-cache set to: $NO_CACHE"
            ;;
        2)
            if [ "$PARALLEL" = "true" ]; then
                PARALLEL="false"
                export PARALLEL="false"
            else
                PARALLEL="true"
                export PARALLEL="true"
            fi
            log_success "Parallel builds set to: $PARALLEL"
            ;;
        3)
            if [ "$PUSH_IMAGES" = "true" ]; then
                PUSH_IMAGES="false"
                export PUSH_IMAGES="false"
            else
                PUSH_IMAGES="true"
                export PUSH_IMAGES="true"
            fi
            log_success "Push images set to: $PUSH_IMAGES"
            ;;
        0)
            return
            ;;
        *)
            log_error "Invalid option"
            ;;
    esac
    
    read -p "Press Enter to continue..." -r
    configure_build_options
}

show_documentation() {
    log_header "📚 Documentation Links"
    echo ""
    echo "Available documentation:"
    echo ""
    echo "📖 Complete Guide:"
    echo "   docs/deployment/OPTIMIZED_DOCKER_GUIDE.md"
    echo ""
    echo "⚡ Quick Reference:"  
    echo "   docs/deployment/OPTIMIZED_BUILD_QUICK_REF.md"
    echo ""
    echo "🗂️  Deployment Index:"
    echo "   docs/deployment/README.md"
    echo ""
    echo "📊 Optimization Results:"
    echo "   docs/AGGRESSIVE_ML_OPTIMIZATION_RESULTS.md"
    echo ""
    log_info "Open these files in your editor for detailed instructions"
}

show_help() {
    echo "Usage: ./sutra-optimize.sh [COMMAND] [OPTIONS]"
    echo ""
    echo "Default Action (no arguments):"
    echo "  ./sutra-optimize.sh              Build all + deploy (simple edition)"
    echo ""
    echo "Commands:"
    echo "  build-all              Build all services"
    echo "  build <service>        Build specific service"
    echo "  build-ml               Build ML services (embedding + nlg)"
    echo "  sizes                  Show built image sizes"
    echo "  status                 Show build status"
    echo "  test                   Test built images"
    echo "  deploy                 Deploy with built images"
    echo "  clean                  Clean Docker cache and images"
    echo "  menu                   Interactive menu"
    echo "  help                   Show this help"
    echo ""
    echo "Options:"
    echo "  SUTRA_EDITION=simple|community|enterprise"
    echo "  NO_CACHE=true         Force rebuild without cache"
    echo "  PARALLEL=true         Build services in parallel"
    echo "  PUSH_IMAGES=true      Push to registry after build"
    echo ""
    echo "Examples:"
    echo "  ./sutra-optimize.sh                    # Build all + deploy (default)"
    echo "  ./sutra-optimize.sh menu               # Interactive menu"
    echo "  ./sutra-optimize.sh build-all          # Build all services only"
    echo "  ./sutra-optimize.sh build embedding    # Build embedding service"
    echo "  ./sutra-optimize.sh sizes              # Show built image sizes"
    echo "  NO_CACHE=true ./sutra-optimize.sh      # Force rebuild + deploy"
    echo ""
}

# Main execution
if [ $# -eq 0 ]; then
    # No arguments - build and deploy simple edition (default behavior)
    log_header "🚀 Default Action: Build & Deploy Simple Edition"
    echo ""
    log_info "Building all services for edition: $EDITION"
    echo ""
    
    if build_all_services; then
        echo ""
        log_header "🚀 Deploying Built Images"
        echo ""
        deploy_optimized
    else
        log_error "Build failed. Deployment skipped."
        echo ""
        log_info "Use './sutra-optimize.sh menu' for interactive options"
        exit 1
    fi
else
    # Command-line mode
    COMMAND="$1"
    shift
    
    case "$COMMAND" in
        build-all)
            build_all_services
            ;;
        build)
            if [ $# -eq 0 ]; then
                log_error "Service name required"
                echo "Available: ml-base embedding nlg hybrid control api bulk-ingester storage client"
                exit 1
            fi
            
            # Handle ml-base specially
            if [ "$1" = "ml-base" ]; then
                build_ml_base
            else
                # For ML services, ensure ml-base is built first
                if [ "$1" = "embedding" ] || [ "$1" = "nlg" ]; then
                    log_info "Building ML base foundation first (required for $1)..."
                    if ! build_ml_base; then
                        log_error "Failed to build ML base - cannot build $1"
                        exit 1
                    fi
                    echo ""
                fi
                build_service "$1"
            fi
            ;;
        build-ml)
            log_header "🤖 Building ML Services"
            log_info "Building ML base foundation first..."
            if ! build_ml_base; then
                log_error "Failed to build ML base - cannot build ML services"
                exit 1
            fi
            echo ""
            build_service "embedding"
            echo ""
            build_service "nlg"
            ;;
        sizes|compare)
            show_size_comparison
            ;;
        status)
            show_optimization_status
            ;;
        test)
            test_optimized_images
            ;;
        deploy)
            deploy_optimized
            ;;
        clean)
            clean_images
            ;;
        menu)
            interactive_menu
            ;;
        help|--help|-h)
            show_help
            ;;
        *)
            log_error "Unknown command: $COMMAND"
            echo ""
            show_help
            exit 1
            ;;
    esac
fi