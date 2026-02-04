#!/usr/bin/env bash
# Dummy storage server for testing Agent spawning
# Usage: ./dummy-storage-server.sh --port PORT --storage-path PATH --node-id ID

PORT=50051
STORAGE_PATH="./storage"
NODE_ID="node-unknown"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --port)
            PORT="$2"
            shift 2
            ;;
        --storage-path)
            STORAGE_PATH="$2"
            shift 2
            ;;
        --node-id)
            NODE_ID="$2"
            shift 2
            ;;
        *)
            shift
            ;;
    esac
done

echo "🚀 Dummy Storage Server starting..."
echo "   Node ID: $NODE_ID"
echo "   Port: $PORT"
echo "   Storage Path: $STORAGE_PATH"
echo ""
echo "💤 Sleeping (press Ctrl+C to exit)..."

# Create the storage directory if it doesn't exist
mkdir -p "$STORAGE_PATH"

# Trap signals to ensure clean shutdown
trap 'echo "📴 Dummy storage server shutting down..."; exit 0' SIGTERM SIGINT

# Just sleep to simulate a running process
while true; do
    sleep 30
    echo "💓 Dummy storage server heartbeat - Node: $NODE_ID, Port: $PORT"
done
