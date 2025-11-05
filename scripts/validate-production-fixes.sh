#!/bin/bash
# Quick validation script for production fixes

set -euo pipefail

echo "🔍 PRODUCTION FIXES VALIDATION"
echo "================================"
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

PASSED=0
FAILED=0

check() {
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✓${NC} $1"
        ((PASSED++))
    else
        echo -e "${RED}✗${NC} $1"
        ((FAILED++))
    fi
}

# 1. Check Python dependencies are pinned
echo "1. Checking Python dependency pinning..."
grep -q "sqlalchemy==2.0.35" packages/sutra-core/pyproject.toml
check "SQLAlchemy pinned to 2.0.35"

grep -q "fastapi==0.115.0" packages/sutra-api/pyproject.toml
check "FastAPI pinned to 0.115.0"

grep -q "pytest==8.3.3" pyproject.toml
check "Root pytest pinned to 8.3.3"

# 2. Check React version consistency
echo ""
echo "2. Checking React version consistency..."
react_versions=$(grep -h '"react":' packages/*/package.json 2>/dev/null | sort -u | wc -l)
if [ "$react_versions" -eq 1 ]; then
    echo -e "${GREEN}✓${NC} Single React version across all packages"
    ((PASSED++))
else
    echo -e "${RED}✗${NC} Multiple React versions found"
    ((FAILED++))
fi

grep -q '"react": "18.2.0"' packages/sutra-ui-framework/package.json
check "UI Framework uses React 18.2.0"

# 3. Check test scripts exist and are executable
echo ""
echo "3. Checking test scripts..."
[ -x scripts/smoke-test-embeddings.sh ]
check "Smoke test script exists and is executable"

[ -x scripts/integration-test.sh ]
check "Integration test script exists and is executable"

# 4. Check pytest configuration
echo ""
echo "4. Checking pytest configuration..."
grep -q "cov-fail-under=70" pytest.ini
check "Coverage threshold set to 70%"

grep -q "cov-report=html:htmlcov" pytest.ini
check "HTML coverage report configured"

grep -q "cov-report=xml:coverage.xml" pytest.ini
check "XML coverage report configured"

# 5. Check .gitignore for coverage artifacts
echo ""
echo "5. Checking .gitignore..."
grep -q "coverage.xml" .gitignore
check "coverage.xml in .gitignore"

grep -q "htmlcov/" .gitignore
check "htmlcov/ in .gitignore"

# Summary
echo ""
echo "================================"
echo "📊 VALIDATION RESULTS"
echo "================================"
echo -e "${GREEN}Passed: $PASSED${NC}"
echo -e "${RED}Failed: $FAILED${NC}"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ All production fixes validated successfully!${NC}"
    exit 0
else
    echo -e "${RED}❌ Some validations failed${NC}"
    exit 1
fi
