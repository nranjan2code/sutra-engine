#!/bin/bash
# Comprehensive Docker Pipeline Diagnostic Script
# Identifies and reports all issues preventing services from running

set -e

echo "=========================================="
echo "🔬 Docker Pipeline Diagnostic Tool"
echo "=========================================="
echo ""

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

ISSUES_FOUND=0

check_issue() {
    echo -e "${RED}❌ ISSUE${NC}: $1"
    echo -e "   ${YELLOW}Fix${NC}: $2"
    echo ""
    ISSUES_FOUND=$((ISSUES_FOUND + 1))
}

check_ok() {
    echo -e "${GREEN}✅ OK${NC}: $1"
}

check_warn() {
    echo -e "${YELLOW}⚠️  WARN${NC}: $1"
}

# ============================================================================
# 1. Check Docker Environment
# ============================================================================
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "1️⃣  Docker Environment"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if ! command -v docker &> /dev/null; then
    check_issue "Docker not installed" "Install Docker Desktop from docker.com"
else
    check_ok "Docker installed"
    echo "   Version: $(docker --version)"
fi

if ! docker info &> /dev/null; then
    check_issue "Docker daemon not running" "Start Docker Desktop"
else
    check_ok "Docker daemon running"
fi

echo ""

# ============================================================================
# 2. Check Required Files
# ============================================================================
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "2️⃣  Required Files"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

check_file() {
    if [ -f "$1" ]; then
        check_ok "Found $1"
    else
        check_issue "Missing $1" "Create this file or restore from backup"
    fi
}

check_file "docker-compose-grid.yml"
check_file "Dockerfile.test-storage"
check_file "packages/sutra-api/Dockerfile"
check_file "packages/sutra-hybrid/Dockerfile"
check_file "packages/sutra-client/Dockerfile"
check_file "packages/sutra-control/Dockerfile"
check_file "packages/sutra-storage-client-tcp/sutra_storage_client/__init__.py"

echo ""

# ============================================================================
# 3. Check Python Package Dependencies
# ============================================================================
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "3️⃣  Python Package Dependencies"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

check_py_package() {
    if [ -d "$1" ]; then
        if [ -f "$1/setup.py" ] || [ -f "$1/pyproject.toml" ]; then
            check_ok "Package $1 is installable"
        else
            check_warn "Package $1 missing setup.py/pyproject.toml"
        fi
    else
        check_issue "Package directory $1 not found" "Check package structure"
    fi
}

check_py_package "packages/sutra-core"
check_py_package "packages/sutra-storage-client-tcp"
check_py_package "packages/sutra-api"
check_py_package "packages/sutra-hybrid"

echo ""

# ============================================================================
# 4. Check Storage Server Rust Build
# ============================================================================
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "4️⃣  Storage Server (Rust)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

cd packages/sutra-storage

if cargo check 2>&1 | grep -q "error"; then
    echo -e "${RED}❌ ISSUE${NC}: Rust storage has compilation errors"
    echo "   Checking errors..."
    cargo check 2>&1 | grep "error\[E" | head -5
    echo ""
    check_issue "Rust compilation errors" "Run 'cd packages/sutra-storage && cargo check' to see full errors"
else
    check_ok "Rust storage compiles successfully"
fi

cd ../..

echo ""

# ============================================================================
# 5. Check API Endpoint Configuration
# ============================================================================
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "5️⃣  API Endpoint Configuration"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check if API uses unified pipeline
if grep -q "learn_concept_v2" packages/sutra-api/sutra_api/main.py; then
    check_ok "API /learn uses unified pipeline"
else
    check_issue "API /learn not using unified pipeline" "Update to use learn_concept_v2()"
fi

if grep -q "learn_batch_v2" packages/sutra-api/sutra_api/main.py; then
    check_ok "API /learn/batch uses batch optimization"
else
    check_issue "API /learn/batch not optimized" "Update to use learn_batch_v2()"
fi

echo ""

# ============================================================================
# 6. Check Storage Client TCP
# ============================================================================
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "6️⃣  Storage Client TCP"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if grep -q "def learn_concept_v2" packages/sutra-storage-client-tcp/sutra_storage_client/__init__.py; then
    check_ok "TCP client has learn_concept_v2()"
else
    check_issue "TCP client missing learn_concept_v2()" "Add unified pipeline method"
fi

if grep -q "def learn_batch_v2" packages/sutra-storage-client-tcp/sutra_storage_client/__init__.py; then
    check_ok "TCP client has learn_batch_v2()"
else
    check_issue "TCP client missing learn_batch_v2()" "Add batch method"
fi

echo ""

# ============================================================================
# 7. Check Docker Compose Configuration
# ============================================================================
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "7️⃣  Docker Compose Configuration"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if grep -q "VECTOR_DIMENSION=768" docker-compose-grid.yml; then
    check_ok "VECTOR_DIMENSION=768 configured"
else
    check_issue "VECTOR_DIMENSION not set to 768" "Add to storage-server environment"
fi

if grep -q "SUTRA_EMBEDDING_MODEL=nomic-embed-text" docker-compose-grid.yml; then
    check_ok "SUTRA_EMBEDDING_MODEL=nomic-embed-text configured"
else
    check_issue "SUTRA_EMBEDDING_MODEL not set" "Add to storage-server and sutra-hybrid"
fi

echo ""

# ============================================================================
# 8. Check Dockerfile Syntax
# ============================================================================
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "8️⃣  Dockerfile Syntax"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

check_dockerfile() {
    if [ -f "$1" ]; then
        if docker build -f "$1" -t test-build:syntax-check . --no-cache --quiet 2>&1 | grep -q "error"; then
            check_issue "Dockerfile $1 has syntax errors" "Check Dockerfile syntax"
        else
            check_ok "Dockerfile $1 syntax valid"
        fi
    fi
}

# Note: This is a lightweight check, full builds would take too long

echo ""

# ============================================================================
# 9. Check Port Conflicts
# ============================================================================
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "9️⃣  Port Availability"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

check_port() {
    if lsof -Pi :$1 -sTCP:LISTEN -t >/dev/null 2>&1; then
        check_warn "Port $1 is already in use"
        echo "   Process: $(lsof -Pi :$1 -sTCP:LISTEN | tail -1 | awk '{print $1}')"
    else
        check_ok "Port $1 is available"
    fi
}

check_port 8000  # API
check_port 8001  # Hybrid
check_port 8080  # Client
check_port 9000  # Control
check_port 11434 # Ollama
check_port 50051 # Storage

echo ""

# ============================================================================
# 10. Check Ollama Availability
# ============================================================================
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔟 Ollama Service"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if command -v ollama &> /dev/null; then
    check_ok "Ollama CLI installed"
    
    if curl -s http://localhost:11434/api/tags >/dev/null 2>&1; then
        check_ok "Ollama service running"
        
        if ollama list 2>/dev/null | grep -q "nomic-embed-text"; then
            check_ok "nomic-embed-text model available"
        else
            check_warn "nomic-embed-text not pulled yet"
            echo "   Run: ollama pull nomic-embed-text"
        fi
    else
        check_warn "Ollama service not running (will start in Docker)"
    fi
else
    check_warn "Ollama not installed locally (will use Docker container)"
fi

echo ""

# ============================================================================
# 11. Check Existing Containers
# ============================================================================
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "1️⃣1️⃣  Existing Containers"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if docker ps -a --filter "name=sutra" --format "table {{.Names}}\t{{.Status}}" | grep -v "NAMES" | wc -l | grep -q "0"; then
    check_ok "No existing containers (clean state)"
else
    echo "Existing Sutra containers:"
    docker ps -a --filter "name=sutra" --format "  {{.Names}}: {{.Status}}"
    echo ""
    check_warn "Existing containers found - may need cleanup"
    echo "   Run: docker-compose -f docker-compose-grid.yml down"
fi

echo ""

# ============================================================================
# Summary & Recommendations
# ============================================================================
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 Diagnostic Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

if [ $ISSUES_FOUND -eq 0 ]; then
    echo -e "${GREEN}🎉 No critical issues found!${NC}"
    echo ""
    echo "Your Docker pipeline should be ready to run."
    echo ""
    echo "Next steps:"
    echo "  1. Build images: docker-compose -f docker-compose-grid.yml build"
    echo "  2. Start services: docker-compose -f docker-compose-grid.yml up -d"
    echo "  3. Check logs: docker-compose -f docker-compose-grid.yml logs -f"
    echo "  4. Verify: ./scripts/smoke-test-embeddings.sh"
    echo ""
else
    echo -e "${RED}⚠️  Found $ISSUES_FOUND critical issue(s)${NC}"
    echo ""
    echo "Please fix the issues above before building Docker images."
    echo ""
    echo "Common fixes:"
    echo "  • Rust errors: cd packages/sutra-storage && cargo fix --allow-dirty"
    echo "  • Missing files: Check git status and restore if needed"
    echo "  • Port conflicts: Stop conflicting services"
    echo ""
fi

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
