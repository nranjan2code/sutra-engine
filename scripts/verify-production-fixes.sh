#!/bin/bash
# Production verification script for unified learning architecture fixes
# Validates all critical fixes are working correctly

set -e

echo "=========================================="
echo "🔍 Production Verification Script"
echo "   Unified Learning Architecture"
echo "=========================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

FAILED_CHECKS=0
PASSED_CHECKS=0

check_pass() {
    echo -e "${GREEN}✅ PASS${NC}: $1"
    PASSED_CHECKS=$((PASSED_CHECKS + 1))
}

check_fail() {
    echo -e "${RED}❌ FAIL${NC}: $1"
    FAILED_CHECKS=$((FAILED_CHECKS + 1))
}

check_warn() {
    echo -e "${YELLOW}⚠️  WARN${NC}: $1"
}

echo "📋 Running production verification checks..."
echo ""

# ============================================================================
# Check 1: Default Embedding Model Configuration
# ============================================================================
echo "1️⃣  Checking default embedding model configuration..."

if grep -q 'default_model.*nomic-embed-text' packages/sutra-storage/src/embedding_client.rs; then
    check_pass "Default embedding model is nomic-embed-text"
else
    check_fail "Default embedding model not set to nomic-embed-text"
fi

# ============================================================================
# Check 2: API Endpoints Use Unified Pipeline
# ============================================================================
echo ""
echo "2️⃣  Checking API endpoints use unified pipeline..."

if grep -q 'learn_concept_v2' packages/sutra-api/sutra_api/main.py; then
    check_pass "API /learn endpoint uses learn_concept_v2()"
else
    check_fail "API /learn endpoint not using unified pipeline"
fi

if grep -q 'learn_batch_v2' packages/sutra-api/sutra_api/main.py; then
    check_pass "API /learn/batch endpoint uses learn_batch_v2()"
else
    check_fail "API /learn/batch endpoint not using batch optimization"
fi

# ============================================================================
# Check 3: Storage Server Learning Pipeline Exists
# ============================================================================
echo ""
echo "3️⃣  Checking storage server learning pipeline..."

if [ -f "packages/sutra-storage/src/learning_pipeline.rs" ]; then
    check_pass "learning_pipeline.rs exists"
else
    check_fail "learning_pipeline.rs not found"
fi

if [ -f "packages/sutra-storage/src/embedding_client.rs" ]; then
    check_pass "embedding_client.rs exists"
else
    check_fail "embedding_client.rs not found"
fi

if [ -f "packages/sutra-storage/src/association_extractor.rs" ]; then
    check_pass "association_extractor.rs exists"
else
    check_fail "association_extractor.rs not found"
fi

# ============================================================================
# Check 4: TCP Client Has learn_concept_v2
# ============================================================================
echo ""
echo "4️⃣  Checking TCP client has unified API..."

if grep -q 'def learn_concept_v2' packages/sutra-storage-client-tcp/sutra_storage_client/__init__.py; then
    check_pass "TCP client has learn_concept_v2()"
else
    check_fail "TCP client missing learn_concept_v2()"
fi

if grep -q 'def learn_batch_v2' packages/sutra-storage-client-tcp/sutra_storage_client/__init__.py; then
    check_pass "TCP client has learn_batch_v2()"
else
    check_fail "TCP client missing learn_batch_v2()"
fi

# ============================================================================
# Check 5: ReasoningEngine Extracts Parameters Correctly
# ============================================================================
echo ""
echo "5️⃣  Checking ReasoningEngine.learn() implementation..."

if grep -q 'generate_embedding = kwargs.get("generate_embedding"' packages/sutra-core/sutra_core/reasoning/engine.py; then
    check_pass "ReasoningEngine extracts parameters from kwargs"
else
    check_fail "ReasoningEngine not extracting parameters correctly"
fi

# Check it doesn't pass options dict
if grep -q 'options=kwargs' packages/sutra-core/sutra_core/reasoning/engine.py; then
    check_fail "ReasoningEngine passing options dict (should pass individual params)"
else
    check_pass "ReasoningEngine passes individual parameters"
fi

# ============================================================================
# Check 6: Test Files Exist
# ============================================================================
echo ""
echo "6️⃣  Checking test coverage..."

if [ -f "tests/test_unified_learning_integration.py" ]; then
    check_pass "Integration tests exist"
else
    check_fail "Integration tests missing"
fi

if [ -f "tests/test_failure_scenarios.py" ]; then
    check_pass "Failure scenario tests exist"
else
    check_fail "Failure scenario tests missing"
fi

# ============================================================================
# Check 7: Documentation
# ============================================================================
echo ""
echo "7️⃣  Checking documentation..."

if [ -f "docs/MIGRATION_UNIFIED_LEARNING.md" ]; then
    check_pass "Migration documentation exists"
else
    check_fail "Migration documentation missing"
fi

if [ -f "docs/UNIFIED_LEARNING_ARCHITECTURE.md" ]; then
    check_pass "Architecture documentation exists"
else
    check_warn "Architecture documentation missing (optional)"
fi

# ============================================================================
# Check 8: Docker Configuration (if exists)
# ============================================================================
echo ""
echo "8️⃣  Checking Docker configuration..."

if [ -f "docker-compose-grid.yml" ]; then
    if grep -q 'VECTOR_DIMENSION.*768' docker-compose-grid.yml; then
        check_pass "Docker config has VECTOR_DIMENSION=768"
    else
        check_warn "Docker config missing VECTOR_DIMENSION=768"
    fi
    
    if grep -q 'SUTRA_EMBEDDING_MODEL.*nomic-embed-text' docker-compose-grid.yml; then
        check_pass "Docker config has SUTRA_EMBEDDING_MODEL=nomic-embed-text"
    else
        check_warn "Docker config missing SUTRA_EMBEDDING_MODEL=nomic-embed-text"
    fi
else
    check_warn "docker-compose-grid.yml not found (may not be needed)"
fi

# ============================================================================
# Check 9: Rust Code Compiles
# ============================================================================
echo ""
echo "9️⃣  Checking Rust code compilation..."

cd packages/sutra-storage
if cargo check --quiet 2>/dev/null; then
    check_pass "Rust storage code compiles"
else
    check_fail "Rust storage code has compilation errors"
fi
cd ../..

# ============================================================================
# Check 10: Python Imports Work
# ============================================================================
echo ""
echo "🔟 Checking Python imports..."

if python3 -c "from sutra_storage_client import StorageClient; print('OK')" 2>/dev/null | grep -q "OK"; then
    check_pass "sutra_storage_client imports successfully"
else
    check_warn "sutra_storage_client not installed or has import errors"
fi

# ============================================================================
# Summary
# ============================================================================
echo ""
echo "=========================================="
echo "📊 Verification Summary"
echo "=========================================="
echo -e "✅ Passed: ${GREEN}$PASSED_CHECKS${NC}"
echo -e "❌ Failed: ${RED}$FAILED_CHECKS${NC}"
echo ""

if [ $FAILED_CHECKS -eq 0 ]; then
    echo -e "${GREEN}🎉 All critical checks passed!${NC}"
    echo ""
    echo "Next steps:"
    echo "  1. Rebuild storage server: cd packages/sutra-storage && cargo build --release"
    echo "  2. Rebuild Docker images: docker-compose -f docker-compose-grid.yml build"
    echo "  3. Run integration tests: pytest tests/test_unified_learning_integration.py -v"
    echo "  4. Run failure tests: pytest tests/test_failure_scenarios.py -v"
    echo "  5. Deploy and verify: ./sutra-deploy.sh up && ./scripts/smoke-test-embeddings.sh"
    echo ""
    exit 0
else
    echo -e "${RED}⚠️  $FAILED_CHECKS critical checks failed!${NC}"
    echo ""
    echo "Please fix the failed checks before deploying to production."
    echo "See MIGRATION_UNIFIED_LEARNING.md for details."
    echo ""
    exit 1
fi
