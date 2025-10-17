#!/usr/bin/env bash
# Launch Sutra Control Center

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🧠 Starting Sutra Control Center...${NC}"
echo ""

# Check if virtual environment exists
if [ ! -d "../../venv" ]; then
    echo "⚠️  Virtual environment not found. Run 'make setup' from project root first."
    exit 1
fi

# Activate virtual environment
source ../../venv/bin/activate

# Install package if needed
if ! python -c "import sutra_control" 2>/dev/null; then
    echo "📦 Installing sutra-control package..."
    pip install -e . > /dev/null
fi

echo -e "${GREEN}✓${NC} Control Center starting on http://localhost:9000"
echo ""
echo "  Dashboard: http://localhost:9000"
echo "  API Docs:  http://localhost:9000/docs"
echo ""
echo "Press Ctrl+C to stop"
echo ""

# Start the server
python -m uvicorn sutra_control.main:app --host 0.0.0.0 --port 9000 --reload
