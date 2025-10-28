# Storage Design

**Vector-based storage architecture for Sutra AI user management system**

## Overview

The Sutra AI user management system uses a **dual-storage architecture** that separates authentication data from business knowledge. User data (users, sessions, tokens) is stored using pure vector embeddings for high-performance search and retrieval.

## Storage Architecture

### Dual-Storage Design
```
┌─────────────────────────────────────┐
│           API Service               │
│         (FastAPI)                   │
└─────────────┬───────────────────────┘
              │
      ┌───────┴────────┐
      ▼                ▼
┌─────────────┐  ┌─────────────────┐
│ User Storage│  │ Domain Storage  │
│ (Vector)    │  │ (Semantic +     │
│ Port: 50053 │  │  Vector)        │
│             │  │ Port: 50051     │
└─────────────┘  └─────────────────┘
```

### Storage Separation
- **User Storage**: Authentication data only (users, sessions, tokens)
- **Domain Storage**: Business knowledge (concepts, rules, entities)

## User Storage Server

### Configuration
```yaml
user-storage-server:
  image: sutra-storage-server:latest
  ports:
    - "50053:50051"  # External:Internal
  environment:
    - SUTRA_SEMANTIC_ANALYSIS=false      # Vector-only storage
    - SUTRA_EMBEDDING_SERVICE_URL=http://embedding-single:8888
    - SUTRA_EMBEDDING_TIMEOUT_SEC=30
    - VECTOR_DIMENSION=768
    - STORAGE_PATH=/data
  volumes:
    - user-storage-data:/data
```

### Key Properties
- **Language**: Rust (high-performance binary)
- **Protocol**: Custom TCP binary with MessagePack serialization
- **Indexing**: HNSW (Hierarchical Navigable Small World) vector index
- **Persistence**: Binary format + WAL (Write-Ahead Log)
- **Performance**: <0.01ms queries, 57K writes/sec

## Storage Protocol

### TCP Binary Protocol
The storage server uses a custom binary protocol over TCP for minimal latency:

```rust
// Message format (simplified)
#[derive(Serialize, Deserialize)]
pub enum StorageRequest {
    LearnConceptV2 {
        content: String,
        options: LearnOptions,
    },
    VectorSearch {
        query_vector: Vec<f32>,
        k: u32,
    },
    QueryConcept {
        concept_id: String,
    },
}
```

### MessagePack Serialization
- **Format**: MessagePack (binary JSON alternative)
- **Size**: ~60% smaller than JSON
- **Speed**: 2-5x faster than JSON parsing
- **Type Safety**: Schema validation in Rust

### Connection Management
```python
# Python client connection
from sutra_storage_client import StorageClient

client = StorageClient('user-storage-server:50051')
# Automatic connection pooling and retry logic
```

## Data Models

### User Concept
```json
{
  "type": "user",
  "email": "user@example.com",
  "password_hash": "$argon2id$v=19$m=65536,t=3,p=4$...", 
  "full_name": "John Doe",
  "organization": "Acme Corporation",
  "role": "user",
  "created_at": "2025-10-28T14:44:05.485569",
  "last_login": null,
  "active": true
}
```

**Storage Process**:
1. JSON serialization (→ ~300-400 bytes)
2. Embedding generation (→ 768-dimensional vector)
3. HNSW indexing (→ O(log n) search)
4. Binary persistence (→ storage.dat + WAL)

### Session Concept  
```json
{
  "type": "session",
  "session_id": "V2oZYJH7_M3PcK2FKm721g",
  "user_id": "3192ed168daf854e0000000000000000",
  "email": "user@example.com",
  "organization": "Acme Corporation",
  "role": "user", 
  "created_at": "2025-10-28T14:22:59.319000",
  "expires_at": "2025-11-04T14:22:59.319000",
  "active": true,
  "last_activity": "2025-10-28T14:22:59.319000"
}
```

**Critical**: Sessions require `generate_embedding: true` for proper storage and retrieval.

## Vector Embedding Process

### Embedding Generation
```python
# Storage options for user data
options = {
    "generate_embedding": True,        # REQUIRED for vector search
    "extract_associations": False,     # No associations for auth data
    # analyze_semantics controlled by SUTRA_SEMANTIC_ANALYSIS=false
}

concept_id = storage.learn_concept_v2(
    content=json.dumps(user_data),
    options=options
)
```

### Embedding Flow
1. **Content**: JSON string → Embedding Service
2. **Vectorization**: Text → 768-dimensional vector (via transformer model)
3. **Indexing**: Vector → HNSW index for fast similarity search
4. **Storage**: Vector + JSON → Binary file with durability guarantees

### Vector Search Process
```python
# Search with dummy vector (broad search)
dummy_vector = [0.0] * 768
results = storage.vector_search(dummy_vector, k=50)

for concept_id, similarity in results:
    concept = storage.query_concept(concept_id)
    if concept:
        data = json.loads(concept['content'])
        # Filter by type, email, etc.
```

## Storage Performance

### Metrics
- **Query Latency**: <0.01ms (HNSW index)
- **Write Throughput**: 57,000 concepts/second
- **Storage Efficiency**: ~1KB per concept (JSON + vector)
- **Memory Usage**: ~0.1KB per concept (excluding vectors)
- **Startup Time**: 3.5ms for 1M vectors (persistent HNSW)

### Scalability
- **Concepts**: 10M+ per storage instance
- **Concurrent Connections**: 1000+ TCP connections
- **Vector Dimensions**: 768 (configurable)
- **Index Type**: HNSW (approximate nearest neighbor)

## Storage Internals

### File Structure
```
/data/
├── storage.dat          # Binary concept + vector storage
├── storage.usearch      # HNSW vector index (persistent)
├── wal.log             # Write-Ahead Log for durability
└── metadata.json       # Storage metadata
```

### Binary Format (V2)
```rust
// Simplified binary layout
struct StorageV2 {
    header: StorageHeader,
    concepts: Vec<ConceptEntry>,
    edges: Vec<EdgeEntry>,
    vectors: Vec<VectorEntry>,
}

struct ConceptEntry {
    id: ConceptId,
    content: Vec<u8>,      // JSON bytes
    metadata: ConceptMetadata,
}

struct VectorEntry {
    concept_id: ConceptId,
    vector: Vec<f32>,      // 768-dimensional
}
```

### Write-Ahead Log (WAL)
- **Purpose**: Crash recovery and durability
- **Format**: Binary log entries with checksums
- **Replay**: Automatic on startup
- **Truncation**: After successful checkpoints

## Configuration Options

### Environment Variables
```bash
# Core storage settings
STORAGE_PATH=/data                    # Storage directory
STORAGE_HOST=0.0.0.0                 # Bind address  
STORAGE_PORT=50051                   # TCP port
VECTOR_DIMENSION=768                 # Vector size

# Embedding integration
SUTRA_EMBEDDING_SERVICE_URL=http://embedding-single:8888
SUTRA_EMBEDDING_TIMEOUT_SEC=30

# Learning pipeline
SUTRA_SEMANTIC_ANALYSIS=false       # Disable for user storage
SUTRA_MIN_ASSOCIATION_CONFIDENCE=0.5
SUTRA_MAX_ASSOCIATIONS_PER_CONCEPT=10

# Performance tuning
RUST_LOG=info                        # Logging level
SUTRA_RECONCILE_INTERVAL_MS=10      # Background sync interval
```

### Storage Options per Request
```python
class LearnOptions:
    generate_embedding: bool = True      # Enable vector generation
    embedding_model: Optional[str] = None  # Model selection
    extract_associations: bool = False   # Association extraction
    min_association_confidence: float = 0.5
    max_associations_per_concept: int = 10
    strength: float = 1.0               # Concept weight
    confidence: float = 1.0             # Classification confidence
```

## Vector Search Strategies

### Search Types

#### Broad Search (User Lookup)
```python
# Use zero vector for broad search
dummy_vector = [0.0] * 768
results = storage.vector_search(dummy_vector, k=50)

# Filter results by content
for concept_id, similarity in results:
    concept = storage.query_concept(concept_id)
    data = json.loads(concept['content'])
    if data.get('type') == 'user' and data.get('email') == target_email:
        return data
```

#### Semantic Search (Future)
```python
# Use actual content embedding for semantic similarity
query_embedding = embedding_service.embed("user authentication session")
results = storage.vector_search(query_embedding, k=20)
```

### Search Performance
- **HNSW Parameters**: 
  - M=16 (connections per node)
  - efConstruction=200 (index build quality)
  - ef=100 (search quality)
- **Complexity**: O(log n) average case
- **Accuracy**: 95%+ recall for typical queries

## Storage Security

### Data Protection
- **Encryption at Rest**: Not implemented (development mode)
- **Access Control**: No authentication in development mode
- **Network Security**: TCP without TLS in development

### Production Security (Future)
```bash
# Enable security mode
SUTRA_SECURE_MODE=true
SUTRA_TLS_ENABLED=true
SUTRA_TLS_CERT=/path/to/cert.pem
SUTRA_TLS_KEY=/path/to/key.pem
SUTRA_AUTH_METHOD=token
SUTRA_AUTH_SECRET=secure-secret
```

## Monitoring & Diagnostics

### Storage Health Check
```python
async def check_storage_health():
    """Verify storage server connectivity and performance."""
    try:
        client = StorageClient('user-storage-server:50051')
        
        # Test vector search
        start_time = time.time()
        dummy_vector = [0.0] * 768
        results = client.vector_search(dummy_vector, k=10)
        search_time = time.time() - start_time
        
        return {
            "status": "healthy",
            "search_latency_ms": search_time * 1000,
            "concept_count": len(results),
            "server": "user-storage-server:50051"
        }
    except Exception as e:
        return {
            "status": "error",
            "error": str(e)
        }
```

### Performance Monitoring
```bash
# Check storage logs
docker logs sutra-user-storage --tail 20

# Monitor storage metrics
docker exec sutra-user-storage ps aux
docker exec sutra-user-storage df -h /data

# Test storage performance
time curl -X POST http://localhost:8000/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"perf@test.com","password":"test123","full_name":"Performance Test","organization":"Test"}'
```

### Debugging Tools
```python
# Direct storage inspection
from sutra_storage_client import StorageClient

client = StorageClient('user-storage-server:50051')

# Count concepts by type
dummy_vector = [0.0] * 768
results = client.vector_search(dummy_vector, k=1000)

type_counts = {}
for concept_id, similarity in results:
    concept = client.query_concept(concept_id)
    if concept:
        try:
            data = json.loads(concept['content'])
            concept_type = data.get('type', 'unknown')
            type_counts[concept_type] = type_counts.get(concept_type, 0) + 1
        except:
            type_counts['parse_error'] = type_counts.get('parse_error', 0) + 1

print("Concept counts by type:", type_counts)
```

## Backup & Recovery

### Backup Strategy
```bash
#!/bin/bash
# backup-user-storage.sh

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="/backups/user-storage-$TIMESTAMP"

# Create backup directory
mkdir -p $BACKUP_DIR

# Stop container (optional for consistency)
docker stop sutra-user-storage

# Backup storage files
docker cp sutra-user-storage:/data/storage.dat $BACKUP_DIR/
docker cp sutra-user-storage:/data/storage.usearch $BACKUP_DIR/
docker cp sutra-user-storage:/data/wal.log $BACKUP_DIR/

# Restart container
docker start sutra-user-storage

echo "Backup completed: $BACKUP_DIR"
```

### Recovery Process
```bash
#!/bin/bash
# restore-user-storage.sh

BACKUP_DIR="/backups/user-storage-20251028_144405"

# Stop container
docker stop sutra-user-storage

# Restore storage files
docker cp $BACKUP_DIR/storage.dat sutra-user-storage:/data/
docker cp $BACKUP_DIR/storage.usearch sutra-user-storage:/data/
docker cp $BACKUP_DIR/wal.log sutra-user-storage:/data/

# Restart container (WAL replay will occur automatically)
docker start sutra-user-storage

echo "Recovery completed from: $BACKUP_DIR"
```

### WAL Recovery
The storage server automatically replays WAL entries on startup:
```
2025-10-28T14:34:29.872058Z INFO Replaying WAL for crash recovery...
2025-10-28T14:34:29.872076Z INFO ✅ Replayed 5 WAL entries from crash recovery
```

---

**AI Context**: This storage design uses vector embeddings exclusively for user authentication data, providing high-performance search capabilities while separating concerns from business knowledge storage. The binary TCP protocol and HNSW indexing enable sub-millisecond query performance with horizontal scalability.

**Last Updated**: 2025-10-28  
**Critical Dependencies**: Embedding service health is required for new concept storage. Existing concepts can be queried without embedding service.