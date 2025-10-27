# Data Model

The Sutra Storage Engine uses a graph-based data model optimized for AI reasoning. This document explains the core data structures, their relationships, and how to work with them effectively.

## Core Data Types

### 1. Concept

A **concept** is the fundamental unit of knowledge in Sutra. Every piece of information is stored as a concept.

```rust
struct ConceptRecord {
    concept_id: ConceptId,        // 16-byte MD5 hash (deterministic)
    strength: f32,                // Importance weight (0.0-1.0) 
    confidence: f32,              // Quality/reliability score (0.0-1.0)
    access_count: u32,            // Usage tracking for optimization
    created: u64,                 // Unix timestamp
    last_accessed: u64,           // Last read/write time
    content_offset: u64,          // Location in storage file
    content_length: u32,          // Size of content in bytes
    embedding_offset: u64,        // Location of vector embedding
    // ... additional metadata
}
```

#### Concept Properties

| Field | Type | Purpose | Range |
|-------|------|---------|-------|
| `concept_id` | ConceptId | Unique identifier (MD5 hash) | 16 bytes |
| `content` | String | UTF-8 text content | Variable |
| `embedding` | Vec<f32> | Semantic vector representation | 768 dimensions |
| `strength` | f32 | Importance/weight in reasoning | 0.0-1.0 |
| `confidence` | f32 | Quality/reliability score | 0.0-1.0 |
| `metadata` | ConceptMetadata | Type, organization, tags | Optional |

#### Concept Types

Sutra supports different concept types for organizing knowledge:

```rust
enum ConceptType {
    // Domain knowledge (stored in domain-storage.dat)
    DomainConcept = 0,         // Facts, procedures, domain expertise
    
    // User/organization data (stored in user-storage.dat)  
    User = 10,                 // User accounts
    Session = 11,              // Login sessions
    Organization = 12,         // Multi-tenant organizations
    
    // Conversation data
    Conversation = 20,         // Chat threads
    Message = 21,              // Individual messages
    Space = 22,                // Workspaces/channels
    
    // Access control
    Permission = 30,           // RBAC permissions
    Role = 31,                 // User roles
    
    // Audit/compliance
    AuditLog = 40,            // System audit events
}
```

### 2. Association

An **association** represents a typed relationship between two concepts. This is how knowledge connections are stored.

```rust
struct AssociationRecord {
    source_id: ConceptId,      // Origin concept
    target_id: ConceptId,      // Destination concept
    assoc_type: u8,            // Relationship type
    confidence: f32,           // Strength of connection (0.0-1.0)
    weight: f32,               // Reasoning weight
    created: u64,              // When relationship was established
    last_used: u64,            // Last time used in reasoning
    // ... additional tracking data
}
```

#### Association Types

Sutra defines several types of relationships for different reasoning patterns:

```rust
enum AssociationType {
    // Domain knowledge relationships
    Semantic = 0,              // Similar meaning/context
    Causal = 1,                // Cause-and-effect relationship
    Temporal = 2,              // Time-based sequence
    Hierarchical = 3,          // Parent-child/category relationship
    Compositional = 4,         // Part-of/contains relationship
    
    // User/organization relationships
    HasSession = 10,           // User -> Session
    BelongsToOrganization = 11, // User -> Organization
    HasRole = 12,              // User -> Role
    HasPermission = 13,        // Role -> Permission
    
    // Conversation relationships
    OwnsConversation = 20,     // User -> Conversation
    HasMessage = 21,           // Conversation -> Message  
    AuthoredBy = 22,           // Message -> User
    InSpace = 23,              // Conversation -> Space
    
    // Cross-storage links
    UsesKnowledgeBase = 30,    // Conversation -> Domain knowledge
    
    // Audit relationships
    TriggeredBy = 40,          // AuditLog -> User
    RelatesTo = 41,            // AuditLog -> Any concept
}
```

### 3. ConceptId

Concept identifiers are deterministic 16-byte MD5 hashes, enabling:
- **Consistency**: Same content always produces same ID
- **Deduplication**: Identical concepts automatically merge
- **Distribution**: Predictable sharding across storage nodes

```python
# Python example
import hashlib

def generate_concept_id(content: str) -> str:
    """Generate deterministic concept ID from content."""
    hash_bytes = hashlib.md5(content.encode('utf-8')).digest()
    return hash_bytes.hex()

# Usage
concept_id = generate_concept_id("Machine learning is a subset of AI")
# Always produces: "a1b2c3d4e5f6789..." (same for identical content)
```

## Multi-Tenancy and Metadata

### ConceptMetadata

Modern Sutra concepts include rich metadata for organization and filtering:

```rust
struct ConceptMetadata {
    concept_type: ConceptType,              // Type classification
    organization_id: Option<String>,        // Tenant isolation (required for user data)
    created_by: Option<String>,            // User who created this
    tags: Vec<String>,                     // Custom tags for filtering
    attributes: HashMap<String, String>,    // Key-value metadata
    deleted: bool,                         // Soft delete flag
    schema_version: u32,                   // Forward compatibility
}
```

#### Multi-Tenant Isolation

User storage concepts **must** have an organization_id:

```python
# Valid user concept
user_metadata = ConceptMetadata(
    concept_type=ConceptType.User,
    organization_id="org-abc123",      # Required!
    created_by="user-xyz789",
    tags=["active", "premium"],
    attributes={
        "email": "alice@company.com",
        "role": "admin"
    }
)

# Domain concepts don't need organization_id
domain_metadata = ConceptMetadata(
    concept_type=ConceptType.DomainConcept,
    organization_id=None,              # Optional for domain knowledge
    tags=["medical", "protocol"]
)
```

## Storage Architecture

### Dual Storage System

Sutra uses separate storage files for different data types:

```
├── domain-storage.dat     # Domain-specific knowledge
│   ├── Medical protocols
│   ├── Legal precedents  
│   ├── Financial regulations
│   └── Technical documentation
│
└── user-storage.dat       # Multi-tenant user data
    ├── Users and sessions
    ├── Conversations and messages
    ├── Organizations and roles
    └── Audit logs
```

### Benefits of Separation

1. **Performance**: Domain knowledge optimized for read-heavy reasoning
2. **Security**: User data isolated with tenant-specific access control
3. **Compliance**: Audit logs separate from business logic
4. **Scaling**: Different sharding strategies per data type

## Working with the Data Model

### Creating Concepts

```python
from sutra_storage_client import StorageClient

client = StorageClient("localhost:50051")

# Domain concept (automatic embedding generation)
concept_id = client.learn_concept_v2(
    content="Diabetes requires regular blood glucose monitoring."
)

# User concept with metadata
user_concept = client.learn_concept_with_metadata(
    content="Dr. Sarah Johnson, Endocrinologist",
    metadata={
        "concept_type": ConceptType.User,
        "organization_id": "hospital-123",
        "attributes": {
            "email": "sarah.johnson@hospital.com",
            "specialty": "endocrinology",
            "license": "MD-12345"
        }
    }
)
```

### Creating Associations  

```python
# Semantic relationship between medical concepts
client.learn_association(
    source_id=diabetes_concept_id,
    target_id=monitoring_concept_id, 
    assoc_type=AssociationType.Semantic,
    confidence=0.95
)

# Hierarchical relationship (procedure -> category)
client.learn_association(
    source_id=glucose_test_id,
    target_id=diagnostic_procedures_id,
    assoc_type=AssociationType.Hierarchical, 
    confidence=0.90
)

# User ownership relationship
client.learn_association(
    source_id=user_id,
    target_id=conversation_id,
    assoc_type=AssociationType.OwnsConversation,
    confidence=1.0
)
```

### Querying Data

```python
# Get concept details
concept = client.query_concept(concept_id)
print(f"Content: {concept['content']}")
print(f"Confidence: {concept['confidence']}")
print(f"Metadata: {concept['metadata']}")

# Find related concepts
neighbors = client.get_neighbors(concept_id)
for neighbor_id in neighbors:
    neighbor = client.query_concept(neighbor_id)
    print(f"Related: {neighbor['content']}")

# Search by metadata filters
medical_users = client.query_by_metadata(
    concept_type=ConceptType.User,
    organization_id="hospital-123", 
    attributes={"specialty": "endocrinology"}
)
```

## Data Validation

### Schema Validation

Sutra includes built-in validation for different concept types:

```python
# User concepts require email, password_hash, salt
user_attributes = {
    "email": "doctor@hospital.com",
    "password_hash": "hashed_password_123",
    "salt": "random_salt_456",
    "full_name": "Dr. Smith"
}

# Session concepts require token and expiration
session_attributes = {
    "session_token": "token_abc123",
    "expires_at": "1698765432",  # Unix timestamp
    "ip_address": "192.168.1.100"
}

# Message concepts require role
message_attributes = {
    "role": "assistant",  # Must be "user" or "assistant"
    "confidence": "0.95",
    "token_count": "150"
}
```

### Content Templates

Standard content formatting for consistency:

```python
# Formatted content examples
user_content = f"User: {username} ({email})"
session_content = f"Session for user {user_id} created at {timestamp}"
conversation_content = f"Conversation: '{title}' (created by {user_id})"
message_content = f"[{role}]: {message_text}"
audit_content = f"AUDIT: {action} on {resource_type} {resource_id}"
```

## Best Practices

### 1. Concept Design
- **Atomic Content**: Each concept should represent one coherent piece of knowledge
- **Meaningful IDs**: Content should be normalized for consistent ID generation
- **Appropriate Types**: Use correct ConceptType for proper storage routing

### 2. Association Strategy  
- **Semantic**: Use for similar concepts that could be alternatives
- **Causal**: For cause-and-effect relationships in processes
- **Hierarchical**: For category/subcategory and part-of relationships
- **Compositional**: For components that make up larger systems

### 3. Multi-Tenancy
- **Always set organization_id** for user storage concepts
- **Use consistent tenant naming** conventions
- **Implement tenant-aware queries** in application logic

### 4. Performance
- **Batch operations** when possible for better throughput
- **Cache frequently accessed concepts** in application layer  
- **Use appropriate confidence thresholds** to filter low-quality associations

## Next Steps

- [**Storage Format**](./02-storage-format.md) - Learn about binary persistence
- [**TCP Protocol**](./03-tcp-protocol.md) - Understand the communication interface
- [**Client Usage**](./04-client-usage.md) - See practical code examples

---

*The data model provides the foundation for building sophisticated AI reasoning applications with full explainability and audit trails.*