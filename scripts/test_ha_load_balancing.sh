#!/bin/bash
# Test HAProxy Load Balancing for Enterprise HA
# Sends requests through HAProxy and verifies distribution across replicas

set -e

echo "🔄 Testing Enterprise HA Load Balancing"
echo "========================================"
echo

# Test Embedding Service Load Balancing
echo "📊 Testing Embedding Service (embedder-ha:8888)"
echo "Sending 30 embedding requests..."
echo

for i in {1..30}; do
    curl -s -X POST http://localhost:8888/embed \
        -H "Content-Type: application/json" \
        -d '{"texts": ["test concept '$i'"], "normalize": true}' \
        > /dev/null 2>&1 && echo -n "." || echo -n "✗"
done

echo
echo "✅ Embedding requests completed"
echo

# Check HAProxy stats for embedding service
echo "📈 HAProxy Embedding Stats (port 9998):"
curl -s http://localhost:9998/stats | grep -A5 "embedder-backend" | head -10 || echo "Stats endpoint not accessible"
echo

# Test NLG Service Load Balancing
echo "📊 Testing NLG Service (nlg-ha:8003)"
echo "Sending 30 NLG requests..."
echo

for i in {1..30}; do
    curl -s -X POST http://localhost:8003/generate \
        -H "Content-Type: application/json" \
        -d '{"prompt": "test '$i'", "max_tokens": 10}' \
        > /dev/null 2>&1 && echo -n "." || echo -n "✗"
done

echo
echo "✅ NLG requests completed"
echo

# Check HAProxy stats for NLG service
echo "📈 HAProxy NLG Stats (port 9997):"
curl -s http://localhost:9997/stats | grep -A5 "nlg-backend" | head -10 || echo "Stats endpoint not accessible"
echo

# Check replica logs for request distribution
echo "📋 Checking Request Distribution:"
echo
echo "Embedder Replica 1 logs:"
docker logs sutra-works-embedder-1 2>&1 | grep -E "POST /embed|embedding" | tail -3 || echo "No logs yet"
echo
echo "Embedder Replica 2 logs:"
docker logs sutra-works-embedder-2 2>&1 | grep -E "POST /embed|embedding" | tail -3 || echo "No logs yet"
echo
echo "Embedder Replica 3 logs:"
docker logs sutra-works-embedder-3 2>&1 | grep -E "POST /embed|embedding" | tail -3 || echo "No logs yet"
echo

echo "✅ Load balancing test complete!"
echo "Review the logs above to verify requests are distributed across all replicas."
