#!/bin/bash
# Quick start script for Sutra Engine

# Defaults
export STORAGE_PATH="${STORAGE_PATH:-./data}"
export STORAGE_PORT="${STORAGE_PORT:-50051}"
export RUST_LOG="${RUST_LOG:-info}"

# Ensure data dir exists
mkdir -p "$STORAGE_PATH"

echo "Starting Sutra Engine on port $STORAGE_PORT..."
echo "Data: $STORAGE_PATH"

./sutra-engine
