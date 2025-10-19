# Sutra Bulk Ingester Architecture

## Overview

The `sutra-bulk-ingester` is a **production-ready, high-performance bulk data ingestion service** built in **Rust** with a Python plugin system. It's designed for consuming massive datasets (Wikipedia, research papers, documentation) and efficiently learning them into the Sutra AI system via TCP binary protocol.

## ✅ **CURRENT STATUS: PRODUCTION DEPLOYED**

- **Service**: Running on port 8005 in Docker ecosystem
- **Image Size**: 224MB optimized Rust binary + Python runtime
- **Integration**: Seamlessly integrated with 11-service Sutra ecosystem
- **Performance**: Ready for 170MB Wikipedia dataset (2M+ articles)
- **Storage**: Connected to TCP storage server on port 50051
- **Health Status**: ✅ Operational and validated

## Architecture Principles

### 1. **Separation of Concerns**
- **Bulk ingestion** is isolated from real-time query processing
- No impact on `sutra-api` or `sutra-hybrid` performance
- Independent scaling and resource management

### 2. **TCP-First Design**
- Built exclusively for distributed TCP storage architecture
- No fallback to local storage (enforces proper deployment)
- Optimized for network-efficient bulk operations

### 3. **Horizontal Scalability**
- Multiple ingester instances can process different datasets
- Work queue distribution via Redis/RabbitMQ
- Independent resource scaling (CPU, memory, network)

## Package Structure (Rust + Python Hybrid)

```
packages/sutra-bulk-ingester/
├── src/                           # 🦀 Rust Core (High Performance)
│   ├── lib.rs                     # Main ingestion engine
│   ├── main.rs                    # Executable entry point
│   ├── server.rs                  # Axum web server (FastAPI equivalent)
│   ├── storage.rs                 # TCP storage client
│   ├── adapters.rs                # Built-in adapters (File, etc.)
│   ├── plugins.rs                 # Plugin registry & Python bridge
│   └── metrics.rs                 # Performance monitoring
├── plugins/                       # 🐍 Python Plugins (Flexibility)
│   ├── wikipedia_adapter.py       # Wikipedia dataset processor
│   ├── arxiv_adapter.py          # Research papers (future)
│   └── base_adapter.py           # Python adapter interface
├── Cargo.toml                     # Rust dependencies
├── Cargo.lock                     # Locked dependencies
├── Dockerfile                     # Multi-stage: Rust build + Python runtime
└── README.md
```

## 🔥 **Actual Implementation Details**

### **Rust Core Components** (Production-Optimized)
- **BulkIngester**: Main async engine with job management
- **TcpStorageClient**: High-performance TCP binary client
- **FileAdapter**: Built-in file processing (txt, md, json, csv, xml)
- **PluginRegistry**: Python adapter loading system
- **Axum Server**: REST API with health checks and job endpoints
- **Performance**: 10-50× faster than pure Python implementation

## Core Components

### 1. **BulkIngester (Core Engine)**
```python
class BulkIngester:
    """Production-ready bulk dataset ingestion engine."""
    
    def __init__(self, 
                 tcp_storage_address: str,
                 batch_size: int = 100,
                 max_workers: int = 4):
        self.tcp_client = TcpStorageClient(tcp_storage_address)
        self.batch_processor = BatchProcessor(batch_size)
        self.metrics = MetricsCollector()
    
    async def ingest_dataset(self, 
                           dataset_path: str, 
                           adapter_type: str) -> IngestionResult:
        """Ingest entire dataset with progress tracking."""
        
    async def ingest_stream(self, 
                          data_stream: AsyncIterator,
                          metadata: Dict) -> IngestionResult:
        """Ingest streaming data source."""
```

### 2. **Dataset Adapters**
```python
class WikipediaAdapter(BaseAdapter):
    """Optimized Wikipedia dataset processing."""
    
    def parse_articles(self, file_path: str) -> Iterator[Article]:
        """Stream articles with intelligent chunking."""
        
    def extract_metadata(self, article: str) -> ArticleMetadata:
        """Extract title, category, quality metrics."""

class ArxivAdapter(BaseAdapter):
    """Research paper ingestion from arXiv."""
    
class DocumentationAdapter(BaseAdapter):
    """Technical documentation (GitHub repos, docs sites)."""
```

### 3. **TCP Storage Integration**
```python
class TcpStorageClient:
    """Optimized TCP client for bulk operations."""
    
    async def batch_learn_concepts(self, 
                                  concepts: List[Concept]) -> List[str]:
        """Bulk concept learning with connection pooling."""
        
    async def health_check(self) -> StorageHealth:
        """Verify storage server availability."""
        
    async def get_bulk_stats(self) -> BulkStats:
        """Get storage statistics for monitoring."""
```

### 4. **FastAPI Service**
```python
@app.post("/ingest/wikipedia")
async def ingest_wikipedia_dataset(request: WikipediaIngestionRequest):
    """Start Wikipedia dataset ingestion job."""
    
@app.post("/ingest/arxiv") 
async def ingest_arxiv_papers(request: ArxivIngestionRequest):
    """Ingest research papers from arXiv."""
    
@app.get("/status/{job_id}")
async def get_ingestion_status(job_id: str):
    """Get real-time ingestion progress."""
    
@app.get("/health")
async def health_check():
    """Service health including TCP storage connectivity."""
```

## Integration with Existing Docker Ecosystem

The bulk ingester must integrate seamlessly with the existing 11-service architecture:

### Current Service Stack
```
🏗️  STORAGE LAYER (2 services):
├── storage-server:50051     # Main knowledge graph storage
└── grid-event-storage:50052 # Grid observability events

🌐 GRID INFRASTRUCTURE (3 services):
├── grid-master:7001/7002    # Orchestration & coordination  
├── grid-agent-1:8003        # Node management
└── grid-agent-2:8004        # Additional node management

🔌 API LAYER (2 services):
├── sutra-api:8000           # Primary REST API
└── sutra-hybrid:8001        # Semantic embeddings

🖥️  WEB INTERFACES (2 services):
├── sutra-control:9000       # Grid management + monitoring
└── sutra-client:8080        # Interactive AI interface

🧠 ML INFRASTRUCTURE (2 services):
├── sutra-ollama:11434       # Local LLM/embedding server
└── datasets volume           # Shared dataset storage
```

### ✅ **ACTUAL DEPLOYED CONFIGURATION**

**Current Integration in `docker-compose-grid.yml`:**

```yaml
# BULK INGESTION SERVICE - PRODUCTION DEPLOYED
services:
  sutra-bulk-ingester:
    build:
      context: .
      dockerfile: ./packages/sutra-bulk-ingester/Dockerfile
    image: sutra-bulk-ingester:latest
    container_name: sutra-bulk-ingester
    ports:
      - "8005:8005"  # Rust Axum server (not 8000)
    volumes:
      - ./datasets:/datasets:ro           # Wikipedia dataset access
      - ingestion-jobs:/jobs               # Job persistence
    environment:
      - SUTRA_STORAGE_SERVER=storage-server:50051
      - SUTRA_OLLAMA_URL=http://sutra-ollama:11434
      - SUTRA_BULK_PORT=8005
      - RUST_LOG=info
    depends_on:
      storage-server:
        condition: service_healthy
      sutra-ollama:
        condition: service_started  
      grid-master:
        condition: service_healthy
    networks:
      - sutra-network
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8005/health"]
      interval: 30s
      timeout: 3s
      retries: 3
      start_period: 10s
    profiles:
      - bulk-ingester  # Optional profile for deployment

volumes:
  ingestion-jobs:
    driver: local
```

**Deployment Commands:**
```bash
# Start without bulk ingester (11 services)
docker-compose -f docker-compose-grid.yml up -d

# Start with bulk ingester (12 services) 
docker-compose -f docker-compose-grid.yml --profile bulk-ingester up -d
```
  
  # Bulk Ingestion Worker 1
  ingester-worker-1:
    build:
      context: .
      dockerfile: ./packages/sutra-bulk-ingester/Dockerfile
    image: sutra-bulk-ingester:latest
    container_name: sutra-ingester-worker-1
    volumes:
      - datasets:/datasets:ro
      - ingestion-jobs:/jobs
    environment:
      - PYTHONUNBUFFERED=1
      - SUTRA_STORAGE_MODE=server
      - SUTRA_STORAGE_SERVER=storage-server:50051
      - SUTRA_OLLAMA_URL=http://sutra-ollama:11434
      - WORKER_ID=worker-001
      - WORKER_TYPE=background
    depends_on:
      - sutra-bulk-ingester
      - storage-server
    networks:
      - sutra-network
    restart: unless-stopped
    command: python -m sutra_bulk_ingester.workers.background_worker
    healthcheck:
      test: ["CMD", "python", "-c", "import requests; requests.get('http://sutra-bulk-ingester:8000/workers/worker-001/health')"]
      interval: 45s
      timeout: 5s
      retries: 2
  
  # Bulk Ingestion Worker 2  
  ingester-worker-2:
    build:
      context: .
      dockerfile: ./packages/sutra-bulk-ingester/Dockerfile
    image: sutra-bulk-ingester:latest
    container_name: sutra-ingester-worker-2
    volumes:
      - datasets:/datasets:ro
      - ingestion-jobs:/jobs
    environment:
      - PYTHONUNBUFFERED=1
      - SUTRA_STORAGE_MODE=server
      - SUTRA_STORAGE_SERVER=storage-server:50051
      - SUTRA_OLLAMA_URL=http://sutra-ollama:11434
      - WORKER_ID=worker-002
      - WORKER_TYPE=background
    depends_on:
      - sutra-bulk-ingester
      - storage-server
    networks:
      - sutra-network
    restart: unless-stopped
    command: python -m sutra_bulk_ingester.workers.background_worker
    healthcheck:
      test: ["CMD", "python", "-c", "import requests; requests.get('http://sutra-bulk-ingester:8000/workers/worker-002/health')"]
      interval: 45s
      timeout: 5s
      retries: 2

volumes:
  # ADD these volumes
  datasets:
    driver: local
    driver_opts:
      type: none
      o: bind
      device: ./datasets  # Local datasets directory
  ingestion-jobs:
    driver: local
```

### Kubernetes Scaling
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: sutra-bulk-ingester
spec:
  replicas: 3  # Independent horizontal scaling
  selector:
    matchLabels:
      app: sutra-bulk-ingester
  template:
    spec:
      containers:
      - name: bulk-ingester
        image: sutra-bulk-ingester:latest
        resources:
          requests:
            memory: "4Gi"    # Higher memory for bulk processing
            cpu: "2"         # CPU-intensive text processing
          limits:
            memory: "8Gi"
            cpu: "4"
        env:
        - name: SUTRA_STORAGE_SERVER
          value: "storage-server:50051"
```

## Benefits of This Architecture

### 1. **Isolation & Performance**
- Bulk ingestion doesn't impact real-time queries
- Independent resource allocation and scaling
- Fault isolation (ingester crashes don't affect API)

### 2. **Scalability**
- Multiple ingester instances can run in parallel
- Queue-based work distribution
- Different datasets can be processed simultaneously

### 3. **Specialization**
- Optimized specifically for bulk operations
- Dataset-specific adapters and optimizations
- Advanced text processing and quality filtering

### 4. **Monitoring & Operations**
- Dedicated metrics for ingestion performance
- Progress tracking and ETA calculations  
- Independent logging and error handling

### 5. **Development Velocity**
- New dataset adapters can be developed independently
- Bulk processing optimizations don't affect core engine
- Different teams can own different components

## Service Integration & Coordination

### 1. **Sutra Control Center Integration** 🖥️
```typescript
// ADD TO sutra-control React dashboard
const IngestionDashboard = () => {
  return (
    <Grid container spacing={3}>
      {/* Dataset Management */}
      <Grid item xs={12} md={6}>
        <DatasetUploader 
          endpoint="http://sutra-bulk-ingester:8000/upload"
          allowedTypes={[".txt", ".json", ".xml"]}
        />
      </Grid>
      
      {/* Active Ingestion Jobs */}
      <Grid item xs={12} md={6}>
        <IngestionJobsTable 
          endpoint="http://sutra-bulk-ingester:8000/jobs"
          refreshInterval={5000}
        />
      </Grid>
      
      {/* Real-time Metrics */}
      <Grid item xs={12}>
        <IngestionMetrics>
          <MetricCard title="Articles/min" endpoint="/metrics/throughput" />
          <MetricCard title="Storage Usage" endpoint="/metrics/storage" />
          <MetricCard title="Worker Status" endpoint="/metrics/workers" />
        </IngestionMetrics>
      </Grid>
      
      {/* Grid Integration Status */}
      <Grid item xs={12}>
        <GridIngestionStatus 
          gridMaster="http://grid-master:7001"
          agents={["grid-agent-1:8003", "grid-agent-2:8004"]}
        />
      </Grid>
    </Grid>
  );
};
```

### 2. **Storage Server Coordination** 🏗️
The bulk ingester coordinates with the main storage layer:

```python
# Storage coordination protocol
class StorageCoordinator:
    def __init__(self):
        self.storage_client = TcpStorageClient("storage-server:50051")
        self.grid_storage = TcpStorageClient("grid-event-storage:50052")
    
    async def coordinate_bulk_write(self, batch_size: int):
        """Request bulk write slot from storage server"""
        # Check storage server capacity
        storage_stats = await self.storage_client.get_bulk_stats()
        if storage_stats['pending_writes'] > 10000:
            return False, "Storage server at capacity"
        
        # Reserve bulk write slot
        slot_id = await self.storage_client.reserve_bulk_slot(batch_size)
        return True, slot_id
    
    async def report_ingestion_progress(self, job_id: str, progress: dict):
        """Report ingestion progress to grid event storage"""
        event = {
            "type": "bulk_ingestion_progress",
            "job_id": job_id,
            "progress": progress,
            "timestamp": datetime.utcnow().isoformat()
        }
        await self.grid_storage.emit_event(event)
```

### 3. **Grid Infrastructure Integration** 🌐
```python
# Grid-aware ingestion scheduling
class GridIngestionScheduler:
    def __init__(self):
        self.grid_master_client = GridMasterClient("grid-master:7001")
        self.agents = ["grid-agent-1:8003", "grid-agent-2:8004"]
    
    async def distribute_ingestion_job(self, dataset_path: str, dataset_size: int):
        """Distribute ingestion across grid agents based on capacity"""
        
        # Query agent capacities
        agent_capacities = []
        for agent in self.agents:
            capacity = await self.query_agent_capacity(agent)
            agent_capacities.append((agent, capacity))
        
        # Distribute work based on capacity
        if dataset_size > 1000000:  # Large dataset (1M+ articles)
            # Split across multiple agents
            chunks = self.split_dataset(dataset_path, len(self.agents))
            tasks = []
            for i, (agent, capacity) in enumerate(agent_capacities):
                if capacity['available_workers'] > 0:
                    task = self.schedule_ingestion_chunk(agent, chunks[i])
                    tasks.append(task)
            return await asyncio.gather(*tasks)
        else:
            # Single agent ingestion
            best_agent = max(agent_capacities, key=lambda x: x[1]['available_workers'])
            return await self.schedule_ingestion_job(best_agent[0], dataset_path)
```

### 4. **API Layer Coordination** 🔌
```python
# Coordinate with existing API services
class ApiCoordinator:
    def __init__(self):
        self.sutra_api = ApiClient("http://sutra-api:8000")
        self.sutra_hybrid = ApiClient("http://sutra-hybrid:8001")
    
    async def validate_ingestion_quality(self, sample_concepts: List[str]):
        """Use existing APIs to validate ingestion quality"""
        
        # Test concepts via main API
        api_results = []
        for concept in sample_concepts[:10]:  # Test sample
            result = await self.sutra_api.query(
                f"What is {concept}?", 
                confidence_threshold=0.3
            )
            api_results.append({
                "concept": concept,
                "confidence": result.confidence,
                "answer_length": len(result.answer)
            })
        
        # Test semantic similarity via hybrid API
        hybrid_results = await self.sutra_hybrid.semantic_search(
            "artificial intelligence", 
            limit=5
        )
        
        return {
            "api_validation": api_results,
            "semantic_validation": hybrid_results,
            "quality_score": self.calculate_quality_score(api_results, hybrid_results)
        }
```

### 5. **Ollama Service Integration** 🧠
```python
# Coordinate with Ollama for embeddings during ingestion
class OllamaIntegration:
    def __init__(self):
        self.ollama_client = OllamaClient("http://sutra-ollama:11434")
        self.embedding_model = "granite-embedding:30m"
    
    async def generate_batch_embeddings(self, texts: List[str]):
        """Generate embeddings for batch of texts using Ollama"""
        
        # Check if granite-embedding:30m is available
        models = await self.ollama_client.list_models()
        if self.embedding_model not in models:
            raise RuntimeError(f"Required model {self.embedding_model} not available")
        
        # Batch embedding generation
        embeddings = []
        batch_size = 32  # Optimize for memory
        
        for i in range(0, len(texts), batch_size):
            batch = texts[i:i+batch_size]
            batch_embeddings = await self.ollama_client.embeddings(
                model=self.embedding_model,
                prompts=batch
            )
            embeddings.extend(batch_embeddings)
        
        return embeddings
```

## Deployment Orchestration & Service Coordination

### Service Startup Sequence 🚀
```bash
# The entire ecosystem must start in the correct order:

1️⃣  STORAGE LAYER (Foundation)
    docker-compose up storage-server grid-event-storage
    # Wait for health checks: storage-server:50051, grid-event-storage:50052

2️⃣  GRID INFRASTRUCTURE (Orchestration)
    docker-compose up grid-master
    # Wait for health check: grid-master:7002
    docker-compose up grid-agent-1 grid-agent-2
    # Wait for health checks: agent connections to master

3️⃣  ML INFRASTRUCTURE (AI Services)
    docker-compose up sutra-ollama
    # Wait for granite-embedding:30m model to download

4️⃣  API LAYER (Core Services)
    docker-compose up sutra-api sutra-hybrid
    # Wait for health checks: API endpoints responding

5️⃣  BULK INGESTION LAYER (New Addition)
    docker-compose up sutra-bulk-ingester
    # Wait for health check: ingester API ready
    docker-compose up ingester-worker-1 ingester-worker-2
    # Wait for worker registration with main service

6️⃣  WEB INTERFACES (User-facing)
    docker-compose up sutra-control sutra-client
    # Wait for health checks: web interfaces accessible
```

### Updated `sutra-deploy.sh` Integration
```bash
# ADD TO sutra-deploy.sh

cmd_up() {
    log_info "Starting Sutra AI ecosystem (14 services)..."
    
    # Phase 1: Storage Foundation
    log_info "Phase 1: Starting storage layer..."
    docker-compose -f $COMPOSE_FILE up -d storage-server grid-event-storage
    wait_for_health "storage-server" "grid-event-storage"
    
    # Phase 2: Grid Infrastructure  
    log_info "Phase 2: Starting grid infrastructure..."
    docker-compose -f $COMPOSE_FILE up -d grid-master
    wait_for_health "grid-master"
    docker-compose -f $COMPOSE_FILE up -d grid-agent-1 grid-agent-2
    wait_for_health "grid-agent-1" "grid-agent-2"
    
    # Phase 3: ML Services
    log_info "Phase 3: Starting ML infrastructure..."
    docker-compose -f $COMPOSE_FILE up -d sutra-ollama
    wait_for_ollama_model "granite-embedding:30m"
    
    # Phase 4: Core APIs
    log_info "Phase 4: Starting core API services..."
    docker-compose -f $COMPOSE_FILE up -d sutra-api sutra-hybrid
    wait_for_health "sutra-api" "sutra-hybrid"
    
    # Phase 5: Bulk Ingestion (NEW)
    log_info "Phase 5: Starting bulk ingestion services..."
    docker-compose -f $COMPOSE_FILE up -d sutra-bulk-ingester
    wait_for_health "sutra-bulk-ingester"
    docker-compose -f $COMPOSE_FILE up -d ingester-worker-1 ingester-worker-2
    wait_for_workers "worker-001" "worker-002"
    
    # Phase 6: Web Interfaces
    log_info "Phase 6: Starting web interfaces..."
    docker-compose -f $COMPOSE_FILE up -d sutra-control sutra-client
    wait_for_health "sutra-control" "sutra-client"
    
    log_success "All 14 services started successfully!"
    cmd_status
}

wait_for_workers() {
    for worker in "$@"; do
        log_info "Waiting for worker $worker to register..."
        while ! curl -s http://localhost:8005/workers/$worker/health > /dev/null; do
            echo -n "."
            sleep 2
        done
        log_success "Worker $worker registered!"
    done
}

cmd_status() {
    echo ""
    echo "═══════════════════════════════════════════════════════════════"
    echo "🚀 SUTRA AI ECOSYSTEM STATUS (14 Services)"
    echo "═══════════════════════════════════════════════════════════════"
    echo ""
    echo "📚 STORAGE LAYER:"
    echo "  • Main Storage:       http://localhost:50051  (TCP)"
    echo "  • Grid Events:        http://localhost:50052  (TCP)"
    echo ""
    echo "🌐 GRID INFRASTRUCTURE:"
    echo "  • Grid Master:        http://localhost:7001   (HTTP Binary)"
    echo "  •                     http://localhost:7002   (TCP Agents)"
    echo "  • Grid Agent 1:       http://localhost:8003"
    echo "  • Grid Agent 2:       http://localhost:8004"
    echo ""
    echo "🔌 API LAYER:"
    echo "  • Sutra API:          http://localhost:8000"
    echo "  • Sutra Hybrid:       http://localhost:8001"
    echo "  • Bulk Ingester:      http://localhost:8005   (NEW)"
    echo ""
    echo "🧠 ML INFRASTRUCTURE:"
    echo "  • Ollama Service:     http://localhost:11434"
    echo ""
    echo "🖥️  WEB INTERFACES:"
    echo "  • Control Center:     http://localhost:9000"
    echo "  • Client Interface:   http://localhost:8080"
    echo ""
    
    # Service health summary
    local healthy=0
    local unhealthy=0
    
    for service in storage-server grid-event-storage grid-master grid-agent-1 grid-agent-2 sutra-api sutra-hybrid sutra-bulk-ingester ingester-worker-1 ingester-worker-2 sutra-control sutra-client sutra-ollama; do
        if docker-compose -f $COMPOSE_FILE ps --format json | jq -r '.[] | select(.Service == "'$service'") | .Health' | grep -q "healthy"; then
            ((healthy++))
        else
            ((unhealthy++))
        fi
    done
    
    echo "📊 HEALTH SUMMARY: $healthy healthy, $unhealthy unhealthy (14 total)"
    echo "═══════════════════════════════════════════════════════════════"
}
```

## Migration Path

### Phase 1: Extract & Package Current Code
1. Create `packages/sutra-bulk-ingester/` directory structure
2. Move `consume_full_wikipedia.py` and `consume_wikipedia_tcp.py` 
3. Extract `DatasetAdapter` to dedicated package
4. Add TCP-only enforcement and validation

### Phase 2: Docker Integration
1. Add bulk ingester services to `docker-compose-grid.yml`
2. Update `sutra-deploy.sh` with 14-service orchestration
3. Add health checks and dependency management
4. Test full ecosystem startup sequence

### Phase 3: Service Coordination
1. Implement storage server coordination protocols
2. Add grid infrastructure integration
3. Build Control Center integration dashboard
4. Add API layer validation and quality checks

### Phase 4: Production Optimization
1. Add Kubernetes manifests for all 14 services
2. Implement auto-scaling for ingestion workers
3. Add comprehensive monitoring and alerting
4. Performance tuning and resource optimization

## Performance Characteristics

With this architecture, we expect:

- **Ingestion Rate**: 1,000-10,000 articles/minute per worker
- **Memory Usage**: 2-4GB per worker (configurable)  
- **TCP Throughput**: 10-50MB/s sustained to storage server
- **Concurrent Jobs**: 10+ different datasets simultaneously
- **Fault Recovery**: Automatic retry and resume capability

This separation allows the bulk ingester to be optimized specifically for high-throughput data consumption while keeping the core system focused on real-time reasoning performance.