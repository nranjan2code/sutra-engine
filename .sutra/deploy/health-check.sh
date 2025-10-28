#!/bin/bash
# Sutra AI - Health Check Utility
# Universal health check script for all services
# Usage: health-check <url> [timeout] [retries]

URL="$1"
TIMEOUT="${2:-3}"
RETRIES="${3:-1}"

if [ -z "$URL" ]; then
    echo "Usage: health-check <url> [timeout] [retries]"
    exit 1
fi

# Function to check URL
check_health() {
    if command -v curl >/dev/null 2>&1; then
        curl -f -s --max-time "$TIMEOUT" "$URL" >/dev/null
    elif command -v wget >/dev/null 2>&1; then
        wget -q -T "$TIMEOUT" -O /dev/null "$URL"
    else
        echo "Neither curl nor wget available for health check"
        exit 1
    fi
}

# Retry logic
for i in $(seq 1 "$RETRIES"); do
    if check_health; then
        exit 0
    fi
    
    if [ "$i" -lt "$RETRIES" ]; then
        sleep 1
    fi
done

exit 1