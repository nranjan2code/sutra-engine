#!/bin/bash
# Sutra Engine Standalone Release Builder
# Compiles the Rust core and packages it with docs, client, and examples.
set -e

# Configuration
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RELEASE_DIR="$PROJECT_ROOT/release/sutra-engine"
BINARY_NAME="sutra-engine"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}📦 Building Sutra Storage Engine...${NC}"

# Navigate to root
cd "$PROJECT_ROOT"

# Build the storage server binary
# We use -p sutra-storage to target the package and --bin to select the binary
cargo build --release --bin storage-server -p sutra-storage

echo -e "${BLUE}📂 Preparing release directory...${NC}"
rm -rf "$RELEASE_DIR"
mkdir -p "$RELEASE_DIR"

# Copy binary
SOURCE_BIN="$PROJECT_ROOT/target/release/storage-server"
if [ ! -f "$SOURCE_BIN" ]; then
    echo "Error: Binary not found at $SOURCE_BIN"
    exit 1
fi

cp "$SOURCE_BIN" "$RELEASE_DIR/$BINARY_NAME"

# Copy Documentation & Assets
echo -e "${BLUE}📄 Copying documentation and assets...${NC}"
cp -r "$PROJECT_ROOT/release/sutra-engine/docs" "$RELEASE_DIR/" 2>/dev/null || true
cp -r "$PROJECT_ROOT/release/sutra-engine/examples" "$RELEASE_DIR/" 2>/dev/null || true
cp -r "$PROJECT_ROOT/release/sutra-engine/sutra_engine_client" "$RELEASE_DIR/" 2>/dev/null || true
cp "$PROJECT_ROOT/release/sutra-engine/setup.py" "$RELEASE_DIR/" 2>/dev/null || true
cp "$PROJECT_ROOT/LICENSE" "$RELEASE_DIR/"

# We already have a specific README in the repo, copy it over the previous generated one
# if it exists, otherwise generate the simple one.
if [ -f "$PROJECT_ROOT/release/sutra-engine/README.md" ]; then
    cp "$PROJECT_ROOT/release/sutra-engine/README.md" "$RELEASE_DIR/README.md"
else
    # Fallback to simple generated README if the detailed one doesn't exist
    cat > "$RELEASE_DIR/README.md" << EOF
# Sutra Storage Engine
...
EOF
fi

# Create a convenience launch script
cat > "$RELEASE_DIR/start-engine.sh" << EOF
#!/bin/bash
# Quick start script for Sutra Engine

# Defaults
export STORAGE_PATH="\${STORAGE_PATH:-./data}"
export STORAGE_PORT="\${STORAGE_PORT:-50051}"
export RUST_LOG="\${RUST_LOG:-info}"

# Ensure data dir exists
mkdir -p "\$STORAGE_PATH"

echo "Starting Sutra Engine on port \$STORAGE_PORT..."
echo "Data: \$STORAGE_PATH"

./$BINARY_NAME
EOF
chmod +x "$RELEASE_DIR/start-engine.sh"

echo -e "${GREEN}✅ Release created successfully!${NC}"
echo -e "   Location: $RELEASE_DIR"
echo -e "   Binary:   $RELEASE_DIR/$BINARY_NAME"
echo -e "   Script:   $RELEASE_DIR/start-engine.sh"
