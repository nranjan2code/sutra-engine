#!/bin/bash
# Production Smoke Test Suite for Sutra AI
# Tests critical services after deployment

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
STORAGE_HOST="${STORAGE_HOST:-localhost}"
STORAGE_PORT="${STORAGE_PORT:-50051}"
API_HOST="${API_HOST:-localhost}"
API_PORT="${API_PORT:-8000}"
HYBRID_HOST="${HYBRID_HOST:-localhost}"
HYBRID_PORT="${HYBRID_PORT:-8001}"
EMBEDDING_HOST="${EMBEDDING_HOST:-localhost}"
EMBEDDING_PORT="${EMBEDDING_PORT:-8888}"
NLG_HOST="${NLG_HOST:-localhost}"
NLG_PORT="${NLG_PORT:-8003}"
CLIENT_HOST="${CLIENT_HOST:-localhost}"
CLIENT_PORT="${CLIENT_PORT:-8080}"
CONTROL_HOST="${CONTROL_HOST:-localhost}"
CONTROL_PORT="${CONTROL_PORT:-9000}"

TIMEOUT="${SMOKE_TEST_TIMEOUT:-5}"
FAILED_TESTS=0
PASSED_TESTS=0

# Helper functions
log_info() {
    echo -e "${BLUE}ℹ ${NC}$1"
}

log_success() {
    echo -e "${GREEN}✓${NC} $1"
    ((PASSED_TESTS++))
}

log_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

log_error() {
    echo -e "${RED}✗${NC} $1"
    ((FAILED_TESTS++))
}

# Test TCP connection
test_tcp_connection() {
    local host=$1
    local port=$2
    local service=$3
    
    log_info "Testing TCP connection to $service ($host:$port)..."
    
    if timeout "$TIMEOUT" bash -c "exec 3<>/dev/tcp/$host/$port" 2>/dev/null; then
        log_success "$service TCP port is accessible"
        return 0
    else
        log_error "$service TCP port is NOT accessible"
        return 1
    fi
}

# Test HTTP endpoint
test_http_endpoint() {
    local url=$1
    local service=$2
    local expected_status=${3:-200}
    
    log_info "Testing HTTP endpoint: $url"
    
    response=$(curl -s -o /dev/null -w "%{http_code}" --max-time "$TIMEOUT" "$url" 2>/dev/null || echo "000")
    
    if [ "$response" = "$expected_status" ]; then
        log_success "$service HTTP endpoint returned $response"
        return 0
    else
        log_error "$service HTTP endpoint returned $response (expected $expected_status)"
        return 1
    fi
}

# Test embedding service
test_embedding_service() {
    log_info "Testing Embedding Service functionality..."
    
    # Test health endpoint
    if ! test_http_endpoint "http://$EMBEDDING_HOST:$EMBEDDING_PORT/health" "Embedding Service"; then
        return 1
    fi
    
    # Test embedding generation
    log_info "Testing embedding generation..."
    response=$(curl -s --max-time "$TIMEOUT" \
        -X POST "http://$EMBEDDING_HOST:$EMBEDDING_PORT/embed" \
        -H "Content-Type: application/json" \
        -d '{"text": "test embedding"}' 2>/dev/null || echo "{}")
    
    if echo "$response" | jq -e '.embedding | length > 0' >/dev/null 2>&1; then
        log_success "Embedding generation successful"
        return 0
    else
        log_error "Embedding generation failed"
        return 1
    fi
}

# Test storage server
test_storage_server() {
    log_info "Testing Storage Server..."
    
    if test_tcp_connection "$STORAGE_HOST" "$STORAGE_PORT" "Storage Server"; then
        return 0
    else
        return 1
    fi
}

# Test API server
test_api_server() {
    log_info "Testing API Server..."
    
    if ! test_http_endpoint "http://$API_HOST:$API_PORT/health" "API Server"; then
        return 1
    fi
    
    # Test basic learn endpoint (should return 401 if auth enabled, or work if disabled)
    log_info "Testing API learn endpoint..."
    response=$(curl -s -o /dev/null -w "%{http_code}" --max-time "$TIMEOUT" \
        -X POST "http://$API_HOST:$API_PORT/api/learn" \
        -H "Content-Type: application/json" \
        -d '{"content": "test concept"}' 2>/dev/null || echo "000")
    
    # 200 (success), 401 (auth required), or 422 (validation) are all acceptable
    if [[ "$response" =~ ^(200|401|422)$ ]]; then
        log_success "API learn endpoint is functional (status: $response)"
        return 0
    else
        log_error "API learn endpoint failed (status: $response)"
        return 1
    fi
}

# Test hybrid service
test_hybrid_service() {
    log_info "Testing Hybrid Service..."
    
    if test_http_endpoint "http://$HYBRID_HOST:$HYBRID_PORT/ping" "Hybrid Service"; then
        return 0
    else
        return 1
    fi
}

# Test NLG service (optional - may not be deployed)
test_nlg_service() {
    log_info "Testing NLG Service (optional)..."
    
    if test_http_endpoint "http://$NLG_HOST:$NLG_PORT/health" "NLG Service"; then
        return 0
    else
        log_warning "NLG Service not available (this is optional)"
        return 0  # Don't fail if NLG is not deployed
    fi
}

# Test client UI
test_client_ui() {
    log_info "Testing Client UI..."
    
    if test_http_endpoint "http://$CLIENT_HOST:$CLIENT_PORT/" "Client UI"; then
        return 0
    else
        return 1
    fi
}

# Test control center
test_control_center() {
    log_info "Testing Control Center..."
    
    if test_http_endpoint "http://$CONTROL_HOST:$CONTROL_PORT/health" "Control Center"; then
        return 0
    else
        return 1
    fi
}

# Main test suite
main() {
    echo ""
    echo "═══════════════════════════════════════════════════════════"
    echo "   🧪 SUTRA AI PRODUCTION SMOKE TEST SUITE"
    echo "═══════════════════════════════════════════════════════════"
    echo ""
    
    log_info "Starting smoke tests with ${TIMEOUT}s timeout per test..."
    echo ""
    
    # Core services (critical)
    echo "━━━ Core Services ━━━"
    test_storage_server
    test_embedding_service
    test_api_server
    test_hybrid_service
    echo ""
    
    # Optional services
    echo "━━━ Optional Services ━━━"
    test_nlg_service
    echo ""
    
    # UI services
    echo "━━━ UI Services ━━━"
    test_client_ui
    test_control_center
    echo ""
    
    # Summary
    echo "═══════════════════════════════════════════════════════════"
    echo "   📊 TEST RESULTS"
    echo "═══════════════════════════════════════════════════════════"
    echo ""
    echo -e "${GREEN}Passed:${NC} $PASSED_TESTS"
    echo -e "${RED}Failed:${NC} $FAILED_TESTS"
    echo ""
    
    if [ $FAILED_TESTS -eq 0 ]; then
        echo -e "${GREEN}✓ All smoke tests PASSED${NC}"
        echo ""
        return 0
    else
        echo -e "${RED}✗ Some smoke tests FAILED${NC}"
        echo ""
        return 1
    fi
}

# Run tests
main
exit $?
