"""
High-performance TCP storage client with connection pooling and adaptive backpressure.

Eliminates connection storm and implements intelligent request queuing to 
achieve sustained high throughput without resource exhaustion.
"""

import socket
import struct
import time
import logging
from typing import Dict, List, Optional, Tuple, Any
from threading import Lock, Event
from collections import deque
import msgpack

from .connection_pool import get_connection_pool

logger = logging.getLogger(__name__)

class AdaptiveBackpressure:
    """
    Intelligent backpressure control to prevent storage server overload.
    
    Monitors response times and automatically throttles requests when
    the server shows signs of stress.
    """
    
    def __init__(self):
        self.response_times = deque(maxlen=50)  # Last 50 response times
        self.slow_request_threshold = 10.0  # 10 seconds = slow
        self.critical_threshold = 20.0       # 20 seconds = critical
        self.throttle_factor = 1.0           # Current throttling (1.0 = no throttling)
        self.last_adjustment = time.time()
        self._lock = Lock()
    
    def record_response_time(self, response_time: float):
        """Record a response time for adaptive control."""
        with self._lock:
            self.response_times.append(response_time)
            
            # Adjust throttling every 5 responses
            if len(self.response_times) % 5 == 0:
                self._adjust_throttling()
    
    def _adjust_throttling(self):
        """Adjust throttling based on recent response times."""
        if not self.response_times:
            return
        
        current_time = time.time()
        if current_time - self.last_adjustment < 5.0:  # Don't adjust too frequently
            return
        
        # Calculate recent performance metrics
        recent_times = list(self.response_times)[-10:]  # Last 10 responses
        avg_time = sum(recent_times) / len(recent_times)
        slow_count = sum(1 for t in recent_times if t > self.slow_request_threshold)
        critical_count = sum(1 for t in recent_times if t > self.critical_threshold)
        
        old_factor = self.throttle_factor
        
        # Increase throttling if server is stressed
        if critical_count >= 3:  # 30%+ critical responses
            self.throttle_factor = min(10.0, self.throttle_factor * 2.0)
            logger.warning(f"Critical server load detected, throttling increased to {self.throttle_factor:.1f}x")
        elif slow_count >= 5:    # 50%+ slow responses
            self.throttle_factor = min(5.0, self.throttle_factor * 1.5)
            logger.warning(f"Slow server responses detected, throttling increased to {self.throttle_factor:.1f}x")
        elif avg_time < 2.0 and slow_count == 0:  # Server is healthy, reduce throttling
            self.throttle_factor = max(1.0, self.throttle_factor * 0.8)
            if old_factor > 1.0:
                logger.info(f"Server performance improved, throttling reduced to {self.throttle_factor:.1f}x")
        
        self.last_adjustment = current_time
    
    def get_delay(self) -> float:
        """Get current delay to apply before making request."""
        with self._lock:
            if self.throttle_factor <= 1.0:
                return 0.0
            
            # Base delay of 100ms, scaled by throttle factor
            base_delay = 0.1
            return base_delay * (self.throttle_factor - 1.0)

class HighPerformanceStorageClient:
    """
    High-performance storage client with connection pooling and adaptive backpressure.
    
    Designed to eliminate the connection storm and maintain sustained high throughput
    without causing storage server resource exhaustion.
    """
    
    def __init__(self, server_address: str = "localhost:50051"):
        """
        Initialize high-performance client.
        
        Args:
            server_address: Storage server address (host:port)
        """
        host, port = server_address.split(":")
        self.server_address = (host, int(port))
        self.connection_pool = get_connection_pool(self.server_address)
        self.backpressure = AdaptiveBackpressure()
        
        # Request queue for intelligent batching
        self.pending_requests = deque()
        self.request_lock = Lock()
        
        # Performance tracking
        self.stats = {
            'requests_sent': 0,
            'requests_failed': 0,
            'total_response_time': 0.0,
            'throttled_requests': 0,
            'connection_reuses': 0,
            'circuit_breaker_trips': 0
        }
        
        # Default options for learning
        self.default_learn_options = {
            "generate_embedding": True,
            "embedding_model": None,
            "extract_associations": True,
            "min_association_confidence": 0.5,
            "max_associations_per_concept": 10,
            "strength": 1.0,
            "confidence": 1.0,
        }
        
        logger.info(f"Initialized high-performance client for {server_address}")
    
    def _send_request(self, request: Dict) -> Dict:
        """
        Send a single request with connection pooling and adaptive backpressure.
        
        Args:
            request: Request dictionary
            
        Returns:
            Response dictionary
            
        Raises:
            ConnectionError: If unable to connect or communicate
            TimeoutError: If request times out
        """
        # Apply adaptive backpressure
        delay = self.backpressure.get_delay()
        if delay > 0:
            logger.debug(f"Applying backpressure delay: {delay:.2f}s")
            time.sleep(delay)
            self.stats['throttled_requests'] += 1
        
        start_time = time.time()
        connection = None
        
        try:
            # Get connection from pool
            connection = self.connection_pool.get_connection()
            if not connection:
                self.stats['circuit_breaker_trips'] += 1
                raise ConnectionError("Circuit breaker is open or pool exhausted")
            
            # Serialize request
            request_bytes = msgpack.packb(request)
            
            # Send request (length + payload)
            connection.settimeout(60.0)  # 60 second timeout for individual requests
            connection.sendall(struct.pack("!I", len(request_bytes)))
            connection.sendall(request_bytes)
            
            # Read response length
            response_length_bytes = b""
            while len(response_length_bytes) < 4:
                chunk = connection.recv(4 - len(response_length_bytes))
                if not chunk:
                    raise ConnectionError("Connection closed while reading response length")
                response_length_bytes += chunk
            
            response_length = struct.unpack("!I", response_length_bytes)[0]
            
            # Read response payload
            response_bytes = b""
            while len(response_bytes) < response_length:
                chunk = connection.recv(response_length - len(response_bytes))
                if not chunk:
                    raise ConnectionError("Connection closed while reading response")
                response_bytes += chunk
            
            # Deserialize response
            response = msgpack.unpackb(response_bytes, strict_map_key=False)
            # print(f"DEBUG: response: {response}") # For internal debugging if needed
            
            # Record successful request
            response_time = time.time() - start_time
            self.backpressure.record_response_time(response_time)
            self.stats['requests_sent'] += 1
            self.stats['total_response_time'] += response_time
            
            # Return connection to pool (healthy)
            self.connection_pool.return_connection(connection, healthy=True)
            self.stats['connection_reuses'] += 1
            
            return response
            
        except Exception as e:
            # Record failed request
            self.stats['requests_failed'] += 1
            response_time = time.time() - start_time
            self.backpressure.record_response_time(response_time)
            
            # Return connection to pool (unhealthy)
            if connection:
                self.connection_pool.return_connection(connection, healthy=False)
            
            logger.error(f"Request failed after {response_time:.2f}s: {e}")
            raise
    
    def learn_concept_v2(self, content: str, options: Optional[Dict] = None) -> str:
        """
        Learn a single concept using the V2 API.
        
        Args:
            content: Content to learn
            options: Learning options (optional)
            
        Returns:
            Concept ID string
        """
        if not options:
            options = self.default_learn_options
        
        request = {
            "LearnConceptV2": {
                "content": content,
                "options": options
            }
        }
        
        response = self._send_request(request)
        
        if "LearnConceptV2Ok" in response:
            return response["LearnConceptV2Ok"]["concept_id"]
        elif "Error" in response:
            raise RuntimeError(f"Storage error: {response['Error']['message']}")
        else:
            raise RuntimeError(f"Unexpected response: {response}")
    
    def learn_batch(self, contents: List[str], options: Optional[Dict] = None) -> List[str]:
        """
        Learn multiple concepts in batch (high-performance).
        
        Args:
            contents: List of content strings to learn
            options: Learning options (optional)
            
        Returns:
            List of concept ID strings
        """
        if not contents:
            return []
        
        if not options:
            options = self.default_learn_options
        
        request = {
            "LearnBatch": {
                "contents": contents,
                "options": options
            }
        }
        
        response = self._send_request(request)
        
        if "LearnBatchOk" in response:
            return response["LearnBatchOk"]["concept_ids"]
        elif "Error" in response:
            raise RuntimeError(f"Storage error: {response['Error']['message']}")
        else:
            raise RuntimeError(f"Unexpected response: {response}")
    
    def get_stats(self) -> Dict:
        """Get client performance statistics."""
        pool_stats = self.connection_pool.get_stats()
        
        avg_response_time = 0.0
        if self.stats['requests_sent'] > 0:
            avg_response_time = self.stats['total_response_time'] / self.stats['requests_sent']
        
        return {
            **self.stats,
            'avg_response_time': avg_response_time,
            'success_rate': (
                self.stats['requests_sent'] / 
                (self.stats['requests_sent'] + self.stats['requests_failed'])
                if (self.stats['requests_sent'] + self.stats['requests_failed']) > 0 else 0
            ),
            'pool_stats': pool_stats,
            'current_throttle': self.backpressure.throttle_factor,
        }
    
    def close(self):
        """Close the client and clean up resources."""
        # Connection pool will be cleaned up when process exits
        logger.info("High-performance storage client closed")

# For backward compatibility, provide a factory function
def create_optimized_client(server_address: str = "localhost:50051") -> HighPerformanceStorageClient:
    """Create an optimized storage client with connection pooling."""
    return HighPerformanceStorageClient(server_address)