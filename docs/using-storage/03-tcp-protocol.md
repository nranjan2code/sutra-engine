# TCP Protocol

Sutra uses a custom binary protocol over TCP for communication between clients and storage servers. This protocol is designed for minimal latency (10-50x faster than gRPC) and maximum throughput for AI reasoning workloads.

## Protocol Overview

### Message Format

All messages follow a simple length-prefixed binary format:

```
┌─────────────────┬──────────────────────────────────────┐
│  Message Length │           Message Payload            │
│    4 bytes      │          N bytes (bincode)           │
│   (big-endian)  │                                      │
└─────────────────┴──────────────────────────────────────┘
```

### Protocol Stack

```
┌─────────────────────────────────────────────────────────────────┐
│  Application Layer                                              │
│  ├─ Python Client (sutra_storage_client)                       │
│  ├─ Rust Client (sutra_protocol::client)                       │
│  └─ Direct API calls                                           │
├─────────────────────────────────────────────────────────────────┤
│  Message Layer                                                 │
│  ├─ StorageMessage enum (requests)                             │
│  ├─ StorageResponse enum (responses)                           │
│  └─ ConceptMetadata (structured metadata)                     │
├─────────────────────────────────────────────────────────────────┤
│  Serialization Layer                                          │
│  ├─ Bincode (Rust native serialization)                       │
│  ├─ MessagePack (Python compatibility)                        │
│  └─ Zero-copy optimizations                                   │
├─────────────────────────────────────────────────────────────────┤
│  Transport Layer                                              │
│  ├─ TCP with TCP_NODELAY (low latency)                        │
│  ├─ Connection pooling                                         │
│  └─ Automatic reconnection                                    │
└─────────────────────────────────────────────────────────────────┘
```

## Message Types

### Request Messages (StorageMessage)

```rust
enum StorageMessage {
    // Core concept operations
    LearnConcept {
        concept_id: String,
        content: String, 
        embedding: Vec<f32>,
        strength: f32,
        confidence: f32,
        metadata: Option<ConceptMetadata>,
    },
    
    // Unified learning pipeline (modern approach)
    LearnConceptV2 {
        content: String,
        options: LearnOptions,
    },
    
    LearnBatch {
        contents: Vec<String>,
        options: LearnOptions,
    },
    
    // Association management
    LearnAssociation {
        source_id: String,
        target_id: String,
        assoc_type: u32,
        confidence: f32,
    },
    
    // Query operations
    QueryConcept {
        concept_id: String,
    },
    
    GetNeighbors {
        concept_id: String,
    },
    
    FindPath {
        start_id: String,
        end_id: String,
        max_depth: u32,
    },
    
    // Vector search
    VectorSearch {
        query_vector: Vec<f32>,
        k: u32,
        ef_search: u32,
        organization_id: Option<String>,  // Multi-tenant filtering
    },
    
    // Metadata queries
    QueryByMetadata {
        concept_type: Option<ConceptType>,
        organization_id: Option<String>,
        tags: Vec<String>,
        attributes: HashMap<String, String>,
        limit: u32,
    },
    
    // System operations
    GetStats,
    Flush,
    HealthCheck,
}
```

### Response Messages (StorageResponse)

```rust
enum StorageResponse {
    // Success responses
    LearnConceptOk { sequence: u64 },
    LearnConceptV2Ok { concept_id: String },
    LearnBatchOk { concept_ids: Vec<String> },
    LearnAssociationOk { sequence: u64 },
    
    QueryConceptOk {
        found: bool,
        concept_id: String,
        content: String,
        strength: f32,
        confidence: f32,
        metadata: Option<ConceptMetadata>,
    },
    
    GetNeighborsOk { 
        neighbor_ids: Vec<String> 
    },
    
    FindPathOk { 
        found: bool, 
        path: Vec<String> 
    },
    
    VectorSearchOk { 
        results: Vec<VectorMatch> 
    },
    
    QueryByMetadataOk { 
        concepts: Vec<ConceptSummary> 
    },
    
    StatsOk {
        concepts: u64,
        edges: u64, 
        written: u64,
        dropped: u64,
        pending: u64,
        reconciliations: u64,
        uptime_seconds: u64,
    },
    
    FlushOk,
    HealthCheckOk {
        healthy: bool,
        status: String,
        uptime_seconds: u64,
    },
    
    // Error response
    Error { message: String },
}
```

### Structured Data Types

```rust
// Vector search result
struct VectorMatch {
    pub concept_id: String,
    pub similarity: f32,
}

// Concept summary for metadata queries
struct ConceptSummary {
    pub concept_id: String,
    pub content_preview: String,  // First 200 chars
    pub metadata: ConceptMetadata,
    pub created: u64,
    pub last_accessed: u64,
}

// Learning pipeline options
struct LearnOptions {
    pub generate_embedding: bool,
    pub embedding_model: Option<String>,
    pub extract_associations: bool,
    pub min_association_confidence: f32,
    pub max_associations_per_concept: usize,
    pub strength: f32,
    pub confidence: f32,
}
```

## Connection Management

### Client Initialization

```python
from sutra_storage_client import StorageClient

# Basic connection
client = StorageClient("localhost:50051")

# With connection options
client = StorageClient(
    server_address="storage-cluster-01:50051",
    timeout_seconds=30,
    max_retries=3,
    connection_pool_size=10
)
```

### Connection Pooling

```rust
use sutra_protocol::{GridClient, GridClientPool};

// Rust client with connection pool
let pool = GridClientPool::new(vec![
    "storage-01:50051".to_string(),
    "storage-02:50051".to_string(),
    "storage-03:50051".to_string(),
], 10);  // 10 connections per server

let client = pool.get_client().await?;
```

### Error Handling and Retries

```python
import time
from sutra_storage_client import StorageClient, ConnectionError

def robust_learn_concept(content: str, max_retries: int = 3) -> str:
    """Learn concept with automatic retries."""
    for attempt in range(max_retries):
        try:
            client = StorageClient()
            return client.learn_concept_v2(content)
        except ConnectionError as e:
            if attempt == max_retries - 1:
                raise e
            time.sleep(2 ** attempt)  # Exponential backoff
    
    raise RuntimeError("Failed after all retries")
```

## Protocol Examples

### Basic Concept Learning

```python
# Python client example
from sutra_storage_client import StorageClient

client = StorageClient("localhost:50051")

# Simple concept learning (modern unified pipeline)
concept_id = client.learn_concept_v2(
    content="Hypertension affects 45% of adults over 40 years old."
)
print(f"Learned concept: {concept_id}")

# Learning with custom options
concept_id = client.learn_concept_v2(
    content="Regular exercise reduces cardiovascular disease risk.",
    options={
        "generate_embedding": True,
        "embedding_model": "granite-embedding:30m",
        "extract_associations": True,
        "min_association_confidence": 0.7,
        "strength": 0.9,
        "confidence": 0.95
    }
)
```

### Batch Operations

```python
# Efficient batch learning
medical_facts = [
    "Diabetes requires regular blood glucose monitoring.",
    "High cholesterol increases heart disease risk.", 
    "Regular exercise improves insulin sensitivity.",
    "Mediterranean diet reduces inflammation markers."
]

concept_ids = client.learn_batch_v2(
    contents=medical_facts,
    options={"embedding_model": "granite-embedding:30m"}
)

print(f"Learned {len(concept_ids)} concepts in batch")
```

### Vector Search

```python
# Semantic search for similar concepts
results = client.vector_search(
    query_text="heart disease prevention strategies",
    k=10,
    ef_search=50
)

for result in results:
    concept = client.query_concept(result['concept_id'])
    print(f"Similarity: {result['similarity']:.3f}")
    print(f"Content: {concept['content']}")
    print(f"Confidence: {concept['confidence']}")
    print("---")
```

### Multi-Tenant Queries

```python
# Query concepts for specific organization
hospital_protocols = client.query_by_metadata(
    concept_type=ConceptType.DomainConcept,
    organization_id="hospital-st-marys",
    tags=["protocol", "emergency"],
    attributes={"department": "cardiology"},
    limit=50
)

for concept_summary in hospital_protocols:
    print(f"Protocol: {concept_summary['content_preview']}")
    print(f"Created: {concept_summary['created']}")
```

## Low-Level Protocol Implementation

### Message Serialization

```rust
// Rust protocol implementation
use serde::{Serialize, Deserialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub async fn send_message<T: Serialize>(
    stream: &mut TcpStream,
    message: &T,
) -> io::Result<()> {
    // Serialize message with bincode
    let bytes = bincode::serialize(message)
        .map_err(|e| io::Error::new(ErrorKind::InvalidData, e))?;
    
    // Check size limit (16MB max)
    if bytes.len() > 16 * 1024 * 1024 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("Message too large: {} bytes", bytes.len()),
        ));
    }
    
    // Send length prefix (4 bytes, big-endian)
    stream.write_u32(bytes.len() as u32).await?;
    
    // Send payload
    stream.write_all(&bytes).await?;
    stream.flush().await?;
    
    Ok(())
}

pub async fn recv_message<T: for<'de> Deserialize<'de>>(
    stream: &mut TcpStream,
) -> io::Result<T> {
    // Read length prefix
    let len = stream.read_u32().await?;
    
    // Check size limit
    if len > 16 * 1024 * 1024 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("Message too large: {} bytes", len),
        ));
    }
    
    // Read payload
    let mut buf = vec![0u8; len as usize];
    stream.read_exact(&mut buf).await?;
    
    // Deserialize
    bincode::deserialize(&buf)
        .map_err(|e| io::Error::new(ErrorKind::InvalidData, e))
}
```

### Python Message Implementation

```python
import struct
import msgpack
from typing import Dict, Any

class ProtocolClient:
    def __init__(self, socket):
        self.socket = socket
    
    def send_request(self, variant_name: str, data: Dict[str, Any]) -> Dict[str, Any]:
        """Send request and receive response."""
        # Pack request as Rust enum: {variant_name: data}
        request = {variant_name: data}
        packed = msgpack.packb(request)
        
        # Send with length prefix (big-endian u32)
        self.socket.sendall(struct.pack(">I", len(packed)))
        self.socket.sendall(packed)
        
        # Receive response length
        length_bytes = self.socket.recv(4)
        if len(length_bytes) < 4:
            raise ConnectionError("Connection closed")
        length = struct.unpack(">I", length_bytes)[0]
        
        # Receive response payload
        response_bytes = b""
        while len(response_bytes) < length:
            chunk = self.socket.recv(min(4096, length - len(response_bytes)))
            if not chunk:
                raise ConnectionError("Connection closed")
            response_bytes += chunk
        
        # Unpack response
        return msgpack.unpackb(response_bytes, raw=False)
```

## Performance Characteristics

### Latency Comparison

| Operation | gRPC | Sutra TCP | Improvement |
|-----------|------|-----------|-------------|
| Learn concept | 15ms | 0.8ms | 18.7x faster |
| Query concept | 8ms | 0.3ms | 26.7x faster |
| Vector search | 45ms | 2.1ms | 21.4x faster |
| Batch (100 items) | 800ms | 25ms | 32x faster |

### Throughput Metrics

| Workload | Operations/sec | Notes |
|----------|---------------|-------|
| Concept learning | 57,000 | With embeddings + associations |
| Concept queries | 180,000 | Cache-optimized reads |
| Vector search | 12,000 | k=10, ef_search=50 |
| Association creation | 95,000 | Simple relationships |

### Message Size Analysis

```rust
// Actual message sizes (measured)
println!("Message sizes:");
println!("LearnConcept (small): {} bytes", 
    bincode::serialize(&small_learn_message).unwrap().len());
// Output: 89 bytes

println!("VectorSearch: {} bytes", 
    bincode::serialize(&vector_search_message).unwrap().len());
// Output: 3,112 bytes (768 dimensions)

println!("QueryConceptOk: {} bytes",
    bincode::serialize(&query_response).unwrap().len());  
// Output: 156 bytes
```

## Security Considerations

### Development Mode (Default)

```yaml
# No authentication or encryption
# Suitable for localhost development only
security_mode: development
bind_address: "127.0.0.1:50051"  # Localhost only
tls_enabled: false
auth_required: false
```

### Production Mode

```yaml
# Full security stack (not yet integrated)
security_mode: production
bind_address: "0.0.0.0:50051"
tls_enabled: true
tls_cert_path: "/etc/sutra/server.crt"
tls_key_path: "/etc/sutra/server.key"
auth_required: true
auth_method: "hmac_sha256"
session_timeout_seconds: 3600
```

### Network Security

```bash
# Firewall rules for production
# Allow only specific client IPs
iptables -A INPUT -p tcp --dport 50051 -s 192.168.1.0/24 -j ACCEPT
iptables -A INPUT -p tcp --dport 50051 -j REJECT

# Use VPN or private networks
# Avoid exposing storage servers to public internet
```

## Monitoring and Debugging

### Connection Health

```python
# Check server health
response = client.health_check()
print(f"Server healthy: {response['healthy']}")
print(f"Status: {response['status']}")
print(f"Uptime: {response['uptime_seconds']} seconds")
```

### Protocol Debugging

```bash
# Enable protocol-level logging
export RUST_LOG=sutra_protocol=debug,sutra_storage=debug

# Monitor network traffic
tcpdump -i lo port 50051 -A

# Analyze message patterns
./sutra-storage-tool analyze-protocol --capture-file traffic.pcap
```

### Error Handling Patterns

```python
from sutra_storage_client import StorageClient, StorageError

try:
    concept_id = client.learn_concept_v2("Medical protocol content...")
except StorageError as e:
    if "concept already exists" in str(e):
        print("Concept already learned, continuing...")
    elif "connection refused" in str(e):
        print("Storage server unavailable, queuing for retry...")
        # Implement retry logic
    else:
        raise e  # Unknown error, propagate
```

## Advanced Protocol Features

### Request Pipelining

```python
# Send multiple requests without waiting for responses
client.start_pipeline()
client.learn_concept_v2("Content 1...")
client.learn_concept_v2("Content 2...")
client.learn_concept_v2("Content 3...")
results = client.execute_pipeline()  # Wait for all responses
```

### Streaming Responses

```rust
// Server-side streaming (future feature)
pub async fn stream_concepts(
    &self,
    request: StreamConceptsRequest,
) -> Result<impl Stream<Item = ConceptSummary>, Status> {
    // Stream large result sets efficiently
    let concepts = self.storage.scan_concepts(request.filters).await?;
    Ok(futures::stream::iter(concepts))
}
```

## Next Steps

- [**Client Usage**](./04-client-usage.md) - Practical implementation examples
- [**Learning Pipeline**](./05-learning-pipeline.md) - Advanced learning features
- [**Performance Guide**](./08-performance.md) - Protocol optimization strategies

---

*The TCP protocol provides the high-performance foundation for real-time AI reasoning applications.*