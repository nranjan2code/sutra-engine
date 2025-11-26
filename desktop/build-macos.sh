#!/bin/bash
# Sutra Desktop Edition - macOS Build Script
# Builds a self-contained native application bundle

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}ℹ${NC} $1"; }
log_success() { echo -e "${GREEN}✓${NC} $1"; }
log_warning() { echo -e "${YELLOW}⚠${NC} $1"; }
log_error() { echo -e "${RED}✗${NC} $1"; }

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
BUILD_DIR="$SCRIPT_DIR/build"
APP_NAME="Sutra"
APP_BUNDLE="$BUILD_DIR/$APP_NAME.app"
VERSION="${SUTRA_VERSION:-1.0.0}"

# Architecture detection
ARCH=$(uname -m)
if [[ "$ARCH" == "arm64" ]]; then
    RUST_TARGET="aarch64-apple-darwin"
else
    RUST_TARGET="x86_64-apple-darwin"
fi

# Parse arguments
BUILD_UNIVERSAL=false
CREATE_DMG=false
INCLUDE_MODELS=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --universal)
            BUILD_UNIVERSAL=true
            shift
            ;;
        --dmg)
            CREATE_DMG=true
            shift
            ;;
        --with-models)
            INCLUDE_MODELS=true
            shift
            ;;
        *)
            shift
            ;;
    esac
done

echo ""
echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║           🖥️  SUTRA DESKTOP EDITION - macOS Build             ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo ""
log_info "Version: $VERSION"
log_info "Architecture: $ARCH ($RUST_TARGET)"
log_info "Universal: $BUILD_UNIVERSAL"
echo ""

# ============================================================================
# STEP 1: Clean and prepare
# ============================================================================
log_info "Step 1: Preparing build directory..."

rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR"
mkdir -p "$APP_BUNDLE/Contents/MacOS"
mkdir -p "$APP_BUNDLE/Contents/Resources/bin"
mkdir -p "$APP_BUNDLE/Contents/Resources/python"
mkdir -p "$APP_BUNDLE/Contents/Resources/data"
mkdir -p "$APP_BUNDLE/Contents/Resources/config"

log_success "Build directory prepared"

# ============================================================================
# STEP 2: Build Rust storage-server
# ============================================================================
log_info "Step 2: Building Rust storage-server..."

cd "$ROOT_DIR/packages/sutra-storage"

if [[ "$BUILD_UNIVERSAL" == "true" ]]; then
    log_info "Building universal binary (ARM64 + x86_64)..."
    
    # Build for ARM64
    cargo build --release --target aarch64-apple-darwin --bin storage-server
    
    # Build for x86_64
    cargo build --release --target x86_64-apple-darwin --bin storage-server
    
    # Create universal binary
    lipo -create \
        "$ROOT_DIR/target/aarch64-apple-darwin/release/storage-server" \
        "$ROOT_DIR/target/x86_64-apple-darwin/release/storage-server" \
        -output "$APP_BUNDLE/Contents/Resources/bin/storage-server"
else
    cargo build --release --bin storage-server
    cp "$ROOT_DIR/target/release/storage-server" "$APP_BUNDLE/Contents/Resources/bin/"
fi

chmod +x "$APP_BUNDLE/Contents/Resources/bin/storage-server"
log_success "Storage server built: $(du -h "$APP_BUNDLE/Contents/Resources/bin/storage-server" | cut -f1)"

# ============================================================================
# STEP 3: Create minimal Python virtual environment
# ============================================================================
log_info "Step 3: Creating minimal Python environment..."

PYTHON_DIR="$APP_BUNDLE/Contents/Resources/python"
VENV_DIR="$PYTHON_DIR/venv"

# Use system Python or bundled Python
PYTHON_BIN=$(which python3.11 || which python3.10 || which python3)

if [[ -z "$PYTHON_BIN" ]]; then
    log_error "Python 3.10+ required but not found"
    exit 1
fi

log_info "Using Python: $PYTHON_BIN"

# Create virtual environment
$PYTHON_BIN -m venv "$VENV_DIR"
source "$VENV_DIR/bin/activate"

# Upgrade pip
pip install --upgrade pip wheel --quiet

# Install ONLY essential dependencies (no heavy ML libs)
log_info "Installing minimal Python dependencies..."

# Core dependencies only
pip install --quiet \
    msgpack==1.1.0 \
    numpy==1.26.4 \
    fastapi==0.115.0 \
    uvicorn==0.30.6 \
    pydantic==2.9.2 \
    pydantic-settings==2.5.2 \
    python-multipart==0.0.12 \
    argon2-cffi==23.1.0 \
    pyjwt==2.9.0 \
    requests==2.31.0

# Install local packages in minimal mode
log_info "Installing Sutra packages..."

# Storage client (TCP only - just msgpack)
pip install --quiet "$ROOT_DIR/packages/sutra-storage-client-tcp"

# Core (server mode - no sklearn, no hnswlib)
pip install --quiet "$ROOT_DIR/packages/sutra-core"

# Hybrid (no tfidf extras)
pip install --quiet "$ROOT_DIR/packages/sutra-hybrid"

# API
pip install --quiet "$ROOT_DIR/packages/sutra-api"

deactivate

# Calculate size
PYTHON_SIZE=$(du -sh "$VENV_DIR" | cut -f1)
log_success "Python environment created: $PYTHON_SIZE"

# ============================================================================
# STEP 4: Copy Python packages source (for running)
# ============================================================================
log_info "Step 4: Copying Python package sources..."

mkdir -p "$PYTHON_DIR/packages"

# Copy essential packages
cp -r "$ROOT_DIR/packages/sutra-storage-client-tcp/sutra_storage_client" "$PYTHON_DIR/packages/"
cp -r "$ROOT_DIR/packages/sutra-core/sutra_core" "$PYTHON_DIR/packages/"
cp -r "$ROOT_DIR/packages/sutra-hybrid/sutra_hybrid" "$PYTHON_DIR/packages/"
cp -r "$ROOT_DIR/packages/sutra-api/sutra_api" "$PYTHON_DIR/packages/"

log_success "Python packages copied"

# ============================================================================
# STEP 5: Create launcher script
# ============================================================================
log_info "Step 5: Creating launcher..."

cat > "$APP_BUNDLE/Contents/MacOS/sutra-launcher" << 'LAUNCHER'
#!/bin/bash
# Sutra Desktop Launcher
# Manages all Sutra services as native processes

set -e

# Get app bundle path
APP_DIR="$(cd "$(dirname "$0")/.." && pwd)"
RESOURCES="$APP_DIR/Resources"
BIN_DIR="$RESOURCES/bin"
PYTHON_DIR="$RESOURCES/python"
DATA_DIR="${SUTRA_DATA_DIR:-$HOME/Library/Application Support/Sutra}"
LOG_DIR="$DATA_DIR/logs"
PID_DIR="$DATA_DIR/pids"

# Ports
STORAGE_PORT="${SUTRA_STORAGE_PORT:-50051}"
API_PORT="${SUTRA_API_PORT:-8000}"
HYBRID_PORT="${SUTRA_HYBRID_PORT:-8001}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}ℹ${NC} $1"; }
log_success() { echo -e "${GREEN}✓${NC} $1"; }
log_error() { echo -e "${RED}✗${NC} $1"; }

# Ensure directories exist
mkdir -p "$DATA_DIR" "$LOG_DIR" "$PID_DIR" "$DATA_DIR/storage"

# Activate Python venv
activate_python() {
    source "$PYTHON_DIR/venv/bin/activate"
    export PYTHONPATH="$PYTHON_DIR/packages:$PYTHONPATH"
}

# Start storage server
start_storage() {
    if [[ -f "$PID_DIR/storage.pid" ]] && kill -0 $(cat "$PID_DIR/storage.pid") 2>/dev/null; then
        log_info "Storage server already running"
        return 0
    fi
    
    log_info "Starting storage server on port $STORAGE_PORT..."
    
    STORAGE_PATH="$DATA_DIR/storage" \
    STORAGE_HOST="127.0.0.1" \
    STORAGE_PORT="$STORAGE_PORT" \
    VECTOR_DIMENSION="${VECTOR_DIMENSION:-256}" \
    RUST_LOG="${RUST_LOG:-info}" \
    "$BIN_DIR/storage-server" > "$LOG_DIR/storage.log" 2>&1 &
    
    echo $! > "$PID_DIR/storage.pid"
    
    # Wait for startup
    for i in {1..30}; do
        if nc -z 127.0.0.1 $STORAGE_PORT 2>/dev/null; then
            log_success "Storage server started (PID: $(cat "$PID_DIR/storage.pid"))"
            return 0
        fi
        sleep 0.5
    done
    
    log_error "Storage server failed to start"
    cat "$LOG_DIR/storage.log"
    return 1
}

# Start API server
start_api() {
    if [[ -f "$PID_DIR/api.pid" ]] && kill -0 $(cat "$PID_DIR/api.pid") 2>/dev/null; then
        log_info "API server already running"
        return 0
    fi
    
    log_info "Starting API server on port $API_PORT..."
    
    activate_python
    
    SUTRA_STORAGE_SERVER="127.0.0.1:$STORAGE_PORT" \
    SUTRA_USER_STORAGE_SERVER="127.0.0.1:$STORAGE_PORT" \
    SUTRA_EDITION="desktop" \
    python -m uvicorn sutra_api.main:app \
        --host 127.0.0.1 \
        --port $API_PORT \
        --log-level info \
        > "$LOG_DIR/api.log" 2>&1 &
    
    echo $! > "$PID_DIR/api.pid"
    
    # Wait for startup
    for i in {1..30}; do
        if curl -s "http://127.0.0.1:$API_PORT/health" > /dev/null 2>&1; then
            log_success "API server started (PID: $(cat "$PID_DIR/api.pid"))"
            return 0
        fi
        sleep 0.5
    done
    
    log_error "API server failed to start"
    cat "$LOG_DIR/api.log"
    return 1
}

# Start Hybrid server
start_hybrid() {
    if [[ -f "$PID_DIR/hybrid.pid" ]] && kill -0 $(cat "$PID_DIR/hybrid.pid") 2>/dev/null; then
        log_info "Hybrid server already running"
        return 0
    fi
    
    log_info "Starting Hybrid server on port $HYBRID_PORT..."
    
    activate_python
    
    SUTRA_STORAGE_SERVER="127.0.0.1:$STORAGE_PORT" \
    SUTRA_EDITION="desktop" \
    SUTRA_USE_SEMANTIC_EMBEDDINGS="false" \
    python -m uvicorn sutra_hybrid.api:app \
        --host 127.0.0.1 \
        --port $HYBRID_PORT \
        --log-level info \
        > "$LOG_DIR/hybrid.log" 2>&1 &
    
    echo $! > "$PID_DIR/hybrid.pid"
    
    # Wait for startup
    for i in {1..30}; do
        if curl -s "http://127.0.0.1:$HYBRID_PORT/ping" > /dev/null 2>&1; then
            log_success "Hybrid server started (PID: $(cat "$PID_DIR/hybrid.pid"))"
            return 0
        fi
        sleep 0.5
    done
    
    log_error "Hybrid server failed to start"
    cat "$LOG_DIR/hybrid.log"
    return 1
}

# Stop all services
stop_all() {
    log_info "Stopping Sutra services..."
    
    for service in hybrid api storage; do
        if [[ -f "$PID_DIR/$service.pid" ]]; then
            pid=$(cat "$PID_DIR/$service.pid")
            if kill -0 $pid 2>/dev/null; then
                kill $pid 2>/dev/null || true
                log_success "Stopped $service (PID: $pid)"
            fi
            rm -f "$PID_DIR/$service.pid"
        fi
    done
    
    log_success "All services stopped"
}

# Check status
status() {
    echo ""
    echo "Sutra Desktop Status"
    echo "===================="
    
    for service in storage api hybrid; do
        if [[ -f "$PID_DIR/$service.pid" ]] && kill -0 $(cat "$PID_DIR/$service.pid") 2>/dev/null; then
            pid=$(cat "$PID_DIR/$service.pid")
            echo -e "  ${GREEN}●${NC} $service (PID: $pid)"
        else
            echo -e "  ${RED}●${NC} $service (not running)"
        fi
    done
    
    echo ""
    echo "Data directory: $DATA_DIR"
    echo "Logs: $LOG_DIR"
    echo ""
}

# Show logs
logs() {
    service="${1:-all}"
    
    if [[ "$service" == "all" ]]; then
        tail -f "$LOG_DIR"/*.log
    else
        tail -f "$LOG_DIR/$service.log"
    fi
}

# Main command handler
case "${1:-start}" in
    start)
        echo ""
        echo "╔═══════════════════════════════════════╗"
        echo "║     🚀 Starting Sutra Desktop         ║"
        echo "╚═══════════════════════════════════════╝"
        echo ""
        
        start_storage
        start_api
        # start_hybrid  # Optional: uncomment if needed
        
        echo ""
        log_success "Sutra Desktop is running!"
        echo ""
        echo "  API:     http://127.0.0.1:$API_PORT"
        echo "  Health:  http://127.0.0.1:$API_PORT/health"
        echo "  Docs:    http://127.0.0.1:$API_PORT/docs"
        echo ""
        echo "  Data:    $DATA_DIR"
        echo "  Logs:    $LOG_DIR"
        echo ""
        ;;
    
    stop)
        stop_all
        ;;
    
    restart)
        stop_all
        sleep 1
        exec "$0" start
        ;;
    
    status)
        status
        ;;
    
    logs)
        logs "$2"
        ;;
    
    *)
        echo "Usage: sutra-launcher {start|stop|restart|status|logs [service]}"
        exit 1
        ;;
esac
LAUNCHER

chmod +x "$APP_BUNDLE/Contents/MacOS/sutra-launcher"
log_success "Launcher created"

# ============================================================================
# STEP 6: Create Info.plist
# ============================================================================
log_info "Step 6: Creating Info.plist..."

cat > "$APP_BUNDLE/Contents/Info.plist" << PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>sutra-launcher</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>CFBundleIdentifier</key>
    <string>ai.sutra.desktop</string>
    <key>CFBundleName</key>
    <string>Sutra</string>
    <key>CFBundleDisplayName</key>
    <string>Sutra AI</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>$VERSION</string>
    <key>CFBundleVersion</key>
    <string>$VERSION</string>
    <key>LSMinimumSystemVersion</key>
    <string>12.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>LSUIElement</key>
    <false/>
    <key>NSHumanReadableCopyright</key>
    <string>© 2025 Sutra AI. All rights reserved.</string>
</dict>
</plist>
PLIST

log_success "Info.plist created"

# ============================================================================
# STEP 7: Create default configuration
# ============================================================================
log_info "Step 7: Creating default configuration..."

cat > "$APP_BUNDLE/Contents/Resources/config/default.toml" << 'CONFIG'
# Sutra Desktop Configuration

[desktop]
# Edition identifier
edition = "desktop"

# Data storage location (~ expands to home directory)
data_dir = "~/Library/Application Support/Sutra"

[server]
# Storage server settings
storage_port = 50051
api_port = 8000
hybrid_port = 8001

# Bind address (127.0.0.1 = local only, 0.0.0.0 = network accessible)
bind_address = "127.0.0.1"

[storage]
# Vector search settings
vector_dimension = 256  # 256 (fast), 512 (balanced), 768 (accurate)
max_concepts = 100000

# Persistence
wal_enabled = true
mmap_enabled = true

[embedding]
# Embedding generation mode
# - "disabled": No embeddings (text-only mode)
# - "builtin": Use built-in lightweight model
# - "remote": Use remote embedding service
mode = "disabled"

# Remote embedding service URL (if mode = "remote")
# service_url = "http://localhost:8888"

[logging]
# Log level: debug, info, warning, error
level = "info"

# Log rotation
max_size_mb = 50
max_files = 5
CONFIG

log_success "Default configuration created"

# ============================================================================
# STEP 8: Calculate final size
# ============================================================================
echo ""
log_info "Build Summary"
echo "=============="

TOTAL_SIZE=$(du -sh "$APP_BUNDLE" | cut -f1)
BINARY_SIZE=$(du -sh "$APP_BUNDLE/Contents/Resources/bin" | cut -f1)

echo "  App Bundle:    $APP_BUNDLE"
echo "  Total Size:    $TOTAL_SIZE"
echo "  Binaries:      $BINARY_SIZE"
echo "  Python:        $PYTHON_SIZE"
echo "  Architecture:  $ARCH"
echo ""

# ============================================================================
# STEP 9: Create DMG (optional)
# ============================================================================
if [[ "$CREATE_DMG" == "true" ]]; then
    log_info "Step 9: Creating DMG installer..."
    
    DMG_NAME="Sutra-Desktop-$VERSION-$ARCH.dmg"
    DMG_PATH="$BUILD_DIR/$DMG_NAME"
    
    # Create temporary DMG directory
    DMG_TEMP="$BUILD_DIR/dmg_temp"
    mkdir -p "$DMG_TEMP"
    cp -r "$APP_BUNDLE" "$DMG_TEMP/"
    
    # Create Applications symlink
    ln -s /Applications "$DMG_TEMP/Applications"
    
    # Create DMG
    hdiutil create -volname "Sutra Desktop" \
        -srcfolder "$DMG_TEMP" \
        -ov -format UDZO \
        "$DMG_PATH"
    
    rm -rf "$DMG_TEMP"
    
    DMG_SIZE=$(du -sh "$DMG_PATH" | cut -f1)
    log_success "DMG created: $DMG_PATH ($DMG_SIZE)"
fi

# ============================================================================
# Done
# ============================================================================
echo ""
log_success "Build complete!"
echo ""
echo "To run Sutra Desktop:"
echo "  $APP_BUNDLE/Contents/MacOS/sutra-launcher start"
echo ""
echo "Or copy to Applications:"
echo "  cp -r $APP_BUNDLE /Applications/"
echo "  open /Applications/Sutra.app"
echo ""
