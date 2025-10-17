#!/bin/bash

# Colors for output
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

echo -e "${GREEN}🚀 Starting Sutra AI Development Environment${NC}"
echo ""

# Function to cleanup background processes on exit
cleanup() {
    echo ""
    echo -e "${RED}🛑 Shutting down servers...${NC}"
    kill $API_PID $CLIENT_PID 2>/dev/null
    exit 0
}

# Trap Ctrl+C and call cleanup
trap cleanup INT TERM

# Start API server
echo -e "${CYAN}[API]${NC} Starting Sutra API on http://localhost:8000"
source venv/bin/activate
cd packages/sutra-api
python -m sutra_api.main > /tmp/sutra-api.log 2>&1 &
API_PID=$!
cd ../..

# Wait a moment for API to start
sleep 2

# Start Client
echo -e "${MAGENTA}[CLIENT]${NC} Starting Sutra Client on http://localhost:3000"
cd packages/sutra-client
npm run dev > /tmp/sutra-client.log 2>&1 &
CLIENT_PID=$!
cd ../..

echo ""
echo -e "${GREEN}✅ Development servers running${NC}"
echo -e "${CYAN}   API:${NC}    http://localhost:8000"
echo -e "${MAGENTA}   Client:${NC} http://localhost:3000"
echo ""
echo -e "Press ${RED}Ctrl+C${NC} to stop all servers"
echo ""

# Show logs
tail -f /tmp/sutra-api.log /tmp/sutra-client.log &
TAIL_PID=$!

# Wait for background processes
wait $API_PID $CLIENT_PID
