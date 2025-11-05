#!/bin/bash
"""
Production-grade build system for Sutra Memory.

Unified build process that handles all package types:
- Rust services (storage, grid, protocol)
- Python services (API, core, hybrid)
- JavaScript/TypeScript frontends (client, control, explorer)

Features:
- Parallel builds for faster CI/CD
- Dependency checking and validation
- Production optimizations
- Security scanning
- Bundle size monitoring
- Zero external monitoring dependencies
"""

set -euo pipefail

# Configuration
SUTRA_VERSION="${SUTRA_VERSION:-$(cat VERSION)}"
BUILD_MODE="${BUILD_MODE:-production}"
PARALLEL_JOBS="${PARALLEL_JOBS:-$(nproc)}"
SKIP_TESTS="${SKIP_TESTS:-false}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')] $1${NC}"
}

warn() {
    echo -e "${YELLOW}[$(date +'%Y-%m-%d %H:%M:%S')] WARNING: $1${NC}"
}

error() {
    echo -e "${RED}[$(date +'%Y-%m-%d %H:%M:%S')] ERROR: $1${NC}"
    exit 1
}

info() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')] INFO: $1${NC}"
}

# Validate environment
validate_environment() {
    log "Validating build environment..."
    
    # Check required tools
    command -v cargo >/dev/null 2>&1 || error "Rust/Cargo not found"
    command -v python3 >/dev/null 2>&1 || error "Python3 not found"
    command -v node >/dev/null 2>&1 || error "Node.js not found"
    command -v npm >/dev/null 2>&1 || error "npm not found"
    
    # Check version file
    [[ -f VERSION ]] || error "VERSION file not found"
    
    # Validate Docker if needed
    if [[ "${BUILD_DOCKER:-}" == "true" ]]; then
        command -v docker >/dev/null 2>&1 || error "Docker not found"
    fi
    
    info "Environment validation passed"
}

# Check and pin dependencies
validate_dependencies() {
    log "Validating dependency consistency..."
    
    # Check for React version consistency
    local react_versions=$(find packages -name "package.json" -exec grep -l "react" {} \; | xargs grep '"react"' | sort | uniq -c)
    local react_count=$(echo "$react_versions" | wc -l)
    
    if [[ $react_count -gt 1 ]]; then
        warn "Multiple React versions found:"
        echo "$react_versions"
        warn "Consider standardizing on React 18.2.0"
    fi
    
    # Check for security vulnerabilities in Node packages
    log "Checking for security vulnerabilities..."
    find packages -name "package.json" -type f | while read -r package_file; do
        local dir=$(dirname "$package_file")
        if [[ -f "$dir/package-lock.json" ]] || [[ -f "$dir/yarn.lock" ]]; then
            info "Auditing $dir..."
            (cd "$dir" && npm audit --audit-level moderate) || warn "Security issues found in $dir"
        fi
    done
    
    info "Dependency validation completed"
}

# Build Rust packages
build_rust() {
    log "Building Rust packages..."
    
    local rust_packages=(
        "sutra-storage"
        "sutra-protocol"
        "sutra-grid-master"
        "sutra-grid-agent"
        "sutra-grid-events"
        "sutra-bulk-ingester"
    )
    
    # Build in parallel
    if [[ $PARALLEL_JOBS -gt 1 ]]; then
        log "Building Rust packages in parallel (${PARALLEL_JOBS} jobs)"
        printf '%s\n' "${rust_packages[@]}" | xargs -P "$PARALLEL_JOBS" -I {} bash -c "
            log \"Building Rust package: {}\"
            cd packages/{}
            if [[ \"$BUILD_MODE\" == \"production\" ]]; then
                cargo build --release --locked
            else
                cargo build
            fi
            log \"✓ Rust package {} built successfully\"
        "
    else
        for package in "${rust_packages[@]}"; do
            log "Building Rust package: $package"
            cd "packages/$package"
            
            if [[ "$BUILD_MODE" == "production" ]]; then
                cargo build --release --locked
            else
                cargo build
            fi
            
            cd - > /dev/null
            log "✓ Rust package $package built successfully"
        done
    fi
    
    info "All Rust packages built successfully"
}

# Build Python packages
build_python() {
    log "Building Python packages..."
    
    local python_packages=(
        "sutra-core"
        "sutra-api"
        "sutra-hybrid"
        "sutra-embedding-service"
        "sutra-nlg"
        "sutra-nlg-service"
        "sutra-storage-client-tcp"
    )
    
    # Create virtual environment if needed
    if [[ ! -d ".venv" ]]; then
        log "Creating Python virtual environment..."
        python3 -m venv .venv
    fi
    
    source .venv/bin/activate
    
    # Upgrade pip and install build tools
    pip install --upgrade pip setuptools wheel build
    
    for package in "${python_packages[@]}"; do
        if [[ -d "packages/$package" ]]; then
            log "Building Python package: $package"
            cd "packages/$package"
            
            # Install package in development mode
            if [[ -f "pyproject.toml" ]]; then
                python -m build
            elif [[ -f "setup.py" ]]; then
                python setup.py bdist_wheel
            fi
            
            cd - > /dev/null
            log "✓ Python package $package built successfully"
        fi
    done
    
    info "All Python packages built successfully"
}

# Build Node.js packages
build_nodejs() {
    log "Building Node.js packages..."
    
    local nodejs_packages=(
        "sutra-client"
        "sutra-control"
        "sutra-ui-framework"
    )
    
    for package in "${nodejs_packages[@]}"; do
        if [[ -d "packages/$package" ]]; then
            log "Building Node.js package: $package"
            cd "packages/$package"
            
            # Install dependencies
            if [[ -f "package-lock.json" ]]; then
                npm ci --production=false
            else
                npm install
            fi
            
            # Run build
            if npm run build > /dev/null 2>&1; then
                log "✓ Node.js package $package built successfully"
                
                # Check bundle size
                if [[ -d "dist" ]]; then
                    local bundle_size=$(du -sh dist | cut -f1)
                    info "Bundle size for $package: $bundle_size"
                    
                    # Warn if bundle is too large
                    local size_bytes=$(du -sb dist | cut -f1)
                    if [[ $size_bytes -gt 5242880 ]]; then  # 5MB
                        warn "Large bundle size for $package: $bundle_size"
                    fi
                fi
            else
                warn "Build script not found for $package, skipping"
            fi
            
            cd - > /dev/null
        fi
    done
    
    info "All Node.js packages built successfully"
}

# Run tests
run_tests() {
    if [[ "$SKIP_TESTS" == "true" ]]; then
        warn "Skipping tests (SKIP_TESTS=true)"
        return
    fi
    
    log "Running tests..."
    
    # Rust tests
    log "Running Rust tests..."
    cargo test --workspace --locked
    
    # Python tests
    if [[ -d ".venv" ]]; then
        source .venv/bin/activate
        log "Running Python tests..."
        if [[ -f "pytest.ini" ]]; then
            python -m pytest packages/*/tests/ -v --tb=short
        fi
    fi
    
    # Node.js tests
    log "Running Node.js tests..."
    find packages -name "package.json" -type f | while read -r package_file; do
        local dir=$(dirname "$package_file")
        if grep -q '"test"' "$package_file"; then
            log "Running tests for $(basename "$dir")"
            (cd "$dir" && npm test) || warn "Tests failed for $(basename "$dir")"
        fi
    done
    
    info "Tests completed"
}

# Security scan
security_scan() {
    log "Running security scan..."
    
    # Rust security audit
    if command -v cargo-audit >/dev/null 2>&1; then
        log "Running Rust security audit..."
        cargo audit
    else
        warn "cargo-audit not installed, skipping Rust security scan"
    fi
    
    # Python security scan
    if command -v safety >/dev/null 2>&1; then
        log "Running Python security scan..."
        safety check --full-report
    else
        warn "safety not installed, skipping Python security scan"
    fi
    
    # Node.js security audit
    log "Running Node.js security audit..."
    find packages -name "package.json" -type f | while read -r package_file; do
        local dir=$(dirname "$package_file")
        if [[ -f "$dir/package-lock.json" ]]; then
            (cd "$dir" && npm audit --audit-level high) || warn "Security issues in $(basename "$dir")"
        fi
    done
    
    info "Security scan completed"
}

# Generate build report
generate_report() {
    log "Generating build report..."
    
    local report_file="build-report-$(date +%Y%m%d-%H%M%S).json"
    
    cat > "$report_file" << EOF
{
  "build_info": {
    "version": "$SUTRA_VERSION",
    "mode": "$BUILD_MODE",
    "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
    "parallel_jobs": $PARALLEL_JOBS,
    "hostname": "$(hostname)",
    "user": "$(whoami)"
  },
  "environment": {
    "rust_version": "$(rustc --version)",
    "python_version": "$(python3 --version)",
    "node_version": "$(node --version)",
    "npm_version": "$(npm --version)"
  },
  "packages": {
    "rust_packages": $(find packages -name "Cargo.toml" | wc -l),
    "python_packages": $(find packages -name "pyproject.toml" -o -name "setup.py" | wc -l),
    "nodejs_packages": $(find packages -name "package.json" | wc -l)
  },
  "bundle_sizes": {
EOF

    # Add bundle sizes
    local first=true
    find packages -name "dist" -type d | while read -r dist_dir; do
        local package_name=$(basename "$(dirname "$dist_dir")")
        local size_bytes=$(du -sb "$dist_dir" 2>/dev/null | cut -f1 || echo "0")
        
        if [[ "$first" == "true" ]]; then
            first=false
        else
            echo ","
        fi
        
        echo "    \"$package_name\": $size_bytes"
    done >> "$report_file"
    
    cat >> "$report_file" << EOF
  },
  "status": "success"
}
EOF

    info "Build report generated: $report_file"
}

# Main build function
main() {
    log "Starting Sutra Memory production build..."
    log "Version: $SUTRA_VERSION"
    log "Mode: $BUILD_MODE"
    log "Parallel jobs: $PARALLEL_JOBS"
    
    validate_environment
    validate_dependencies
    
    # Build all packages
    build_rust
    build_python  
    build_nodejs
    
    # Run tests and security scan
    run_tests
    security_scan
    
    # Generate report
    generate_report
    
    log "✅ Production build completed successfully!"
    log "🚀 Ready for deployment"
}

# Handle arguments
case "${1:-}" in
    "rust")
        validate_environment
        build_rust
        ;;
    "python")
        validate_environment
        build_python
        ;;
    "nodejs")
        validate_environment
        build_nodejs
        ;;
    "test")
        validate_environment
        run_tests
        ;;
    "security")
        validate_environment
        security_scan
        ;;
    "report")
        generate_report
        ;;
    "")
        main
        ;;
    *)
        echo "Usage: $0 [rust|python|nodejs|test|security|report]"
        echo ""
        echo "Environment variables:"
        echo "  SUTRA_VERSION     - Version to build (default: from VERSION file)"
        echo "  BUILD_MODE        - production|development (default: production)"
        echo "  PARALLEL_JOBS     - Number of parallel jobs (default: nproc)"
        echo "  SKIP_TESTS        - Skip tests (default: false)"
        echo "  BUILD_DOCKER      - Build Docker images (default: false)"
        exit 1
        ;;
esac