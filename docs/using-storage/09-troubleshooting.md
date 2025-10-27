# Troubleshooting

This guide helps diagnose and resolve common issues with Sutra Storage. It covers connection problems, performance issues, data consistency, and provides debugging tools and techniques.

## Quick Diagnosis

### Health Check Procedure

```python
from sutra_storage_client import StorageClient
import time

def quick_health_check(server_address: str = "localhost:50051") -> Dict:
    """Perform rapid health assessment of storage system."""
    health_report = {
        'server_address': server_address,
        'timestamp': time.time(),
        'connection': {'status': 'unknown', 'latency_ms': 0},
        'basic_operations': {'working': False, 'error': None},
        'storage_stats': {},
        'recommendations': []
    }
    
    try:
        # Test connection
        start_time = time.time()
        client = StorageClient(server_address)
        connect_time = (time.time() - start_time) * 1000
        
        health_report['connection'] = {'status': 'connected', 'latency_ms': connect_time}
        
        if connect_time > 100:
            health_report['recommendations'].append(f"High connection latency: {connect_time:.1f}ms")
        
        # Test basic operations
        try:
            # Health check
            health_response = client.health_check()
            if health_response.get('healthy', False):
                health_report['basic_operations']['working'] = True
            else:
                health_report['recommendations'].append("Server reports unhealthy status")
            
            # Get stats
            stats = client.get_stats()
            health_report['storage_stats'] = stats
            
            # Analyze stats
            if stats.get('pending', 0) > 1000:
                health_report['recommendations'].append(f"High pending operations: {stats['pending']}")
            
            if stats.get('dropped', 0) > 0:
                health_report['recommendations'].append(f"Dropped operations detected: {stats['dropped']}")
            
            # Test simple operation
            test_concept_id = client.learn_concept_v2("Health check test concept")
            if test_concept_id:
                # Verify we can read it back
                concept = client.query_concept(test_concept_id)
                if not concept.get('found', False):
                    health_report['recommendations'].append("Write succeeded but read failed - possible consistency issue")
            
        except Exception as e:
            health_report['basic_operations']['error'] = str(e)
            health_report['recommendations'].append(f"Basic operations failing: {e}")
        
    except Exception as e:
        health_report['connection']['status'] = 'failed'
        health_report['connection']['error'] = str(e)
        health_report['recommendations'].append(f"Cannot connect to server: {e}")
    
    return health_report

# Run health check
health = quick_health_check()
print(f"Health Check Report for {health['server_address']}:")
print(f"Connection: {health['connection']['status']} ({health['connection']['latency_ms']:.1f}ms)")
print(f"Basic operations: {'✓' if health['basic_operations']['working'] else '✗'}")

if health['storage_stats']:
    stats = health['storage_stats']
    print(f"Storage: {stats.get('concepts', 0)} concepts, {stats.get('edges', 0)} associations")
    print(f"Performance: {stats.get('uptime_seconds', 0)}s uptime")

if health['recommendations']:
    print("\nRecommendations:")
    for rec in health['recommendations']:
        print(f"  - {rec}")
```

## Connection Issues

### Problem: Cannot Connect to Storage Server

#### Symptoms
```
ConnectionRefusedError: [Errno 111] Connection refused
```

#### Diagnosis Steps

```bash
# 1. Check if storage server is running
ps aux | grep sutra-storage
# or
./sutra-deploy.sh status storage-server

# 2. Check port availability
netstat -tlnp | grep :50051
# or
ss -tlnp | grep :50051

# 3. Test network connectivity
telnet localhost 50051
# or
nc -zv localhost 50051

# 4. Check server logs
./sutra-deploy.sh logs storage-server --tail=50
```

#### Solutions

```bash
# Start storage server if not running
./sutra-deploy.sh start storage-server

# Check configuration
cat docker-compose-grid.yml | grep -A 10 -B 5 storage-server

# Verify port binding
docker ps | grep storage-server
# Should show: 0.0.0.0:50051->50051/tcp

# Restart with debug logging
RUST_LOG=debug ./sutra-deploy.sh restart storage-server
```

### Problem: Intermittent Connection Drops

#### Symptoms
```
ConnectionError: Connection closed by server
```

#### Advanced Connection Diagnostics

```python
import socket
import time
import statistics
from typing import List

def diagnose_connection_stability(server_address: str, test_duration: int = 60) -> Dict:
    """Test connection stability over time."""
    host, port = server_address.split(':')
    port = int(port)
    
    connection_times = []
    failures = []
    
    print(f"Testing connection stability for {test_duration} seconds...")
    
    end_time = time.time() + test_duration
    test_count = 0
    
    while time.time() < end_time:
        test_count += 1
        
        try:
            start = time.time()
            
            # Test TCP connection
            sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            sock.settimeout(5.0)
            sock.connect((host, port))
            
            connect_time = (time.time() - start) * 1000
            connection_times.append(connect_time)
            
            sock.close()
            
        except Exception as e:
            failures.append({
                'timestamp': time.time(),
                'error': str(e),
                'test_number': test_count
            })
        
        time.sleep(1)  # Test every second
    
    # Analysis
    if connection_times:
        avg_latency = statistics.mean(connection_times)
        std_latency = statistics.stdev(connection_times) if len(connection_times) > 1 else 0
        max_latency = max(connection_times)
        min_latency = min(connection_times)
    else:
        avg_latency = std_latency = max_latency = min_latency = 0
    
    return {
        'test_duration': test_duration,
        'total_tests': test_count,
        'successful_connections': len(connection_times),
        'failed_connections': len(failures),
        'success_rate': len(connection_times) / test_count * 100 if test_count > 0 else 0,
        'latency_stats': {
            'avg_ms': avg_latency,
            'std_ms': std_latency,
            'min_ms': min_latency,
            'max_ms': max_latency
        },
        'failures': failures[:5]  # First 5 failures for analysis
    }

# Usage
stability_test = diagnose_connection_stability("localhost:50051", test_duration=30)
print(f"Connection Stability Test Results:")
print(f"Success rate: {stability_test['success_rate']:.1f}%")
print(f"Average latency: {stability_test['latency_stats']['avg_ms']:.2f}ms ± {stability_test['latency_stats']['std_ms']:.2f}ms")

if stability_test['failures']:
    print(f"Failures detected ({len(stability_test['failures'])} shown):")
    for failure in stability_test['failures']:
        print(f"  - Test #{failure['test_number']}: {failure['error']}")
```

#### Solutions for Connection Stability

```python
class RobustStorageClient:
    """Storage client with automatic reconnection and error recovery."""
    
    def __init__(self, server_addresses: List[str], max_retries: int = 3, 
                 retry_delay: float = 1.0):
        self.server_addresses = server_addresses
        self.max_retries = max_retries
        self.retry_delay = retry_delay
        self.current_server_index = 0
        self.client = None
        self.consecutive_failures = 0
        
    def _get_next_server(self) -> str:
        """Round-robin server selection."""
        server = self.server_addresses[self.current_server_index]
        self.current_server_index = (self.current_server_index + 1) % len(self.server_addresses)
        return server
    
    def _connect(self) -> bool:
        """Establish connection with retry logic."""
        for attempt in range(len(self.server_addresses)):
            server = self._get_next_server()
            
            try:
                self.client = StorageClient(server)
                # Test connection
                self.client.health_check()
                
                print(f"Connected to {server}")
                self.consecutive_failures = 0
                return True
                
            except Exception as e:
                print(f"Failed to connect to {server}: {e}")
                continue
        
        return False
    
    def _execute_with_retry(self, operation_func, *args, **kwargs):
        """Execute operation with automatic retry and reconnection."""
        for retry in range(self.max_retries):
            try:
                if self.client is None and not self._connect():
                    raise ConnectionError("All servers unavailable")
                
                result = operation_func(*args, **kwargs)
                self.consecutive_failures = 0
                return result
                
            except (ConnectionError, OSError) as e:
                print(f"Connection error (attempt {retry + 1}): {e}")
                self.client = None  # Force reconnection
                self.consecutive_failures += 1
                
                if retry < self.max_retries - 1:
                    # Exponential backoff
                    delay = self.retry_delay * (2 ** retry)
                    print(f"Retrying in {delay:.1f} seconds...")
                    time.sleep(delay)
                else:
                    raise e
            
            except Exception as e:
                # Non-connection error, don't retry
                print(f"Operation error: {e}")
                raise e
    
    def learn_concept_v2(self, content: str, options: dict = None) -> str:
        return self._execute_with_retry(
            self.client.learn_concept_v2, content, options
        )
    
    def query_concept(self, concept_id: str) -> dict:
        return self._execute_with_retry(
            self.client.query_concept, concept_id
        )
    
    def vector_search(self, query_text: str = None, k: int = 10, ef_search: int = 50) -> list:
        return self._execute_with_retry(
            self.client.vector_search, 
            query_text=query_text, k=k, ef_search=ef_search
        )

# Usage with multiple servers for failover
robust_client = RobustStorageClient([
    "storage-01.company.com:50051",
    "storage-02.company.com:50051",
    "localhost:50051"  # Fallback to local
])

# Operations automatically handle connection failures
concept_id = robust_client.learn_concept_v2("Test concept with robust client")
```

## Performance Issues

### Problem: Slow Learning Operations

#### Symptoms
- Learning operations taking >5 seconds
- Batch operations slower than individual operations
- High memory usage during learning

#### Performance Diagnosis

```python
import psutil
import time
from dataclasses import dataclass

@dataclass
class OperationProfile:
    operation: str
    duration_ms: float
    cpu_before: float
    cpu_after: float
    memory_before_mb: float
    memory_after_mb: float
    
class PerformanceProfiler:
    def __init__(self, storage_client: StorageClient):
        self.client = storage_client
        self.profiles = []
    
    def profile_operation(self, operation_name: str, operation_func, *args, **kwargs):
        """Profile a single operation."""
        # Capture before metrics
        process = psutil.Process()
        cpu_before = process.cpu_percent()
        memory_before = process.memory_info().rss / 1024 / 1024
        
        # Execute operation
        start_time = time.time()
        try:
            result = operation_func(*args, **kwargs)
            success = True
        except Exception as e:
            result = None
            success = False
            print(f"Operation failed: {e}")
        
        end_time = time.time()
        
        # Capture after metrics
        cpu_after = process.cpu_percent()
        memory_after = process.memory_info().rss / 1024 / 1024
        
        profile = OperationProfile(
            operation=operation_name,
            duration_ms=(end_time - start_time) * 1000,
            cpu_before=cpu_before,
            cpu_after=cpu_after,
            memory_before_mb=memory_before,
            memory_after_mb=memory_after
        )
        
        self.profiles.append(profile)
        
        print(f"{operation_name}: {profile.duration_ms:.1f}ms, "
              f"Memory: {memory_before:.1f}→{memory_after:.1f}MB")
        
        return result if success else None
    
    def diagnose_learning_performance(self, test_content: List[str]) -> Dict:
        """Comprehensive learning performance analysis."""
        print("Diagnosing learning performance...")
        
        results = {
            'individual_learning': [],
            'batch_learning': {},
            'recommendations': []
        }
        
        # Test individual learning
        for i, content in enumerate(test_content[:5]):
            concept_id = self.profile_operation(
                f"learn_individual_{i}",
                self.client.learn_concept_v2,
                content
            )
            
            if self.profiles:
                results['individual_learning'].append(self.profiles[-1])
        
        # Test different batch sizes
        batch_sizes = [10, 25, 50, 100]
        for batch_size in batch_sizes:
            if len(test_content) >= batch_size:
                batch = test_content[:batch_size]
                
                concept_ids = self.profile_operation(
                    f"learn_batch_{batch_size}",
                    self.client.learn_batch_v2,
                    batch
                )
                
                if self.profiles:
                    profile = self.profiles[-1]
                    results['batch_learning'][batch_size] = {
                        'total_time_ms': profile.duration_ms,
                        'time_per_concept_ms': profile.duration_ms / batch_size,
                        'memory_increase_mb': profile.memory_after_mb - profile.memory_before_mb
                    }
        
        # Generate recommendations
        if results['individual_learning']:
            avg_individual_time = sum(p.duration_ms for p in results['individual_learning']) / len(results['individual_learning'])
            
            if avg_individual_time > 1000:  # >1 second
                results['recommendations'].append(f"Slow individual learning: {avg_individual_time:.0f}ms avg")
        
        # Find optimal batch size
        if results['batch_learning']:
            best_batch_size = min(results['batch_learning'].keys(), 
                                key=lambda size: results['batch_learning'][size]['time_per_concept_ms'])
            
            best_time = results['batch_learning'][best_batch_size]['time_per_concept_ms']
            results['recommendations'].append(f"Optimal batch size: {best_batch_size} ({best_time:.1f}ms per concept)")
        
        return results

# Usage
profiler = PerformanceProfiler(client)

test_data = [
    "Medical protocol for hypertension management in elderly patients.",
    "Diabetes screening guidelines for adults over 35 years of age.",
    "Cardiac rehabilitation program effectiveness in post-MI patients.",
    "Statin therapy recommendations for primary prevention of CVD.",
    "Blood pressure targets in patients with chronic kidney disease."
] * 5  # Repeat for larger dataset

performance_analysis = profiler.diagnose_learning_performance(test_data)

print("\nPerformance Analysis Results:")
if performance_analysis['individual_learning']:
    avg_time = sum(p.duration_ms for p in performance_analysis['individual_learning']) / len(performance_analysis['individual_learning'])
    print(f"Average individual learning time: {avg_time:.1f}ms")

print("\nBatch Learning Performance:")
for batch_size, metrics in performance_analysis['batch_learning'].items():
    print(f"  Batch size {batch_size}: {metrics['time_per_concept_ms']:.1f}ms per concept")

print("\nRecommendations:")
for rec in performance_analysis['recommendations']:
    print(f"  - {rec}")
```

#### Solutions for Learning Performance

```bash
# 1. Server-side optimizations
# Check server resource usage
docker stats sutra-storage-server

# Increase memory allocation
docker update --memory=4g sutra-storage-server

# Restart with performance settings
RUST_LOG=info SUTRA_BATCH_SIZE=100 ./sutra-deploy.sh restart storage-server

# 2. Client-side optimizations
```

```python
# Optimize client for bulk operations
class OptimizedBulkClient:
    def __init__(self, storage_client: StorageClient):
        self.client = storage_client
        
    def bulk_learn_optimized(self, contents: List[str], 
                           optimal_batch_size: int = 50) -> List[str]:
        """Optimized bulk learning with multiple strategies."""
        
        # Strategy 1: Remove duplicates
        unique_contents = list(dict.fromkeys(contents))  # Preserve order
        print(f"Removed {len(contents) - len(unique_contents)} duplicates")
        
        # Strategy 2: Sort by length (similar content together)
        unique_contents.sort(key=len)
        
        # Strategy 3: Process in optimal batches
        all_concept_ids = []
        failed_contents = []
        
        for i in range(0, len(unique_contents), optimal_batch_size):
            batch = unique_contents[i:i + optimal_batch_size]
            
            try:
                concept_ids = self.client.learn_batch_v2(batch)
                all_concept_ids.extend(concept_ids)
                
            except Exception as e:
                print(f"Batch failed, processing individually: {e}")
                
                # Fallback: process failed batch individually
                for content in batch:
                    try:
                        concept_id = self.client.learn_concept_v2(content)
                        all_concept_ids.append(concept_id)
                    except Exception as individual_error:
                        print(f"Individual learning failed: {individual_error}")
                        failed_contents.append(content)
        
        if failed_contents:
            print(f"Warning: {len(failed_contents)} concepts failed to learn")
        
        return all_concept_ids

# Usage
optimized_client = OptimizedBulkClient(client)
concept_ids = optimized_client.bulk_learn_optimized(large_content_list)
```

### Problem: Slow Vector Search

#### Diagnosis and Solutions

```python
def diagnose_vector_search_performance(client: StorageClient) -> Dict:
    """Analyze vector search performance across different parameters."""
    
    test_queries = [
        "heart disease prevention",
        "diabetes management protocol", 
        "hypertension treatment guidelines",
        "cardiac rehabilitation therapy"
    ]
    
    # Test different parameter combinations
    test_configs = [
        {'k': 5, 'ef_search': 25},
        {'k': 10, 'ef_search': 50},
        {'k': 20, 'ef_search': 100},
        {'k': 50, 'ef_search': 200},
    ]
    
    results = {}
    
    for config in test_configs:
        config_name = f"k={config['k']}_ef={config['ef_search']}"
        
        times = []
        result_counts = []
        
        for query in test_queries:
            start_time = time.time()
            
            try:
                search_results = client.vector_search(
                    query_text=query,
                    k=config['k'],
                    ef_search=config['ef_search']
                )
                
                query_time = (time.time() - start_time) * 1000
                times.append(query_time)
                result_counts.append(len(search_results))
                
            except Exception as e:
                print(f"Search failed for {config_name}: {e}")
        
        if times:
            results[config_name] = {
                'avg_latency_ms': sum(times) / len(times),
                'max_latency_ms': max(times),
                'min_latency_ms': min(times),
                'avg_results': sum(result_counts) / len(result_counts),
                'queries_per_second': 1000 / (sum(times) / len(times))
            }
    
    # Find optimal configuration
    if results:
        # Balance between speed and quality
        optimal_config = min(results.keys(), 
                           key=lambda cfg: results[cfg]['avg_latency_ms'] * (1 + 0.1 * (20 - results[cfg]['avg_results'])))
        
        print(f"Vector Search Performance Analysis:")
        for config, metrics in results.items():
            marker = " ← OPTIMAL" if config == optimal_config else ""
            print(f"  {config}: {metrics['avg_latency_ms']:.1f}ms avg, "
                  f"{metrics['queries_per_second']:.0f} q/s, "
                  f"{metrics['avg_results']:.1f} results{marker}")
    
    return results

# Run diagnosis
search_performance = diagnose_vector_search_performance(client)
```

## Data Consistency Issues

### Problem: Missing Concepts After Learning

#### Diagnosis

```python
def diagnose_data_consistency(client: StorageClient) -> Dict:
    """Check for data consistency issues."""
    
    print("Checking data consistency...")
    
    issues = {
        'missing_concepts': [],
        'orphaned_associations': [],
        'statistics_mismatch': False,
        'index_inconsistencies': []
    }
    
    # Test concept creation and retrieval
    test_concepts = []
    for i in range(10):
        content = f"Consistency test concept {i} - {time.time()}"
        
        try:
            concept_id = client.learn_concept_v2(content)
            test_concepts.append((concept_id, content))
            
            # Immediately try to read it back
            concept = client.query_concept(concept_id)
            
            if not concept.get('found', False):
                issues['missing_concepts'].append({
                    'concept_id': concept_id,
                    'content': content,
                    'issue': 'Created but not readable'
                })
            elif concept.get('content') != content:
                issues['missing_concepts'].append({
                    'concept_id': concept_id,
                    'expected_content': content,
                    'actual_content': concept.get('content'),
                    'issue': 'Content mismatch'
                })
                
        except Exception as e:
            issues['missing_concepts'].append({
                'content': content,
                'issue': f'Creation failed: {e}'
            })
    
    # Check vector search consistency
    if test_concepts:
        for concept_id, content in test_concepts[:3]:
            try:
                # Search for exact content
                search_results = client.vector_search(
                    query_text=content,
                    k=10,
                    ef_search=100
                )
                
                # The concept should appear in its own search results with high similarity
                found_self = any(result['concept_id'] == concept_id 
                               for result in search_results 
                               if result.get('similarity', 0) > 0.9)
                
                if not found_self:
                    issues['index_inconsistencies'].append({
                        'concept_id': concept_id,
                        'issue': 'Not found in own vector search'
                    })
                    
            except Exception as e:
                issues['index_inconsistencies'].append({
                    'concept_id': concept_id,
                    'issue': f'Vector search failed: {e}'
                })
    
    # Clean up test concepts
    print(f"Created {len(test_concepts)} test concepts for consistency check")
    
    return issues

# Run consistency check
consistency_issues = diagnose_data_consistency(client)

print("Data Consistency Check Results:")
if consistency_issues['missing_concepts']:
    print(f"  ⚠️  {len(consistency_issues['missing_concepts'])} concept issues found")
    for issue in consistency_issues['missing_concepts'][:3]:
        print(f"    - {issue}")

if consistency_issues['index_inconsistencies']:
    print(f"  ⚠️  {len(consistency_issues['index_inconsistencies'])} index issues found")
    for issue in consistency_issues['index_inconsistencies'][:3]:
        print(f"    - {issue}")

if not any(consistency_issues.values()):
    print("  ✅ No consistency issues detected")
```

#### Solutions for Data Consistency

```bash
# Force storage checkpoint
./sutra-deploy.sh exec storage-server "kill -USR1 1"

# Or via client
python3 -c "
from sutra_storage_client import StorageClient
client = StorageClient()
client.flush()
print('Checkpoint completed')
"

# Check WAL status
./sutra-deploy.sh logs storage-server | grep -i "wal\|checkpoint"

# Restart storage server to force recovery
./sutra-deploy.sh restart storage-server
```

### Problem: Association Inconsistencies

```python
def diagnose_association_issues(client: StorageClient, 
                              sample_concept_ids: List[str]) -> Dict:
    """Check for association-related issues."""
    
    issues = {
        'missing_neighbors': [],
        'asymmetric_associations': [],
        'broken_references': []
    }
    
    for concept_id in sample_concept_ids[:10]:
        try:
            # Get neighbors
            neighbors = client.get_neighbors(concept_id)
            
            # Check if neighbors exist
            for neighbor_id in neighbors:
                neighbor_concept = client.query_concept(neighbor_id)
                if not neighbor_concept.get('found', False):
                    issues['broken_references'].append({
                        'source_concept': concept_id,
                        'broken_neighbor': neighbor_id
                    })
                else:
                    # Check if association is bidirectional (where expected)
                    reverse_neighbors = client.get_neighbors(neighbor_id)
                    if concept_id not in reverse_neighbors:
                        # This might be expected for directional associations
                        pass
            
        except Exception as e:
            issues['missing_neighbors'].append({
                'concept_id': concept_id,
                'error': str(e)
            })
    
    return issues

# Get some concept IDs for testing
try:
    stats = client.get_stats()
    if stats.get('concepts', 0) > 0:
        # Create a few test concepts to get their IDs
        test_ids = []
        for i in range(5):
            concept_id = client.learn_concept_v2(f"Association test concept {i}")
            test_ids.append(concept_id)
        
        association_issues = diagnose_association_issues(client, test_ids)
        
        if any(association_issues.values()):
            print("Association Issues Found:")
            for issue_type, issues in association_issues.items():
                if issues:
                    print(f"  {issue_type}: {len(issues)} issues")
        else:
            print("✅ No association issues detected")

except Exception as e:
    print(f"Could not check associations: {e}")
```

## Server-Side Debugging

### Log Analysis

```bash
# Get recent logs with filtering
./sutra-deploy.sh logs storage-server --tail=100 | grep -E "(ERROR|WARN|panic)"

# Monitor logs in real-time
./sutra-deploy.sh logs storage-server --follow | grep -v "DEBUG"

# Analyze error patterns
./sutra-deploy.sh logs storage-server --since=1h | \
    grep "ERROR" | \
    sed 's/.*ERROR \([^:]*\):.*/\1/' | \
    sort | uniq -c | sort -nr

# Check for memory issues
./sutra-deploy.sh logs storage-server | grep -i "memory\|oom\|allocation"
```

### Server Resource Monitoring

```bash
# Monitor server container resources
docker stats sutra-storage-server --no-stream

# Check disk usage
./sutra-deploy.sh exec storage-server df -h

# Check memory usage inside container
./sutra-deploy.sh exec storage-server free -h

# Check storage file sizes
./sutra-deploy.sh exec storage-server ls -lh /data/storage/
```

### Advanced Debugging

```python
def debug_server_state(client: StorageClient) -> Dict:
    """Get comprehensive server debugging information."""
    
    debug_info = {
        'timestamp': time.time(),
        'client_connectivity': {},
        'server_stats': {},
        'performance_indicators': {},
        'error_indicators': []
    }
    
    try:
        # Test connectivity
        start = time.time()
        health = client.health_check()
        health_latency = (time.time() - start) * 1000
        
        debug_info['client_connectivity'] = {
            'health_check_latency_ms': health_latency,
            'server_healthy': health.get('healthy', False),
            'server_status': health.get('status', 'unknown'),
            'server_uptime': health.get('uptime_seconds', 0)
        }
        
        if health_latency > 100:
            debug_info['error_indicators'].append(f"High health check latency: {health_latency:.1f}ms")
        
        # Get detailed stats
        stats = client.get_stats()
        debug_info['server_stats'] = stats
        
        # Analyze stats for issues
        if stats.get('dropped', 0) > 0:
            debug_info['error_indicators'].append(f"Dropped operations: {stats['dropped']}")
        
        if stats.get('pending', 0) > 1000:
            debug_info['error_indicators'].append(f"High pending operations: {stats['pending']}")
        
        # Performance indicators
        concepts = stats.get('concepts', 0)
        edges = stats.get('edges', 0)
        uptime = stats.get('uptime_seconds', 1)
        
        debug_info['performance_indicators'] = {
            'concepts_per_second_avg': concepts / uptime if uptime > 0 else 0,
            'associations_per_concept': edges / concepts if concepts > 0 else 0,
            'data_density': 'high' if concepts > 100000 else 'medium' if concepts > 10000 else 'low'
        }
        
    except Exception as e:
        debug_info['error_indicators'].append(f"Failed to get server info: {e}")
    
    return debug_info

# Get comprehensive debug information
debug_info = debug_server_state(client)

print("Server Debug Information:")
print(f"Connectivity: {debug_info['client_connectivity']}")
print(f"Stats: {debug_info['server_stats']}")
print(f"Performance: {debug_info['performance_indicators']}")

if debug_info['error_indicators']:
    print("⚠️  Issues detected:")
    for issue in debug_info['error_indicators']:
        print(f"  - {issue}")
else:
    print("✅ No issues detected")
```

## Emergency Recovery Procedures

### Data Recovery from WAL

```bash
# Stop storage server
./sutra-deploy.sh stop storage-server

# Backup current state
cp -r /data/storage /backup/storage-$(date +%Y%m%d-%H%M%S)

# Check WAL integrity
./sutra-storage-tool verify-wal /data/storage/wal.log

# Recover from WAL if main storage is corrupted
./sutra-storage-tool recover-from-wal \
    --wal-path /data/storage/wal.log \
    --output-path /data/storage/storage-recovered.dat

# Replace main storage if recovery successful
if [ -f /data/storage/storage-recovered.dat ]; then
    mv /data/storage/storage.dat /data/storage/storage-corrupted.dat
    mv /data/storage/storage-recovered.dat /data/storage/storage.dat
fi

# Restart server
./sutra-deploy.sh start storage-server
```

### Emergency Client for Basic Operations

```python
class EmergencyClient:
    """Minimal client for emergency operations when main client fails."""
    
    def __init__(self, server_address: str):
        self.address = server_address
        
    def emergency_health_check(self) -> bool:
        """Basic TCP connection test."""
        import socket
        
        try:
            host, port = self.address.split(':')
            sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            sock.settimeout(5.0)
            result = sock.connect_ex((host, int(port)))
            sock.close()
            return result == 0
        except:
            return False
    
    def emergency_stats_dump(self) -> Dict:
        """Get basic stats using raw TCP."""
        import socket
        import struct
        
        try:
            host, port = self.address.split(':')
            sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            sock.connect((host, int(port)))
            
            # Send minimal stats request (implementation-specific)
            # This is a simplified example
            request = b'{"GetStats": {}}'
            sock.send(struct.pack('>I', len(request)))
            sock.send(request)
            
            # Read response length
            length_bytes = sock.recv(4)
            if len(length_bytes) == 4:
                length = struct.unpack('>I', length_bytes)[0]
                
                # Read response
                response = sock.recv(length).decode()
                sock.close()
                
                return {'raw_response': response, 'status': 'success'}
            
            sock.close()
            return {'status': 'failed', 'reason': 'Invalid response'}
            
        except Exception as e:
            return {'status': 'failed', 'reason': str(e)}

# Emergency operations
emergency = EmergencyClient("localhost:50051")

if emergency.emergency_health_check():
    print("✅ Server accepting connections")
    stats = emergency.emergency_stats_dump()
    print(f"Emergency stats: {stats}")
else:
    print("❌ Server not responding to connections")
```

## Getting Help

### Information to Collect Before Seeking Support

```python
def collect_support_info(client: StorageClient) -> str:
    """Collect comprehensive information for support requests."""
    
    import platform
    import sys
    
    info_lines = [
        "=== Sutra Storage Support Information ===",
        f"Timestamp: {time.strftime('%Y-%m-%d %H:%M:%S UTC', time.gmtime())}",
        "",
        "System Information:",
        f"  OS: {platform.system()} {platform.release()}",
        f"  Python: {sys.version}",
        f"  Architecture: {platform.machine()}",
        ""
    ]
    
    try:
        health = client.health_check()
        stats = client.get_stats()
        
        info_lines.extend([
            "Server Information:",
            f"  Healthy: {health.get('healthy', False)}",
            f"  Status: {health.get('status', 'unknown')}",
            f"  Uptime: {health.get('uptime_seconds', 0)} seconds",
            f"  Concepts: {stats.get('concepts', 0)}",
            f"  Associations: {stats.get('edges', 0)}",
            f"  Pending operations: {stats.get('pending', 0)}",
            f"  Dropped operations: {stats.get('dropped', 0)}",
            ""
        ])
        
    except Exception as e:
        info_lines.extend([
            "Server Information:",
            f"  ERROR: Could not connect to server: {e}",
            ""
        ])
    
    # Recent error patterns (if available)
    info_lines.extend([
        "Recent Operations:",
        "  (Include details of operations that failed)",
        "",
        "Error Messages:",
        "  (Include full error messages and stack traces)",
        ""
    ])
    
    return "\n".join(info_lines)

# Generate support information
support_info = collect_support_info(client)
print(support_info)

# Save to file
with open(f"sutra-support-info-{int(time.time())}.txt", "w") as f:
    f.write(support_info)
    print(f"Support information saved to {f.name}")
```

### Common Support Scenarios

| Issue | Urgency | First Steps |
|-------|---------|-------------|
| Cannot connect | High | Check server status, network, logs |
| Slow performance | Medium | Run performance diagnostics, check resources |
| Data inconsistency | High | Run consistency checks, backup data |
| Memory errors | High | Check available RAM, restart server |
| Disk space full | High | Clean up old WAL files, add storage |
| Association errors | Medium | Check concept relationships, rebuild indexes |

---

*This troubleshooting guide covers the most common issues encountered with Sutra Storage. For additional help, collect the support information and consult the development team or documentation repository.*