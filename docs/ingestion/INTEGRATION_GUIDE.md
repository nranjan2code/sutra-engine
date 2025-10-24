# Sutra Bulk Ingester Integration Guide

## 🚀 **Current Status: Foundation Complete**

✅ **IMPLEMENTED:**
- Rust core engine with async streaming
- FastAPI-like web server with job management
- Plugin system architecture
- Python Wikipedia adapter
- Docker multi-stage build
- Docker Compose integration
- Health checks and monitoring

⚠️ **NEEDS INTEGRATION:**
- TCP storage protocol connection
- PyO3 Python plugin bridge
- Full ecosystem testing

## 🐳 **Quick Docker Test**

### 1. Create Test Dataset
```bash
# Run the test to create a sample Wikipedia dataset
python3 test_docker_ingester.py
```

### 2. Start Full Ecosystem
```bash
# Start all 14 services including bulk ingester
docker-compose -f docker-compose-with-ingester.yml up -d

# Or start just the core services for testing
docker-compose -f docker-compose-grid.yml up -d storage-server
```

### 3. Test the Ingester
```bash
# Check health
curl http://localhost:8005/health

# List available adapters
curl http://localhost:8005/adapters

# Submit a Wikipedia ingestion job
curl -X POST http://localhost:8005/jobs \
  -H 'Content-Type: application/json' \
  -d '{
    "source_type": "file",
    "source_config": {"path": "/datasets/wikipedia.txt"},
    "adapter_name": "file"
  }'

# Check job status
curl http://localhost:8005/jobs
```

## 🎯 **Service Architecture**

```
🌐 BULK INGESTION SERVICES (NEW):
├── sutra-bulk-ingester:8005    # Main API server
├── ingester-worker-1           # Background worker 1  
└── ingester-worker-2           # Background worker 2

📊 INTEGRATED WITH:
├── storage-server:50051        # TCP storage
├── sutra-ollama:11434         # Embeddings
├── grid-master:7001           # Orchestration
└── sutra-control:9000         # Control Center (includes bulk ingester UI ✅)
```

## 🔧 **Next Steps to Complete Integration**

### 1. **Fix TCP Storage Connection**
```rust
// In storage.rs - connect to real TCP storage
impl TcpStorageClient {
    pub async fn new(server_address: &str) -> Result<Self> {
        // Use actual sutra-protocol package
        let client = sutra_protocol::StorageClient::connect(server_address).await?;
        Ok(Self { client })
    }
}
```

### 2. **Complete Python Plugin Bridge**
```rust
// In plugins.rs - add PyO3 integration
#[cfg(feature = "python-plugins")]
pub fn load_python_adapter(path: &Path) -> Result<Box<dyn IngestionAdapter>> {
    pyo3::Python::with_gil(|py| {
        let module = PyModule::from_code(py, &code, path.to_str().unwrap(), "adapter")?;
        let adapter_class = module.getattr("WikipediaAdapter")?;
        let adapter_instance = adapter_class.call0()?;
        Ok(Box::new(PythonAdapter::new(adapter_instance.into())))
    })
}
```

### 3. **Test with Real Wikipedia Dataset**
```bash
# Put real wikipedia.txt in datasets/ directory
ls -la datasets/wikipedia.txt

# Start ecosystem and test
docker-compose -f docker-compose-with-ingester.yml up -d
python3 test_docker_ingester.py
```

## 📋 **Performance Expectations**

With the Rust core + Python adapters architecture:

```
📊 EXPECTED PERFORMANCE:
├── File I/O:           10-20× faster than pure Python
├── Text Processing:    5-15× faster with Rust regex
├── Memory Usage:       2-5× lower with zero-copy ops
├── TCP Communication: 3-10× faster binary protocol
└── Concurrent Jobs:    10× more with async runtime

🎯 TARGET THROUGHPUT:
├── Small datasets:     1,000+ articles/minute
├── Large datasets:     10,000+ articles/minute  
├── Memory usage:       <4GB per worker
└── Scalability:        Multiple workers per dataset
```

## 🚦 **Usage Examples**

### Submit Different Dataset Types
```bash
# Wikipedia dataset
curl -X POST localhost:8005/jobs -d '{
  "source_type": "file",
  "source_config": {"path": "/datasets/wikipedia.txt", "format": "wikipedia"},
  "adapter_name": "file"
}'

# Line-by-line text file
curl -X POST localhost:8005/jobs -d '{
  "source_type": "file", 
  "source_config": {"path": "/datasets/documents.txt", "format": "lines"},
  "adapter_name": "file"
}'

# JSON documents
curl -X POST localhost:8005/jobs -d '{
  "source_type": "file",
  "source_config": {"path": "/datasets/data.json", "format": "json"},
  "adapter_name": "file"
}'
```

### Monitor Progress
```bash
# List all jobs
curl localhost:8005/jobs

# Get specific job status  
curl localhost:8005/jobs/job_12345

# Monitor logs
docker logs -f sutra-bulk-ingester
```

## 🔍 **Troubleshooting**

### Common Issues

1. **"Storage server connection failed"**
   ```bash
   # Check storage server is running
   docker-compose ps storage-server
   # Check network connectivity
   docker exec sutra-bulk-ingester nc -z storage-server 50051
   ```

2. **"Plugin not found"**  
   ```bash
   # Check plugins directory
   docker exec sutra-bulk-ingester ls -la /app/plugins/
   # Verify plugin syntax
   python3 -c "import plugins.wikipedia_adapter"
   ```

3. **"Build failed"**
   ```bash
   # Check Rust dependencies
   docker build --no-cache -f packages/sutra-bulk-ingester/Dockerfile .
   # View build logs for specific errors
   ```

## 📈 **Future Enhancements**

- **Real-time streaming**: Kafka, WebSocket adapters
- **Database connectors**: PostgreSQL, MongoDB adapters  
- **Cloud storage**: S3, GCS adapters
- **GPU acceleration**: CUDA text processing
- **Auto-scaling**: Kubernetes HPA integration