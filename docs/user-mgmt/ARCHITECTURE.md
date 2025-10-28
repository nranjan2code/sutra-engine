# User Management System Architecture

**Technical architecture for Sutra AI's authentication and session management**

## System Overview

The Sutra AI user management system uses a **dual-storage architecture** that separates authentication data from business knowledge, optimized for high-performance vector search.

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Frontend      │    │   API Service    │    │  User Storage   │
│   (React)       │◄──►│   (FastAPI)      │◄──►│   (Vector)      │
│                 │    │   Port: 8000     │    │   Port: 50053   │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                              │                                    
                              ▼                                    
                       ┌──────────────────┐    ┌─────────────────┐
                       │ Embedding Service │    │ Domain Storage  │
                       │    (HA Cluster)   │    │  (Semantic +    │
                       │   Port: 8888      │    │   Vector)       │
                       └──────────────────┘    │ Port: 50051     │
                                               └─────────────────┘
```

## Core Components

### 1. API Service (`sutra-api`)
- **Language**: Python/FastAPI
- **Port**: 8000
- **Purpose**: REST endpoints for authentication
- **Key Module**: `sutra_api.services.user_service.UserService`
- **Storage Client**: TCP binary protocol to user storage

### 2. User Storage Server (`user-storage-server`) 
- **Language**: Rust
- **Port**: 50053 (external), 50051 (internal)
- **Purpose**: Authentication data storage (users, sessions, tokens)
- **Protocol**: Custom TCP binary with MessagePack serialization
- **Configuration**: `SUTRA_SEMANTIC_ANALYSIS=false` (vector-only)

### 3. Domain Storage Server (`storage-server`)
- **Language**: Rust  
- **Port**: 50051
- **Purpose**: Business knowledge storage
- **Configuration**: `SUTRA_SEMANTIC_ANALYSIS=true` (semantic + vector)

### 4. Embedding Service (`embedding-single`)
- **Language**: Python
- **Port**: 8888 (HAProxy load balancer)
- **Replicas**: 3 instances (8889-8891) with automatic failover
- **Purpose**: Generate vector embeddings for all text content

## Data Flow Architecture

### Registration Flow
```
1. Frontend → API Service (POST /auth/register)
2. API Service → UserService.register()  
3. UserService → Argon2 password hashing
4. UserService → User Storage (learn_concept_v2)
5. User Storage → Embedding Service (generate embeddings)
6. User Storage → Vector index storage
7. API Service → Return user data (no session created)
```

### Login Flow  
```
1. Frontend → API Service (POST /auth/login)
2. API Service → UserService.login()
3. UserService → User Storage (vector_search for user)
4. UserService → Argon2 password verification
5. UserService → Generate session data
6. UserService → User Storage (learn_concept_v2 for session)
7. User Storage → Embedding Service (session embeddings)
8. UserService → Generate JWT tokens
9. API Service → Return tokens + user data
```

### Session Validation Flow
```
1. Frontend → API Service (with JWT token)
2. API Service → JWT token validation
3. API Service → Session ID extraction
4. UserService → User Storage (vector_search for session)
5. UserService → Session expiration check
6. API Service → Allow/deny request
```

## Storage Architecture

### User Storage Design
- **Type**: Vector-only storage (no semantic analysis)
- **Embedding**: Required for all concepts (`generate_embedding: true`)
- **Search**: Pure vector similarity search
- **Performance**: <0.01ms query time, 57K writes/sec

### Data Structures

#### User Concept
```json
{
  "type": "user",
  "email": "user@example.com", 
  "password_hash": "$argon2id$v=19$...",
  "full_name": "User Name",
  "organization": "Company",
  "role": "user",
  "created_at": "2025-10-28T14:44:05.485569",
  "last_login": null
}
```

#### Session Concept  
```json
{
  "type": "session",
  "session_id": "V2oZYJH7_M3PcK2FKm721g",
  "user_id": "3192ed168daf854e0000000000000000", 
  "email": "user@example.com",
  "organization": "Company",
  "role": "user",
  "created_at": "2025-10-28T14:22:59.319000",
  "expires_at": "2025-11-04T14:22:59.319000",
  "active": true,
  "last_activity": "2025-10-28T14:22:59.319000"
}
```

## Critical Configuration

### Environment Variables

#### User Storage Server
```bash
SUTRA_SEMANTIC_ANALYSIS=false           # Vector-only storage
SUTRA_EMBEDDING_SERVICE_URL=http://embedding-single:8888
SUTRA_EMBEDDING_TIMEOUT_SEC=30
VECTOR_DIMENSION=768
```

#### Domain Storage Server  
```bash
SUTRA_SEMANTIC_ANALYSIS=true            # Full semantic analysis
SUTRA_EMBEDDING_SERVICE_URL=http://embedding-single:8888
SUTRA_EMBEDDING_TIMEOUT_SEC=30  
VECTOR_DIMENSION=768
```

### Storage Options for User Data
```python
# Correct configuration for user/session storage
options = {
    "generate_embedding": True,          # REQUIRED for storage
    "extract_associations": False,       # Not needed for auth data
    "analyze_semantics": False           # Controlled by env var
}
```

## Performance Characteristics

- **Vector Search**: <0.01ms per query
- **Concept Storage**: 57K writes/second  
- **Embedding Generation**: ~500ms per concept
- **JWT Token TTL**: 24 hours (access), 7 days (refresh)
- **Session TTL**: 7 days default

## Security Model

- **Password Hashing**: Argon2id with salt
- **JWT Signing**: HMAC-SHA256 
- **Transport**: HTTP (development), HTTPS (production)
- **Storage Protocol**: Binary TCP (no encryption in dev mode)
- **Session Storage**: Vector embeddings (not plaintext)

## Failure Modes & Recovery

### Common Issues
1. **Sessions not found**: Restart user storage server (embedding state issue)
2. **Business domain classification**: Check `SUTRA_SEMANTIC_ANALYSIS=false`  
3. **Registration→Login race**: Add 3-second delay between operations
4. **Embedding timeout**: Check embedding service health on port 8888

### Diagnostic Commands
```bash
# Check service health
curl http://localhost:8888/health
curl http://localhost:8000/health

# Check storage logs
docker logs sutra-user-storage --tail 20

# Test vector search
docker exec -it sutra-api python -c "
from sutra_storage_client import StorageClient
client = StorageClient('user-storage-server:50051')
results = client.vector_search([0.0]*768, k=10)
print(f'Found {len(results)} concepts')
"
```

## Integration Points

### With Frontend (React)
- JWT token storage in localStorage/sessionStorage
- Automatic token refresh on expiration
- Logout token invalidation

### With Business Services
- User context injection via JWT middleware
- Organization-based data filtering  
- Role-based access control (RBAC)

---

**AI Context Notes**: This architecture separates concerns between authentication (pure vector storage) and business logic (semantic storage). The dual-storage design prevents domain misclassification of authentication data while maintaining high performance for user operations.

**Last Updated**: 2025-10-28  
**Critical Dependencies**: Embedding service must be healthy for user/session storage to work.