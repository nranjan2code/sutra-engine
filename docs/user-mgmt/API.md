# API Reference

**Complete API documentation for Sutra AI user management endpoints**

## Base Configuration

### Endpoints
- **Base URL**: `http://localhost:8000` (development)
- **API Prefix**: `/auth`
- **Content-Type**: `application/json`
- **Authentication**: Bearer token (JWT)

### Response Format
All responses follow standard JSON format with appropriate HTTP status codes.

```json
{
  "data": {},           // Success response data
  "error": "message"    // Error message (on failure)
}
```

## Authentication Endpoints

### 1. User Registration

**Endpoint**: `POST /auth/register`  
**Description**: Create a new user account  
**Authentication**: Not required

#### Request Body
```json
{
  "email": "user@example.com",
  "password": "securePassword123",
  "full_name": "John Doe",
  "organization": "Acme Corporation"
}
```

#### Request Schema
- `email` (string, required): Valid email address, will be normalized to lowercase
- `password` (string, required): Minimum 8 characters
- `full_name` (string, required): User's full name, minimum 2 characters  
- `organization` (string, required): Organization name, minimum 2 characters

#### Success Response (201)
```json
{
  "user_id": "3192ed168daf854e",
  "email": "user@example.com",
  "organization": "Acme Corporation",
  "role": "user",
  "full_name": "John Doe", 
  "created_at": "2025-10-28T14:44:05.485569",
  "last_login": null
}
```

#### Error Responses
- **400 Bad Request**: Invalid input data
- **409 Conflict**: User already exists
- **500 Internal Server Error**: Registration failed

```json
{
  "detail": "User already exists"
}
```

#### cURL Example
```bash
curl -X POST http://localhost:8000/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "john.doe@example.com",
    "password": "securePass123",
    "full_name": "John Doe",
    "organization": "Acme Corp"
  }'
```

### 2. User Login

**Endpoint**: `POST /auth/login`  
**Description**: Authenticate user and create session  
**Authentication**: Not required

#### Request Body
```json
{
  "email": "user@example.com", 
  "password": "securePassword123"
}
```

#### Request Schema
- `email` (string, required): User's email address
- `password` (string, required): User's password

#### Success Response (200)
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...", 
  "token_type": "bearer",
  "expires_in": 86400,
  "user": {
    "user_id": "3192ed168daf854e0000000000000000",
    "email": "user@example.com",
    "organization": "Acme Corporation",
    "role": "user",
    "full_name": "John Doe",
    "created_at": "2025-10-28T14:44:05.485569",
    "last_login": null
  }
}
```

#### Response Fields
- `access_token`: JWT token for API authentication (24 hour TTL)
- `refresh_token`: JWT token for token renewal (7 day TTL)
- `token_type`: Always "bearer"
- `expires_in`: Access token expiration in seconds
- `user`: Complete user profile data

#### Error Responses
- **401 Unauthorized**: Invalid credentials
- **500 Internal Server Error**: Login failed

```json
{
  "detail": "Invalid credentials"
}
```

#### cURL Example
```bash
curl -X POST http://localhost:8000/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "john.doe@example.com", 
    "password": "securePass123"
  }'
```

### 3. User Logout

**Endpoint**: `POST /auth/logout`  
**Description**: Invalidate current session  
**Authentication**: Bearer token required

#### Request Headers
```
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

#### Request Body
Empty body `{}`

#### Success Response (200)
```json
{
  "message": "Logged out successfully"
}
```

#### Error Responses
- **401 Unauthorized**: Invalid or expired token
- **400 Bad Request**: Invalid token format
- **500 Internal Server Error**: Logout failed

#### cURL Example
```bash
# Extract token from login response
TOKEN="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

curl -X POST http://localhost:8000/auth/logout \
  -H "Authorization: Bearer $TOKEN"
```

## Protected Endpoints

### Authentication Middleware
All protected endpoints require a valid JWT token in the Authorization header.

#### Header Format
```
Authorization: Bearer <access_token>
```

#### Authentication Flow
1. Extract Bearer token from Authorization header
2. Validate JWT signature and expiration
3. Extract session ID from token payload
4. Validate session exists and is active in storage
5. Inject user context into request

### Example Protected Route
```python
from fastapi import Depends, HTTPException
from fastapi.security import HTTPBearer

security = HTTPBearer()

@router.get("/profile")
async def get_user_profile(
    current_user: dict = Depends(get_current_user)
) -> dict:
    return {
        "email": current_user["email"],
        "organization": current_user["organization"],
        "role": current_user["role"]
    }
```

## JWT Token Structure

### Access Token Payload
```json
{
  "user_id": "3192ed168daf854e0000000000000000",
  "session_id": "V2oZYJH7_M3PcK2FKm721g",
  "email": "user@example.com",
  "organization": "Acme Corporation", 
  "role": "user",
  "exp": 1761747779,
  "iat": 1761661379
}
```

### Token Claims
- `user_id`: Unique user identifier from storage
- `session_id`: Session identifier for validation
- `email`: User's email address
- `organization`: User's organization
- `role`: User role for authorization
- `exp`: Token expiration timestamp
- `iat`: Token issued at timestamp

### Token Validation
```python
import jwt

def validate_token(token: str) -> dict:
    try:
        payload = jwt.decode(
            token, 
            JWT_SECRET_KEY, 
            algorithms=["HS256"]
        )
        return payload
    except jwt.ExpiredSignatureError:
        raise ValueError("Token has expired")
    except jwt.InvalidTokenError:
        raise ValueError("Invalid token")
```

## Error Handling

### Standard Error Codes
- **400 Bad Request**: Invalid request format or data
- **401 Unauthorized**: Authentication required or failed  
- **403 Forbidden**: Access denied (insufficient permissions)
- **404 Not Found**: Resource not found
- **409 Conflict**: Resource already exists
- **422 Unprocessable Entity**: Validation errors
- **500 Internal Server Error**: Server-side error

### Error Response Format
```json
{
  "detail": "Human-readable error message"
}
```

### Validation Errors (422)
```json
{
  "detail": [
    {
      "loc": ["body", "email"],
      "msg": "field required",
      "type": "value_error.missing"
    },
    {
      "loc": ["body", "password"], 
      "msg": "ensure this value has at least 8 characters",
      "type": "value_error.any_str.min_length"
    }
  ]
}
```

## Rate Limiting

### Current Implementation
No rate limiting currently implemented.

### Recommended Implementation
```python
from slowapi import Limiter, _rate_limit_exceeded_handler
from slowapi.util import get_remote_address
from slowapi.errors import RateLimitExceeded

limiter = Limiter(key_func=get_remote_address)

@app.state.limiter = limiter
@app.add_exception_handler(RateLimitExceeded, _rate_limit_exceeded_handler)

@router.post("/login")
@limiter.limit("5/minute")  # 5 attempts per minute per IP
async def login(request: Request, ...):
    pass

@router.post("/register") 
@limiter.limit("3/minute")  # 3 registrations per minute per IP
async def register(request: Request, ...):
    pass
```

## SDK Examples

### Python SDK Usage
```python
import requests
import json

class SutraAuthClient:
    def __init__(self, base_url="http://localhost:8000"):
        self.base_url = base_url
        self.access_token = None
        
    def register(self, email, password, full_name, organization):
        response = requests.post(
            f"{self.base_url}/auth/register",
            json={
                "email": email,
                "password": password,
                "full_name": full_name,
                "organization": organization
            }
        )
        response.raise_for_status()
        return response.json()
    
    def login(self, email, password):
        response = requests.post(
            f"{self.base_url}/auth/login",
            json={
                "email": email,
                "password": password
            }
        )
        response.raise_for_status()
        data = response.json()
        self.access_token = data["access_token"]
        return data
    
    def logout(self):
        if not self.access_token:
            return
            
        response = requests.post(
            f"{self.base_url}/auth/logout",
            headers={"Authorization": f"Bearer {self.access_token}"}
        )
        response.raise_for_status()
        self.access_token = None
        return response.json()
    
    def get_headers(self):
        if self.access_token:
            return {"Authorization": f"Bearer {self.access_token}"}
        return {}

# Usage example
client = SutraAuthClient()

# Register
user_data = client.register(
    "test@example.com", 
    "password123",
    "Test User", 
    "Test Org"
)

# Login  
auth_data = client.login("test@example.com", "password123")
print(f"Logged in as: {auth_data['user']['email']}")

# Make authenticated request
response = requests.get(
    "http://localhost:8000/protected-resource",
    headers=client.get_headers()
)

# Logout
client.logout()
```

### JavaScript/React SDK Usage
```javascript
class SutraAuthClient {
  constructor(baseURL = 'http://localhost:8000') {
    this.baseURL = baseURL;
    this.accessToken = localStorage.getItem('access_token');
  }

  async register(userData) {
    const response = await fetch(`${this.baseURL}/auth/register`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(userData),
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.detail);
    }

    return response.json();
  }

  async login(email, password) {
    const response = await fetch(`${this.baseURL}/auth/login`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ email, password }),
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.detail);
    }

    const data = await response.json();
    this.accessToken = data.access_token;
    localStorage.setItem('access_token', data.access_token);
    localStorage.setItem('refresh_token', data.refresh_token);

    return data;
  }

  async logout() {
    try {
      if (this.accessToken) {
        await fetch(`${this.baseURL}/auth/logout`, {
          method: 'POST',
          headers: {
            'Authorization': `Bearer ${this.accessToken}`,
          },
        });
      }
    } catch (error) {
      console.error('Logout API call failed:', error);
    } finally {
      this.accessToken = null;
      localStorage.removeItem('access_token');
      localStorage.removeItem('refresh_token');
    }
  }

  getAuthHeaders() {
    return this.accessToken 
      ? { 'Authorization': `Bearer ${this.accessToken}` }
      : {};
  }

  isAuthenticated() {
    return !!this.accessToken;
  }
}

// Usage example
const authClient = new SutraAuthClient();

// Register
try {
  const user = await authClient.register({
    email: 'test@example.com',
    password: 'password123',
    full_name: 'Test User',
    organization: 'Test Org'
  });
  console.log('User registered:', user.email);
} catch (error) {
  console.error('Registration failed:', error.message);
}

// Login
try {
  const authData = await authClient.login('test@example.com', 'password123');
  console.log('Logged in as:', authData.user.email);
} catch (error) {
  console.error('Login failed:', error.message);
}

// Make authenticated request
if (authClient.isAuthenticated()) {
  const response = await fetch('/api/protected-resource', {
    headers: authClient.getAuthHeaders(),
  });
}

// Logout
await authClient.logout();
```

## Testing & Development

### Health Check Endpoints
```bash
# Check API service health
curl http://localhost:8000/health

# Check embedding service health  
curl http://localhost:8888/health

# Check user storage health
docker logs sutra-user-storage --tail 5
```

### Development Environment Setup
```bash
# Start all services
SUTRA_EDITION=simple ./sutra deploy

# Check service status
./sutra status

# View logs
docker logs sutra-api --tail 20
docker logs sutra-user-storage --tail 20
```

### API Testing Scripts
```bash
#!/bin/bash
# test-auth-flow.sh

BASE_URL="http://localhost:8000"
EMAIL="test-$(date +%s)@example.com"
PASSWORD="password123"

echo "Testing complete authentication flow..."

# 1. Register
echo "1. Registering user..."
REGISTER_RESPONSE=$(curl -s -X POST $BASE_URL/auth/register \
  -H "Content-Type: application/json" \
  -d "{
    \"email\": \"$EMAIL\",
    \"password\": \"$PASSWORD\",
    \"full_name\": \"Test User\",
    \"organization\": \"Test Org\"
  }")

echo "Registration response: $REGISTER_RESPONSE"

# 2. Login
echo "2. Logging in..."
LOGIN_RESPONSE=$(curl -s -X POST $BASE_URL/auth/login \
  -H "Content-Type: application/json" \
  -d "{
    \"email\": \"$EMAIL\",
    \"password\": \"$PASSWORD\"
  }")

TOKEN=$(echo $LOGIN_RESPONSE | jq -r '.access_token')
echo "Login successful, token: ${TOKEN:0:50}..."

# 3. Test protected endpoint
echo "3. Testing protected endpoint..."
PROTECTED_RESPONSE=$(curl -s -X GET $BASE_URL/protected-resource \
  -H "Authorization: Bearer $TOKEN")

echo "Protected response: $PROTECTED_RESPONSE"

# 4. Logout
echo "4. Logging out..."
LOGOUT_RESPONSE=$(curl -s -X POST $BASE_URL/auth/logout \
  -H "Authorization: Bearer $TOKEN")

echo "Logout response: $LOGOUT_RESPONSE"

echo "Authentication flow test complete!"
```

---

**AI Context**: This API reference provides complete documentation for the Sutra AI user management system, including all endpoints, request/response formats, authentication mechanisms, and usage examples. The API uses JWT tokens for stateless authentication with vector-based session storage for high performance.

**Last Updated**: 2025-10-28  
**Version**: 2.0.0  
**Base URL**: http://localhost:8000 (development)