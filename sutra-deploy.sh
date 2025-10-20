#!/bin/bash
# Sutra Grid Deployment Manager
# Single source of truth for all deployment operations

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
COMPOSE_FILE="docker-compose-grid.yml"
PROJECT_NAME="sutra-grid"

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

print_header() {
    echo ""
    echo "╔═══════════════════════════════════════════════════════════════╗"
    echo "║          Sutra Grid Deployment Manager v1.0                  ║"
    echo "╚═══════════════════════════════════════════════════════════════╝"
    echo ""
}

# Command functions
cmd_build() {
    log_info "Building self-contained Docker images..."
    
    ./build.sh
    
    log_success "All images built successfully!"
    echo ""
    log_info "Built images:"
    docker images | grep -E "(sutra-|sutra-base)" | head -20
}

cmd_up() {
    log_info "Starting Sutra Grid system..."
    
    docker-compose -f $COMPOSE_FILE up -d
    
    echo ""
    log_success "Sutra Grid system started!"
    echo ""
    cmd_status
    
    # Validate critical services
    echo ""
    log_info "Validating critical services..."
    if validate_embedding_service && validate_hybrid_service; then
        log_success "All critical services validated successfully!"
    else
        log_warning "Some validations failed. Check service logs for details."
        log_info "Run './sutra-deploy.sh validate' for detailed health checks."
    fi
}

cmd_validate() {
    log_info "Running comprehensive service validation..."
    echo ""
    
    local failures=0
    
    # Check if all containers are running
    log_info "Checking container status..."
    local containers=("sutra-storage" "sutra-embedding-service" "sutra-hybrid" "sutra-api" "sutra-control" "sutra-client" "sutra-grid-master" "sutra-grid-events" "sutra-grid-agent-1" "sutra-grid-agent-2")
    
    for container in "${containers[@]}"; do
        if docker ps | grep -q "$container"; then
            log_success "$container: Running"
        else
            log_error "$container: Not running"
            ((failures++))
        fi
    done
    
    echo ""
    
    # Validate embedding service
    if ! validate_embedding_service; then
        ((failures++))
    fi
    echo ""
    
    # Validate hybrid service
    if ! validate_hybrid_service; then
        ((failures++))
    fi
    echo ""
    
    # Test key endpoints
    log_info "Testing service endpoints..."
    local endpoints=(
        "http://localhost:8000/health:API"
        "http://localhost:8001/ping:Hybrid"
        "http://localhost:8888/health:Embedding"
        "http://localhost:9000/health:Control"
        "http://localhost:7001/health:Grid Master"
    )
    
    for endpoint in "${endpoints[@]}"; do
        local url="${endpoint%:*}"
        local name="${endpoint#*:}"
        if curl -f -s "$url" > /dev/null 2>&1; then
            log_success "$name endpoint: Responding"
        else
            log_error "$name endpoint: Not responding"
            ((failures++))
        fi
    done
    
    echo ""
    
    # Final summary
    if [ $failures -eq 0 ]; then
        log_success "All validation checks passed! System is production-ready."
        return 0
    else
        log_error "$failures validation checks failed. System is NOT production-ready."
        log_info "Check service logs and fix issues before deployment."
        return 1
    fi
}

cmd_down() {
    log_info "Stopping Sutra Grid system..."
    
    docker-compose -f $COMPOSE_FILE down
    
    log_success "Sutra Grid system stopped!"
}

cmd_restart() {
    log_info "Restarting Sutra Grid system..."
    
    cmd_down
    sleep 2
    cmd_up
}

cmd_status() {
    log_info "System Status:"
    echo ""
    docker-compose -f $COMPOSE_FILE ps
    echo ""
    
    log_info "Service URLs:"
    echo "  • Sutra Control Center: http://localhost:9000"
    echo "  • Sutra Client (UI):    http://localhost:8080"
    echo "  • Sutra API:            http://localhost:8000"
    echo "  • Sutra Hybrid API:     http://localhost:8001"
    echo "  • Sutra Embedding Service: http://localhost:8888"
    echo "  • Grid Master (HTTP):   http://localhost:7001"
    echo "  • Grid Master (gRPC):   localhost:7002"
    echo "  • Storage Server:       localhost:50051"
}

validate_embedding_service() {
    log_info "Validating critical embedding service..."
    
    # Check if container is running
    if ! docker ps | grep -q "sutra-embedding-service"; then
        log_error "Embedding service container not running!"
        return 1
    fi
    
    # Wait for service to be ready (up to 90 seconds)
    log_info "Waiting for embedding service to load model..."
    for i in {1..18}; do
        if curl -f -s http://localhost:8888/health > /dev/null 2>&1; then
            break
        fi
        echo -n "."
        sleep 5
    done
    echo ""
    
    # Test health endpoint
    if ! curl -f -s http://localhost:8888/health > /dev/null 2>&1; then
        log_error "Embedding service health check failed!"
        log_info "Checking logs:"
        docker logs sutra-embedding-service --tail 10
        return 1
    fi
    
    # Validate model and dimension
    local health_response
    health_response=$(curl -s http://localhost:8888/health 2>/dev/null || echo "{}")
    
    local model_loaded
    local dimension
    model_loaded=$(echo "$health_response" | jq -r '.model_loaded // false' 2>/dev/null || echo "false")
    dimension=$(echo "$health_response" | jq -r '.dimension // 0' 2>/dev/null || echo "0")
    
    if [ "$model_loaded" != "true" ]; then
        log_error "Embedding model not loaded!"
        return 1
    fi
    
    if [ "$dimension" != "768" ]; then
        log_error "Wrong embedding dimension: $dimension (expected 768)"
        return 1
    fi
    
    # Test embedding generation
    local embed_response
    embed_response=$(curl -s -X POST http://localhost:8888/embed \
        -H "Content-Type: application/json" \
        -d '{"texts": ["test"], "normalize": true}' 2>/dev/null || echo "{}")
    
    local embed_length
    embed_length=$(echo "$embed_response" | jq -r '.embeddings[0] | length' 2>/dev/null || echo "0")
    
    if [ "$embed_length" != "768" ]; then
        log_error "Embedding generation failed (got $embed_length dimensions)"
        return 1
    fi
    
    log_success "Embedding service fully operational (nomic-embed-text-v1.5, 768-d)"
    return 0
}

validate_hybrid_service() {
    log_info "Validating hybrid service connection to embedding service..."
    
    # Check if hybrid is healthy (not restarting)
    local hybrid_status
    hybrid_status=$(docker ps --format "table {{.Names}}\t{{.Status}}" | grep sutra-hybrid | awk '{print $2}' || echo "Down")
    
    if echo "$hybrid_status" | grep -q "Restarting"; then
        log_error "Hybrid service is restarting! Check embedding service connection."
        log_info "Recent hybrid logs:"
        docker logs sutra-hybrid --tail 5 2>&1 | grep -E "(embedding|PRODUCTION)"
        return 1
    fi
    
    # Test hybrid ping endpoint
    if ! curl -f -s http://localhost:8001/ping > /dev/null 2>&1; then
        log_warning "Hybrid service ping endpoint not responding"
        return 1
    fi
    
    log_success "Hybrid service operational and connected to embedding service"
    return 0
}

cmd_logs() {
    local service=$1
    
    if [ -z "$service" ]; then
        log_info "Showing logs for all services (Ctrl+C to exit)..."
        docker-compose -f $COMPOSE_FILE logs -f
    else
        log_info "Showing logs for $service (Ctrl+C to exit)..."
        docker-compose -f $COMPOSE_FILE logs -f $service
    fi
}

cmd_clean() {
    log_warning "This will remove all containers, volumes, and images!"
    read -p "Are you sure? (yes/no): " confirm
    
    if [ "$confirm" != "yes" ]; then
        log_info "Clean cancelled."
        return
    fi
    
    log_info "Cleaning up..."
    
    # Stop and remove containers
    docker-compose -f $COMPOSE_FILE down -v
    
    # Remove images
    docker images | grep sutra | awk '{print $3}' | xargs -r docker rmi -f
    
    log_success "Cleanup complete!"
}

cmd_install() {
    log_info "Installing Sutra Grid system..."
    
    # Check prerequisites
    log_info "Checking prerequisites..."
    
    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed. Please install Docker first."
        exit 1
    fi
    
    if ! command -v docker-compose &> /dev/null; then
        log_error "Docker Compose is not installed. Please install Docker Compose first."
        exit 1
    fi
    
    log_success "Prerequisites OK"
    
    # Build images
    cmd_build
    
    # Start services
    cmd_up
    
    echo ""
    log_success "Installation complete!"
    log_info "Access the Control Center at: http://localhost:9000"
}

cmd_maintenance() {
    echo ""
    echo "╔═══════════════════════════════════════════════════════════════╗"
    echo "║                    Maintenance Menu                           ║"
    echo "╚═══════════════════════════════════════════════════════════════╝"
    echo ""
    echo "  1) View system status"
    echo "  2) Check service health"
    echo "  3) Restart unhealthy services"
    echo "  4) View logs"
    echo "  5) Clean up unused resources"
    echo "  6) Backup data volumes"
    echo "  7) Exit"
    echo ""
    read -p "Select option: " option
    
    case $option in
        1)
            cmd_status
            ;;
        2)
            log_info "Checking service health..."
            docker-compose -f $COMPOSE_FILE ps --format json | jq -r '.[] | "\(.Service): \(.Health)"'
            ;;
        3)
            log_info "Restarting unhealthy services..."
            unhealthy=$(docker-compose -f $COMPOSE_FILE ps --format json | jq -r '.[] | select(.Health == "unhealthy") | .Service')
            if [ -z "$unhealthy" ]; then
                log_success "All services are healthy!"
            else
                for service in $unhealthy; do
                    log_warning "Restarting $service..."
                    docker-compose -f $COMPOSE_FILE restart $service
                done
                log_success "Services restarted!"
            fi
            ;;
        4)
            read -p "Enter service name (or press Enter for all): " service
            cmd_logs $service
            ;;
        5)
            log_info "Cleaning up unused Docker resources..."
            docker system prune -f
            log_success "Cleanup complete!"
            ;;
        6)
            log_info "Backing up data volumes..."
            timestamp=$(date +%Y%m%d_%H%M%S)
            mkdir -p backups
            docker run --rm -v sutra-models_storage-data:/data -v $(pwd)/backups:/backup alpine tar czf /backup/storage-data-$timestamp.tar.gz -C /data .
            docker run --rm -v sutra-models_grid-event-data:/data -v $(pwd)/backups:/backup alpine tar czf /backup/grid-event-data-$timestamp.tar.gz -C /data .
            log_success "Backups saved to: backups/"
            ;;
        7)
            exit 0
            ;;
        *)
            log_error "Invalid option"
            ;;
    esac
}

cmd_help() {
    echo "Usage: ./sutra-deploy.sh [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  install      - First-time installation (build + start)"
    echo "  build        - Build all Docker images"
    echo "  up           - Start all services"
    echo "  down         - Stop all services"
    echo "  restart      - Restart all services"
    echo "  status       - Show service status and URLs"
    echo "  validate     - Run comprehensive health checks (including embedding service)"
    echo "  logs [svc]   - Show logs (optionally for specific service)"
    echo "  clean        - Remove all containers, volumes, and images"
    echo "  maintenance  - Interactive maintenance menu"
    echo "  help         - Show this help message"
    echo ""
    echo "Examples:"
    echo "  ./sutra-deploy.sh install         # First-time setup"
    echo "  ./sutra-deploy.sh up              # Start services"
    echo "  ./sutra-deploy.sh validate        # Check system health"
    echo "  ./sutra-deploy.sh logs sutra-api  # View API logs"
    echo "  ./sutra-deploy.sh maintenance     # Maintenance menu"
    echo ""
}

# Main script
print_header

# Check if compose file exists
if [ ! -f "$COMPOSE_FILE" ]; then
    log_error "Docker Compose file not found: $COMPOSE_FILE"
    exit 1
fi

# Parse command
COMMAND=${1:-help}

case $COMMAND in
    install)
        cmd_install
        ;;
    build)
        cmd_build
        ;;
    up|start)
        cmd_up
        ;;
    down|stop)
        cmd_down
        ;;
    restart)
        cmd_restart
        ;;
    status)
        cmd_status
        ;;
    validate)
        cmd_validate
        ;;
    logs)
        cmd_logs $2
        ;;
    clean)
        cmd_clean
        ;;
    maintenance|maint)
        cmd_maintenance
        ;;
    help|--help|-h)
        cmd_help
        ;;
    *)
        log_error "Unknown command: $COMMAND"
        echo ""
        cmd_help
        exit 1
        ;;
esac

echo ""
