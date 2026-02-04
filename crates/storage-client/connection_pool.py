"""
High-performance connection pool for TCP storage clients.

Implements intelligent connection reuse, adaptive pooling, and circuit breaker
patterns to eliminate the connection storm that causes 25-30 second timeouts.
"""

import socket
import threading
import time
from collections import deque
from typing import Optional, Dict
import logging

logger = logging.getLogger(__name__)

class ConnectionPool:
    """
    Thread-safe connection pool with adaptive sizing and circuit breaker.
    
    Eliminates the connection storm that causes storage server exhaustion
    by reusing TCP connections intelligently.
    """
    
    def __init__(self, 
                 server_address: tuple,
                 min_connections: int = 2,
                 max_connections: int = 10,
                 max_idle_time: float = 300.0,  # 5 minutes
                 health_check_interval: float = 60.0):  # 1 minute
        """
        Initialize connection pool.
        
        Args:
            server_address: (host, port) tuple
            min_connections: Minimum connections to maintain
            max_connections: Maximum connections allowed
            max_idle_time: Max idle time before closing connection
            health_check_interval: How often to check connection health
        """
        self.server_address = server_address
        self.min_connections = min_connections
        self.max_connections = max_connections
        self.max_idle_time = max_idle_time
        self.health_check_interval = health_check_interval
        
        # Thread-safe connection storage
        self._available = deque()  # Available connections
        self._in_use = set()       # Connections currently in use
        self._connection_info = {}  # Connection metadata
        self._lock = threading.RLock()
        self._next_health_check = time.time() + health_check_interval
        
        # Circuit breaker state
        self._failed_attempts = 0
        self._last_failure_time = 0
        self._circuit_open = False
        
        # Statistics
        self.stats = {
            'created': 0,
            'reused': 0,
            'closed': 0,
            'failed': 0,
            'circuit_trips': 0
        }
        
        # Initialize minimum connections
        self._initialize_pool()
    
    def _initialize_pool(self):
        """Create initial pool connections."""
        with self._lock:
            for _ in range(self.min_connections):
                try:
                    conn = self._create_connection()
                    if conn:
                        self._available.append(conn)
                except Exception as e:
                    logger.warning(f"Failed to create initial connection: {e}")
    
    def _create_connection(self) -> Optional[socket.socket]:
        """Create a new TCP connection to storage server."""
        try:
            sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            sock.settimeout(30.0)  # 30 second timeout
            sock.setsockopt(socket.IPPROTO_TCP, socket.TCP_NODELAY, 1)  # Disable Nagle
            sock.connect(self.server_address)
            
            # Track connection info
            conn_id = id(sock)
            self._connection_info[conn_id] = {
                'created_at': time.time(),
                'last_used': time.time(),
                'use_count': 0
            }
            
            self.stats['created'] += 1
            logger.debug(f"Created new connection {conn_id}")
            return sock
            
        except Exception as e:
            logger.error(f"Failed to create connection: {e}")
            self.stats['failed'] += 1
            self._handle_connection_failure()
            return None
    
    def _handle_connection_failure(self):
        """Handle connection failure for circuit breaker."""
        self._failed_attempts += 1
        self._last_failure_time = time.time()
        
        # Open circuit if too many failures
        if self._failed_attempts >= 5:
            self._circuit_open = True
            self.stats['circuit_trips'] += 1
            logger.warning("Circuit breaker opened due to connection failures")
    
    def _is_circuit_open(self) -> bool:
        """Check if circuit breaker is open."""
        if not self._circuit_open:
            return False
        
        # Auto-reset after 30 seconds
        if time.time() - self._last_failure_time > 30:
            self._circuit_open = False
            self._failed_attempts = 0
            logger.info("Circuit breaker reset")
            return False
        
        return True
    
    def _is_connection_healthy(self, sock: socket.socket) -> bool:
        """Check if connection is still healthy."""
        try:
            # Use SO_ERROR to check for socket errors
            error = sock.getsockopt(socket.SOL_SOCKET, socket.SO_ERROR)
            if error != 0:
                return False
            
            # Check if connection is still alive with non-blocking peek
            sock.settimeout(0.1)
            try:
                data = sock.recv(1, socket.MSG_PEEK | socket.MSG_DONTWAIT)
                return True  # Connection has data or is alive
            except socket.error as e:
                if e.errno in (socket.EAGAIN, socket.EWOULDBLOCK):
                    return True  # No data available, but connection is alive
                return False  # Connection is dead
                
        except Exception:
            return False
    
    def get_connection(self) -> Optional[socket.socket]:
        """
        Get a connection from the pool.
        
        Returns:
            TCP socket connection or None if circuit breaker is open
        """
        if self._is_circuit_open():
            logger.warning("Circuit breaker is open, rejecting request")
            return None
        
        with self._lock:
            # Clean up and health check if needed
            if time.time() > self._next_health_check:
                self._cleanup_idle_connections()
                self._next_health_check = time.time() + self.health_check_interval
            
            # Try to reuse existing connection
            while self._available:
                conn = self._available.popleft()
                conn_id = id(conn)
                
                # Check if connection is healthy
                if self._is_connection_healthy(conn):
                    self._in_use.add(conn)
                    self._connection_info[conn_id]['last_used'] = time.time()
                    self._connection_info[conn_id]['use_count'] += 1
                    self.stats['reused'] += 1
                    logger.debug(f"Reused connection {conn_id}")
                    return conn
                else:
                    # Connection is dead, clean it up
                    self._close_connection(conn)
            
            # Create new connection if under limit
            if len(self._in_use) < self.max_connections:
                conn = self._create_connection()
                if conn:
                    self._in_use.add(conn)
                    # Reset circuit breaker on successful connection
                    self._failed_attempts = 0
                    return conn
            
            logger.warning(f"Connection pool exhausted ({len(self._in_use)}/{self.max_connections})")
            return None
    
    def return_connection(self, conn: socket.socket, healthy: bool = True):
        """
        Return a connection to the pool.
        
        Args:
            conn: TCP socket to return
            healthy: Whether the connection is still healthy
        """
        with self._lock:
            self._in_use.discard(conn)
            
            if healthy and self._is_connection_healthy(conn):
                self._available.append(conn)
                logger.debug(f"Returned connection {id(conn)} to pool")
            else:
                self._close_connection(conn)
                logger.debug(f"Closed unhealthy connection {id(conn)}")
    
    def _close_connection(self, conn: socket.socket):
        """Close connection and clean up metadata."""
        try:
            conn.close()
            conn_id = id(conn)
            if conn_id in self._connection_info:
                del self._connection_info[conn_id]
            self.stats['closed'] += 1
        except Exception as e:
            logger.debug(f"Error closing connection: {e}")
    
    def _cleanup_idle_connections(self):
        """Remove idle connections that exceed max_idle_time."""
        current_time = time.time()
        connections_to_close = []
        
        # Check available connections for idle timeout
        for conn in list(self._available):
            conn_id = id(conn)
            if conn_id in self._connection_info:
                idle_time = current_time - self._connection_info[conn_id]['last_used']
                if idle_time > self.max_idle_time:
                    connections_to_close.append(conn)
        
        # Close idle connections
        for conn in connections_to_close:
            self._available.remove(conn)
            self._close_connection(conn)
        
        if connections_to_close:
            logger.debug(f"Closed {len(connections_to_close)} idle connections")
    
    def close_all(self):
        """Close all connections in the pool."""
        with self._lock:
            # Close available connections
            while self._available:
                conn = self._available.popleft()
                self._close_connection(conn)
            
            # Close in-use connections (aggressive cleanup)
            for conn in list(self._in_use):
                self._close_connection(conn)
            self._in_use.clear()
            
            logger.info("Closed all connections in pool")
    
    def get_stats(self) -> Dict:
        """Get pool statistics."""
        with self._lock:
            return {
                **self.stats,
                'available': len(self._available),
                'in_use': len(self._in_use),
                'total': len(self._available) + len(self._in_use),
                'circuit_open': self._circuit_open,
                'failed_attempts': self._failed_attempts
            }

# Global connection pools (one per server address)
_connection_pools = {}
_pools_lock = threading.Lock()

def get_connection_pool(server_address: tuple) -> ConnectionPool:
    """Get or create connection pool for server address."""
    with _pools_lock:
        if server_address not in _connection_pools:
            _connection_pools[server_address] = ConnectionPool(server_address)
        return _connection_pools[server_address]

def close_all_pools():
    """Close all connection pools (for cleanup)."""
    with _pools_lock:
        for pool in _connection_pools.values():
            pool.close_all()
        _connection_pools.clear()