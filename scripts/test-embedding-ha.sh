#!/bin/bash
# Embedding Service HA Failover Test
# 
# Tests automatic failover when embedding service instances fail
# Expected behavior: Zero downtime, requests continue succeeding

set -e

EMBEDDING_ENDPOINT="http://localhost:8888"
HAPROXY_STATS="http://localhost:8404/stats"
TEST_DURATION=60
REQUEST_INTERVAL=0.5

echo "🧪 Embedding Service HA Failover Test"
echo "======================================"
echo ""
echo "Endpoints:"
echo "  Embedding Service: $EMBEDDING_ENDPOINT"
echo "  HAProxy Stats: $HAPROXY_STATS"
echo "  Test Duration: ${TEST_DURATION}s"
echo ""

# Colors
RED='\033[0:31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counter
TOTAL_REQUESTS=0
SUCCESSFUL_REQUESTS=0
FAILED_REQUESTS=0

# Function to make test request
test_request() {
    local start_time=$(date +%s%N)
    
    response=$(curl -s -w "\n%{http_code}" -X POST "$EMBEDDING_ENDPOINT/embed" \
        -H "Content-Type: application/json" \
        -d '{"texts": ["test embedding request"], "normalize": true}' 2>&1)
    
    local end_time=$(date +%s%N)
    local latency_ms=$(( (end_time - start_time) / 1000000 ))
    
    http_code=$(echo "$response" | tail -n1)
    
    TOTAL_REQUESTS=$((TOTAL_REQUESTS + 1))
    
    if [ "$http_code" = "200" ]; then
        SUCCESSFUL_REQUESTS=$((SUCCESSFUL_REQUESTS + 1))
        echo -e "${GREEN}✓${NC} Request #${TOTAL_REQUESTS}: ${latency_ms}ms (HTTP $http_code)"
        return 0
    else
        FAILED_REQUESTS=$((FAILED_REQUESTS + 1))
        echo -e "${RED}✗${NC} Request #${TOTAL_REQUESTS}: FAILED (HTTP $http_code)"
        return 1
    fi
}

# Function to check HAProxy stats
check_haproxy_stats() {
    echo -e "\n${YELLOW}📊 HAProxy Status:${NC}"
    curl -s "$HAPROXY_STATS;csv" | grep "embedding-" | while IFS=',' read -r fields; do
        server_name=$(echo "$fields" | cut -d',' -f2)
        status=$(echo "$fields" | cut -d',' -f18)
        echo "  - $server_name: $status"
    done
}

# Function to stop a container
stop_container() {
    local container=$1
    echo -e "\n${YELLOW}⚠️  Stopping $container...${NC}"
    docker stop "$container" >/dev/null 2>&1
    echo "  Container stopped"
}

# Function to start a container
start_container() {
    local container=$1
    echo -e "\n${YELLOW}🔄 Starting $container...${NC}"
    docker start "$container" >/dev/null 2>&1
    echo "  Container started"
    sleep 5  # Wait for health check
}

# Cleanup function
cleanup() {
    echo -e "\n\n${YELLOW}🧹 Cleaning up...${NC}"
    # Ensure all containers are running
    docker start embedding-1 embedding-2 embedding-3 >/dev/null 2>&1
    echo "  All embedding containers restarted"
}

trap cleanup EXIT

echo "Phase 1: Baseline Testing (30s)"
echo "--------------------------------"
end_time=$(($(date +%s) + 30))
while [ $(date +%s) -lt $end_time ]; do
    test_request || true
    sleep $REQUEST_INTERVAL
done

echo ""
check_haproxy_stats

echo -e "\nPhase 2: Single Instance Failure Test"
echo "--------------------------------------"
stop_container "embedding-1"
sleep 3  # Wait for HAProxy to detect failure

echo "Testing with embedding-1 down..."
end_time=$(($(date +%s) + 20))
while [ $(date +%s) -lt $end_time ]; do
    test_request || true
    sleep $REQUEST_INTERVAL
done

check_haproxy_stats

echo -e "\nPhase 3: Two Instance Failure Test"
echo "-----------------------------------"
stop_container "embedding-2"
sleep 3

echo "Testing with embedding-1 and embedding-2 down..."
end_time=$(($(date +%s) + 15))
while [ $(date +%s) -lt $end_time ]; do
    test_request || true
    sleep $REQUEST_INTERVAL
done

check_haproxy_stats

echo -e "\nPhase 4: Recovery Test"
echo "----------------------"
start_container "embedding-1"
start_container "embedding-2"

echo "Testing recovery..."
end_time=$(($(date +%s) + 20))
while [ $(date +%s) -lt $end_time ]; do
    test_request || true
    sleep $REQUEST_INTERVAL
done

check_haproxy_stats

# Calculate results
SUCCESS_RATE=$(awk "BEGIN {printf \"%.2f\", ($SUCCESSFUL_REQUESTS / $TOTAL_REQUESTS) * 100}")

echo ""
echo "======================================"
echo "📊 Test Results"
echo "======================================"
echo "Total Requests:      $TOTAL_REQUESTS"
echo "Successful:          $SUCCESSFUL_REQUESTS"
echo "Failed:              $FAILED_REQUESTS"
echo "Success Rate:        ${SUCCESS_RATE}%"
echo ""

# Determine pass/fail
if (( $(echo "$SUCCESS_RATE >= 95.0" | bc -l) )); then
    echo -e "${GREEN}✅ PASS: HA system maintained >95% availability${NC}"
    exit 0
else
    echo -e "${RED}❌ FAIL: HA system availability below 95%${NC}"
    exit 1
fi
