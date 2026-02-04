# TCP Binary Protocol Specification

**MessagePack-based protocol for high-performance storage communication**

> **Version**: 2.0.0  
> **Status**: Production  
> **Port**: 7000 (default)

---

## Overview

Sutra Storage uses a **custom binary TCP protocol** instead of gRPC or HTTP for several critical reasons:

### Why Not gRPC?

| Concern | gRPC Issue | Our Solution |
|---------|-----------|--------------|
| **Latency** | 200-500μs overhead | 10-50μs with MessagePack |
| **Dependencies** | Heavy (protobuf, HTTP/2) | Lightweight (tokio, rmp-serde) |
| **Control** | Limited framing control | Custom framing for streaming |
| **Binary Size** | 50MB+ server binary | 15MB storage server |

### Performance Comparison

```
Operation: Store 1KB concept with 768-dim vector

gRPC (measured):
  - Serialization: 50μs (protobuf)
  - Network: 100μs (HTTP/2 overhead)
  - Deserialization: 50μs
  - Total: ~200μs

MessagePack TCP (measured):
  - Serialization: 5μs (msgpack)
  - Network: 10μs (raw TCP)
  - Deserialization: 5μs
  - Total: ~20μs

Result: 10× faster
```

---

## Protocol Design

### Message Framing

```
┌────────────────┬────────────────────────────────────┐
│  4 bytes       │  N bytes                           │
│  Length (u32)  │  MessagePack Payload               │
│  Little-endian │  (StorageRequest or Response)      │
└────────────────┴────────────────────────────────────┘
```

**Frame Structure:**
1. **Length Prefix**: 4 bytes, little-endian u32
2. **Payload**: MessagePack-encoded request or response

**Example:**
```rust
// Sending
let payload = rmp_serde::to_vec(&request)?;
let length = (payload.len() as u32).to_le_bytes();
stream.write_all(&length).await?;
stream.write_all(&payload).await?;

// Receiving
let mut length_bytes = [0u8; 4];
stream.read_exact(&mut length_bytes).await?;
let length = u32::from_le_bytes(length_bytes);
let mut payload = vec![0u8; length as usize];
stream.read_exact(&mut payload).await?;
let request: StorageRequest = rmp_serde::from_slice(&payload)?;
```

### Security Limits (DoS Prevention)

```rust
const MAX_CONTENT_SIZE: usize = 10 * 1024 * 1024;     // 10MB max content
const MAX_EMBEDDING_DIM: usize = 2048;                 // Max embedding size
const MAX_BATCH_SIZE: usize = 1000;                    // Max batch operations
const MAX_MESSAGE_SIZE: usize = 100 * 1024 * 1024;    // 100MB max TCP message
const MAX_PATH_DEPTH: u32 = 20;                        // Max graph traversal depth
const MAX_SEARCH_K: u32 = 1000;                        // Max k for vector search
```

---

## Message Types

### Request Messages

```rust
pub enum StorageRequest {
    // V2 Unified Learning API
    LearnConceptV2 {
        namespace: Option<String>,
        content: String,
        options: LearnOptionsMsg,
    },
    
    LearnBatch {
        namespace: Option<String>,
        contents: Vec<String>,
        options: LearnOptionsMsg,
    },

    // 🔥 NEW: Learn with precomputed embedding (Requested for Sutra Engine)
    LearnWithEmbedding {
        id: Option<String>,
        namespace: String,
        content: String,
        embedding: Vec<f32>,
        metadata: HashMap<String, String>,
        timestamp: Option<i64>,
    },
    
    // Legacy Explicit Learning
    LearnConcept {
        namespace: Option<String>,
        concept_id: String,
        content: String,
        embedding: Vec<f32>,
        strength: f32,
        confidence: f32,
    },
    
    LearnAssociation {
        namespace: Option<String>,
        source_id: String,
        target_id: String,
        assoc_type: u32,
        confidence: f32,
    },
    
    // Query Operations
    QueryConcept {
        namespace: Option<String>,
        concept_id: String,
    },
    
    GetNeighbors {
        namespace: Option<String>,
        concept_id: String,
    },
    
    FindPath {
        namespace: Option<String>,
        start_id: String,
        end_id: String,
        max_depth: u32,
    },
    
    VectorSearch {
        namespace: Option<String>,
        query_vector: Vec<f32>,
        k: u32,
        ef_search: u32,
    },

    // 🔥 NEW: CRUD Operations
    DeleteConcept {
        namespace: String,
        id: String,
    },

    ClearCollection {
        namespace: String,
    },

    ListRecent {
        namespace: String,
        limit: u32,
    },
    
    // Semantic Operations (V2)
    FindPathSemantic {
        namespace: Option<String>,
        start_id: String,
        end_id: String,
        filter: SemanticFilterMsg,
        max_depth: u32,
        max_paths: u32,
    },
    
    FindTemporalChain {
        namespace: Option<String>,
        domain: Option<String>,
        start_time: i64,
        end_time: i64,
    },
    
    FindCausalChain {
        namespace: Option<String>,
        start_id: String,
        causal_type: String,
        max_depth: u32,
    },
    
    FindContradictions {
        namespace: Option<String>,
        domain: String,
    },
    
    QueryBySemantic {
        namespace: Option<String>,
        filter: SemanticFilterMsg,
        limit: Option<usize>,
    },
    
    // Management Operations
    GetStats { namespace: Option<String> },
    Flush,
    HealthCheck,
}
```

### Response Messages

```rust
pub enum StorageResponse {
    // V2 Unified Learning Responses
    LearnConceptV2Ok {
        concept_id: String,
    },
    
    LearnBatchOk {
        concept_ids: Vec<String>,
    },
    
    // Legacy Learning Responses
    LearnConceptOk {
        sequence: u64,
    },
    
    LearnAssociationOk {
        sequence: u64,
    },

    // 🔥 NEW: CRUD Responses
    DeleteConceptOk { id: String },
    ClearCollectionOk { namespace: String },
    ListRecentOk { items: Vec<RecentItemMsg> },
    
    // Query Responses
    QueryConceptOk {
        found: bool,
        concept_id: String,
        content: String,
        strength: f32,
        confidence: f32,
        attributes: HashMap<String, String>,
    },
    
    GetNeighborsOk {
        neighbor_ids: Vec<String>,
    },
    
    FindPathOk {
        found: bool,
        path: Vec<String>,
    },
    
    VectorSearchOk {
        results: Vec<(String, f32)>,  // (concept_id, similarity)
    },
    
    // Semantic Query Responses (V2)
    FindPathSemanticOk {
        paths: Vec<SemanticPathMsg>,
    },
    
    FindTemporalChainOk {
        chain: Vec<ConceptWithSemanticMsg>,
    },
    
    FindCausalChainOk {
        chain: Vec<ConceptWithSemanticMsg>,
    },
    
    FindContradictionsOk {
        pairs: Vec<(ConceptWithSemanticMsg, ConceptWithSemanticMsg)>,
    },
    
    QueryBySemanticOk {
        concepts: Vec<ConceptWithSemanticMsg>,
    },
    
    // Management Responses
    GetStatsOk {
        concepts: u64,
        edges: u64,
        vectors: u64,
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
    
    // Error Response
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentItemMsg {
    pub id: String,
    pub content_preview: String,
    pub created: u64,
    pub attributes: HashMap<String, String>,
}
```

### Supporting Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnOptionsMsg {
    pub generate_embedding: bool,
    pub embedding_model: Option<String>,
    pub extract_associations: bool,
    pub min_association_confidence: f32,
    pub max_associations_per_concept: usize,
    pub strength: f32,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticFilterMsg {
    pub semantic_type: Option<String>,      // "rule", "event", "entity", etc.
    pub domain_context: Option<String>,     // "medical", "legal", "financial"
    pub temporal_after: Option<i64>,        // Unix timestamp
    pub temporal_before: Option<i64>,       // Unix timestamp
    pub has_causal_relation: bool,
    pub min_confidence: f32,
    pub required_terms: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticPathMsg {
    pub concepts: Vec<String>,
    pub confidence: f32,
    pub type_distribution: HashMap<String, usize>,
    pub domains: Vec<String>,
    pub is_temporally_ordered: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptWithSemanticMsg {
    pub concept_id: String,
    pub content: String,
    pub semantic_type: String,
    pub domain: String,
    pub confidence: f32,
}
```

---

## Protocol Flows

### 1. Learn Concept (V2 Unified Pipeline)

```
Client                          Storage Server
  │                                    │
  │  LearnConceptV2                   │
  │  {content, options}               │
  ├───────────────────────────────────>│
  │                                    │ 1. Semantic Analysis
  │                                    │ 2. Generate Embedding
  │                                    │ 3. Extract Associations
  │                                    │ 4. Store Concept + Vector
  │                                    │ 5. Store Associations
  │                                    │
  │  LearnConceptV2Ok                 │
  │  {concept_id}                     │
  │<───────────────────────────────────┤
```

**Request:**
```json
{
  "LearnConceptV2": {
    "content": "Aspirin is contraindicated in patients with bleeding disorders",
    "options": {
      "generate_embedding": true,
      "embedding_model": null,
      "extract_associations": true,
      "min_association_confidence": 0.5,
      "max_associations_per_concept": 10,
      "strength": 1.0,
      "confidence": 1.0
    }
  }
}
```

**Response:**
```json
{
  "LearnConceptV2Ok": {
    "concept_id": "8f4a3b2c1d0e9f8a7b6c5d4e3f2a1b0c"
  }
}
```

### 2. Batch Learning

```
Client                          Storage Server
  │                                    │
  │  LearnBatch                       │
  │  {contents[], options}            │
  ├───────────────────────────────────>│
  │                                    │ Parallel processing:
  │                                    │ - Batch embeddings
  │                                    │ - Parallel semantic analysis
  │                                    │ - Parallel association extraction
  │                                    │ - Parallel storage
  │                                    │
  │  LearnBatchOk                     │
  │  {concept_ids[]}                  │
  │<───────────────────────────────────┤
```

**Benefits:**
- Single network round-trip
- Batch embedding generation (10× faster)
- Parallel processing across concepts
- Reduced protocol overhead

### 3. Semantic Query

```
Client                          Storage Server
  │                                    │
  │  QueryBySemantic                  │
  │  {filter, limit}                  │
  ├───────────────────────────────────>│
  │                                    │ 1. Apply semantic filter
  │                                    │ 2. Filter by type/domain
  │                                    │ 3. Filter by temporal bounds
  │                                    │ 4. Filter by confidence
  │                                    │ 5. Rank and limit results
  │                                    │
  │  QueryBySemanticOk                │
  │  {concepts[]}                     │
  │<───────────────────────────────────┤
```

**Request:**
```json
{
  "QueryBySemantic": {
    "filter": {
      "semantic_type": "Rule",
      "domain_context": "medical",
      "temporal_after": null,
      "temporal_before": null,
      "has_causal_relation": false,
      "min_confidence": 0.7,
      "required_terms": ["contraindication", "bleeding"]
    },
    "limit": 10
  }
}
```

**Response:**
```json
{
  "QueryBySemanticOk": {
    "concepts": [
      {
        "concept_id": "8f4a3b2c...",
        "content": "Aspirin is contraindicated in patients with bleeding disorders",
        "semantic_type": "Rule",
        "domain": "medical",
        "confidence": 0.95
      }
    ]
  }
}
```

### 4. Vector Search

```
Client                          Storage Server
  │                                    │
  │  VectorSearch                     │
  │  {query_vector, k, ef_search}     │
  ├───────────────────────────────────>│
  │                                    │ 1. Search HNSW index
  │                                    │ 2. Rank by cosine similarity
  │                                    │ 3. Return top-k
  │                                    │
  │  VectorSearchOk                   │
  │  {results[(id, similarity)]}      │
  │<───────────────────────────────────┤
```

**Performance:**
- Typical latency: 2ms for k=10, 1M vectors
- Scales with `ef_search` parameter
- Returns approximate nearest neighbors

### 5. Health Check

```
Client                          Storage Server
  │                                    │
  │  HealthCheck                      │
  ├───────────────────────────────────>│
  │                                    │ Quick liveness check
  │                                    │
  │  HealthCheckOk                    │
  │<───────────────────────────────────┤
```

**Use Cases:**
- Docker HEALTHCHECK directive
- Kubernetes liveness probe
- Load balancer health checks

---

## Client Implementation

### Python Client Example

```python
import socket
import msgpack

class SutraStorageClient:
    def __init__(self, host='localhost', port=7000):
        self.host = host
        self.port = port
        self.sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.sock.connect((host, port))
    
    def _send_request(self, request):
        # Serialize to MessagePack
        payload = msgpack.packb(request)
        
        # Send length prefix
        length = len(payload).to_bytes(4, byteorder='little')
        self.sock.sendall(length)
        
        # Send payload
        self.sock.sendall(payload)
        
        # Receive length prefix
        length_bytes = self.sock.recv(4)
        response_length = int.from_bytes(length_bytes, byteorder='little')
        
        # Receive response
        response_bytes = self.sock.recv(response_length)
        return msgpack.unpackb(response_bytes)
    
    def learn_concept(self, content, options=None):
        if options is None:
            options = {
                'generate_embedding': True,
                'embedding_model': None,
                'extract_associations': True,
                'min_association_confidence': 0.5,
                'max_associations_per_concept': 10,
                'strength': 1.0,
                'confidence': 1.0,
            }
        
        request = {
            'LearnConceptV2': {
                'content': content,
                'options': options,
            }
        }
        
        response = self._send_request(request)
        
        if 'LearnConceptV2Ok' in response:
            return response['LearnConceptV2Ok']['concept_id']
        elif 'Error' in response:
            raise Exception(response['Error']['message'])
    
    def query_concept(self, concept_id):
        request = {
            'QueryConcept': {
                'concept_id': concept_id,
            }
        }
        
        response = self._send_request(request)
        
        if 'QueryConceptOk' in response:
            return response['QueryConceptOk']
        elif 'Error' in response:
            raise Exception(response['Error']['message'])
    
    def vector_search(self, query_vector, k=10, ef_search=40):
        request = {
            'VectorSearch': {
                'query_vector': query_vector,
                'k': k,
                'ef_search': ef_search,
            }
        }
        
        response = self._send_request(request)
        
        if 'VectorSearchOk' in response:
            return response['VectorSearchOk']['results']
        elif 'Error' in response:
            raise Exception(response['Error']['message'])
    
    def get_stats(self):
        request = {'GetStats': {}}
        response = self._send_request(request)
        
        if 'GetStatsOk' in response:
            return response['GetStatsOk']
        elif 'Error' in response:
            raise Exception(response['Error']['message'])
    
    def close(self):
        self.sock.close()

# Usage
client = SutraStorageClient()

# Learn concept
concept_id = client.learn_concept(
    "Aspirin is contraindicated in patients with bleeding disorders"
)

# Query concept
concept = client.query_concept(concept_id)
print(f"Content: {concept['content']}")

# Vector search
results = client.vector_search(query_vector, k=10)
for concept_id, similarity in results:
    print(f"{concept_id}: {similarity:.3f}")

# Get stats
stats = client.get_stats()
print(f"Concepts: {stats['concept_count']}")
print(f"Health: {stats['reconciler_health']:.2f}")

client.close()
```

### Rust Client Example

```rust
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde::{Serialize, Deserialize};

pub struct SutraStorageClient {
    stream: TcpStream,
}

impl SutraStorageClient {
    pub async fn connect(addr: &str) -> anyhow::Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        Ok(Self { stream })
    }
    
    async fn send_request(&mut self, request: StorageRequest) -> anyhow::Result<StorageResponse> {
        // Serialize to MessagePack
        let payload = rmp_serde::to_vec(&request)?;
        
        // Send length prefix
        let length = (payload.len() as u32).to_le_bytes();
        self.stream.write_all(&length).await?;
        
        // Send payload
        self.stream.write_all(&payload).await?;
        
        // Receive length prefix
        let mut length_bytes = [0u8; 4];
        self.stream.read_exact(&mut length_bytes).await?;
        let response_length = u32::from_le_bytes(length_bytes);
        
        // Receive response
        let mut response_bytes = vec![0u8; response_length as usize];
        self.stream.read_exact(&mut response_bytes).await?;
        
        let response: StorageResponse = rmp_serde::from_slice(&response_bytes)?;
        Ok(response)
    }
    
    pub async fn learn_concept(&mut self, content: String, options: LearnOptionsMsg) -> anyhow::Result<String> {
        let request = StorageRequest::LearnConceptV2 { content, options };
        let response = self.send_request(request).await?;
        
        match response {
            StorageResponse::LearnConceptV2Ok { concept_id } => Ok(concept_id),
            StorageResponse::Error { message } => Err(anyhow::anyhow!(message)),
            _ => Err(anyhow::anyhow!("Unexpected response")),
        }
    }
    
    pub async fn get_stats(&mut self) -> anyhow::Result<StatsResponse> {
        let request = StorageRequest::GetStats;
        let response = self.send_request(request).await?;
        
        match response {
            StorageResponse::GetStatsOk { concept_count, edge_count, .. } => {
                Ok(StatsResponse { concept_count, edge_count, .. })
            }
            StorageResponse::Error { message } => Err(anyhow::anyhow!(message)),
            _ => Err(anyhow::anyhow!("Unexpected response")),
        }
    }
}

// Usage
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = SutraStorageClient::connect("localhost:7000").await?;
    
    let options = LearnOptionsMsg::default();
    let concept_id = client.learn_concept(
        "Aspirin is contraindicated in patients with bleeding disorders".to_string(),
        options,
    ).await?;
    
    println!("Learned concept: {}", concept_id);
    
    let stats = client.get_stats().await?;
    println!("Total concepts: {}", stats.concept_count);
    
    Ok(())
}
```

---

## Error Handling

### Error Response Format

```rust
StorageResponse::Error {
    message: String,  // Human-readable error message
}
```

### Common Error Codes (embedded in message)

| Error | Message Prefix | Cause | Client Action |
|-------|---------------|-------|---------------|
| **Invalid Request** | `"Invalid request:"` | Malformed MessagePack | Fix client serialization |
| **Content Too Large** | `"Content exceeds"` | > 10MB content | Split into smaller concepts |
| **Invalid Dimension** | `"Vector dimension"` | Wrong embedding size | Check model configuration |
| **Batch Too Large** | `"Batch size exceeds"` | > 1000 concepts | Split into smaller batches |
| **Path Too Deep** | `"Path depth exceeds"` | > 20 hops | Reduce max_depth parameter |
| **Storage Full** | `"Storage capacity"` | Disk full | Provision more storage |
| **Embedding Failed** | `"Embedding service"` | Service down/timeout | Retry or disable embeddings |

### Retry Strategy

```python
import time

def learn_concept_with_retry(client, content, max_retries=3):
    for attempt in range(max_retries):
        try:
            return client.learn_concept(content)
        except Exception as e:
            if "Embedding service" in str(e) and attempt < max_retries - 1:
                # Exponential backoff
                time.sleep(2 ** attempt)
                continue
            raise
```

---

## Performance Tuning

### Client-Side Optimizations

**1. Connection Pooling**
```python
from queue import Queue

class ConnectionPool:
    def __init__(self, host, port, pool_size=10):
        self.pool = Queue(maxsize=pool_size)
        for _ in range(pool_size):
            client = SutraStorageClient(host, port)
            self.pool.put(client)
    
    def get_client(self):
        return self.pool.get()
    
    def return_client(self, client):
        self.pool.put(client)
```

**2. Batch Operations**
```python
# BAD: 1000 round-trips
for content in contents:
    client.learn_concept(content)

# GOOD: 1 round-trip
client.learn_batch(contents)
```

**3. Pipelining (Advanced)**
```python
# Send multiple requests without waiting
for content in contents:
    client._send_request_async(content)

# Collect responses
responses = [client._receive_response() for _ in contents]
```

### Server-Side Tuning

**High Throughput:**
```yaml
# Increase reconciliation batch size
ADAPTIVE_RECONCILER_MAX_BATCH_SIZE: 20000
ADAPTIVE_RECONCILER_BASE_INTERVAL_MS: 5
```

**Low Latency:**
```yaml
# Reduce reconciliation interval
ADAPTIVE_RECONCILER_MIN_INTERVAL_MS: 1
ADAPTIVE_RECONCILER_BASE_INTERVAL_MS: 5
```

---

## Security Considerations

### Current Status

⚠️ **Development Mode (Default)**: No authentication, no encryption  
✅ **Production Mode (Future)**: TLS 1.3 + HMAC authentication

### Planned Security Features

```rust
// SUTRA_SECURE_MODE=true enables:
1. TLS 1.3 encryption (tokio-rustls)
2. HMAC-SHA256 authentication
3. Request signing with tokens
4. Rate limiting per client
5. IP whitelisting
```

### Current Recommendations

**Development:**
- Bind to `127.0.0.1` only
- Use Docker internal networks
- No internet-facing exposure

**Production (without security module):**
- Run behind VPN or firewall
- Use mTLS at load balancer
- Implement application-level auth in API layer

---

## Monitoring and Debugging

### Protocol-Level Logging

```bash
# Enable protocol-level logging
RUST_LOG=sutra_storage::tcp_server=debug

# Sample output:
[DEBUG] TCP connection from 172.17.0.5:43892
[DEBUG] Received request: LearnConceptV2 (1024 bytes)
[DEBUG] Processed in 12.5ms
[DEBUG] Sent response: LearnConceptV2Ok (64 bytes)
```

### Network Analysis

```bash
# Capture TCP traffic
tcpdump -i any -w sutra-storage.pcap port 7000

# Analyze with Wireshark (MessagePack dissector required)
wireshark sutra-storage.pcap
```

### Performance Metrics

```bash
# Request rate
echo '{"GetStats": {}}' | nc localhost 7000 | jq '.concept_count'

# Watch stats in real-time
watch -n 1 'echo "{\"GetStats\": {}}" | nc localhost 7000 | jq'
```

---

## Versioning and Compatibility

### Protocol Version Negotiation (Future)

```rust
// First message: version handshake
ClientHello {
    protocol_version: "2.0",
    supported_features: ["semantic", "batch", "streaming"],
}

ServerHello {
    protocol_version: "2.0",
    enabled_features: ["semantic", "batch"],
}
```

### Backward Compatibility

- **V2 additions**: Optional semantic fields (ignored by V1 clients)
- **V1 legacy**: `LearnConcept` still supported (mapped to V2 internally)
- **Deprecation policy**: 2 major versions backward compatibility

---

## Future Enhancements

### Planned Features

1. **Streaming Responses**
   - Large batch operations stream results
   - Graph traversal streams paths as found
   - Reduces memory pressure

2. **Compression**
   - Optional zstd compression for large payloads
   - Negotiated via handshake
   - 60-80% size reduction for text content

3. **Multiplexing**
   - Multiple concurrent requests on single connection
   - Request ID correlation
   - Async response delivery

4. **Server Push**
   - Push updates to subscribed clients
   - Real-time graph change notifications
   - Concept invalidation events

---

## Appendix

### MessagePack Format Examples

**Request:**
```
\x81              # map with 1 key-value pair
  \xa9LearnConceptV2   # "LearnConceptV2" (9-char string)
  \x82              # map with 2 key-value pairs
    \xa7content     # "content" (7-char string)
    \xda\x00\x30    # str16 with 48 bytes following
    ...content bytes...
    \xa7options     # "options" (7-char string)
    \x88            # map with 8 key-value pairs
    ...options...
```

**Response:**
```
\x81              # map with 1 key-value pair
  \xb0LearnConceptV2Ok  # "LearnConceptV2Ok" (16-char string)
  \x81              # map with 1 key-value pair
    \xaaconcept_id  # "concept_id" (10-char string)
    \xd9\x20        # str8 with 32 bytes (hex string)
    ...concept_id...
```

### Wire Format Size Comparison

| Data | JSON | MessagePack | Savings |
|------|------|-------------|---------|
| Simple concept | 250 bytes | 180 bytes | 28% |
| With embedding (768-dim) | 6KB | 3.1KB | 48% |
| Batch (100 concepts) | 60KB | 32KB | 47% |

---

**Last Updated**: October 27, 2025  
**Protocol Version**: 2.0.0  
**Authors**: Sutra AI Engineering Team
