#!/bin/bash
# Production smoke test for embedding configuration and semantic search
# Validates that all services are correctly configured with nomic-embed-text

set -e

echo "=== Embedding Production Smoke Test ==="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

FAILED=0

# Test 1: Model availability
echo "1. Checking nomic-embed-text availability..."
if ollama list | grep -q nomic-embed-text; then
    echo -e "${GREEN}✅ PASS${NC}"
else
    echo -e "${RED}❌ FAIL: nomic-embed-text not available${NC}"
    echo "   Fix: ollama pull nomic-embed-text"
    FAILED=1
fi

# Test 2: Storage server configuration
echo "2. Checking storage server configuration..."
if docker logs sutra-storage 2>&1 | grep -q "Vector dimension: 768"; then
    echo -e "${GREEN}✅ PASS${NC}"
else
    echo -e "${RED}❌ FAIL: Storage not using 768 dimensions${NC}"
    echo "   Fix: Set VECTOR_DIMENSION=768 in docker-compose-grid.yml"
    FAILED=1
fi

# Test 3: Hybrid service configuration  
echo "3. Checking hybrid service configuration..."
if docker logs sutra-hybrid 2>&1 | grep -q "nomic-embed-text"; then
    echo -e "${GREEN}✅ PASS${NC}"
else
    echo -e "${RED}❌ FAIL: Hybrid not using nomic-embed-text${NC}"
    echo "   Fix: Set SUTRA_EMBEDDING_MODEL=nomic-embed-text in docker-compose-grid.yml"
    FAILED=1
fi

# Test 4: No fallback warnings
echo "4. Checking for fallback warnings..."
if docker logs sutra-hybrid 2>&1 | grep -qi "fallback"; then
    echo -e "${RED}❌ FAIL: Fallback detected in logs${NC}"
    echo "   This indicates missing or incorrect SUTRA_EMBEDDING_MODEL configuration"
    FAILED=1
else
    echo -e "${GREEN}✅ PASS${NC}"
fi

# Test 5: Storage server is running
echo "5. Checking storage server health..."
if docker ps | grep -q "sutra-storage.*Up"; then
    echo -e "${GREEN}✅ PASS${NC}"
else
    echo -e "${RED}❌ FAIL: Storage server not running${NC}"
    echo "   Fix: docker-compose -f docker-compose-grid.yml up -d storage-server"
    FAILED=1
fi

# Test 6: Hybrid service is running
echo "6. Checking hybrid service health..."
if docker ps | grep -q "sutra-hybrid.*Up"; then
    echo -e "${GREEN}✅ PASS${NC}"
else
    echo -e "${RED}❌ FAIL: Hybrid service not running${NC}"
    echo "   Fix: docker-compose -f docker-compose-grid.yml up -d sutra-hybrid"
    FAILED=1
fi

# Test 7: End-to-end semantic search (only if services are running)
if [ $FAILED -eq 0 ]; then
    echo "7. Testing end-to-end semantic search..."
    RESPONSE=$(curl -s -X POST http://localhost:8001/sutra/query \
        -H "Content-Type: application/json" \
        -d '{"query":"test semantic search","use_semantic":true}' 2>&1)
    
    if echo "$RESPONSE" | jq -e '.reasoning_paths' > /dev/null 2>&1; then
        echo -e "${GREEN}✅ PASS${NC}"
    else
        echo -e "${RED}❌ FAIL: No reasoning paths in response${NC}"
        echo "   Response: $RESPONSE"
        FAILED=1
    fi
else
    echo "7. Skipping end-to-end test (previous failures detected)"
fi

echo ""
echo "========================================"
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}=== All Tests Passed ✅ ===${NC}"
    echo ""
    echo "Your system is correctly configured for production."
    echo "All services are using nomic-embed-text (768-d) embeddings."
    exit 0
else
    echo -e "${RED}=== Tests Failed ❌ ===${NC}"
    echo ""
    echo "Please fix the issues above and run this script again."
    echo "See PRODUCTION_REQUIREMENTS.md for detailed troubleshooting."
    exit 1
fi
