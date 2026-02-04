#!/bin/bash
set -e

echo "🧪 Sutra Grid Integration Test"
echo "================================"
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Build everything
echo "📦 Building master and CLI..."
cargo build --release 2>&1 | grep -E "(Compiling|Finished)" || true
echo ""

# Check if master is running
echo "🔍 Checking if master is running on port 7000..."
if lsof -i :7000 >/dev/null 2>&1; then
    echo -e "${GREEN}✅ Master is running${NC}"
else
    echo -e "${RED}❌ Master is not running. Start it with:${NC}"
    echo "   cd packages/sutra-grid-master && RUST_LOG=info cargo run --release"
    exit 1
fi
echo ""

# Check if agent is running
echo "🔍 Checking if agent is registered..."
AGENT_COUNT=$(./target/release/sutra-grid-cli list-agents 2>/dev/null | grep -c "Agent:" || echo "0")

if [ "$AGENT_COUNT" -eq "0" ]; then
    echo -e "${RED}❌ No agents registered. Start an agent with:${NC}"
    echo "   cd packages/sutra-grid-agent && RUST_LOG=info cargo run --release"
    exit 1
else
    echo -e "${GREEN}✅ Found $AGENT_COUNT agent(s)${NC}"
    AGENT_ID=$(./target/release/sutra-grid-cli list-agents 2>/dev/null | grep "Agent:" | head -1 | awk '{print $3}')
    echo "   Using agent: $AGENT_ID"
fi
echo ""

# Test 1: Cluster Status
echo "📊 Test 1: Get Cluster Status"
./target/release/sutra-grid-cli status
echo -e "${GREEN}✅ Cluster status retrieved${NC}"
echo ""

# Test 2: List Agents
echo "📋 Test 2: List Agents"
./target/release/sutra-grid-cli list-agents
echo -e "${GREEN}✅ Agents listed${NC}"
echo ""

# Test 3: Spawn a storage node
echo "📦 Test 3: Spawn Storage Node"
SPAWN_OUTPUT=$(./target/release/sutra-grid-cli spawn \
    --agent "$AGENT_ID" \
    --port 50051 \
    --storage-path /tmp/test-storage-$$)

echo "$SPAWN_OUTPUT"

if echo "$SPAWN_OUTPUT" | grep -q "spawned successfully"; then
    echo -e "${GREEN}✅ Node spawned successfully${NC}"
    NODE_ID=$(echo "$SPAWN_OUTPUT" | grep "Node ID:" | awk '{print $3}')
    echo "   Node ID: $NODE_ID"
else
    echo -e "${RED}❌ Failed to spawn node${NC}"
    exit 1
fi
echo ""

# Wait for node to start
echo "⏳ Waiting 2 seconds for node to initialize..."
sleep 2
echo ""

# Test 4: Query node status
echo "❓ Test 4: Query Node Status"
./target/release/sutra-grid-cli node-status --node "$NODE_ID"
echo -e "${GREEN}✅ Node status retrieved${NC}"
echo ""

# Test 5: Stop the node
echo "🛑 Test 5: Stop Storage Node"
STOP_OUTPUT=$(./target/release/sutra-grid-cli stop \
    --node "$NODE_ID" \
    --agent "$AGENT_ID")

echo "$STOP_OUTPUT"

if echo "$STOP_OUTPUT" | grep -q "stopped successfully"; then
    echo -e "${GREEN}✅ Node stopped successfully${NC}"
else
    echo -e "${RED}❌ Failed to stop node${NC}"
    exit 1
fi
echo ""

# Final status
echo "📊 Final Cluster Status:"
./target/release/sutra-grid-cli status
echo ""

echo -e "${GREEN}🎉 All integration tests passed!${NC}"
