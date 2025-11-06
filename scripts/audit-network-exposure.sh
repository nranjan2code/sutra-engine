#!/bin/bash
# Network Exposure Audit Script
# Verifies that internal services are not exposed to the host

set -e

echo "=================================================="
echo "Sutra AI - Network Exposure Audit"
echo "=================================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

ISSUES_FOUND=0

echo "1. Checking Docker Compose Configuration..."
echo "-------------------------------------------"

COMPOSE_FILE=".sutra/compose/production.yml"

if [ ! -f "$COMPOSE_FILE" ]; then
    echo -e "${RED}ERROR: Docker Compose file not found: $COMPOSE_FILE${NC}"
    exit 1
fi

# Check for services that should NOT have exposed ports
INTERNAL_SERVICES=(
    "storage-server"
    "grid-event-storage"
    "user-storage-server"
    "grid-master"
    "grid-agent-1"
    "grid-agent-2"
    "ml-base-service"
    "embedding-single"
    "embedding-ha"
    "nlg-single"
    "nlg-ha"
)

echo "Checking internal services are not exposed in Docker Compose..."
for service in "${INTERNAL_SERVICES[@]}"; do
    # Check if service has "ports:" directive
    if grep -A 2 "^  $service:" "$COMPOSE_FILE" | grep -q "ports:"; then
        echo -e "${RED}✗ FAIL: $service has exposed ports in docker-compose${NC}"
        ISSUES_FOUND=$((ISSUES_FOUND + 1))
    else
        echo -e "${GREEN}✓ PASS: $service uses expose: (internal only)${NC}"
    fi
done

echo ""
echo "2. Checking Running Containers..."
echo "-----------------------------------"

# Check if Docker is running
if ! docker ps > /dev/null 2>&1; then
    echo -e "${YELLOW}WARNING: Docker is not running or not accessible${NC}"
    echo "Cannot check running containers"
else
    echo "Checking for exposed ports on running containers..."

    # Get list of running containers with ports
    EXPOSED_CONTAINERS=$(docker ps --format "{{.Names}}\t{{.Ports}}" | grep -v "sutra-works-nginx-proxy" || true)

    if [ -z "$EXPOSED_CONTAINERS" ]; then
        echo -e "${GREEN}✓ PASS: No internal services have exposed ports${NC}"
    else
        echo -e "${YELLOW}Found containers with exposed ports (excluding nginx-proxy):${NC}"
        echo "$EXPOSED_CONTAINERS"
        echo ""

        # Check each internal service
        for service in "${INTERNAL_SERVICES[@]}"; do
            if echo "$EXPOSED_CONTAINERS" | grep -q "$service"; then
                # Check if it's using host port mapping
                PORTS=$(docker ps --filter "name=$service" --format "{{.Ports}}")
                if echo "$PORTS" | grep -q "0.0.0.0"; then
                    echo -e "${RED}✗ FAIL: $service is exposed to host network${NC}"
                    echo "  Ports: $PORTS"
                    ISSUES_FOUND=$((ISSUES_FOUND + 1))
                fi
            fi
        done
    fi
fi

echo ""
echo "3. Checking Nginx Proxy Configuration..."
echo "------------------------------------------"

NGINX_RUNNING=$(docker ps --filter "name=sutra-works-nginx-proxy" --format "{{.Names}}" || true)

if [ -z "$NGINX_RUNNING" ]; then
    echo -e "${YELLOW}WARNING: Nginx proxy is not running${NC}"
else
    echo -e "${GREEN}✓ Nginx proxy is running${NC}"

    # Check if nginx is exposing correct ports
    NGINX_PORTS=$(docker ps --filter "name=sutra-works-nginx-proxy" --format "{{.Ports}}")
    echo "Nginx exposed ports: $NGINX_PORTS"

    # Verify nginx has ports 80, 443, 8080
    if echo "$NGINX_PORTS" | grep -q "80" && echo "$NGINX_PORTS" | grep -q "443" && echo "$NGINX_PORTS" | grep -q "8080"; then
        echo -e "${GREEN}✓ PASS: Nginx has correct port mappings (80, 443, 8080)${NC}"
    else
        echo -e "${RED}✗ FAIL: Nginx port mappings incorrect${NC}"
        ISSUES_FOUND=$((ISSUES_FOUND + 1))
    fi

    # Test nginx configuration
    if docker exec sutra-works-nginx-proxy nginx -t > /dev/null 2>&1; then
        echo -e "${GREEN}✓ PASS: Nginx configuration is valid${NC}"
    else
        echo -e "${RED}✗ FAIL: Nginx configuration has errors${NC}"
        docker exec sutra-works-nginx-proxy nginx -t
        ISSUES_FOUND=$((ISSUES_FOUND + 1))
    fi
fi

echo ""
echo "4. Checking Network Accessibility..."
echo "--------------------------------------"

# Test that internal services are NOT accessible from host
echo "Testing that internal services are not accessible from host..."

INTERNAL_PORTS=(50051 50052 50053 7001 7002 8887 8888)

for port in "${INTERNAL_PORTS[@]}"; do
    if nc -z localhost "$port" 2>/dev/null; then
        echo -e "${RED}✗ FAIL: Port $port is accessible from host (should be internal only)${NC}"
        ISSUES_FOUND=$((ISSUES_FOUND + 1))
    else
        echo -e "${GREEN}✓ PASS: Port $port is not accessible from host${NC}"
    fi
done

# Test that proxy ports ARE accessible
echo ""
echo "Testing that proxy ports are accessible..."

PROXY_PORTS=(80 8080)

for port in "${PROXY_PORTS[@]}"; do
    if nc -z localhost "$port" 2>/dev/null; then
        echo -e "${GREEN}✓ PASS: Port $port is accessible (nginx proxy)${NC}"
    else
        echo -e "${YELLOW}WARNING: Port $port is not accessible (nginx may not be running)${NC}"
    fi
done

echo ""
echo "5. Security Headers Check..."
echo "------------------------------"

if [ -n "$NGINX_RUNNING" ]; then
    echo "Checking security headers..."

    HEADERS=$(curl -s -I http://localhost:8080/health 2>/dev/null || echo "")

    if [ -z "$HEADERS" ]; then
        echo -e "${YELLOW}WARNING: Could not fetch headers (nginx may not be responding)${NC}"
    else
        # Check for security headers
        SECURITY_HEADERS=(
            "X-Frame-Options"
            "X-Content-Type-Options"
            "X-XSS-Protection"
        )

        for header in "${SECURITY_HEADERS[@]}"; do
            if echo "$HEADERS" | grep -qi "$header"; then
                echo -e "${GREEN}✓ PASS: $header is present${NC}"
            else
                echo -e "${RED}✗ FAIL: $header is missing${NC}"
                ISSUES_FOUND=$((ISSUES_FOUND + 1))
            fi
        done
    fi
else
    echo -e "${YELLOW}SKIP: Nginx not running, cannot check headers${NC}"
fi

echo ""
echo "=================================================="
echo "Audit Summary"
echo "=================================================="

if [ $ISSUES_FOUND -eq 0 ]; then
    echo -e "${GREEN}✓ All checks passed! Network configuration is secure.${NC}"
    exit 0
else
    echo -e "${RED}✗ Found $ISSUES_FOUND issue(s)${NC}"
    echo ""
    echo "Recommendations:"
    echo "1. Review .sutra/compose/production.yml"
    echo "2. Ensure internal services use 'expose:' instead of 'ports:'"
    echo "3. Restart services: docker-compose -f .sutra/compose/production.yml up -d"
    echo "4. Review docs/deployment/NETWORK_SECURITY.md for details"
    exit 1
fi
