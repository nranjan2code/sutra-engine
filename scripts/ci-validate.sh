#!/bin/bash
# Production CI/CD validation script
# Runs all quality gates before allowing deployment

set -e  # Exit on error
set -o pipefail  # Catch pipe errors

echo "🔍 Running Production Quality Gates..."

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track failures
FAILURES=0

# Function to report results
report() {
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✓${NC} $1"
    else
        echo -e "${RED}✗${NC} $1"
        ((FAILURES++))
    fi
}

# 1. Python Code Quality
echo ""
echo "📋 Python Code Quality... SKIPPED (Pure Rust)"

# 2. Python Security
echo ""
echo "🔒 Python Security Scanning... SKIPPED (Pure Rust)"

# 3. JavaScript/TypeScript Quality
echo ""
echo "📋 JavaScript/TypeScript Quality..."
echo "Skipping JS check (Frontend moved to Product repo)."

# 4. Rust Quality
echo ""
echo "📋 Rust Code Quality..."
if command -v cargo &> /dev/null; then
    cargo fmt --check && report "Cargo formatting" || report "Cargo formatting"
    cargo clippy -- -D warnings && report "Cargo clippy" || report "Cargo clippy"
else
    echo -e "${YELLOW}⚠${NC} Cargo not installed, skipping"
fi

# 5. Security Checks
echo ""
echo "🔒 Secret Detection..."
if command -v detect-secrets &> /dev/null; then
    detect-secrets scan --baseline .secrets.baseline && report "Secret detection" || report "Secret detection"
else
    echo -e "${YELLOW}⚠${NC} detect-secrets not installed, skipping"
fi

# 6. Tests
echo ""
echo "🧪 Running Tests..."
if command -v cargo &> /dev/null; then
    cargo test --workspace && report "Rust tests" || report "Rust tests"
else
    echo -e "${YELLOW}⚠${NC} Cargo not installed, skipping"
fi

# 7. Bundle Size Check
echo ""
echo "📦 Bundle Size Validation..."
echo "Skipping (Frontend moved to Product repo)."

# 8. Docker Image Validation
echo ""
echo "🐳 Docker Image Validation..."
if command -v docker &> /dev/null; then
    # Check if images exist
    for img in sutra-storage; do
        if docker images | grep -q "$img"; then
            report "$img image exists"
        else
            echo -e "${YELLOW}⚠${NC} $img image not built"
        fi
    done
else
    echo -e "${YELLOW}⚠${NC} Docker not installed, skipping"
fi

# Final Report
echo ""
echo "========================================="
if [ $FAILURES -eq 0 ]; then
    echo -e "${GREEN}✓ All quality gates passed!${NC}"
    echo "Ready for production deployment."
    exit 0
else
    echo -e "${RED}✗ $FAILURES quality gate(s) failed!${NC}"
    echo "Fix issues before deployment."
    exit 1
fi
