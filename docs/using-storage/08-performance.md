# Performance Guide

This guide covers optimization strategies, benchmarking, and troubleshooting techniques to maximize Sutra Storage performance for your AI reasoning workloads.

## Performance Overview

### Target Performance Metrics

| Operation | Target Latency | Target Throughput | Notes |
|-----------|----------------|-------------------|-------|
| Concept Read | <0.01ms | 10M ops/sec | Memory-mapped access |
| Concept Write | <1ms | 57K ops/sec | With WAL durability |
| Vector Search (k=10) | <1ms | 12K ops/sec | HNSW index |
| Association Query | <0.01ms | 5M ops/sec | Fixed-size records |
| Batch Learning (100) | <50ms | 2K batches/sec | Unified pipeline |

### Hardware Requirements

#### Minimum (Development)
```yaml
CPU: 4 cores, 2.0GHz
Memory: 8GB RAM
Storage: 50GB SSD
Network: 1Gbps
```

#### Recommended (Production)
```yaml
CPU: 16+ cores, 3.0GHz+ (Intel/AMD)
Memory: 64GB+ RAM (128GB for large datasets)
Storage: 500GB+ NVMe SSD (3000+ IOPS)
Network: 10Gbps+ with low latency
```

#### Optimal (High Performance)
```yaml
CPU: 32+ cores, 3.5GHz+ with high cache
Memory: 256GB+ RAM, DDR4-3200+
Storage: 2TB+ NVMe SSD, 100K+ IOPS
Network: 25Gbps+ with RDMA support
```

## Storage Optimization

### 1. Memory Configuration

#### OS-Level Tuning

```bash
# /etc/sysctl.conf optimizations
# Reduce swapping (keep data in RAM)
vm.swappiness=1

# Optimize file system cache
vm.vfs_cache_pressure=50

# Increase maximum memory map areas
vm.max_map_count=262144

# Optimize network buffers
net.core.rmem_default=134217728
net.core.rmem_max=134217728
net.core.wmem_default=134217728  
net.core.wmem_max=134217728

# Apply changes
sudo sysctl -p
```

#### Memory Allocation Strategy

```rust
// Rust storage configuration
pub struct StorageConfig {
    // Memory-mapped file settings
    pub mmap_populate: bool,           // Pre-fault pages
    pub mmap_huge_pages: bool,         // Use 2MB pages
    pub mmap_lock: bool,               // Lock in RAM (no swap)
    
    // Buffer pool configuration
    pub buffer_pool_size_mb: usize,    // RAM for hot data
    pub wal_buffer_size_mb: usize,     // WAL write buffer
    pub index_cache_size_mb: usize,    // HNSW index cache
    
    // Concurrency settings
    pub max_concurrent_reads: usize,    // Reader threads
    pub max_concurrent_writes: usize,   // Writer threads
    pub batch_size_limit: usize,       // Max batch size
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            mmap_populate: true,
            mmap_huge_pages: true,
            mmap_lock: false,  // Set true for ultra-low latency
            
            buffer_pool_size_mb: 1024,    // 1GB buffer
            wal_buffer_size_mb: 256,      // 256MB WAL buffer
            index_cache_size_mb: 512,     // 512MB index cache
            
            max_concurrent_reads: 64,
            max_concurrent_writes: 16,
            batch_size_limit: 1000,
        }
    }
}
```

### 2. File System Optimization

#### File System Selection
```bash
# XFS (recommended for large files)
mkfs.xfs -f -K /dev/nvme0n1p1
mount -o noatime,nobarrier,logbufs=8,logbsize=32k /dev/nvme0n1p1 /data

# ext4 (good general purpose)
mkfs.ext4 -F /dev/nvme0n1p1
mount -o noatime,nobarrier,data=writeback /dev/nvme0n1p1 /data

# ZFS (advanced features, higher overhead)
zpool create -o ashift=12 storage /dev/nvme0n1p1
zfs set compression=lz4 storage
zfs set recordsize=64K storage
```

#### Directory Layout Optimization
```bash
# Optimal storage layout
/data/storage/
├── domain/                 # Domain knowledge storage
│   ├── storage.dat         # Main data (on fastest SSD)
│   ├── vectors.idx         # HNSW index (on fast SSD)
│   └── wal.log            # WAL (on separate disk for safety)
├── user/                   # Multi-tenant user data
│   ├── storage.dat
│   ├── vectors.idx
│   └── wal.log
└── temp/                   # Temporary files (RAM disk)
    ├── compaction/
    └── exports/

# Optional: Use RAM disk for temporary files
mkdir /tmp/sutra-temp
mount -t tmpfs -o size=4G tmpfs /tmp/sutra-temp
```

### 3. I/O Optimization

#### Direct I/O and Async Operations

```rust
use tokio::fs::OpenOptions;
use std::os::unix::fs::OpenOptionsExt;

pub async fn optimized_file_operations() -> Result<()> {
    // Use direct I/O for large sequential writes (WAL)
    let wal_file = OpenOptions::new()
        .write(true)
        .create(true)
        .custom_flags(libc::O_DIRECT | libc::O_SYNC)
        .open("storage/wal.log")
        .await?;
    
    // Use async I/O for concurrent operations
    let storage_file = OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(libc::O_ASYNC)
        .open("storage/storage.dat")
        .await?;
    
    Ok(())
}
```

#### Batch I/O Operations

```python
# Python client optimizations
class HighPerformanceClient:
    def __init__(self, server_address: str):
        self.client = StorageClient(server_address)
        self.pending_operations = []
        self.batch_threshold = 100
    
    def learn_concept_buffered(self, content: str) -> str:
        """Buffer concept learning for batch processing."""
        self.pending_operations.append(('learn', content))
        
        if len(self.pending_operations) >= self.batch_threshold:
            return self._flush_operations()
        
        # Return estimated concept ID for immediate use
        import hashlib
        return hashlib.md5(content.encode()).hexdigest()
    
    def _flush_operations(self) -> List[str]:
        """Execute batched operations."""
        if not self.pending_operations:
            return []
        
        # Separate operation types
        learn_ops = [op[1] for op in self.pending_operations if op[0] == 'learn']
        
        concept_ids = []
        if learn_ops:
            # Use batch learning for efficiency
            batch_ids = self.client.learn_batch_v2(learn_ops)
            concept_ids.extend(batch_ids)
        
        self.pending_operations.clear()
        return concept_ids

# Usage
high_perf_client = HighPerformanceClient("localhost:50051")

# Operations are automatically batched
for i in range(500):
    concept_id = high_perf_client.learn_concept_buffered(f"Medical fact {i}")

# Force flush remaining operations
remaining = high_perf_client._flush_operations()
```

## Network Optimization

### 1. Connection Pooling

```python
import asyncio
import aiohttp
from typing import List, Optional

class ConnectionPoolManager:
    def __init__(self, server_addresses: List[str], pool_size: int = 10):
        self.server_addresses = server_addresses
        self.pool_size = pool_size
        self.connections = {}
        self.current_server = 0
        
    async def get_connection(self) -> StorageClient:
        """Get connection from pool with load balancing."""
        server_addr = self.server_addresses[self.current_server]
        self.current_server = (self.current_server + 1) % len(self.server_addresses)
        
        if server_addr not in self.connections:
            self.connections[server_addr] = []
        
        pool = self.connections[server_addr]
        
        # Reuse existing connection if available
        if pool:
            return pool.pop()
        
        # Create new connection if under limit
        if len([c for pool_list in self.connections.values() 
                for c in pool_list]) < self.pool_size:
            return StorageClient(server_addr)
        
        # Wait for connection to become available
        while not any(self.connections.values()):
            await asyncio.sleep(0.001)  # 1ms wait
        
        # Get from any available pool
        for pool_list in self.connections.values():
            if pool_list:
                return pool_list.pop()
    
    async def return_connection(self, connection: StorageClient, server_addr: str):
        """Return connection to pool."""
        if server_addr in self.connections:
            self.connections[server_addr].append(connection)
```

### 2. Protocol Optimization

#### TCP Settings

```bash
# Client-side TCP optimization
# Increase TCP buffer sizes
echo 'net.core.rmem_max = 134217728' >> /etc/sysctl.conf
echo 'net.core.wmem_max = 134217728' >> /etc/sysctl.conf
echo 'net.ipv4.tcp_rmem = 4096 87380 134217728' >> /etc/sysctl.conf
echo 'net.ipv4.tcp_wmem = 4096 65536 134217728' >> /etc/sysctl.conf

# Reduce latency
echo 'net.ipv4.tcp_low_latency = 1' >> /etc/sysctl.conf
echo 'net.core.busy_poll = 50' >> /etc/sysctl.conf

# Apply settings
sudo sysctl -p
```

#### Message Batching

```python
class BatchedStorageClient:
    def __init__(self, storage_client: StorageClient, batch_window_ms: int = 10):
        self.client = storage_client
        self.batch_window_ms = batch_window_ms
        self.pending_requests = []
        self.batch_timer = None
        
    async def learn_concept_v2_batched(self, content: str) -> str:
        """Add to batch and return future result."""
        future = asyncio.Future()
        self.pending_requests.append((content, future))
        
        # Start batch timer if not already running
        if self.batch_timer is None:
            self.batch_timer = asyncio.create_task(self._batch_timer())
        
        return await future
    
    async def _batch_timer(self):
        """Execute batch after timeout."""
        await asyncio.sleep(self.batch_window_ms / 1000.0)
        
        if self.pending_requests:
            contents = [req[0] for req in self.pending_requests]
            futures = [req[1] for req in self.pending_requests]
            
            try:
                # Execute batch
                concept_ids = self.client.learn_batch_v2(contents)
                
                # Resolve futures
                for future, concept_id in zip(futures, concept_ids):
                    future.set_result(concept_id)
                    
            except Exception as e:
                # Resolve all futures with error
                for future in futures:
                    future.set_exception(e)
            
            self.pending_requests.clear()
        
        self.batch_timer = None

# Usage with automatic batching
batched_client = BatchedStorageClient(client, batch_window_ms=5)

# Multiple concurrent operations get automatically batched
tasks = []
for i in range(50):
    task = batched_client.learn_concept_v2_batched(f"Concept {i}")
    tasks.append(task)

# All operations execute in efficient batches
concept_ids = await asyncio.gather(*tasks)
```

## Vector Search Optimization

### 1. HNSW Index Tuning

```rust
// Optimal HNSW parameters for different use cases
pub struct HNSWConfig {
    pub m: usize,              // Connections per node
    pub ef_construction: usize, // Build-time search width
    pub ml: f64,               // Level factor
    pub max_m: usize,          // Max connections per node
    pub max_m0: usize,         // Max connections for level 0
}

impl HNSWConfig {
    // Balanced performance (recommended)
    pub fn balanced() -> Self {
        Self {
            m: 16,
            ef_construction: 200,
            ml: 1.0 / (2.0_f64).ln(),
            max_m: 16,
            max_m0: 32,
        }
    }
    
    // High recall (slower build, better search quality)
    pub fn high_recall() -> Self {
        Self {
            m: 32,
            ef_construction: 400,
            ml: 1.0 / (2.0_f64).ln(),
            max_m: 32,
            max_m0: 64,
        }
    }
    
    // Fast search (faster build, lower recall)
    pub fn fast_search() -> Self {
        Self {
            m: 8,
            ef_construction: 100,
            ml: 1.0 / (2.0_f64).ln(),
            max_m: 8,
            max_m0: 16,
        }
    }
}
```

### 2. Query Optimization

```python
class OptimizedVectorSearch:
    def __init__(self, storage_client: StorageClient):
        self.client = storage_client
        self.query_cache = {}  # Simple LRU cache
        self.cache_size = 1000
        
    def search_with_cache(self, query_text: str, k: int = 10, 
                         ef_search: int = 50) -> List[Dict]:
        """Vector search with caching for repeated queries."""
        cache_key = f"{hash(query_text)}_{k}_{ef_search}"
        
        if cache_key in self.query_cache:
            return self.query_cache[cache_key]
        
        # Execute search
        results = self.client.vector_search(
            query_text=query_text,
            k=k,
            ef_search=ef_search
        )
        
        # Cache result (with size limit)
        if len(self.query_cache) >= self.cache_size:
            # Remove oldest entry
            oldest_key = next(iter(self.query_cache))
            del self.query_cache[oldest_key]
        
        self.query_cache[cache_key] = results
        return results
    
    def parallel_search(self, queries: List[str], k: int = 10) -> List[List[Dict]]:
        """Execute multiple searches in parallel."""
        import concurrent.futures
        
        with concurrent.futures.ThreadPoolExecutor(max_workers=8) as executor:
            futures = [
                executor.submit(self.search_with_cache, query, k)
                for query in queries
            ]
            
            results = []
            for future in concurrent.futures.as_completed(futures):
                try:
                    result = future.result(timeout=5)  # 5 second timeout
                    results.append(result)
                except Exception as e:
                    print(f"Search failed: {e}")
                    results.append([])  # Empty result on failure
        
        return results

# Usage
search_optimizer = OptimizedVectorSearch(client)

# Single search with caching
results = search_optimizer.search_with_cache(
    "heart disease prevention", 
    k=15, 
    ef_search=75
)

# Parallel batch search
medical_queries = [
    "diabetes management protocols",
    "hypertension treatment guidelines", 
    "cardiac emergency procedures",
    "preventive care recommendations"
]

batch_results = search_optimizer.parallel_search(medical_queries, k=10)
```

### 3. Index Maintenance

```python
def optimize_vector_index(client: StorageClient) -> Dict:
    """Perform index maintenance for optimal performance."""
    stats = {
        'before': {},
        'after': {},
        'optimization_time': 0.0,
        'actions_taken': []
    }
    
    start_time = time.time()
    
    try:
        # Get initial stats
        initial_stats = client.get_stats()
        stats['before'] = initial_stats
        
        # Force checkpoint to ensure consistency
        client.flush()
        stats['actions_taken'].append('Checkpoint completed')
        
        # Rebuild indexes if needed (server-side operation)
        # This would be a server admin operation
        print("Index optimization complete - server-side rebuilding may be needed")
        stats['actions_taken'].append('Index rebuild recommended')
        
        # Get final stats
        final_stats = client.get_stats()
        stats['after'] = final_stats
        
    except Exception as e:
        stats['error'] = str(e)
    finally:
        stats['optimization_time'] = time.time() - start_time
    
    return stats

# Scheduled maintenance
def schedule_maintenance():
    """Run maintenance during low-usage periods."""
    import schedule
    
    def maintenance_job():
        print("Starting scheduled maintenance...")
        results = optimize_vector_index(client)
        print(f"Maintenance completed in {results['optimization_time']:.2f}s")
        for action in results['actions_taken']:
            print(f"- {action}")
    
    # Schedule for 2 AM daily
    schedule.every().day.at("02:00").do(maintenance_job)
    
    # Run scheduler
    while True:
        schedule.run_pending()
        time.sleep(60)  # Check every minute

# Start maintenance scheduler in background
import threading
maintenance_thread = threading.Thread(target=schedule_maintenance, daemon=True)
maintenance_thread.start()
```

## Monitoring and Benchmarking

### 1. Performance Monitoring

```python
import psutil
import time
from dataclasses import dataclass
from typing import Dict, List

@dataclass
class PerformanceMetrics:
    timestamp: float
    cpu_percent: float
    memory_mb: float
    disk_read_mb: float
    disk_write_mb: float
    network_recv_mb: float
    network_sent_mb: float
    storage_ops_per_sec: float

class PerformanceMonitor:
    def __init__(self, storage_client: StorageClient):
        self.client = storage_client
        self.metrics_history: List[PerformanceMetrics] = []
        self.monitoring = False
        
    def start_monitoring(self, interval_seconds: float = 1.0):
        """Start continuous performance monitoring."""
        self.monitoring = True
        
        def monitor_loop():
            last_disk_io = psutil.disk_io_counters()
            last_net_io = psutil.net_io_counters()
            last_time = time.time()
            
            while self.monitoring:
                time.sleep(interval_seconds)
                
                current_time = time.time()
                time_delta = current_time - last_time
                
                # System metrics
                cpu_percent = psutil.cpu_percent()
                memory = psutil.virtual_memory()
                
                # I/O metrics
                current_disk_io = psutil.disk_io_counters()
                current_net_io = psutil.net_io_counters()
                
                disk_read_mb = (current_disk_io.read_bytes - last_disk_io.read_bytes) / 1024 / 1024 / time_delta
                disk_write_mb = (current_disk_io.write_bytes - last_disk_io.write_bytes) / 1024 / 1024 / time_delta
                net_recv_mb = (current_net_io.bytes_recv - last_net_io.bytes_recv) / 1024 / 1024 / time_delta
                net_sent_mb = (current_net_io.bytes_sent - last_net_io.bytes_sent) / 1024 / 1024 / time_delta
                
                # Storage-specific metrics
                try:
                    storage_stats = self.client.get_stats()
                    storage_ops_per_sec = storage_stats.get('operations_per_second', 0)
                except:
                    storage_ops_per_sec = 0
                
                # Record metrics
                metrics = PerformanceMetrics(
                    timestamp=current_time,
                    cpu_percent=cpu_percent,
                    memory_mb=memory.used / 1024 / 1024,
                    disk_read_mb=disk_read_mb,
                    disk_write_mb=disk_write_mb,
                    network_recv_mb=net_recv_mb,
                    network_sent_mb=net_sent_mb,
                    storage_ops_per_sec=storage_ops_per_sec
                )
                
                self.metrics_history.append(metrics)
                
                # Keep only last 1000 metrics
                if len(self.metrics_history) > 1000:
                    self.metrics_history.pop(0)
                
                last_disk_io = current_disk_io
                last_net_io = current_net_io
                last_time = current_time
        
        import threading
        self.monitor_thread = threading.Thread(target=monitor_loop, daemon=True)
        self.monitor_thread.start()
    
    def stop_monitoring(self):
        """Stop performance monitoring."""
        self.monitoring = False
    
    def get_performance_summary(self, duration_minutes: int = 5) -> Dict:
        """Get performance summary for recent period."""
        cutoff_time = time.time() - (duration_minutes * 60)
        recent_metrics = [m for m in self.metrics_history if m.timestamp > cutoff_time]
        
        if not recent_metrics:
            return {}
        
        return {
            'duration_minutes': duration_minutes,
            'samples': len(recent_metrics),
            'avg_cpu_percent': sum(m.cpu_percent for m in recent_metrics) / len(recent_metrics),
            'avg_memory_mb': sum(m.memory_mb for m in recent_metrics) / len(recent_metrics),
            'avg_disk_read_mb_s': sum(m.disk_read_mb for m in recent_metrics) / len(recent_metrics),
            'avg_disk_write_mb_s': sum(m.disk_write_mb for m in recent_metrics) / len(recent_metrics),
            'avg_storage_ops_per_sec': sum(m.storage_ops_per_sec for m in recent_metrics) / len(recent_metrics),
            'max_cpu_percent': max(m.cpu_percent for m in recent_metrics),
            'max_memory_mb': max(m.memory_mb for m in recent_metrics),
            'max_storage_ops_per_sec': max(m.storage_ops_per_sec for m in recent_metrics),
        }

# Usage
monitor = PerformanceMonitor(client)
monitor.start_monitoring(interval_seconds=0.5)  # Monitor every 500ms

# Run some operations...
time.sleep(60)  # Monitor for 1 minute

# Get performance report
summary = monitor.get_performance_summary(duration_minutes=1)
print(f"Performance Summary (last 1 minute):")
print(f"- Average CPU: {summary['avg_cpu_percent']:.1f}%")
print(f"- Average Memory: {summary['avg_memory_mb']:.0f} MB")
print(f"- Average Storage Ops/sec: {summary['avg_storage_ops_per_sec']:.0f}")
print(f"- Peak Storage Ops/sec: {summary['max_storage_ops_per_sec']:.0f}")

monitor.stop_monitoring()
```

### 2. Benchmarking Framework

```python
class StorageBenchmark:
    def __init__(self, storage_client: StorageClient):
        self.client = storage_client
        
    def benchmark_concept_learning(self, content_list: List[str], 
                                 batch_sizes: List[int] = [1, 10, 50, 100]) -> Dict:
        """Benchmark concept learning performance with different batch sizes."""
        results = {}
        
        for batch_size in batch_sizes:
            print(f"Benchmarking batch size {batch_size}...")
            
            start_time = time.time()
            concepts_processed = 0
            
            # Process in batches
            for i in range(0, len(content_list), batch_size):
                batch = content_list[i:i + batch_size]
                
                if batch_size == 1:
                    # Single concept learning
                    for content in batch:
                        concept_id = self.client.learn_concept_v2(content)
                        concepts_processed += 1
                else:
                    # Batch learning
                    concept_ids = self.client.learn_batch_v2(batch)
                    concepts_processed += len(concept_ids)
            
            end_time = time.time()
            total_time = end_time - start_time
            
            results[batch_size] = {
                'total_time_seconds': total_time,
                'concepts_processed': concepts_processed,
                'concepts_per_second': concepts_processed / total_time if total_time > 0 else 0,
                'avg_latency_ms': (total_time / concepts_processed * 1000) if concepts_processed > 0 else 0
            }
        
        return results
    
    def benchmark_vector_search(self, queries: List[str], 
                              k_values: List[int] = [1, 5, 10, 20],
                              ef_values: List[int] = [25, 50, 100]) -> Dict:
        """Benchmark vector search performance."""
        results = {}
        
        for k in k_values:
            for ef in ef_values:
                test_name = f"k={k}_ef={ef}"
                print(f"Benchmarking vector search: {test_name}")
                
                start_time = time.time()
                total_results = 0
                
                for query in queries:
                    search_results = self.client.vector_search(
                        query_text=query,
                        k=k,
                        ef_search=ef
                    )
                    total_results += len(search_results)
                
                end_time = time.time()
                total_time = end_time - start_time
                
                results[test_name] = {
                    'total_time_seconds': total_time,
                    'queries_processed': len(queries),
                    'queries_per_second': len(queries) / total_time if total_time > 0 else 0,
                    'avg_query_latency_ms': (total_time / len(queries) * 1000) if queries else 0,
                    'total_results_returned': total_results,
                    'avg_results_per_query': total_results / len(queries) if queries else 0
                }
        
        return results
    
    def run_full_benchmark(self, test_data: List[str]) -> Dict:
        """Run comprehensive benchmark suite."""
        print("Starting comprehensive storage benchmark...")
        
        benchmark_results = {
            'test_timestamp': time.time(),
            'test_data_size': len(test_data),
            'system_info': {
                'cpu_count': psutil.cpu_count(),
                'memory_gb': psutil.virtual_memory().total / 1024 / 1024 / 1024,
                'python_version': sys.version,
            }
        }
        
        # Benchmark learning
        print("Benchmarking concept learning...")
        learning_sample = test_data[:100]  # Use subset for learning benchmark
        benchmark_results['concept_learning'] = self.benchmark_concept_learning(learning_sample)
        
        # Learn all test data for search benchmarks
        print("Loading test data for search benchmarks...")
        concept_ids = self.client.learn_batch_v2(test_data[:500])  # Limit for benchmark
        
        # Benchmark vector search
        print("Benchmarking vector search...")
        search_queries = test_data[500:520] if len(test_data) > 520 else test_data[:10]
        benchmark_results['vector_search'] = self.benchmark_vector_search(search_queries)
        
        # Benchmark concept queries
        print("Benchmarking concept queries...")
        start_time = time.time()
        for concept_id in concept_ids[:100]:
            concept = self.client.query_concept(concept_id)
        
        query_time = time.time() - start_time
        benchmark_results['concept_queries'] = {
            'total_time_seconds': query_time,
            'queries_processed': min(100, len(concept_ids)),
            'queries_per_second': min(100, len(concept_ids)) / query_time if query_time > 0 else 0,
            'avg_latency_ms': query_time / min(100, len(concept_ids)) * 1000 if concept_ids else 0
        }
        
        return benchmark_results

# Usage
benchmark = StorageBenchmark(client)

# Prepare test data
medical_test_data = [
    "Hypertension management requires lifestyle modification and medication adherence.",
    "Diabetes screening should begin at age 35 for average-risk adults.",
    "Statins are recommended for primary prevention in high-risk patients.",
    # ... more test data
] * 10  # Repeat for larger dataset

# Run comprehensive benchmark
results = benchmark.run_full_benchmark(medical_test_data)

# Print results
print("\n=== Benchmark Results ===")
print(f"Test data size: {results['test_data_size']} items")
print(f"System: {results['system_info']['cpu_count']} CPUs, {results['system_info']['memory_gb']:.1f}GB RAM")

print("\nConcept Learning Performance:")
for batch_size, metrics in results['concept_learning'].items():
    print(f"  Batch size {batch_size}: {metrics['concepts_per_second']:.0f} concepts/sec, "
          f"{metrics['avg_latency_ms']:.2f}ms avg latency")

print("\nVector Search Performance:")
for config, metrics in results['vector_search'].items():
    print(f"  {config}: {metrics['queries_per_second']:.0f} queries/sec, "
          f"{metrics['avg_query_latency_ms']:.2f}ms avg latency")

print(f"\nConcept Query Performance: {results['concept_queries']['queries_per_second']:.0f} queries/sec")
```

## Next Steps

- [**Vector Search**](./07-vector-search.md) - Advanced semantic search optimization
- [**Troubleshooting**](./09-troubleshooting.md) - Solving performance issues
- [**Multi-Tenancy**](./06-multi-tenancy.md) - Scaling with organization isolation

---

*Performance optimization is crucial for production AI systems. These strategies ensure Sutra Storage delivers consistent sub-millisecond response times at scale.*