#!/bin/bash
# Local Sync Script for Sutra Engine Standalone
# Use this when GitHub Actions are unavailable

set -e

# Configuration
SOURCE_DIR="release/sutra-engine"
TARGET_REPO="https://github.com/nranjan2code/sutra-engine.git"
TEMP_DIR=".tmp_engine_sync"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}🚀 Preparing to sync to $TARGET_REPO...${NC}"

# Ensure source exists
if [ ! -d "$SOURCE_DIR" ]; then
    echo "Error: Source directory $SOURCE_DIR not found. Run scripts/release_storage_engine.sh first."
    exit 1
fi

# Clean temp dir
rm -rf "$TEMP_DIR"

# Clone the target repo
echo -e "${BLUE}📂 Cloning target repository...${NC}"
git clone "$TARGET_REPO" "$TEMP_DIR"

# Copy files (excluding .git)
echo -e "${BLUE}📦 Copying files...${NC}"
cp -r "$SOURCE_DIR/"* "$TEMP_DIR/"
cp "$SOURCE_DIR/.gitignore" "$TEMP_DIR/" 2>/dev/null || true

# Navigate to temp dir
cd "$TEMP_DIR"

# Commit and Push
echo -e "${BLUE}📤 Committing and Pushing...${NC}"
git add .
if git diff --staged --quiet; then
    echo "No changes to sync."
else
    git commit -m "Manual sync from local release folder"
    git push origin main
fi

# Cleanup
cd ..
rm -rf "$TEMP_DIR"

echo -e "${GREEN}✅ Sync Complete! Check your repo: $TARGET_REPO${NC}"
