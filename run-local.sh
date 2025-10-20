#!/bin/bash

# Run Sutra AI services directly on host (no Docker dependencies)
# This approach eliminates all external dependencies

set -e

echo "🚀 Running Sutra AI services locally (host-based)"

# Check if Python 3 is available
if ! command -v python3 &> /dev/null; then
    echo "❌ Python 3 not found. Please install Python 3.11+"
    exit 1
fi

echo "📦 Setting up Python virtual environment..."
cd packages/sutra-embedding-service

# Create virtual environment if it doesn't exist
if [ ! -d "venv" ]; then
    python3 -m venv venv
fi

# Activate virtual environment
source venv/bin/activate

# Install requirements
pip install --no-cache-dir -r requirements.txt

echo "🔥 Starting embedding service with local nomic-1.5 model..."

# Set environment variables
export PORT=8888
export EMBEDDING_MODEL="nomic-ai/nomic-embed-text-v1.5"

# Start the service
python3 main.py &
EMBEDDING_PID=$!

echo "✅ Embedding service started (PID: $EMBEDDING_PID) on port 8888"

cd ../..

echo "🦀 Building and starting Rust storage server..."
cd packages/sutra-storage

# Build storage server
cargo build --release

# Start storage server
RUST_LOG=info ./target/release/storage-server &
STORAGE_PID=$!

echo "✅ Storage server started (PID: $STORAGE_PID) on port 50051"

cd ../..

echo ""
echo "🎉 Sutra AI services running locally:"
echo "  📡 Embedding Service: http://localhost:8888"
echo "  💾 Storage Server:    tcp://localhost:50051"
echo ""
echo "To stop services:"
echo "  kill $EMBEDDING_PID $STORAGE_PID"
echo ""
echo "To test embedding service:"
echo '  curl -X POST http://localhost:8888/embed -H "Content-Type: application/json" -d '\''{"texts":["test"]}'\'

# Keep script running
wait