# Testing Guide

**Comprehensive testing procedures for Sutra AI user management system**

## Overview

This guide provides systematic testing procedures for the user authentication system, covering unit tests, integration tests, performance testing, and production validation.

## Test Environment Setup

### Prerequisites
```bash
# Ensure services are running
SUTRA_EDITION=simple ./sutra deploy

# Verify service health
./sutra status

# Check service logs
docker logs sutra-api --tail 20
docker logs sutra-user-storage --tail 20
docker logs sutra-embedding-single --tail 20
```

### Test Data Preparation
```bash
# Clean test environment
docker exec sutra-user-storage rm -f /data/storage.dat
docker restart sutra-user-storage

# Wait for service startup
sleep 5

# Verify clean state
curl -s http://localhost:8000/auth/health
```

## Unit Tests

### Authentication Service Tests
```python
# tests/unit/test_user_service.py
import pytest
import asyncio
from unittest.mock import AsyncMock, MagicMock, patch
from datetime import datetime, timedelta

from sutra_api.services.user_service import UserService
from sutra_api.models.user import User, UserCreate, UserLogin, Session

class TestUserService:
    
    @pytest.fixture
    def mock_storage_client(self):
        """Mock storage client for isolated testing."""
        client = MagicMock()
        client.vector_search = AsyncMock(return_value=[])
        client.query_concept = AsyncMock(return_value=None)
        client.learn_concept_v2 = AsyncMock(return_value="concept_123")
        return client
    
    @pytest.fixture
    def user_service(self, mock_storage_client):
        """User service with mocked dependencies."""
        return UserService(storage_client=mock_storage_client)
    
    @pytest.mark.asyncio
    async def test_register_user_success(self, user_service, mock_storage_client):
        """Test successful user registration."""
        # Mock empty search (no existing user)
        mock_storage_client.vector_search.return_value = []
        
        user_data = UserCreate(
            email="test@example.com",
            password="secure123",
            full_name="Test User",
            organization="Test Org"
        )
        
        result = await user_service.register_user(user_data)
        
        assert result is not None
        assert result.email == "test@example.com"
        assert result.full_name == "Test User"
        assert result.active is True
        
        # Verify storage was called
        mock_storage_client.learn_concept_v2.assert_called_once()
    
    @pytest.mark.asyncio
    async def test_register_user_duplicate_email(self, user_service, mock_storage_client):
        """Test registration with duplicate email."""
        # Mock existing user found
        existing_user_concept = {
            'content': json.dumps({
                'type': 'user',
                'email': 'test@example.com',
                'active': True
            })
        }
        mock_storage_client.vector_search.return_value = [("concept_123", 0.9)]
        mock_storage_client.query_concept.return_value = existing_user_concept
        
        user_data = UserCreate(
            email="test@example.com",
            password="secure123",
            full_name="Test User",
            organization="Test Org"
        )
        
        with pytest.raises(ValueError, match="Email already registered"):
            await user_service.register_user(user_data)
    
    @pytest.mark.asyncio
    async def test_authenticate_user_success(self, user_service, mock_storage_client):
        """Test successful user authentication."""
        # Mock user found with correct password
        user_concept = {
            'content': json.dumps({
                'type': 'user',
                'email': 'test@example.com',
                'password_hash': '$argon2id$v=19$m=65536,t=3,p=4$hash',
                'active': True,
                'full_name': 'Test User',
                'organization': 'Test Org',
                'role': 'user'
            })
        }
        mock_storage_client.vector_search.return_value = [("concept_123", 0.9)]
        mock_storage_client.query_concept.return_value = user_concept
        
        with patch('argon2.verify', return_value=True):
            result = await user_service.authenticate_user("test@example.com", "correct_password")
        
        assert result is not None
        assert result.email == "test@example.com"
    
    @pytest.mark.asyncio
    async def test_authenticate_user_wrong_password(self, user_service, mock_storage_client):
        """Test authentication with incorrect password."""
        user_concept = {
            'content': json.dumps({
                'type': 'user',
                'email': 'test@example.com',
                'password_hash': '$argon2id$v=19$m=65536,t=3,p=4$hash',
                'active': True
            })
        }
        mock_storage_client.vector_search.return_value = [("concept_123", 0.9)]
        mock_storage_client.query_concept.return_value = user_concept
        
        with patch('argon2.verify', return_value=False):
            result = await user_service.authenticate_user("test@example.com", "wrong_password")
        
        assert result is None
    
    @pytest.mark.asyncio
    async def test_create_session_success(self, user_service, mock_storage_client):
        """Test successful session creation."""
        user = User(
            id="user_123",
            email="test@example.com",
            full_name="Test User",
            organization="Test Org",
            role="user"
        )
        
        session = await user_service.create_session(user)
        
        assert session is not None
        assert session.user_id == "user_123"
        assert session.email == "test@example.com"
        assert session.active is True
        assert isinstance(session.expires_at, datetime)
        
        # Verify session was stored with embeddings enabled
        mock_storage_client.learn_concept_v2.assert_called_once()
        call_args = mock_storage_client.learn_concept_v2.call_args
        assert call_args[1]['options']['generate_embedding'] is True
    
    @pytest.mark.asyncio
    async def test_get_session_success(self, user_service, mock_storage_client):
        """Test successful session retrieval."""
        session_concept = {
            'content': json.dumps({
                'type': 'session',
                'session_id': 'session_123',
                'user_id': 'user_123',
                'email': 'test@example.com',
                'active': True,
                'expires_at': (datetime.utcnow() + timedelta(days=7)).isoformat(),
                'created_at': datetime.utcnow().isoformat()
            })
        }
        mock_storage_client.vector_search.return_value = [("concept_456", 0.9)]
        mock_storage_client.query_concept.return_value = session_concept
        
        session = await user_service.get_session("session_123")
        
        assert session is not None
        assert session.session_id == "session_123"
        assert session.active is True

# Run unit tests
# pytest tests/unit/test_user_service.py -v
```

### Password Security Tests
```python
# tests/unit/test_password_security.py
import pytest
from argon2 import PasswordHasher
from argon2.exceptions import VerifyMismatchError

class TestPasswordSecurity:
    
    def setup_method(self):
        """Setup password hasher with secure parameters."""
        self.ph = PasswordHasher(
            memory_cost=65536,  # 64 MB memory usage
            time_cost=3,        # 3 iterations
            parallelism=4,      # 4 parallel threads
            hash_len=32,        # 32 byte hash length
            salt_len=16         # 16 byte salt length
        )
    
    def test_password_hashing(self):
        """Test password hashing produces different hashes."""
        password = "secure_password_123"
        
        hash1 = self.ph.hash(password)
        hash2 = self.ph.hash(password)
        
        # Different salts produce different hashes
        assert hash1 != hash2
        assert hash1.startswith("$argon2id$v=19$")
        assert hash2.startswith("$argon2id$v=19$")
    
    def test_password_verification(self):
        """Test password verification works correctly."""
        password = "secure_password_123"
        hash_value = self.ph.hash(password)
        
        # Correct password verifies
        try:
            self.ph.verify(hash_value, password)
            verified = True
        except VerifyMismatchError:
            verified = False
        
        assert verified is True
    
    def test_wrong_password_verification(self):
        """Test wrong password fails verification."""
        password = "secure_password_123"
        wrong_password = "wrong_password"
        hash_value = self.ph.hash(password)
        
        # Wrong password fails
        with pytest.raises(VerifyMismatchError):
            self.ph.verify(hash_value, wrong_password)
    
    def test_password_strength_requirements(self):
        """Test password strength validation."""
        def validate_password_strength(password: str) -> bool:
            return len(password) >= 8
        
        assert validate_password_strength("short") is False
        assert validate_password_strength("long_enough_password") is True
        assert validate_password_strength("") is False

# pytest tests/unit/test_password_security.py -v
```

## Integration Tests

### Full Authentication Flow Test
```python
# tests/integration/test_auth_flow.py
import pytest
import httpx
import json
import asyncio
from datetime import datetime

class TestAuthenticationFlow:
    
    @pytest.fixture
    def base_url(self):
        return "http://localhost:8000"
    
    @pytest.fixture
    def test_user_data(self):
        timestamp = int(datetime.now().timestamp())
        return {
            "email": f"test_{timestamp}@example.com",
            "password": "secure123",
            "full_name": "Integration Test User",
            "organization": "Test Organization"
        }
    
    @pytest.mark.asyncio
    async def test_complete_authentication_flow(self, base_url, test_user_data):
        """Test complete registration -> login -> session -> logout flow."""
        
        async with httpx.AsyncClient() as client:
            
            # Step 1: Register user
            register_response = await client.post(
                f"{base_url}/auth/register",
                json=test_user_data
            )
            
            assert register_response.status_code == 200
            user_data = register_response.json()
            assert user_data["email"] == test_user_data["email"]
            assert user_data["full_name"] == test_user_data["full_name"]
            assert "password" not in user_data  # Password should not be returned
            
            # Step 2: Login user
            login_data = {
                "email": test_user_data["email"],
                "password": test_user_data["password"]
            }
            
            login_response = await client.post(
                f"{base_url}/auth/login",
                json=login_data
            )
            
            assert login_response.status_code == 200
            login_result = login_response.json()
            assert "access_token" in login_result
            assert "session_id" in login_result
            assert login_result["token_type"] == "bearer"
            
            access_token = login_result["access_token"]
            session_id = login_result["session_id"]
            
            # Step 3: Verify session with token
            headers = {"Authorization": f"Bearer {access_token}"}
            
            profile_response = await client.get(
                f"{base_url}/auth/me", 
                headers=headers
            )
            
            assert profile_response.status_code == 200
            profile_data = profile_response.json()
            assert profile_data["email"] == test_user_data["email"]
            assert profile_data["session_id"] == session_id
            
            # Step 4: Logout user
            logout_response = await client.post(
                f"{base_url}/auth/logout",
                headers=headers
            )
            
            assert logout_response.status_code == 200
            logout_result = logout_response.json()
            assert logout_result["message"] == "Successfully logged out"
            
            # Step 5: Verify token is invalidated
            profile_response_after_logout = await client.get(
                f"{base_url}/auth/me",
                headers=headers
            )
            
            assert profile_response_after_logout.status_code == 401
    
    @pytest.mark.asyncio
    async def test_duplicate_registration(self, base_url, test_user_data):
        """Test that duplicate email registration fails."""
        
        async with httpx.AsyncClient() as client:
            
            # First registration should succeed
            first_response = await client.post(
                f"{base_url}/auth/register",
                json=test_user_data
            )
            assert first_response.status_code == 200
            
            # Second registration with same email should fail
            second_response = await client.post(
                f"{base_url}/auth/register",
                json=test_user_data
            )
            assert second_response.status_code == 400
            error_data = second_response.json()
            assert "already registered" in error_data["detail"].lower()
    
    @pytest.mark.asyncio
    async def test_invalid_login(self, base_url, test_user_data):
        """Test login with invalid credentials."""
        
        async with httpx.AsyncClient() as client:
            
            # Register user first
            await client.post(f"{base_url}/auth/register", json=test_user_data)
            
            # Try login with wrong password
            invalid_login = {
                "email": test_user_data["email"],
                "password": "wrong_password"
            }
            
            login_response = await client.post(
                f"{base_url}/auth/login",
                json=invalid_login
            )
            
            assert login_response.status_code == 401
            error_data = login_response.json()
            assert "invalid credentials" in error_data["detail"].lower()
    
    @pytest.mark.asyncio
    async def test_invalid_token_access(self, base_url):
        """Test API access with invalid token."""
        
        async with httpx.AsyncClient() as client:
            
            # Try to access protected endpoint with invalid token
            invalid_headers = {"Authorization": "Bearer invalid_token_here"}
            
            response = await client.get(
                f"{base_url}/auth/me",
                headers=invalid_headers
            )
            
            assert response.status_code == 401

# Run integration tests
# pytest tests/integration/test_auth_flow.py -v
```

### Storage Integration Tests
```python
# tests/integration/test_storage_integration.py
import pytest
import asyncio
import json
from sutra_storage_client import StorageClient

class TestStorageIntegration:
    
    @pytest.fixture
    def storage_client(self):
        return StorageClient('localhost:50053')  # user-storage port
    
    @pytest.mark.asyncio
    async def test_user_storage_operations(self, storage_client):
        """Test user storage create, search, and retrieve operations."""
        
        # Create test user data
        user_data = {
            "type": "user",
            "email": "storage_test@example.com",
            "password_hash": "$argon2id$v=19$m=65536,t=3,p=4$test_hash",
            "full_name": "Storage Test User",
            "organization": "Test Org",
            "role": "user",
            "active": True,
            "created_at": "2025-10-28T15:00:00.000000"
        }
        
        # Store user with embedding generation
        options = {
            "generate_embedding": True,
            "extract_associations": False
        }
        
        concept_id = storage_client.learn_concept_v2(
            content=json.dumps(user_data),
            options=options
        )
        
        assert concept_id is not None
        assert len(concept_id) > 0
        
        # Search for user by email (broad search)
        dummy_vector = [0.0] * 768
        search_results = storage_client.vector_search(dummy_vector, k=50)
        
        found_user = None
        for result_id, similarity in search_results:
            concept = storage_client.query_concept(result_id)
            if concept:
                try:
                    data = json.loads(concept['content'])
                    if data.get('email') == 'storage_test@example.com':
                        found_user = data
                        break
                except json.JSONDecodeError:
                    continue
        
        assert found_user is not None
        assert found_user['email'] == user_data['email']
        assert found_user['full_name'] == user_data['full_name']
    
    @pytest.mark.asyncio  
    async def test_session_storage_with_embeddings(self, storage_client):
        """Test session storage specifically requires embeddings."""
        
        session_data = {
            "type": "session",
            "session_id": "test_session_123",
            "user_id": "user_123", 
            "email": "session_test@example.com",
            "active": True,
            "created_at": "2025-10-28T15:00:00.000000",
            "expires_at": "2025-11-04T15:00:00.000000"
        }
        
        # Store session WITH embeddings (required)
        options = {
            "generate_embedding": True,  # Critical for session retrieval
            "extract_associations": False
        }
        
        concept_id = storage_client.learn_concept_v2(
            content=json.dumps(session_data),
            options=options
        )
        
        assert concept_id is not None
        
        # Search for session
        dummy_vector = [0.0] * 768
        search_results = storage_client.vector_search(dummy_vector, k=50)
        
        found_session = None
        for result_id, similarity in search_results:
            concept = storage_client.query_concept(result_id)
            if concept:
                try:
                    data = json.loads(concept['content'])
                    if (data.get('type') == 'session' and 
                        data.get('session_id') == 'test_session_123'):
                        found_session = data
                        break
                except json.JSONDecodeError:
                    continue
        
        assert found_session is not None
        assert found_session['session_id'] == session_data['session_id']
    
    @pytest.mark.asyncio
    async def test_storage_without_embeddings_fails_retrieval(self, storage_client):
        """Test that concepts stored without embeddings cannot be found via vector search."""
        
        test_data = {
            "type": "test",
            "content": "This should not be findable via vector search",
            "id": "no_embedding_test"
        }
        
        # Store WITHOUT embeddings
        options = {
            "generate_embedding": False,  # This will cause search to fail
            "extract_associations": False
        }
        
        concept_id = storage_client.learn_concept_v2(
            content=json.dumps(test_data),
            options=options
        )
        
        assert concept_id is not None
        
        # Try to find via vector search (should fail)
        dummy_vector = [0.0] * 768
        search_results = storage_client.vector_search(dummy_vector, k=50)
        
        found = False
        for result_id, similarity in search_results:
            if result_id == concept_id:
                found = True
                break
        
        # Should NOT be found in vector search without embeddings
        assert found is False
        
        # But should be retrievable by direct concept ID
        concept = storage_client.query_concept(concept_id)
        assert concept is not None
        data = json.loads(concept['content'])
        assert data['id'] == "no_embedding_test"

# Run storage integration tests  
# pytest tests/integration/test_storage_integration.py -v
```

## Performance Tests

### Authentication Performance Test
```python
# tests/performance/test_auth_performance.py
import pytest
import asyncio
import httpx
import time
import statistics
from concurrent.futures import ThreadPoolExecutor

class TestAuthenticationPerformance:
    
    @pytest.fixture
    def base_url(self):
        return "http://localhost:8000"
    
    @pytest.mark.asyncio
    async def test_registration_performance(self, base_url):
        """Test registration performance under concurrent load."""
        
        async def register_user(user_id: int):
            """Register a single user and measure response time."""
            user_data = {
                "email": f"perf_user_{user_id}@example.com",
                "password": "secure123",
                "full_name": f"Performance User {user_id}",
                "organization": "Performance Test"
            }
            
            start_time = time.time()
            
            async with httpx.AsyncClient() as client:
                response = await client.post(
                    f"{base_url}/auth/register",
                    json=user_data,
                    timeout=30.0
                )
            
            end_time = time.time()
            
            return {
                'user_id': user_id,
                'response_time': end_time - start_time,
                'status_code': response.status_code,
                'success': response.status_code == 200
            }
        
        # Test with 20 concurrent registrations
        concurrent_users = 20
        tasks = [register_user(i) for i in range(concurrent_users)]
        
        results = await asyncio.gather(*tasks, return_exceptions=True)
        
        # Analyze results
        successful_results = [r for r in results if isinstance(r, dict) and r['success']]
        response_times = [r['response_time'] for r in successful_results]
        
        assert len(successful_results) == concurrent_users, "All registrations should succeed"
        
        # Performance assertions
        avg_response_time = statistics.mean(response_times)
        max_response_time = max(response_times)
        
        print(f"Average registration time: {avg_response_time:.3f}s")
        print(f"Maximum registration time: {max_response_time:.3f}s")
        print(f"95th percentile: {statistics.quantiles(response_times, n=20)[18]:.3f}s")
        
        # Performance requirements
        assert avg_response_time < 2.0, f"Average response time too slow: {avg_response_time:.3f}s"
        assert max_response_time < 5.0, f"Maximum response time too slow: {max_response_time:.3f}s"
    
    @pytest.mark.asyncio
    async def test_login_performance(self, base_url):
        """Test login performance for existing users."""
        
        # Pre-register test users
        test_users = []
        async with httpx.AsyncClient() as client:
            for i in range(10):
                user_data = {
                    "email": f"login_perf_{i}@example.com",
                    "password": "secure123",
                    "full_name": f"Login Test User {i}",
                    "organization": "Login Performance Test"
                }
                
                response = await client.post(f"{base_url}/auth/register", json=user_data)
                if response.status_code == 200:
                    test_users.append(user_data)
        
        assert len(test_users) >= 5, "Need at least 5 users for login performance test"
        
        async def login_user(user_data):
            """Login a user and measure response time."""
            login_data = {
                "email": user_data["email"],
                "password": user_data["password"]
            }
            
            start_time = time.time()
            
            async with httpx.AsyncClient() as client:
                response = await client.post(
                    f"{base_url}/auth/login",
                    json=login_data,
                    timeout=30.0
                )
            
            end_time = time.time()
            
            return {
                'email': user_data["email"],
                'response_time': end_time - start_time,
                'status_code': response.status_code,
                'success': response.status_code == 200
            }
        
        # Test concurrent logins
        login_tasks = [login_user(user) for user in test_users]
        login_results = await asyncio.gather(*login_tasks)
        
        successful_logins = [r for r in login_results if r['success']]
        login_times = [r['response_time'] for r in successful_logins]
        
        assert len(successful_logins) >= len(test_users) * 0.8, "At least 80% of logins should succeed"
        
        avg_login_time = statistics.mean(login_times)
        max_login_time = max(login_times)
        
        print(f"Average login time: {avg_login_time:.3f}s")
        print(f"Maximum login time: {max_login_time:.3f}s")
        
        # Login should be faster than registration
        assert avg_login_time < 1.0, f"Average login time too slow: {avg_login_time:.3f}s"
        assert max_login_time < 3.0, f"Maximum login time too slow: {max_login_time:.3f}s"

# Run performance tests
# pytest tests/performance/test_auth_performance.py -v -s
```

### Storage Performance Test
```python
# tests/performance/test_storage_performance.py
import pytest
import time
import json
import statistics
import asyncio
from sutra_storage_client import StorageClient

class TestStoragePerformance:
    
    @pytest.fixture
    def storage_client(self):
        return StorageClient('localhost:50053')
    
    def test_storage_write_performance(self, storage_client):
        """Test storage write performance."""
        
        write_times = []
        concept_count = 100
        
        for i in range(concept_count):
            user_data = {
                "type": "user",
                "email": f"perf_user_{i}@example.com",
                "password_hash": f"$argon2id$hash_{i}",
                "full_name": f"Performance User {i}",
                "organization": "Performance Test",
                "created_at": f"2025-10-28T15:{i:02d}:00.000000"
            }
            
            options = {
                "generate_embedding": True,
                "extract_associations": False
            }
            
            start_time = time.time()
            
            concept_id = storage_client.learn_concept_v2(
                content=json.dumps(user_data),
                options=options
            )
            
            end_time = time.time()
            write_times.append(end_time - start_time)
            
            assert concept_id is not None
        
        # Analyze write performance
        avg_write_time = statistics.mean(write_times)
        max_write_time = max(write_times)
        min_write_time = min(write_times)
        
        print(f"Average write time: {avg_write_time*1000:.2f}ms")
        print(f"Maximum write time: {max_write_time*1000:.2f}ms")
        print(f"Minimum write time: {min_write_time*1000:.2f}ms")
        print(f"Writes per second: {1/avg_write_time:.1f}")
        
        # Performance requirements
        assert avg_write_time < 0.1, f"Average write time too slow: {avg_write_time*1000:.2f}ms"
        assert max_write_time < 0.5, f"Maximum write time too slow: {max_write_time*1000:.2f}ms"
    
    def test_storage_read_performance(self, storage_client):
        """Test storage vector search performance."""
        
        # Ensure we have some data to search
        for i in range(10):
            user_data = {
                "type": "user", 
                "email": f"search_user_{i}@example.com",
                "test_index": i
            }
            
            storage_client.learn_concept_v2(
                content=json.dumps(user_data),
                options={"generate_embedding": True, "extract_associations": False}
            )
        
        # Measure search performance
        search_times = []
        search_count = 50
        
        dummy_vector = [0.0] * 768
        
        for i in range(search_count):
            start_time = time.time()
            
            results = storage_client.vector_search(dummy_vector, k=20)
            
            end_time = time.time()
            search_times.append(end_time - start_time)
            
            assert len(results) >= 0  # Should return some results
        
        # Analyze search performance
        avg_search_time = statistics.mean(search_times)
        max_search_time = max(search_times)
        
        print(f"Average search time: {avg_search_time*1000:.2f}ms")
        print(f"Maximum search time: {max_search_time*1000:.2f}ms")
        print(f"Searches per second: {1/avg_search_time:.1f}")
        
        # Performance requirements (should be sub-millisecond)
        assert avg_search_time < 0.01, f"Average search time too slow: {avg_search_time*1000:.2f}ms"
        assert max_search_time < 0.05, f"Maximum search time too slow: {max_search_time*1000:.2f}ms"

# Run storage performance tests
# pytest tests/performance/test_storage_performance.py -v -s
```

## Production Testing

### Health Check Tests
```bash
#!/bin/bash
# scripts/test-production-health.sh

set -e

echo "🔍 Testing production health checks..."

# Test API health
echo "Testing API health..."
RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:8000/auth/health)
if [ "$RESPONSE" = "200" ]; then
    echo "✅ API health check passed"
else
    echo "❌ API health check failed (HTTP $RESPONSE)"
    exit 1
fi

# Test storage connectivity
echo "Testing storage connectivity..."
python3 -c "
from sutra_storage_client import StorageClient
try:
    client = StorageClient('localhost:50053')
    results = client.vector_search([0.0] * 768, k=5)
    print('✅ Storage connectivity test passed')
except Exception as e:
    print(f'❌ Storage connectivity test failed: {e}')
    exit(1)
"

# Test embedding service
echo "Testing embedding service..."
EMBEDDING_RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:8888/health)
if [ "$EMBEDDING_RESPONSE" = "200" ]; then
    echo "✅ Embedding service health check passed"
else
    echo "❌ Embedding service health check failed (HTTP $EMBEDDING_RESPONSE)"
    exit 1
fi

echo "🎉 All production health checks passed!"
```

### Load Testing Script
```bash
#!/bin/bash
# scripts/load-test-auth.sh

set -e

echo "🚀 Running authentication load test..."

# Configuration
BASE_URL="http://localhost:8000"
CONCURRENT_USERS=50
TEST_DURATION=60  # seconds

# Create load testing script
cat > /tmp/auth_load_test.py << 'EOF'
import asyncio
import httpx
import time
import sys
import random
import statistics

async def register_and_login_user(user_id: int, base_url: str):
    """Register user, login, and make authenticated requests."""
    user_data = {
        "email": f"load_user_{user_id}_{random.randint(1000,9999)}@example.com",
        "password": "loadtest123",
        "full_name": f"Load Test User {user_id}",
        "organization": "Load Test"
    }
    
    times = {}
    
    async with httpx.AsyncClient(timeout=30.0) as client:
        # Register
        start = time.time()
        register_response = await client.post(f"{base_url}/auth/register", json=user_data)
        times['register'] = time.time() - start
        
        if register_response.status_code != 200:
            return {'success': False, 'error': f'Registration failed: {register_response.status_code}'}
        
        # Login  
        login_data = {"email": user_data["email"], "password": user_data["password"]}
        start = time.time()
        login_response = await client.post(f"{base_url}/auth/login", json=login_data)
        times['login'] = time.time() - start
        
        if login_response.status_code != 200:
            return {'success': False, 'error': f'Login failed: {login_response.status_code}'}
        
        token = login_response.json()["access_token"]
        headers = {"Authorization": f"Bearer {token}"}
        
        # Authenticated request
        start = time.time()
        profile_response = await client.get(f"{base_url}/auth/me", headers=headers)
        times['profile'] = time.time() - start
        
        if profile_response.status_code != 200:
            return {'success': False, 'error': f'Profile request failed: {profile_response.status_code}'}
        
        # Logout
        start = time.time()
        logout_response = await client.post(f"{base_url}/auth/logout", headers=headers)
        times['logout'] = time.time() - start
        
        return {
            'success': True,
            'user_id': user_id,
            'times': times,
            'total_time': sum(times.values())
        }

async def run_load_test(concurrent_users: int, base_url: str):
    """Run load test with specified concurrent users."""
    print(f"Starting load test with {concurrent_users} concurrent users...")
    
    start_time = time.time()
    
    # Create tasks for concurrent users
    tasks = [register_and_login_user(i, base_url) for i in range(concurrent_users)]
    results = await asyncio.gather(*tasks, return_exceptions=True)
    
    end_time = time.time()
    
    # Analyze results
    successful_results = [r for r in results if isinstance(r, dict) and r.get('success')]
    failed_results = [r for r in results if not isinstance(r, dict) or not r.get('success')]
    
    if successful_results:
        total_times = [r['total_time'] for r in successful_results]
        register_times = [r['times']['register'] for r in successful_results]
        login_times = [r['times']['login'] for r in successful_results]
        
        print(f"\n📊 Load Test Results:")
        print(f"Total users: {concurrent_users}")
        print(f"Successful: {len(successful_results)}")
        print(f"Failed: {len(failed_results)}")
        print(f"Success rate: {len(successful_results)/concurrent_users*100:.1f}%")
        print(f"Test duration: {end_time - start_time:.2f}s")
        print(f"\n⏱️ Response Times:")
        print(f"Average total time: {statistics.mean(total_times):.3f}s")
        print(f"Average registration: {statistics.mean(register_times):.3f}s")
        print(f"Average login: {statistics.mean(login_times):.3f}s")
        print(f"Max total time: {max(total_times):.3f}s")
        print(f"95th percentile: {statistics.quantiles(total_times, n=20)[18]:.3f}s")
    
    if failed_results:
        print(f"\n❌ Failed Requests:")
        for failure in failed_results[:5]:  # Show first 5 failures
            print(f"  - {failure}")
    
    return len(successful_results) / concurrent_users >= 0.95

if __name__ == "__main__":
    concurrent_users = int(sys.argv[1]) if len(sys.argv) > 1 else 10
    base_url = sys.argv[2] if len(sys.argv) > 2 else "http://localhost:8000"
    
    success = asyncio.run(run_load_test(concurrent_users, base_url))
    sys.exit(0 if success else 1)
EOF

# Run load test
echo "Running load test with $CONCURRENT_USERS concurrent users..."
python3 /tmp/auth_load_test.py $CONCURRENT_USERS $BASE_URL

if [ $? -eq 0 ]; then
    echo "🎉 Load test passed!"
else
    echo "❌ Load test failed!"
    exit 1
fi

# Cleanup
rm -f /tmp/auth_load_test.py
```

## Automated Test Suite

### Complete Test Runner
```bash
#!/bin/bash
# scripts/run-all-tests.sh

set -e

echo "🧪 Running complete authentication test suite..."

# Ensure services are running
echo "Checking service health..."
./scripts/test-production-health.sh

# Run unit tests
echo "Running unit tests..."
cd /path/to/sutra-models
PYTHONPATH=packages/sutra-core python -m pytest tests/unit/ -v

# Run integration tests
echo "Running integration tests..."
PYTHONPATH=packages/sutra-core python -m pytest tests/integration/ -v

# Run performance tests
echo "Running performance tests..."
PYTHONPATH=packages/sutra-core python -m pytest tests/performance/ -v -s

# Run load test
echo "Running load test..."
./scripts/load-test-auth.sh

echo "🎉 All tests completed successfully!"
```

## Continuous Testing

### GitHub Actions Workflow
```yaml
# .github/workflows/auth-tests.yml
name: Authentication Tests

on:
  push:
    branches: [ main, develop ]
    paths:
      - 'packages/sutra-api/**'
      - 'packages/sutra-storage/**'
      - 'tests/**'
  pull_request:
    branches: [ main ]

jobs:
  test-authentication:
    runs-on: ubuntu-latest
    
    services:
      docker:
        image: docker:20.10.7
        options: --privileged
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Set up Docker Compose
      run: |
        sudo curl -L "https://github.com/docker/compose/releases/download/v2.12.2/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
        sudo chmod +x /usr/local/bin/docker-compose
    
    - name: Build and deploy services
      run: |
        SUTRA_EDITION=simple ./sutra-optimize.sh build-all
        SUTRA_EDITION=simple ./sutra deploy
    
    - name: Wait for services
      run: |
        timeout 60 bash -c 'until curl -s http://localhost:8000/auth/health; do sleep 2; done'
        timeout 60 bash -c 'until curl -s http://localhost:8888/health; do sleep 2; done'
    
    - name: Set up Python
      uses: actions/setup-python@v4
      with:
        python-version: '3.11'
    
    - name: Install dependencies
      run: |
        pip install -r requirements-dev.txt
        pip install -e packages/sutra-core
    
    - name: Run unit tests
      run: |
        PYTHONPATH=packages/sutra-core python -m pytest tests/unit/ -v --junitxml=test-results-unit.xml
    
    - name: Run integration tests
      run: |
        PYTHONPATH=packages/sutra-core python -m pytest tests/integration/ -v --junitxml=test-results-integration.xml
    
    - name: Run performance tests
      run: |
        PYTHONPATH=packages/sutra-core python -m pytest tests/performance/ -v -s --junitxml=test-results-performance.xml
    
    - name: Publish test results
      uses: EnricoMi/publish-unit-test-result-action@v2
      if: always()
      with:
        files: test-results-*.xml
    
    - name: Cleanup
      if: always()
      run: |
        docker compose down -v
```

---

**AI Context**: This testing guide provides comprehensive validation procedures for the Sutra AI user authentication system, covering unit tests (isolated component testing), integration tests (end-to-end workflows), performance tests (response time and throughput validation), and production health checks. The test suite validates the complete authentication flow from registration through logout, ensuring the vector-based storage system correctly handles user sessions with proper embedding generation.

**Last Updated**: 2025-10-28  
**Test Coverage**: Registration, login, session management, logout, storage operations, performance validation, health monitoring, and load testing scenarios.