#!/bin/bash
# Bootstrap reserved storage instance for Grid events

set -e

STORAGE_PORT=50052
STORAGE_PATH="/tmp/grid-events-storage"

echo "🗄️  Bootstrapping Grid Events Storage"
echo "===================================="
echo ""
echo "Port: $STORAGE_PORT"
echo "Path: $STORAGE_PATH"
echo ""

# Clean and create storage directory
rm -rf "$STORAGE_PATH"
mkdir -p "$STORAGE_PATH"

echo "✅ Storage directory created"
echo ""
echo "🚀 Starting storage server..."
echo "   (Press Ctrl+C to stop)"
echo ""

# Start storage server
# This is a reserved instance dedicated to Grid observability events
STORAGE_PORT="$STORAGE_PORT" STORAGE_PATH="$STORAGE_PATH" cargo run --release --bin storage_server
