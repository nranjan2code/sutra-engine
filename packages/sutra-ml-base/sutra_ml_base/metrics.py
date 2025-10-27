"""
Metrics Collection and Observability for Sutra ML Services

Provides production-grade metrics collection with:
- Request timing and throughput tracking
- Error rate monitoring and alerting
- Resource usage monitoring (CPU, memory, GPU)
- Edition-aware quota and limit tracking
- Performance trend analysis
"""

import logging
import time
from dataclasses import dataclass, field
from typing import Dict, List, Optional, Any
from threading import Lock
from collections import defaultdict, deque
from datetime import datetime, timedelta

logger = logging.getLogger(__name__)


@dataclass
class ServiceMetrics:
    """Service performance metrics snapshot"""
    # Request metrics
    total_requests: int = 0
    successful_requests: int = 0
    failed_requests: int = 0
    
    # Timing metrics  
    avg_response_time_ms: float = 0.0
    min_response_time_ms: float = 0.0
    max_response_time_ms: float = 0.0
    
    # Throughput metrics
    requests_per_second: float = 0.0
    
    # Error metrics
    error_rate: float = 0.0
    
    # Resource metrics
    memory_usage_gb: float = 0.0
    cpu_usage_percent: float = 0.0
    
    # Timestamp
    timestamp: str = field(default_factory=lambda: datetime.utcnow().isoformat())
    
    @property
    def success_rate(self) -> float:
        """Calculate success rate percentage"""
        if self.total_requests == 0:
            return 100.0
        return (self.successful_requests / self.total_requests) * 100


class MetricsCollector:
    """Thread-safe metrics collector for ML services"""
    
    def __init__(self, window_size_minutes: int = 60, max_history: int = 1000):
        """Initialize metrics collector
        
        Args:
            window_size_minutes: Size of sliding window for rate calculations
            max_history: Maximum number of historical records to keep
        """
        self.window_size = timedelta(minutes=window_size_minutes)
        self.max_history = max_history
        
        # Thread safety
        self._lock = Lock()
        
        # Request tracking
        self._request_times: deque = deque(maxlen=max_history)
        self._response_times: deque = deque(maxlen=max_history)
        self._error_counts: Dict[str, int] = defaultdict(int)
        self._endpoint_stats: Dict[str, Dict[str, Any]] = defaultdict(lambda: {
            'count': 0,
            'total_time': 0.0,
            'errors': 0
        })
        
        # Counters
        self._total_requests = 0
        self._successful_requests = 0
        self._failed_requests = 0
        
        # Performance tracking
        self._min_response_time = float('inf')
        self._max_response_time = 0.0
        self._total_response_time = 0.0
        
        # Resource tracking (set externally)
        self._memory_usage_gb = 0.0
        self._cpu_usage_percent = 0.0
        
        logger.info(f"Metrics collector initialized (window: {window_size_minutes}min)")
    
    def record_request(
        self, 
        endpoint: str, 
        method: str, 
        status_code: int, 
        processing_time_ms: float
    ):
        """Record a completed request
        
        Args:
            endpoint: API endpoint path
            method: HTTP method
            status_code: HTTP status code
            processing_time_ms: Processing time in milliseconds
        """
        with self._lock:
            now = datetime.utcnow()
            
            # Update counters
            self._total_requests += 1
            
            if 200 <= status_code < 400:
                self._successful_requests += 1
            else:
                self._failed_requests += 1
                self._error_counts[f"{status_code}"] += 1
            
            # Update timing stats
            self._total_response_time += processing_time_ms
            self._min_response_time = min(self._min_response_time, processing_time_ms)
            self._max_response_time = max(self._max_response_time, processing_time_ms)
            
            # Record for windowed calculations
            self._request_times.append((now, status_code))
            self._response_times.append((now, processing_time_ms))
            
            # Update per-endpoint stats
            endpoint_key = f"{method} {endpoint}"
            stats = self._endpoint_stats[endpoint_key]
            stats['count'] += 1
            stats['total_time'] += processing_time_ms
            
            if status_code >= 400:
                stats['errors'] += 1
    
    def record_error(self, error_type: str, error_message: str):
        """Record an application error
        
        Args:
            error_type: Type of error (e.g., "MODEL_ERROR", "VALIDATION_ERROR")
            error_message: Error message for logging
        """
        with self._lock:
            self._error_counts[error_type] += 1
        
        logger.error(f"Recorded error - {error_type}: {error_message}")
    
    def update_resource_usage(self, memory_gb: float, cpu_percent: float):
        """Update resource usage metrics
        
        Args:
            memory_gb: Current memory usage in GB
            cpu_percent: Current CPU usage percentage
        """
        with self._lock:
            self._memory_usage_gb = memory_gb
            self._cpu_usage_percent = cpu_percent
    
    def get_stats(self) -> ServiceMetrics:
        """Get current metrics snapshot"""
        with self._lock:
            # Calculate averages
            avg_response_time = (
                self._total_response_time / self._total_requests
                if self._total_requests > 0 else 0.0
            )
            
            min_response_time = (
                self._min_response_time 
                if self._min_response_time != float('inf') else 0.0
            )
            
            error_rate = (
                (self._failed_requests / self._total_requests) * 100
                if self._total_requests > 0 else 0.0
            )
            
            # Calculate requests per second (last minute)
            rps = self._calculate_requests_per_second()
            
            return ServiceMetrics(
                total_requests=self._total_requests,
                successful_requests=self._successful_requests,
                failed_requests=self._failed_requests,
                avg_response_time_ms=avg_response_time,
                min_response_time_ms=min_response_time,
                max_response_time_ms=self._max_response_time,
                requests_per_second=rps,
                error_rate=error_rate,
                memory_usage_gb=self._memory_usage_gb,
                cpu_usage_percent=self._cpu_usage_percent
            )
    
    def get_endpoint_stats(self) -> Dict[str, Dict[str, Any]]:
        """Get per-endpoint statistics"""
        with self._lock:
            stats = {}
            
            for endpoint, data in self._endpoint_stats.items():
                if data['count'] > 0:
                    avg_time = data['total_time'] / data['count']
                    error_rate = (data['errors'] / data['count']) * 100
                    
                    stats[endpoint] = {
                        'requests': data['count'],
                        'avg_response_time_ms': avg_time,
                        'errors': data['errors'],
                        'error_rate': error_rate,
                        'success_rate': 100 - error_rate
                    }
            
            return stats
    
    def get_error_summary(self) -> Dict[str, int]:
        """Get error counts by type"""
        with self._lock:
            return dict(self._error_counts)
    
    def _calculate_requests_per_second(self) -> float:
        """Calculate requests per second in the current window"""
        if not self._request_times:
            return 0.0
        
        now = datetime.utcnow()
        window_start = now - timedelta(minutes=1)
        
        # Count requests in the last minute
        recent_requests = sum(
            1 for timestamp, _ in self._request_times
            if timestamp >= window_start
        )
        
        return recent_requests / 60.0  # Per second
    
    def reset_stats(self):
        """Reset all statistics (useful for testing)"""
        with self._lock:
            self._request_times.clear()
            self._response_times.clear()
            self._error_counts.clear()
            self._endpoint_stats.clear()
            
            self._total_requests = 0
            self._successful_requests = 0
            self._failed_requests = 0
            
            self._min_response_time = float('inf')
            self._max_response_time = 0.0
            self._total_response_time = 0.0
            
            logger.info("Metrics reset")
    
    def export_prometheus_metrics(self) -> str:
        """Export metrics in Prometheus format (for monitoring integration)"""
        stats = self.get_stats()
        endpoint_stats = self.get_endpoint_stats()
        
        metrics = []
        
        # Service-level metrics
        metrics.append(f"sutra_ml_requests_total {stats.total_requests}")
        metrics.append(f"sutra_ml_requests_successful {stats.successful_requests}")
        metrics.append(f"sutra_ml_requests_failed {stats.failed_requests}")
        metrics.append(f"sutra_ml_response_time_avg_ms {stats.avg_response_time_ms}")
        metrics.append(f"sutra_ml_response_time_min_ms {stats.min_response_time_ms}")
        metrics.append(f"sutra_ml_response_time_max_ms {stats.max_response_time_ms}")
        metrics.append(f"sutra_ml_requests_per_second {stats.requests_per_second}")
        metrics.append(f"sutra_ml_error_rate_percent {stats.error_rate}")
        metrics.append(f"sutra_ml_memory_usage_gb {stats.memory_usage_gb}")
        metrics.append(f"sutra_ml_cpu_usage_percent {stats.cpu_usage_percent}")
        
        # Per-endpoint metrics
        for endpoint, data in endpoint_stats.items():
            # Sanitize endpoint name for Prometheus
            endpoint_label = endpoint.replace("/", "_").replace("-", "_")
            metrics.append(f'sutra_ml_endpoint_requests{{endpoint="{endpoint_label}"}} {data["requests"]}')
            metrics.append(f'sutra_ml_endpoint_avg_time{{endpoint="{endpoint_label}"}} {data["avg_response_time_ms"]}')
            metrics.append(f'sutra_ml_endpoint_errors{{endpoint="{endpoint_label}"}} {data["errors"]}')
        
        return "\n".join(metrics) + "\n"