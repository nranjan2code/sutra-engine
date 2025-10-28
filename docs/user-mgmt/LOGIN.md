# User Login Process

**Complete workflow for user authentication and session creation in Sutra AI**

## Overview

The login process authenticates users via vector search, verifies passwords using Argon2, creates secure sessions stored as vector embeddings, and returns JWT tokens for authenticated access.

## Login Workflow

### 1. Frontend Request
```javascript
// Frontend (React) login call
const loginUser = async (credentials) => {
  const response = await axios.post('http://localhost:8000/auth/login', {
    email: 'user@example.com',
    password: 'securePassword123'
  });
  
  // Store tokens for authenticated requests
  localStorage.setItem('access_token', response.data.access_token);
  localStorage.setItem('refresh_token', response.data.refresh_token);
  
  return response.data;
};
```

### 2. API Endpoint Processing
**Endpoint**: `POST /auth/login`  
**Handler**: `sutra_api.routes.auth.login`  
**Service**: `sutra_api.services.user_service.UserService.login()`

```python
async def login(self, email: str, password: str) -> Dict:
```

### 3. User Lookup via Vector Search
```python
# Find user by email using vector similarity search
email = email.lower().strip()
dummy_vector = [0.0] * 768  # Zero vector for broad search

vector_results = self.storage.vector_search(dummy_vector, k=50)
user_data = None
user_concept_id = None

for concept_id, similarity in vector_results:
    concept = self.storage.query_concept(concept_id)
    if concept and concept.get("content"):
        try:
            data = json.loads(concept["content"])
            if data.get("type") == "user" and data.get("email") == email:
                user_data = data
                user_concept_id = concept_id
                break
        except json.JSONDecodeError:
            continue

if not user_data:
    raise ValueError("Invalid credentials")
```

### 4. Password Verification
```python
# Verify password using Argon2
stored_hash = user_data["password_hash"]
if not argon2.verify(password, stored_hash):
    raise ValueError("Invalid credentials")

logger.info(f"Password verified: {email}")
```

**Security Properties**:
- Constant-time comparison prevents timing attacks
- Argon2 verification handles salt extraction automatically
- No plaintext password storage or transmission

### 5. Session Creation
```python
# Generate secure session
session_id = secrets.token_urlsafe(16)  # Cryptographically secure
now = datetime.utcnow()
expires_at = now + timedelta(days=7)

session_data = {
    "type": "session",
    "session_id": session_id,
    "user_id": user_concept_id,
    "email": user_data["email"],
    "organization": user_data["organization"],
    "role": user_data["role"],
    "created_at": now.isoformat(),
    "expires_at": expires_at.isoformat(),
    "active": True,
    "last_activity": now.isoformat(),
}
```

### 6. Session Storage (Critical)
```python
# Store session with embeddings enabled (REQUIRED)
session_concept_id = self.storage.learn_concept_v2(
    content=json.dumps(session_data),
    options={
        "generate_embedding": True,    # CRITICAL: Must be True
        "extract_associations": False, # No associations for sessions
    }
)
```

**Why Embeddings Are Required**:
- Sessions stored without embeddings are not retrievable via vector search
- User storage server uses vector-only architecture  
- Embedding generation takes ~500ms but ensures session persistence

### 7. JWT Token Generation
```python
# Generate access token (short-lived)
access_payload = {
    "user_id": user_concept_id,
    "session_id": session_id,
    "email": user_data["email"],
    "organization": user_data["organization"],
    "role": user_data["role"],
    "exp": datetime.utcnow() + timedelta(days=1),  # 24 hours
    "iat": datetime.utcnow()
}
access_token = jwt.encode(access_payload, JWT_SECRET_KEY, algorithm="HS256")

# Generate refresh token (long-lived)  
refresh_payload = {
    "user_id": user_concept_id,
    "session_id": session_id,
    "email": user_data["email"],
    "organization": user_data["organization"],
    "role": user_data["role"],
    "exp": datetime.utcnow() + timedelta(days=7),  # 7 days
    "iat": datetime.utcnow()
}
refresh_token = jwt.encode(refresh_payload, JWT_SECRET_KEY, algorithm="HS256")
```

### 8. Response Generation
```python
return {
    "access_token": access_token,
    "refresh_token": refresh_token,
    "token_type": "bearer",
    "expires_in": 86400,  # 24 hours in seconds
    "user": {
        "user_id": user_concept_id,
        "email": user_data["email"],
        "organization": user_data["organization"],
        "role": user_data["role"], 
        "full_name": user_data.get("full_name"),
        "created_at": user_data.get("created_at"),
        "last_login": user_data.get("last_login")
    }
}
```

## Complete Code Implementation

### API Route Handler
```python
@router.post("/login", response_model=dict)
async def login(
    credentials: UserLoginRequest,
    user_service: UserService = Depends(get_user_service)
) -> dict:
    try:
        result = await user_service.login(
            email=credentials.email,
            password=credentials.password
        )
        return result
    except ValueError as e:
        raise HTTPException(status_code=401, detail="Invalid credentials")
    except Exception as e:
        logger.error(f"Login failed: {e}")
        raise HTTPException(status_code=500, detail="Login failed")
```

### UserService.login() Method
```python
async def login(self, email: str, password: str) -> Dict:
    """
    Authenticate user and create session via vector search.
    
    Returns JWT tokens and user data.
    """
    try:
        # 1. Normalize email
        email = email.lower().strip()
        logger.info(f"Vector search login: {email}")
        
        # 2. Find user via vector search
        dummy_vector = [0.0] * 768
        vector_results = self.storage.vector_search(dummy_vector, k=50)
        
        user_data = None
        user_concept_id = None
        
        logger.info(f"Searching {len(vector_results)} concepts for user")
        for concept_id, similarity in vector_results:
            concept = self.storage.query_concept(concept_id)
            if concept and concept.get("content"):
                try:
                    data = json.loads(concept["content"])
                    if data.get("type") == "user" and data.get("email") == email:
                        user_data = data
                        user_concept_id = concept_id
                        logger.info(f"User found: {email}")
                        break
                except json.JSONDecodeError:
                    continue
        
        if not user_data:
            logger.warning(f"User not found: {email}")
            raise ValueError("Invalid credentials")
        
        # 3. Verify password with Argon2
        stored_hash = user_data["password_hash"]
        if not argon2.verify(password, stored_hash):
            logger.warning(f"Invalid password: {email}")
            raise ValueError("Invalid credentials")
        
        logger.info(f"Password verified: {email}")
        
        # 4. Create session data
        session_id = secrets.token_urlsafe(16)
        now = datetime.utcnow()
        expires_at = now + timedelta(days=7)
        
        session_data = {
            "type": "session",
            "session_id": session_id,
            "user_id": user_concept_id,
            "email": user_data["email"],
            "organization": user_data["organization"],
            "role": user_data["role"],
            "created_at": now.isoformat(),
            "expires_at": expires_at.isoformat(),
            "active": True,
            "last_activity": now.isoformat(),
        }
        
        # 5. Store session with embeddings (CRITICAL)
        session_concept_id = self.storage.learn_concept_v2(
            content=json.dumps(session_data),
            options={
                "generate_embedding": True,  # Sessions NEED embeddings for storage
                "extract_associations": False,
            }
        )
        
        logger.info(f"✅ Session created: {email} -> {session_id[:8]}")
        
        # 6. Generate JWT tokens
        access_payload = {
            "user_id": user_concept_id,
            "session_id": session_id,
            "email": user_data["email"],
            "organization": user_data["organization"],
            "role": user_data["role"],
            "exp": datetime.utcnow() + timedelta(days=1),
            "iat": datetime.utcnow()
        }
        
        refresh_payload = {
            "user_id": user_concept_id,
            "session_id": session_id,
            "email": user_data["email"],
            "organization": user_data["organization"], 
            "role": user_data["role"],
            "exp": datetime.utcnow() + timedelta(days=7),
            "iat": datetime.utcnow()
        }
        
        access_token = jwt.encode(access_payload, JWT_SECRET_KEY, algorithm="HS256")
        refresh_token = jwt.encode(refresh_payload, JWT_SECRET_KEY, algorithm="HS256")
        
        # 7. Return authentication data
        return {
            "access_token": access_token,
            "refresh_token": refresh_token,
            "token_type": "bearer",
            "expires_in": 86400,
            "user": {
                "user_id": user_concept_id,
                "email": user_data["email"],
                "organization": user_data["organization"],
                "role": user_data["role"],
                "full_name": user_data.get("full_name"),
                "created_at": user_data.get("created_at"),
                "last_login": user_data.get("last_login")
            }
        }
        
    except ValueError:
        raise  # Re-raise credential errors
    except Exception as e:
        logger.error(f"Login failed: {email}: {e}")
        raise ValueError("Login failed")
```

## Data Validation Schema

```python
from pydantic import BaseModel, EmailStr

class UserLoginRequest(BaseModel):
    email: EmailStr
    password: str
```

## Session Data Flow

### Storage Process
1. **JSON Serialization**: Session data → JSON string
2. **Embedding Generation**: JSON → 768-dimensional vector (via embedding service)
3. **Vector Indexing**: Vector stored in HNSW index for fast retrieval
4. **Binary Storage**: JSON + vector stored in user-storage.dat
5. **WAL Entry**: Write-Ahead Log ensures durability

### Retrieval Process  
1. **Vector Search**: Query storage with dummy vector
2. **Content Filtering**: Parse JSON content to find sessions
3. **Session Validation**: Check expiration, active status
4. **Authorization**: Grant/deny access based on session state

## Critical Configuration

### Session Storage Requirements
```python
# Sessions MUST be stored with embeddings enabled
session_options = {
    "generate_embedding": True,        # REQUIRED - without this, sessions disappear
    "extract_associations": False,     # Not needed for session data
}
```

### User Storage Server Configuration
```yaml
environment:
  - SUTRA_SEMANTIC_ANALYSIS=false      # No semantic analysis for auth data
  - SUTRA_EMBEDDING_SERVICE_URL=http://embedding-single:8888
  - SUTRA_EMBEDDING_TIMEOUT_SEC=30
```

## Performance Characteristics

- **User Lookup**: <10ms (vector search)
- **Password Verification**: ~100ms (Argon2)
- **Session Creation**: ~500ms (embedding generation)
- **JWT Generation**: <1ms
- **Total Login Time**: ~600-800ms

## Error Handling

### Authentication Errors
- **401 Unauthorized**: Invalid email/password combination
- **500 Internal Server Error**: Storage failure, embedding service down
- **400 Bad Request**: Invalid request format

### Common Failure Modes
1. **User not found**: Email doesn't exist in system
2. **Invalid password**: Password doesn't match stored hash
3. **Storage failure**: User storage server unavailable
4. **Embedding timeout**: Embedding service overloaded
5. **Session storage failure**: Sessions created but not persisted

## Testing Login

### Manual Testing
```bash
# Valid login
curl -X POST http://localhost:8000/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "password123"
  }'

# Expected response
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "bearer",
  "expires_in": 86400,
  "user": {
    "user_id": "3192ed168daf854e0000000000000000",
    "email": "test@example.com", 
    "organization": "Test Org",
    "role": "user",
    "full_name": "Test User",
    "created_at": "2025-10-28T14:44:05.485569",
    "last_login": null
  }
}
```

### Session Verification
```python
# Verify session is stored correctly
from sutra_storage_client import StorageClient
import json

client = StorageClient('user-storage-server:50051')
dummy_vector = [0.0] * 768
results = client.vector_search(dummy_vector, k=20)

for concept_id, similarity in results:
    concept = client.query_concept(concept_id)
    if concept:
        data = json.loads(concept['content'])
        if data.get('type') == 'session':
            print(f"✅ Session found: {data['session_id'][:8]}... for {data['email']}")
```

## Race Condition Handling

### Registration→Login Timing
```python
# If login immediately after registration, add delay
time.sleep(3)  # Allow embedding processing to complete

# Or implement retry logic with exponential backoff
for attempt in range(3):
    try:
        return await login(email, password)
    except ValueError as e:
        if attempt < 2:
            await asyncio.sleep(2 ** attempt)
        else:
            raise e
```

---

**AI Context**: The login process uses vector search to find users efficiently, creates secure sessions with embeddings (critical for persistence), and returns JWT tokens for stateless authentication. The system separates user lookup from session management for optimal performance.

**Last Updated**: 2025-10-28  
**Critical Dependencies**: User storage server, embedding service, Argon2 library, JWT library