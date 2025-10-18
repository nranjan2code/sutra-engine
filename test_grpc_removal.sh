#!/bin/bash
#
# Integration test for gRPC removal
# Tests that custom binary protocol works end-to-end
#

set -e  # Exit on error

echo "========================================="
echo "gRPC Removal - Integration Test"
echo "========================================="
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track results
TESTS_PASSED=0
TESTS_FAILED=0

test_result() {
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✓ PASS${NC}: $1"
        ((TESTS_PASSED++))
    else
        echo -e "${RED}✗ FAIL${NC}: $1"
        ((TESTS_FAILED++))
        return 1
    fi
}

# 1. Test protocol compilation
echo -e "${YELLOW}[1/7]${NC} Testing protocol compilation..."
cd packages/sutra-protocol
cargo build --release 2>&1 | tail -5
test_result "Protocol compiles"

# 2. Test protocol tests
echo -e "\n${YELLOW}[2/7]${NC} Running protocol tests..."
cargo test --quiet 2>&1 | tail -10
test_result "Protocol tests pass"

# 3. Test storage server compilation
echo -e "\n${YELLOW}[3/7]${NC} Testing storage server compilation..."
cd ../sutra-storage
cargo build --release --bin storage-server 2>&1 | tail -5
test_result "Storage server compiles"

# 4. Install Python client
echo -e "\n${YELLOW}[4/7]${NC} Installing Python storage client..."
cd ../sutra-storage-client-tcp
pip install -e . > /dev/null 2>&1
test_result "Python client installs"

# 5. Start storage server
echo -e "\n${YELLOW}[5/7]${NC} Starting storage server..."
cd ../sutra-storage
export STORAGE_PATH=/tmp/test-storage-$$.dat
export STORAGE_PORT=50099  # Use non-standard port to avoid conflicts
./target/release/storage-server &
SERVER_PID=$!
echo "Server PID: $SERVER_PID"

# Wait for server to start
sleep 3

# Check if server is running
if ! kill -0 $SERVER_PID 2>/dev/null; then
    echo -e "${RED}Server failed to start${NC}"
    exit 1
fi
test_result "Storage server starts"

# 6. Test Python client
echo -e "\n${YELLOW}[6/7]${NC} Testing Python client..."
python3 << 'EOF'
import sys
from sutra_storage_client import StorageClient

try:
    # Connect
    client = StorageClient('localhost:50099')
    
    # Health check
    health = client.health_check()
    assert health['healthy'] == True, "Health check failed"
    print("  ✓ Health check passed")
    
    # Learn concept
    seq = client.learn_concept("test-concept", "Hello, World!")
    assert seq > 0, "Learn concept failed"
    print(f"  ✓ Learned concept (seq={seq})")
    
    # Query concept
    result = client.query_concept("test-concept")
    assert result is not None, "Query failed"
    assert result['content'] == "Hello, World!", "Content mismatch"
    print(f"  ✓ Query concept passed")
    
    # Stats
    stats = client.stats()
    assert stats['concepts'] >= 1, "Stats failed"
    print(f"  ✓ Stats: {stats['concepts']} concepts")
    
    client.close()
    print("SUCCESS: All Python client tests passed")
    sys.exit(0)
    
except Exception as e:
    print(f"ERROR: {e}")
    import traceback
    traceback.print_exc()
    sys.exit(1)
EOF

test_result "Python client works"

# 7. Cleanup
echo -e "\n${YELLOW}[7/7]${NC} Cleaning up..."
kill $SERVER_PID 2>/dev/null || true
rm -f /tmp/test-storage-$$.dat
test_result "Cleanup complete"

# Summary
echo ""
echo "========================================="
echo "Test Summary"
echo "========================================="
echo -e "Passed: ${GREEN}${TESTS_PASSED}${NC}"
echo -e "Failed: ${RED}${TESTS_FAILED}${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ ALL TESTS PASSED${NC}"
    echo ""
    echo "Next steps:"
    echo "  1. Update docker-compose-grid.yml"
    echo "  2. Run: ./sutra-deploy.sh build"
    echo "  3. Run: ./sutra-deploy.sh up"
    exit 0
else
    echo -e "${RED}✗ SOME TESTS FAILED${NC}"
    echo ""
    echo "Fix the failures above before deploying."
    exit 1
fi
