#!/bin/bash
# Validate Sutra Works Naming Conventions
# Ensures all containers and images follow the sutra-works- prefix standard

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "=================================================="
echo "Sutra Works - Naming Convention Validation"
echo "=================================================="
echo ""

ISSUES_FOUND=0

# Check Docker is available
if ! docker ps > /dev/null 2>&1; then
    echo -e "${RED}ERROR: Docker is not running or not accessible${NC}"
    exit 1
fi

echo "1. Validating Container Names..."
echo "-----------------------------------"

# Get all Sutra containers (both correct and incorrect naming)
ALL_SUTRA_CONTAINERS=$(docker ps -a --format "{{.Names}}" | grep "^sutra-" || true)

if [ -z "$ALL_SUTRA_CONTAINERS" ]; then
    echo -e "${YELLOW}No Sutra containers found${NC}"
else
    echo "Found Sutra containers:"
    echo "$ALL_SUTRA_CONTAINERS" | sed 's/^/  - /'
    echo ""

    # Check for containers with incorrect prefix
    INVALID_CONTAINERS=$(echo "$ALL_SUTRA_CONTAINERS" | grep -v "^sutra-works-" || true)

    if [ -z "$INVALID_CONTAINERS" ]; then
        echo -e "${GREEN}✓ PASS: All containers use 'sutra-works-' prefix${NC}"
    else
        echo -e "${RED}✗ FAIL: Found containers with incorrect naming:${NC}"
        echo "$INVALID_CONTAINERS" | sed 's/^/  - /'
        echo ""
        echo "Expected format: sutra-works-<component>"
        ISSUES_FOUND=$((ISSUES_FOUND + 1))
    fi
fi

echo ""
echo "2. Validating Image Names..."
echo "-------------------------------"

# Get all Sutra images (both correct and incorrect naming)
ALL_SUTRA_IMAGES=$(docker images --format "{{.Repository}}" | grep "^sutra-" | sort -u || true)

if [ -z "$ALL_SUTRA_IMAGES" ]; then
    echo -e "${YELLOW}No Sutra images found${NC}"
else
    echo "Found Sutra images:"
    echo "$ALL_SUTRA_IMAGES" | sed 's/^/  - /'
    echo ""

    # Check for images with incorrect prefix
    INVALID_IMAGES=$(echo "$ALL_SUTRA_IMAGES" | grep -v "^sutra-works-" || true)

    if [ -z "$INVALID_IMAGES" ]; then
        echo -e "${GREEN}✓ PASS: All images use 'sutra-works-' prefix${NC}"
    else
        echo -e "${RED}✗ FAIL: Found images with incorrect naming:${NC}"
        echo "$INVALID_IMAGES" | sed 's/^/  - /'
        echo ""
        echo "Expected format: sutra-works-<component>"
        ISSUES_FOUND=$((ISSUES_FOUND + 1))
    fi
fi

echo ""
echo "3. Validating Docker Compose Configuration..."
echo "------------------------------------------------"

COMPOSE_FILE=".sutra/compose/production.yml"

if [ ! -f "$COMPOSE_FILE" ]; then
    echo -e "${RED}ERROR: Docker Compose file not found: $COMPOSE_FILE${NC}"
    exit 1
fi

# Check for container_name directives with incorrect naming
INVALID_COMPOSE_CONTAINERS=$(grep "container_name:" "$COMPOSE_FILE" | grep -v "sutra-works-" | grep "sutra-" || true)

if [ -z "$INVALID_COMPOSE_CONTAINERS" ]; then
    echo -e "${GREEN}✓ PASS: All container_name directives use 'sutra-works-' prefix${NC}"
else
    echo -e "${RED}✗ FAIL: Found container_name with incorrect naming in docker-compose:${NC}"
    echo "$INVALID_COMPOSE_CONTAINERS"
    ISSUES_FOUND=$((ISSUES_FOUND + 1))
fi

# Check for image directives with incorrect naming (exclude external images)
INVALID_COMPOSE_IMAGES=$(grep "image: sutra-" "$COMPOSE_FILE" | grep -v "sutra-works-" || true)

if [ -z "$INVALID_COMPOSE_IMAGES" ]; then
    echo -e "${GREEN}✓ PASS: All image directives use 'sutra-works-' prefix${NC}"
else
    echo -e "${RED}✗ FAIL: Found image with incorrect naming in docker-compose:${NC}"
    echo "$INVALID_COMPOSE_IMAGES"
    ISSUES_FOUND=$((ISSUES_FOUND + 1))
fi

echo ""
echo "4. Checking for Common Naming Issues..."
echo "------------------------------------------"

# Check for containers with spaces or special characters
if [ -n "$ALL_SUTRA_CONTAINERS" ]; then
    SPECIAL_CHAR_CONTAINERS=$(echo "$ALL_SUTRA_CONTAINERS" | grep -E '[^a-zA-Z0-9_-]' || true)

    if [ -n "$SPECIAL_CHAR_CONTAINERS" ]; then
        echo -e "${RED}✗ FAIL: Found containers with special characters:${NC}"
        echo "$SPECIAL_CHAR_CONTAINERS" | sed 's/^/  - /'
        ISSUES_FOUND=$((ISSUES_FOUND + 1))
    else
        echo -e "${GREEN}✓ PASS: No special characters in container names${NC}"
    fi
fi

# Check for excessively long names (Docker limit is 63 characters)
if [ -n "$ALL_SUTRA_CONTAINERS" ]; then
    LONG_NAMES=$(echo "$ALL_SUTRA_CONTAINERS" | awk 'length > 63')

    if [ -n "$LONG_NAMES" ]; then
        echo -e "${RED}✗ FAIL: Found containers exceeding 63 character limit:${NC}"
        echo "$LONG_NAMES" | sed 's/^/  - /'
        ISSUES_FOUND=$((ISSUES_FOUND + 1))
    else
        echo -e "${GREEN}✓ PASS: All container names within length limit${NC}"
    fi
fi

echo ""
echo "5. Validating Service Consistency..."
echo "---------------------------------------"

# Check that container names match expected patterns from service definitions
EXPECTED_CONTAINERS=(
    "sutra-works-nginx-proxy"
    "sutra-works-storage"
    "sutra-works-api"
    "sutra-works-hybrid"
    "sutra-works-client"
)

echo "Checking core services are using correct names..."
for container in "${EXPECTED_CONTAINERS[@]}"; do
    if docker ps -a --format "{{.Names}}" | grep -q "^${container}$"; then
        echo -e "${GREEN}✓${NC} Found: $container"
    else
        echo -e "${YELLOW}⚠${NC}  Not running: $container"
    fi
done

echo ""
echo "=================================================="
echo "Validation Summary"
echo "=================================================="

if [ $ISSUES_FOUND -eq 0 ]; then
    echo -e "${GREEN}✓ All naming conventions validated successfully!${NC}"
    echo ""
    echo "Summary:"
    echo "  - All containers use 'sutra-works-' prefix"
    echo "  - All images use 'sutra-works-' prefix"
    echo "  - Docker Compose configuration is correct"
    echo "  - No naming conflicts detected"
    exit 0
else
    echo -e "${RED}✗ Found $ISSUES_FOUND issue(s) with naming conventions${NC}"
    echo ""
    echo "Recommendations:"
    echo "1. Review .sutra/compose/production.yml"
    echo "2. Update container_name and image fields to use 'sutra-works-' prefix"
    echo "3. Rebuild affected services:"
    echo "   docker-compose -f .sutra/compose/production.yml build"
    echo "4. Restart services:"
    echo "   docker-compose -f .sutra/compose/production.yml up -d"
    echo "5. Review docs/deployment/NAMING_CONVENTIONS.md for details"
    exit 1
fi
