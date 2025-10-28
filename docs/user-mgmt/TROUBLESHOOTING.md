# Troubleshooting Guide

**Common issues and solutions for Sutra AI user management system**

## Overview

This guide provides systematic troubleshooting procedures for the user authentication system, covering common problems, diagnostic techniques, and resolution strategies.

## Common Issues

### 1. Sessions Not Persisting After Login

**Symptoms:**
- User can login successfully and receive JWT token
- `/auth/me` endpoint returns 401 Unauthorized immediately after login
- Session appears created but cannot be retrieved

**Root Cause:**
Sessions stored without vector embeddings cannot be found via vector search.

**Diagnosis:**
```bash
# Check session storage configuration
docker logs sutra-user-storage | grep -i "semantic\|embedding"

# Verify embedding service connectivity
curl -s http://localhost:8888/health

# Check if sessions are being stored with embeddings
python3 -c "
from sutra_storage_client import StorageClient
import json

client = StorageClient('localhost:50053')
dummy_vector = [0.0] * 768
results = client.vector_search(dummy_vector, k=50)

session_count = 0
for concept_id, similarity in results:
    concept = client.query_concept(concept_id)
    if concept:
        try:
            data = json.loads(concept['content'])
            if data.get('type') == 'session':
                session_count += 1
                print(f'Found session: {data.get(\"session_id\", \"unknown\")}')
        except: pass
        
print(f'Total sessions found: {session_count}')
"
```

**Solution:**
1. **Enable embeddings for session storage** in `UserService.create_session()`:
```python
# In packages/sutra-api/sutra_api/services/user_service.py
options = {
    "generate_embedding": True,        # REQUIRED for session retrieval
    "extract_associations": False      # Sessions don't need associations
}

concept_id = await self.storage_client.learn_concept_v2(
    content=json.dumps(session_dict),
    options=options
)
```

2. **Verify embedding service is running**:
```bash
docker logs sutra-embedding-single --tail 20
curl -v http://localhost:8888/health
```

3. **Restart user storage server** after configuration changes:
```bash
docker restart sutra-user-storage
```

### 2. User Registration Fails with Storage Errors

**Symptoms:**
- Registration endpoint returns 500 Internal Server Error
- User data appears valid
- Storage server logs show connection errors

**Root Cause:**
User storage server not running or not accessible on expected port.

**Diagnosis:**
```bash
# Check user storage server status
docker ps | grep user-storage

# Verify port mapping
docker port sutra-user-storage

# Test direct storage connectivity
telnet localhost 50053

# Check storage server logs
docker logs sutra-user-storage --tail 50
```

**Solution:**
1. **Verify storage server is running**:
```bash
# Check service status
./sutra status

# If not running, start services
SUTRA_EDITION=simple ./sutra deploy
```

2. **Check port configuration** in `docker-compose.yml`:
```yaml
user-storage-server:
  ports:
    - "50053:50051"  # External port 50053 maps to internal 50051
```

3. **Verify API service connection configuration**:
```bash
# Check API service environment
docker exec sutra-api env | grep STORAGE
# Should show: USER_STORAGE_HOST=user-storage-server:50051
```

### 3. Semantic Analysis Interfering with User Data

**Symptoms:**
- User/session data classified as "Business" domain
- Unexpected associations created between users
- Performance degradation in user operations

**Root Cause:**
Semantic analysis enabled on user storage server, treating auth data as business knowledge.

**Diagnosis:**
```bash
# Check semantic analysis setting
docker exec sutra-user-storage env | grep SUTRA_SEMANTIC_ANALYSIS

# Check for unwanted associations in user data
python3 -c "
from sutra_storage_client import StorageClient
import json

client = StorageClient('localhost:50053')
dummy_vector = [0.0] * 768
results = client.vector_search(dummy_vector, k=50)

for concept_id, similarity in results:
    concept = client.query_concept(concept_id)
    if concept:
        try:
            data = json.loads(concept['content'])
            if data.get('type') in ['user', 'session']:
                # Check for unwanted classifications
                if 'classification' in concept:
                    print(f'User data classified as: {concept[\"classification\"]}')
                if 'associations' in concept:
                    print(f'User data has associations: {len(concept[\"associations\"])}')
        except: pass
"
```

**Solution:**
1. **Disable semantic analysis** for user storage in `docker-compose.yml`:
```yaml
user-storage-server:
  environment:
    - SUTRA_SEMANTIC_ANALYSIS=false      # Disable semantic analysis
    - SUTRA_EMBEDDING_SERVICE_URL=http://embedding-single:8888
```

2. **Restart user storage service**:
```bash
docker-compose restart user-storage-server
```

3. **Clear existing semantic associations** (if needed):
```bash
# Backup first
docker exec sutra-user-storage cp -r /data /data.backup

# Clean restart  
docker stop sutra-user-storage
docker rm sutra-user-storage
docker volume rm sutra-models_user-storage-data
SUTRA_EDITION=simple ./sutra deploy
```

### 4. Password Verification Failures

**Symptoms:**
- Users cannot login with correct passwords
- Registration succeeds but login always fails
- Password hash format appears incorrect

**Root Cause:**
Argon2 configuration mismatch or dependency issues.

**Diagnosis:**
```bash
# Check Argon2 installation
docker exec sutra-api python -c "
import argon2
ph = argon2.PasswordHasher()
print('Argon2 version:', argon2.__version__)

# Test hash/verify
password = 'test123'
hash_value = ph.hash(password)
print('Hash format:', hash_value[:50])

try:
    ph.verify(hash_value, password)
    print('Verification: SUCCESS')
except Exception as e:
    print('Verification: FAILED -', e)
"

# Check user password hashes in storage
python3 -c "
from sutra_storage_client import StorageClient
import json

client = StorageClient('localhost:50053')
dummy_vector = [0.0] * 768
results = client.vector_search(dummy_vector, k=20)

for concept_id, similarity in results:
    concept = client.query_concept(concept_id)
    if concept:
        try:
            data = json.loads(concept['content'])
            if data.get('type') == 'user':
                hash_format = data.get('password_hash', '')[:50]
                print(f'User {data.get(\"email\")}: {hash_format}...')
        except: pass
"
```

**Solution:**
1. **Verify Argon2 configuration** in `UserService.__init__()`:
```python
self.password_hasher = PasswordHasher(
    memory_cost=65536,    # 64 MB
    time_cost=3,          # 3 iterations  
    parallelism=4,        # 4 threads
    hash_len=32,          # 32 byte hash
    salt_len=16           # 16 byte salt
)
```

2. **Test password hashing manually**:
```bash
docker exec -it sutra-api python -c "
from argon2 import PasswordHasher
ph = PasswordHasher(memory_cost=65536, time_cost=3, parallelism=4)

# Test with known password
password = 'test123'
hash_val = ph.hash(password)
print('Generated hash:', hash_val)

# Verify it works
try:
    ph.verify(hash_val, password)
    print('✅ Hash verification successful')
except Exception as e:
    print('❌ Hash verification failed:', e)
"
```

3. **Reinstall Argon2 dependencies** if needed:
```bash
# Rebuild API service
docker-compose build sutra-api
docker-compose up -d sutra-api
```

### 5. JWT Token Validation Errors

**Symptoms:**
- `/auth/me` returns "Invalid token" for valid tokens
- Token appears correctly formatted
- Session exists but token validation fails

**Root Cause:**
JWT secret key mismatch or token expiration issues.

**Diagnosis:**
```bash
# Check JWT secret configuration
docker exec sutra-api env | grep JWT_SECRET

# Decode token manually (without verification)
python3 -c "
import base64
import json

# Get token from login response
token = 'eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyX2lkIjoiM...'

# Decode header and payload (without signature verification)
try:
    header, payload, signature = token.split('.')
    
    # Add padding if needed
    payload += '=' * (4 - len(payload) % 4)
    
    decoded_payload = json.loads(base64.urlsafe_b64decode(payload))
    print('Token payload:', json.dumps(decoded_payload, indent=2))
    
    # Check expiration
    import time
    current_time = time.time()
    exp_time = decoded_payload.get('exp', 0)
    
    print(f'Current time: {current_time}')
    print(f'Expiration: {exp_time}')
    print(f'Valid: {current_time < exp_time}')
    
except Exception as e:
    print(f'Token decode error: {e}')
"

# Test token generation and validation
docker exec sutra-api python -c "
from sutra_api.core.auth import create_access_token, verify_token

# Test token creation
token = create_access_token({
    'user_id': 'test_123',
    'session_id': 'session_456',
    'email': 'test@example.com'
})
print('Generated token:', token[:50] + '...')

# Test token verification
try:
    payload = verify_token(token)
    print('✅ Token verification successful:', payload)
except Exception as e:
    print('❌ Token verification failed:', e)
"
```

**Solution:**
1. **Verify JWT secret is set**:
```bash
# Check environment variable
docker exec sutra-api env | grep JWT_SECRET

# If not set, add to docker-compose.yml
# JWT_SECRET_KEY=your-secret-key-here
```

2. **Check token expiration settings**:
```python
# In packages/sutra-api/sutra_api/core/auth.py
def create_access_token(data: dict, expires_delta: timedelta = None):
    if expires_delta:
        expire = datetime.utcnow() + expires_delta
    else:
        expire = datetime.utcnow() + timedelta(minutes=ACCESS_TOKEN_EXPIRE_MINUTES)
    
    # Ensure ACCESS_TOKEN_EXPIRE_MINUTES is reasonable (e.g., 60)
```

3. **Regenerate tokens** after configuration changes:
```bash
# Restart API service
docker-compose restart sutra-api

# Users will need to login again to get new tokens
```

### 6. Embedding Service Connectivity Issues

**Symptoms:**
- Concept storage fails with embedding timeouts
- Registration/session creation succeeds intermittently
- Storage logs show embedding service errors

**Root Cause:**
Embedding service not running, overloaded, or connectivity issues.

**Diagnosis:**
```bash
# Check embedding service status
docker ps | grep embedding

# Test embedding service directly
curl -s http://localhost:8888/health

# Check service logs
docker logs sutra-embedding-single --tail 20

# Test embedding generation
curl -X POST http://localhost:8888/embed \
  -H "Content-Type: application/json" \
  -d '{"text": "test embedding generation"}'

# Check storage connection to embedding service
docker exec sutra-user-storage env | grep EMBEDDING
```

**Solution:**
1. **Ensure embedding service is running**:
```bash
# Check service status
./sutra status

# If embedding service is down
docker-compose up -d sutra-embedding-single
```

2. **Verify network connectivity**:
```bash
# Test from storage container
docker exec sutra-user-storage curl -s http://embedding-single:8888/health

# Check Docker network
docker network inspect sutra-models_default
```

3. **Increase embedding timeout** if service is slow:
```yaml
# In docker-compose.yml
user-storage-server:
  environment:
    - SUTRA_EMBEDDING_TIMEOUT_SEC=60  # Increase from 30 to 60 seconds
```

4. **Scale embedding service** for high load:
```bash
# Use HA embedding setup
SUTRA_EDITION=community ./sutra deploy
# This deploys 3 embedding replicas with HAProxy load balancer
```

### 7. Storage Data Corruption

**Symptoms:**
- Storage server fails to start
- Vector search returns no results
- WAL replay errors during startup

**Root Cause:**
Storage files corrupted due to improper shutdown or disk issues.

**Diagnosis:**
```bash
# Check storage server startup logs
docker logs sutra-user-storage | grep -i "error\|corruption\|wal"

# Verify storage files exist and have reasonable sizes
docker exec sutra-user-storage ls -la /data/
docker exec sutra-user-storage du -sh /data/*

# Check file integrity
docker exec sutra-user-storage file /data/storage.dat
docker exec sutra-user-storage file /data/storage.usearch
```

**Solution:**
1. **Attempt WAL recovery**:
```bash
# Stop storage server
docker stop sutra-user-storage

# Start with WAL replay enabled (default)
docker start sutra-user-storage

# Check startup logs
docker logs sutra-user-storage --tail 50
```

2. **Restore from backup** if recovery fails:
```bash
# Stop service
docker stop sutra-user-storage

# Restore from backup (if available)
docker cp /backups/user-storage-20251028/storage.dat sutra-user-storage:/data/
docker cp /backups/user-storage-20251028/storage.usearch sutra-user-storage:/data/

# Start service
docker start sutra-user-storage
```

3. **Clean restart** as last resort:
```bash
# WARNING: This will delete all user data
docker stop sutra-user-storage
docker rm sutra-user-storage
docker volume rm sutra-models_user-storage-data

# Redeploy
SUTRA_EDITION=simple ./sutra deploy
```

## Diagnostic Tools

### Health Check Script
```bash
#!/bin/bash
# scripts/diagnose-auth-system.sh

echo "🔍 Sutra Authentication System Diagnosis"
echo "========================================"

# Check service status
echo "📋 Service Status:"
docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}" | grep -E "(api|storage|embedding)"

# Test API health
echo -e "\n🏥 API Health Check:"
curl -s -w "Status: %{http_code}, Time: %{time_total}s\n" http://localhost:8000/auth/health || echo "❌ API not responding"

# Test storage connectivity
echo -e "\n💾 Storage Connectivity:"
python3 -c "
from sutra_storage_client import StorageClient
import time
try:
    start = time.time()
    client = StorageClient('localhost:50053')
    dummy_vector = [0.0] * 768
    results = client.vector_search(dummy_vector, k=5)
    duration = time.time() - start
    print(f'✅ Storage responsive: {len(results)} results in {duration:.3f}s')
except Exception as e:
    print(f'❌ Storage error: {e}')
"

# Test embedding service
echo -e "\n🧠 Embedding Service:"
curl -s -w "Status: %{http_code}, Time: %{time_total}s\n" http://localhost:8888/health || echo "❌ Embedding service not responding"

# Check user storage configuration
echo -e "\n⚙️  User Storage Configuration:"
docker exec sutra-user-storage env | grep -E "(SUTRA_SEMANTIC_ANALYSIS|SUTRA_EMBEDDING)" || echo "❌ Cannot access user storage environment"

# Count stored concepts by type
echo -e "\n📊 Stored Concepts:"
python3 -c "
from sutra_storage_client import StorageClient
import json

try:
    client = StorageClient('localhost:50053')
    dummy_vector = [0.0] * 768
    results = client.vector_search(dummy_vector, k=100)
    
    types = {}
    for concept_id, similarity in results:
        concept = client.query_concept(concept_id)
        if concept:
            try:
                data = json.loads(concept['content'])
                concept_type = data.get('type', 'unknown')
                types[concept_type] = types.get(concept_type, 0) + 1
            except: 
                types['parse_error'] = types.get('parse_error', 0) + 1
    
    for concept_type, count in types.items():
        print(f'  {concept_type}: {count}')
        
    if not types:
        print('  No concepts found')
        
except Exception as e:
    print(f'❌ Error counting concepts: {e}')
"

echo -e "\n✅ Diagnosis complete"
```

### Performance Monitor
```bash
#!/bin/bash
# scripts/monitor-auth-performance.sh

echo "📈 Authentication Performance Monitor"
echo "==================================="

# Function to test operation performance
test_operation() {
    local operation=$1
    local url=$2
    local data=$3
    
    echo "Testing $operation..."
    
    for i in {1..5}; do
        if [ -n "$data" ]; then
            time_result=$(curl -s -w "%{time_total}" -o /dev/null -X POST "$url" -H "Content-Type: application/json" -d "$data")
        else
            time_result=$(curl -s -w "%{time_total}" -o /dev/null "$url")
        fi
        echo "  Attempt $i: ${time_result}s"
    done
}

# Test registration performance
TIMESTAMP=$(date +%s)
REG_DATA="{\"email\":\"perf_$TIMESTAMP@example.com\",\"password\":\"test123\",\"full_name\":\"Performance Test\",\"organization\":\"Test\"}"

test_operation "Registration" "http://localhost:8000/auth/register" "$REG_DATA"

# Test login performance (reuse the registered user)
LOGIN_DATA="{\"email\":\"perf_$TIMESTAMP@example.com\",\"password\":\"test123\"}"
test_operation "Login" "http://localhost:8000/auth/login" "$LOGIN_DATA"

# Test health endpoint performance
test_operation "Health Check" "http://localhost:8000/auth/health"

echo "✅ Performance monitoring complete"
```

### Log Analyzer
```bash
#!/bin/bash
# scripts/analyze-auth-logs.sh

echo "📋 Authentication Log Analysis"
echo "============================="

# Analyze API logs
echo "🔍 API Errors (last 100 lines):"
docker logs sutra-api --tail 100 | grep -i "error\|exception\|traceback" | tail -10

echo -e "\n🔍 Storage Logs (last 50 lines):"
docker logs sutra-user-storage --tail 50 | grep -i "error\|warn\|fail" | tail -5

echo -e "\n🔍 Embedding Service Logs:"
docker logs sutra-embedding-single --tail 30 | grep -i "error\|warn" | tail -3

# Analyze request patterns
echo -e "\n📊 Recent Authentication Requests:"
docker logs sutra-api --tail 200 | grep -E "(POST /auth/register|POST /auth/login|GET /auth/me)" | tail -10

echo -e "\n✅ Log analysis complete"
```

## Prevention Strategies

### Regular Health Monitoring
```bash
#!/bin/bash
# scripts/auth-health-cron.sh
# Add to crontab: */5 * * * * /path/to/auth-health-cron.sh

LOG_FILE="/var/log/sutra-auth-health.log"
TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')

# Test critical endpoints
API_HEALTH=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:8000/auth/health 2>/dev/null || echo "000")
EMBEDDING_HEALTH=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:8888/health 2>/dev/null || echo "000")

# Log results
echo "[$TIMESTAMP] API: $API_HEALTH, Embedding: $EMBEDDING_HEALTH" >> $LOG_FILE

# Alert on failures
if [ "$API_HEALTH" != "200" ] || [ "$EMBEDDING_HEALTH" != "200" ]; then
    echo "[$TIMESTAMP] ALERT: Service health check failed!" >> $LOG_FILE
    # Add notification logic (email, Slack, etc.)
fi
```

### Backup Automation
```bash
#!/bin/bash
# scripts/backup-user-storage-cron.sh
# Add to crontab: 0 2 * * * /path/to/backup-user-storage-cron.sh

BACKUP_DIR="/backups/user-storage"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
TARGET_DIR="$BACKUP_DIR/$TIMESTAMP"

mkdir -p $TARGET_DIR

# Create backup
docker exec sutra-user-storage tar czf - /data | cat > $TARGET_DIR/user-storage-data.tar.gz

# Verify backup
if [ -f "$TARGET_DIR/user-storage-data.tar.gz" ] && [ -s "$TARGET_DIR/user-storage-data.tar.gz" ]; then
    echo "$(date): Backup successful - $TARGET_DIR" >> $BACKUP_DIR/backup.log
    
    # Clean old backups (keep last 7 days)
    find $BACKUP_DIR -name "20*" -type d -mtime +7 -exec rm -rf {} \;
else
    echo "$(date): Backup failed - $TARGET_DIR" >> $BACKUP_DIR/backup.log
fi
```

### Configuration Validation
```python
#!/usr/bin/env python3
# scripts/validate-auth-config.py

import os
import json
import asyncio
from sutra_storage_client import StorageClient

async def validate_configuration():
    """Validate authentication system configuration."""
    
    issues = []
    
    # Check environment variables
    required_env_vars = [
        'JWT_SECRET_KEY',
        'USER_STORAGE_HOST', 
        'SUTRA_EMBEDDING_SERVICE_URL'
    ]
    
    print("🔍 Checking environment configuration...")
    for var in required_env_vars:
        if not os.getenv(var):
            issues.append(f"Missing environment variable: {var}")
    
    # Test storage connectivity
    print("🔍 Testing storage connectivity...")
    try:
        client = StorageClient(os.getenv('USER_STORAGE_HOST', 'localhost:50053'))
        dummy_vector = [0.0] * 768
        results = client.vector_search(dummy_vector, k=1)
        print(f"✅ Storage responsive: {len(results)} results")
    except Exception as e:
        issues.append(f"Storage connectivity error: {e}")
    
    # Test embedding service
    print("🔍 Testing embedding service...")
    import httpx
    try:
        async with httpx.AsyncClient() as client:
            response = await client.get(os.getenv('SUTRA_EMBEDDING_SERVICE_URL', 'http://localhost:8888') + '/health')
            if response.status_code == 200:
                print("✅ Embedding service responsive")
            else:
                issues.append(f"Embedding service returned HTTP {response.status_code}")
    except Exception as e:
        issues.append(f"Embedding service error: {e}")
    
    # Report results
    if issues:
        print(f"\n❌ Configuration issues found:")
        for issue in issues:
            print(f"  - {issue}")
        return False
    else:
        print(f"\n✅ Configuration validation passed")
        return True

if __name__ == "__main__":
    success = asyncio.run(validate_configuration())
    exit(0 if success else 1)
```

---

**AI Context**: This troubleshooting guide addresses the most common issues in the Sutra AI user authentication system, with particular focus on the vector-based storage architecture and embedding requirements. The guide provides systematic diagnosis procedures and step-by-step solutions for session persistence issues, storage connectivity problems, and performance degradation scenarios.

**Last Updated**: 2025-10-28  
**Critical Focus Areas**: Session embedding requirements, storage server configuration, JWT token validation, embedding service connectivity, and data integrity verification.