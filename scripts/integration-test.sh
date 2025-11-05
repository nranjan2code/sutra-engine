#!/bin/bash
# Integration Test Suite for Sutra AI
# Tests end-to-end workflows after deployment

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
API_URL="${API_URL:-http://localhost:8000}"
HYBRID_URL="${HYBRID_URL:-http://localhost:8001}"
EMBEDDING_URL="${EMBEDDING_URL:-http://localhost:8888}"
TIMEOUT=10

TESTS_PASSED=0
TESTS_FAILED=0

log_info() { echo -e "${BLUE}ℹ ${NC}$1"; }
log_success() { echo -e "${GREEN}✓${NC} $1"; ((TESTS_PASSED++)); }
log_error() { echo -e "${RED}✗${NC} $1"; ((TESTS_FAILED++)); }

# Test 1: Learn a concept
test_learn_concept() {
    log_info "Test 1: Learning a concept..."
    
    response=$(curl -s -X POST "$API_URL/api/learn" \
        -H "Content-Type: application/json" \
        -d '{"content": "The sun is a star at the center of our solar system"}' \
        --max-time "$TIMEOUT" 2>/dev/null || echo '{"error": "failed"}')
    
    if echo "$response" | jq -e '.concept_id' >/dev/null 2>&1; then
        concept_id=$(echo "$response" | jq -r '.concept_id')
        log_success "Learned concept with ID: $concept_id"
        echo "$concept_id" > /tmp/sutra_test_concept_id
        return 0
    else
        log_error "Failed to learn concept"
        return 1
    fi
}

# Test 2: Query the learned concept
test_query_concept() {
    log_info "Test 2: Querying learned concept..."
    
    if [ ! -f /tmp/sutra_test_concept_id ]; then
        log_error "No concept ID from previous test"
        return 1
    fi
    
    concept_id=$(cat /tmp/sutra_test_concept_id)
    
    response=$(curl -s -X GET "$API_URL/api/concept/$concept_id" \
        --max-time "$TIMEOUT" 2>/dev/null || echo '{"error": "failed"}')
    
    if echo "$response" | jq -e '.content' >/dev/null 2>&1; then
        log_success "Retrieved concept successfully"
        return 0
    else
        log_error "Failed to retrieve concept"
        return 1
    fi
}

# Test 3: Semantic search
test_semantic_search() {
    log_info "Test 3: Semantic search..."
    
    response=$(curl -s -X POST "$HYBRID_URL/search" \
        -H "Content-Type: application/json" \
        -d '{"query": "solar system", "limit": 5}' \
        --max-time "$TIMEOUT" 2>/dev/null || echo '{"error": "failed"}')
    
    if echo "$response" | jq -e '.results' >/dev/null 2>&1; then
        count=$(echo "$response" | jq '.results | length')
        log_success "Semantic search returned $count results"
        return 0
    else
        log_error "Semantic search failed"
        return 1
    fi
}

# Test 4: Embedding generation
test_embedding_generation() {
    log_info "Test 4: Embedding generation..."
    
    response=$(curl -s -X POST "$EMBEDDING_URL/embed" \
        -H "Content-Type: application/json" \
        -d '{"text": "test embedding generation"}' \
        --max-time "$TIMEOUT" 2>/dev/null || echo '{"error": "failed"}')
    
    if echo "$response" | jq -e '.embedding | length > 0' >/dev/null 2>&1; then
        dimension=$(echo "$response" | jq '.embedding | length')
        log_success "Generated embedding with dimension $dimension"
        return 0
    else
        log_error "Embedding generation failed"
        return 1
    fi
}

# Test 5: Reasoning query
test_reasoning_query() {
    log_info "Test 5: Reasoning query..."
    
    response=$(curl -s -X POST "$API_URL/api/reason" \
        -H "Content-Type: application/json" \
        -d '{"query": "What is the sun?", "max_depth": 3}' \
        --max-time "$TIMEOUT" 2>/dev/null || echo '{"error": "failed"}')
    
    if echo "$response" | jq -e '.reasoning_paths' >/dev/null 2>&1; then
        paths=$(echo "$response" | jq '.reasoning_paths | length')
        log_success "Reasoning returned $paths paths"
        return 0
    else
        log_error "Reasoning query failed"
        return 1
    fi
}

# Main test suite
main() {
    echo ""
    echo "═══════════════════════════════════════════════════════════"
    echo "   🧪 SUTRA AI INTEGRATION TEST SUITE"
    echo "═══════════════════════════════════════════════════════════"
    echo ""
    
    log_info "Testing against:"
    echo "  API: $API_URL"
    echo "  Hybrid: $HYBRID_URL"
    echo "  Embedding: $EMBEDDING_URL"
    echo ""
    
    # Run tests
    test_learn_concept || true
    test_query_concept || true
    test_semantic_search || true
    test_embedding_generation || true
    test_reasoning_query || true
    
    # Cleanup
    rm -f /tmp/sutra_test_concept_id
    
    # Summary
    echo ""
    echo "═══════════════════════════════════════════════════════════"
    echo "   📊 TEST RESULTS"
    echo "═══════════════════════════════════════════════════════════"
    echo ""
    echo -e "${GREEN}Passed:${NC} $TESTS_PASSED"
    echo -e "${RED}Failed:${NC} $TESTS_FAILED"
    echo ""
    
    if [ $TESTS_FAILED -eq 0 ]; then
        echo -e "${GREEN}✓ All integration tests PASSED${NC}"
        return 0
    else
        echo -e "${RED}✗ Some integration tests FAILED${NC}"
        return 1
    fi
}

main
exit $?
