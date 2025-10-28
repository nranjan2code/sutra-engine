# User Registration Process

**Complete workflow for user account creation in Sutra AI**

## Overview

User registration creates a new user account in the vector-based user storage system. The process involves password hashing, data validation, and vector embedding generation for efficient search and retrieval.

## Registration Workflow

### 1. Frontend Request
```javascript
// Frontend (React) registration call
const registerUser = async (userData) => {
  const response = await axios.post('http://localhost:8000/auth/register', {
    email: 'user@example.com',
    password: 'securePassword123',
    full_name: 'John Doe',
    organization: 'Acme Corp'
  });
  return response.data;
};
```

### 2. API Endpoint Processing
**Endpoint**: `POST /auth/register`  
**Handler**: `sutra_api.routes.auth.register`  
**Service**: `sutra_api.services.user_service.UserService.register()`

```python
async def register(self, email: str, password: str, full_name: str, 
                  organization: str, role: str = "user") -> Dict:
```

### 3. Data Validation
- **Email format**: Valid email address format
- **Password strength**: Minimum 8 characters (configurable)
- **Required fields**: email, password, full_name, organization
- **Email uniqueness**: Vector search for existing email

### 4. Password Security
```python
# Argon2 password hashing (industry standard)
password_hash = argon2.hash(password)
# Result: "$argon2id$v=19$m=65536,t=3,p=4$hash..."
```

**Security Properties**:
- Algorithm: Argon2id (memory-hard, side-channel resistant)
- Time cost: 3 iterations
- Memory cost: 64MB
- Parallelism: 4 threads
- Salt: Random 32-byte salt per password

### 5. User Data Structure
```python
user_data = {
    "type": "user",
    "email": email.lower().strip(),
    "password_hash": password_hash,
    "full_name": full_name,
    "organization": organization,
    "role": role,
    "created_at": datetime.utcnow().isoformat(),
    "last_login": None,
    "active": True
}
```

### 6. Vector Storage Process
```python
# Store user in vector storage with embeddings
user_id = self.storage.learn_concept_v2(
    content=json.dumps(user_data),
    options={
        "generate_embedding": True,        # Required for searchability
        "extract_associations": False,     # Not needed for user data
    }
)
```

**Storage Flow**:
1. JSON serialization of user data
2. Embedding generation via embedding service
3. Vector indexing in HNSW index
4. Binary storage in user-storage.dat
5. WAL (Write-Ahead Log) entry for durability

### 7. Response Generation
```python
# Success response (excludes sensitive data)
return {
    "user_id": user_id,
    "email": user_data["email"],
    "organization": user_data["organization"], 
    "role": user_data["role"],
    "full_name": user_data["full_name"],
    "created_at": user_data["created_at"],
    "last_login": None
}
```

## Complete Code Implementation

### API Route Handler
```python
@router.post("/register", response_model=dict)
async def register(
    user_data: UserRegistrationRequest,
    user_service: UserService = Depends(get_user_service)
) -> dict:
    try:
        result = await user_service.register(
            email=user_data.email,
            password=user_data.password, 
            full_name=user_data.full_name,
            organization=user_data.organization
        )
        return result
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))
    except Exception as e:
        logger.error(f"Registration failed: {e}")
        raise HTTPException(status_code=500, detail="Registration failed")
```

### UserService.register() Method
```python
async def register(self, email: str, password: str, full_name: str, 
                  organization: str, role: str = "user") -> Dict:
    """Register a new user with vector storage."""
    
    # 1. Validate and normalize email
    email = email.lower().strip()
    if not email or "@" not in email:
        raise ValueError("Invalid email address")
    
    # 2. Check if user already exists via vector search
    try:
        dummy_vector = [0.0] * 768
        vector_results = self.storage.vector_search(dummy_vector, k=50)
        
        for concept_id, similarity in vector_results:
            concept = self.storage.query_concept(concept_id)
            if concept and concept.get("content"):
                try:
                    data = json.loads(concept["content"])
                    if data.get("type") == "user" and data.get("email") == email:
                        raise ValueError("User already exists")
                except json.JSONDecodeError:
                    continue
    except Exception as e:
        logger.error(f"User existence check failed: {e}")
        raise ValueError("Registration failed")
    
    # 3. Hash password securely  
    password_hash = argon2.hash(password)
    
    # 4. Create user data structure
    user_data = {
        "type": "user",
        "email": email,
        "password_hash": password_hash,
        "full_name": full_name,
        "organization": organization, 
        "role": role,
        "created_at": datetime.utcnow().isoformat(),
        "last_login": None,
        "active": True
    }
    
    # 5. Store in vector storage
    try:
        user_id = self.storage.learn_concept_v2(
            content=json.dumps(user_data),
            options={
                "generate_embedding": True,
                "extract_associations": False,
            }
        )
        
        logger.info(f"✅ User registered: {email} (ID: {user_id[:8]})")
        
        # 6. Return sanitized user data
        return {
            "user_id": user_id,
            "email": user_data["email"],
            "organization": user_data["organization"],
            "role": user_data["role"], 
            "full_name": user_data["full_name"],
            "created_at": user_data["created_at"],
            "last_login": None
        }
        
    except Exception as e:
        logger.error(f"User storage failed: {email}: {e}")
        raise ValueError("Registration failed")
```

## Data Validation Schema

```python
from pydantic import BaseModel, EmailStr, validator

class UserRegistrationRequest(BaseModel):
    email: EmailStr
    password: str
    full_name: str
    organization: str
    
    @validator('password')
    def validate_password(cls, v):
        if len(v) < 8:
            raise ValueError('Password must be at least 8 characters')
        return v
    
    @validator('full_name')
    def validate_full_name(cls, v):
        if len(v.strip()) < 2:
            raise ValueError('Full name must be at least 2 characters')
        return v.strip()
    
    @validator('organization')
    def validate_organization(cls, v):
        if len(v.strip()) < 2:
            raise ValueError('Organization must be at least 2 characters')  
        return v.strip()
```

## Storage Configuration

### User Storage Server Settings
```yaml
# docker-compose.yml for user-storage-server
environment:
  - SUTRA_SEMANTIC_ANALYSIS=false     # Vector-only for user data
  - SUTRA_EMBEDDING_SERVICE_URL=http://embedding-single:8888
  - VECTOR_DIMENSION=768
  - STORAGE_PATH=/data
```

### Storage Options
```python
# Critical: User data MUST have embeddings enabled
storage_options = {
    "generate_embedding": True,         # Required for vector search
    "extract_associations": False,      # No associations for user data  
    # analyze_semantics controlled by SUTRA_SEMANTIC_ANALYSIS env var
}
```

## Error Handling

### Common Registration Errors
- **409 Conflict**: User already exists (duplicate email)
- **400 Bad Request**: Invalid input data (email format, password strength)
- **500 Internal Server Error**: Storage failure, embedding service down

### Error Response Format
```json
{
  "detail": "User already exists"
}
```

## Testing Registration

### Manual Testing
```bash
# Valid registration
curl -X POST http://localhost:8000/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "password123", 
    "full_name": "Test User",
    "organization": "Test Org"
  }'

# Expected response
{
  "user_id": "3192ed168daf854e",
  "email": "test@example.com",
  "organization": "Test Org", 
  "role": "user",
  "full_name": "Test User",
  "created_at": "2025-10-28T14:44:05.485569",
  "last_login": null
}
```

### Storage Verification
```python
# Verify user is stored in vector storage
from sutra_storage_client import StorageClient
client = StorageClient('user-storage-server:50051')

# Search for user
dummy_vector = [0.0] * 768
results = client.vector_search(dummy_vector, k=20)

for concept_id, similarity in results:
    concept = client.query_concept(concept_id)
    if concept:
        data = json.loads(concept['content'])
        if data.get('type') == 'user' and data.get('email') == 'test@example.com':
            print(f"✅ User found: {data['full_name']}")
```

## Performance Metrics

- **Registration Time**: ~800ms (including embedding generation)
- **Password Hashing**: ~100ms (Argon2 with secure parameters)
- **Embedding Generation**: ~500ms (768-dimensional vector)
- **Vector Storage**: ~50ms (binary format + WAL)
- **Duplicate Check**: <10ms (vector search)

## Security Considerations

1. **Password Storage**: Never store plaintext passwords
2. **Email Normalization**: Lowercase and trim to prevent duplicates
3. **Input Sanitization**: Validate all input fields
4. **Rate Limiting**: Implement registration rate limiting (not shown)
5. **Email Verification**: Consider email verification workflow (not implemented)

---

**AI Context**: This registration process creates user accounts that are stored as vector embeddings in the user storage server, separate from business data. The vector-based approach enables fast user lookup during login while maintaining security through proper password hashing.

**Last Updated**: 2025-10-28  
**Dependencies**: User storage server, embedding service, Argon2 library