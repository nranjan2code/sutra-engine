# Sutra Storage Quick Start

**Get running in 5 minutes**

---

## 🚀 Single Storage Mode (Development/Small Scale)

### Docker Compose

```yaml
# docker-compose.yml
version: '3.8'
services:
  storage:
    image: sutra/storage-server:2.0.0
    ports:
      - "7000:7000"
    environment:
      - SUTRA_STORAGE_MODE=single
      - STORAGE_PATH=/data
      - VECTOR_DIMENSION=768
      - WAL_FSYNC=true
      - SUTRA_EMBEDDING_SERVICE_URL=http://embedding:8888
    volumes:
      - ./storage-data:/data
    healthcheck:
      test: ["CMD", "sh", "-c", "echo '{\"HealthCheck\":{}}' | nc localhost 7000"]
      interval: 30s
      timeout: 10s
      retries: 3
```

```bash
docker-compose up -d
docker-compose logs -f storage
```

---

## 🏢 Sharded Storage Mode (Production/Large Scale)

### Docker Compose

```yaml
# docker-compose.yml
version: '3.8'
services:
  storage:
    image: sutra/storage-server:2.0.0
    ports:
      - "7000:7000"
    environment:
      - SUTRA_STORAGE_MODE=sharded
      - SUTRA_NUM_SHARDS=16
      - STORAGE_PATH=/data
      - VECTOR_DIMENSION=768
      - WAL_FSYNC=true
      - SUTRA_EMBEDDING_SERVICE_URL=http://embedding:8888
    volumes:
      - ./sharded-data:/data
    deploy:
      resources:
        limits:
          memory: 16G
          cpus: '8'
    healthcheck:
      test: ["CMD", "sh", "-c", "echo '{\"HealthCheck\":{}}' | nc localhost 7000"]
      interval: 30s
      timeout: 10s
      retries: 3
```

```bash
docker-compose up -d
docker-compose logs -f storage
```

---

## 🐍 Python Client

### Installation

```bash
pip install msgpack
```

### Basic Usage

```python
import socket
import msgpack

class SutraClient:
    def __init__(self, host='localhost', port=7000):
        self.sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.sock.connect((host, port))
    
    def _send(self, request):
        payload = msgpack.packb(request)
        length = len(payload).to_bytes(4, byteorder='little')
        self.sock.sendall(length + payload)
        
        resp_len = int.from_bytes(self.sock.recv(4), byteorder='little')
        return msgpack.unpackb(self.sock.recv(resp_len))
    
    def learn(self, content):
        resp = self._send({
            'LearnConceptV2': {
                'content': content,
                'options': {
                    'generate_embedding': True,
                    'extract_associations': True,
                    'min_association_confidence': 0.5,
                    'max_associations_per_concept': 10,
                    'strength': 1.0,
                    'confidence': 1.0,
                }
            }
        })
        return resp['LearnConceptV2Ok']['concept_id']
    
    def search(self, vector, k=10):
        resp = self._send({
            'VectorSearch': {
                'query_vector': vector,
                'k': k,
                'ef_search': 40,
            }
        })
        return resp['VectorSearchOk']['results']
    
    def stats(self):
        resp = self._send({'GetStats': {}})
        return resp['GetStatsOk']

# Usage
client = SutraClient()

# Learn concept
concept_id = client.learn("Aspirin treats headaches")
print(f"Learned: {concept_id}")

# Get stats
stats = client.stats()
print(f"Concepts: {stats['concept_count']}")
print(f"Health: {stats['reconciler_health']:.2f}")
```

---

## 🦀 Rust Client

### Cargo.toml

```toml
[dependencies]
tokio = { version = "1.40", features = ["full"] }
rmp-serde = "1.1"
serde = { version = "1.0", features = ["derive"] }
```

### Basic Usage

```rust
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde::{Serialize, Deserialize};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut stream = TcpStream::connect("localhost:7000").await?;
    
    // Learn concept
    let request = StorageRequest::LearnConceptV2 {
        content: "Aspirin treats headaches".to_string(),
        options: LearnOptionsMsg::default(),
    };
    
    // Send
    let payload = rmp_serde::to_vec(&request)?;
    let length = (payload.len() as u32).to_le_bytes();
    stream.write_all(&length).await?;
    stream.write_all(&payload).await?;
    
    // Receive
    let mut length_bytes = [0u8; 4];
    stream.read_exact(&mut length_bytes).await?;
    let resp_len = u32::from_le_bytes(length_bytes);
    let mut resp_bytes = vec![0u8; resp_len as usize];
    stream.read_exact(&mut resp_bytes).await?;
    
    let response: StorageResponse = rmp_serde::from_slice(&resp_bytes)?;
    println!("Response: {:?}", response);
    
    Ok(())
}
```

---

## 📊 Monitoring

### Health Check

```bash
echo '{"HealthCheck":{}}' | nc localhost 7000
# Response: {"HealthCheckOk":{}}
```

### Get Statistics

```bash
echo '{"GetStats":{}}' | nc localhost 7000 | python -m json.tool

# Sample output:
{
  "GetStatsOk": {
    "concept_count": 125000,
    "edge_count": 375000,
    "vector_count": 125000,
    "write_log_pending": 42,
    "reconciler_health": 0.95
  }
}
```

### Watch Stats

```bash
watch -n 1 'echo "{\"GetStats\":{}}" | nc localhost 7000 | jq ".GetStatsOk"'
```

---

## 🔧 Configuration Recipes

### High Throughput (Batch Ingestion)

```yaml
environment:
  - SUTRA_STORAGE_MODE=single
  - STORAGE_PATH=/data
  - VECTOR_DIMENSION=768
  - WAL_FSYNC=false  # Risk data loss for speed
  # ConcurrentConfig (internal):
  #   memory_threshold: 100000
  #   max_batch_size: 20000
  #   base_interval_ms: 5
```

### Low Latency (Real-Time Queries)

```yaml
environment:
  - SUTRA_STORAGE_MODE=single
  - STORAGE_PATH=/data
  - VECTOR_DIMENSION=768
  - WAL_FSYNC=true
  # ConcurrentConfig (internal):
  #   memory_threshold: 10000
  #   min_interval_ms: 1
  #   max_batch_size: 5000
```

### Memory Constrained (4GB RAM)

```yaml
environment:
  - SUTRA_STORAGE_MODE=single
  - STORAGE_PATH=/data
  - VECTOR_DIMENSION=768
  - WAL_FSYNC=true
  # ConcurrentConfig (internal):
  #   memory_threshold: 10000
  #   max_batch_size: 1000
  #   max_elements: 50000
```

---

## 🐛 Troubleshooting

### Storage Won't Start

```bash
# Check logs
docker logs storage

# Common issues:
# - Missing embedding service
# - Insufficient disk space
# - Invalid VECTOR_DIMENSION
```

### High Memory Usage

```bash
# Check stats
echo '{"GetStats":{}}' | nc localhost 7000 | jq '.GetStatsOk.write_log_pending'

# If > 70000: Memory threshold too high
# Solution: Restart with lower memory_threshold
```

### Poor Search Quality

```bash
# Check HNSW index
docker exec storage ls -lh /data/storage.usearch

# If missing: Index not built
# Solution: docker restart storage
```

---

## 📚 Next Steps

- [Complete Storage Guide](./STORAGE_GUIDE.md) - Full architecture and operations
- [TCP Protocol Spec](./TCP_BINARY_PROTOCOL.md) - Protocol details
- [Performance Tuning](./STORAGE_GUIDE.md#performance-tuning) - Optimization guide

---

**Questions?** See [README.md](./README.md) for documentation index
