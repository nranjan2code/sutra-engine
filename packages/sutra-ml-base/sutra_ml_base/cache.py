"""
Caching and Performance Optimization for Sutra ML Services

Provides efficient caching mechanisms:
- Model output caching (embeddings, generations)
- LRU cache with TTL and size limits
- Edition-aware cache sizing and features
- Persistent cache storage
- Cache warming and precomputation
"""

import logging
import os
import time
import hashlib
import pickle
from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import Any, Optional, Dict, Tuple, Union
from threading import Lock
from pathlib import Path

logger = logging.getLogger(__name__)


@dataclass
class CacheConfig:
    """Configuration for caching behavior"""
    # Cache sizing
    max_memory_mb: int = 1024
    max_items: int = 10000
    
    # TTL settings
    default_ttl_seconds: int = 3600  # 1 hour
    max_ttl_seconds: int = 86400     # 24 hours
    
    # Storage settings
    persistent: bool = True
    cache_dir: str = "/tmp/.cache/sutra"
    
    # Features
    enable_compression: bool = True
    enable_encryption: bool = False
    
    def __post_init__(self):
        """Validate and create cache directory"""
        if self.persistent:
            Path(self.cache_dir).mkdir(parents=True, exist_ok=True)


class CacheEntry:
    """Single cache entry with metadata"""
    
    def __init__(self, value: Any, ttl_seconds: int, size_bytes: int = 0):
        self.value = value
        self.created_at = time.time()
        self.ttl_seconds = ttl_seconds
        self.size_bytes = size_bytes
        self.access_count = 0
        self.last_accessed = self.created_at
    
    @property
    def is_expired(self) -> bool:
        """Check if entry has expired"""
        if self.ttl_seconds <= 0:  # Never expires
            return False
        return time.time() - self.created_at > self.ttl_seconds
    
    def access(self) -> Any:
        """Access the cached value and update stats"""
        self.access_count += 1
        self.last_accessed = time.time()
        return self.value
    
    @property
    def age_seconds(self) -> float:
        """Get age of entry in seconds"""
        return time.time() - self.created_at


class BaseCacheStore(ABC):
    """Abstract base class for cache storage backends"""
    
    @abstractmethod
    def get(self, key: str) -> Optional[CacheEntry]:
        """Get cache entry by key"""
        pass
    
    @abstractmethod
    def set(self, key: str, entry: CacheEntry) -> bool:
        """Set cache entry"""
        pass
    
    @abstractmethod
    def delete(self, key: str) -> bool:
        """Delete cache entry"""
        pass
    
    @abstractmethod
    def clear(self) -> bool:
        """Clear all entries"""
        pass
    
    @abstractmethod
    def size(self) -> int:
        """Get number of entries"""
        pass
    
    @abstractmethod
    def get_stats(self) -> Dict[str, Any]:
        """Get cache statistics"""
        pass


class MemoryCacheStore(BaseCacheStore):
    """In-memory cache store with LRU eviction"""
    
    def __init__(self, config: CacheConfig):
        self.config = config
        self._cache: Dict[str, CacheEntry] = {}
        self._access_order: List[str] = []
        self._lock = Lock()
        self._total_size_bytes = 0
        
        # Statistics
        self._hits = 0
        self._misses = 0
        self._evictions = 0
    
    def get(self, key: str) -> Optional[CacheEntry]:
        """Get cache entry by key"""
        with self._lock:
            entry = self._cache.get(key)
            
            if entry is None:
                self._misses += 1
                return None
            
            if entry.is_expired:
                # Remove expired entry
                self._remove_entry(key)
                self._misses += 1
                return None
            
            # Update access order
            if key in self._access_order:
                self._access_order.remove(key)
            self._access_order.append(key)
            
            self._hits += 1
            return entry
    
    def set(self, key: str, entry: CacheEntry) -> bool:
        """Set cache entry"""
        with self._lock:
            # Remove existing entry if present
            if key in self._cache:
                self._remove_entry(key)
            
            # Check size limits
            if not self._can_fit_entry(entry):
                self._evict_entries(entry.size_bytes)
            
            # Add new entry
            self._cache[key] = entry
            self._access_order.append(key)
            self._total_size_bytes += entry.size_bytes
            
            return True
    
    def delete(self, key: str) -> bool:
        """Delete cache entry"""
        with self._lock:
            if key in self._cache:
                self._remove_entry(key)
                return True
            return False
    
    def clear(self) -> bool:
        """Clear all entries"""
        with self._lock:
            self._cache.clear()
            self._access_order.clear()
            self._total_size_bytes = 0
            return True
    
    def size(self) -> int:
        """Get number of entries"""
        return len(self._cache)
    
    def get_stats(self) -> Dict[str, Any]:
        """Get cache statistics"""
        with self._lock:
            total_requests = self._hits + self._misses
            hit_rate = (self._hits / total_requests) if total_requests > 0 else 0.0
            
            return {
                "entries": len(self._cache),
                "size_bytes": self._total_size_bytes,
                "size_mb": self._total_size_bytes / (1024 * 1024),
                "max_size_mb": self.config.max_memory_mb,
                "hit_rate": hit_rate,
                "hits": self._hits,
                "misses": self._misses,
                "evictions": self._evictions
            }
    
    def _can_fit_entry(self, entry: CacheEntry) -> bool:
        """Check if entry can fit in cache"""
        max_bytes = self.config.max_memory_mb * 1024 * 1024
        return (
            len(self._cache) < self.config.max_items and
            self._total_size_bytes + entry.size_bytes <= max_bytes
        )
    
    def _evict_entries(self, needed_bytes: int):
        """Evict LRU entries to make space"""
        while (
            self._access_order and
            (
                len(self._cache) >= self.config.max_items or
                self._total_size_bytes + needed_bytes > self.config.max_memory_mb * 1024 * 1024
            )
        ):
            lru_key = self._access_order[0]
            self._remove_entry(lru_key)
            self._evictions += 1
    
    def _remove_entry(self, key: str):
        """Remove entry and update tracking"""
        if key in self._cache:
            entry = self._cache[key]
            del self._cache[key]
            self._total_size_bytes -= entry.size_bytes
            
            if key in self._access_order:
                self._access_order.remove(key)


class CacheManager:
    """High-level cache manager for ML services"""
    
    def __init__(self, config: CacheConfig):
        self.config = config
        self.store = MemoryCacheStore(config)
        
        logger.info(f"Cache manager initialized (max: {config.max_memory_mb}MB, {config.max_items} items)")
    
    def get(self, key: str) -> Optional[Any]:
        """Get cached value by key
        
        Args:
            key: Cache key
            
        Returns:
            Cached value or None if not found/expired
        """
        entry = self.store.get(key)
        return entry.access() if entry else None
    
    def set(self, key: str, value: Any, ttl_seconds: Optional[int] = None) -> bool:
        """Set cached value
        
        Args:
            key: Cache key
            value: Value to cache
            ttl_seconds: Time to live (uses default if None)
            
        Returns:
            True if cached successfully
        """
        if ttl_seconds is None:
            ttl_seconds = self.config.default_ttl_seconds
        
        # Calculate size estimate
        size_bytes = self._estimate_size(value)
        
        entry = CacheEntry(value, ttl_seconds, size_bytes)
        return self.store.set(key, entry)
    
    def delete(self, key: str) -> bool:
        """Delete cached value
        
        Args:
            key: Cache key
            
        Returns:
            True if deleted successfully
        """
        return self.store.delete(key)
    
    def clear(self) -> bool:
        """Clear all cached values
        
        Returns:
            True if cleared successfully
        """
        return self.store.clear()
    
    def get_or_compute(self, key: str, compute_fn: callable, ttl_seconds: Optional[int] = None) -> Any:
        """Get cached value or compute and cache it
        
        Args:
            key: Cache key
            compute_fn: Function to compute value if not cached
            ttl_seconds: Time to live (uses default if None)
            
        Returns:
            Cached or computed value
        """
        # Try to get from cache first
        cached_value = self.get(key)
        if cached_value is not None:
            return cached_value
        
        # Compute new value
        try:
            new_value = compute_fn()
            self.set(key, new_value, ttl_seconds)
            return new_value
        except Exception as e:
            logger.error(f"Failed to compute value for key '{key}': {e}")
            raise
    
    def cache_key(self, prefix: str, *args, **kwargs) -> str:
        """Generate cache key from arguments
        
        Args:
            prefix: Key prefix
            *args: Positional arguments
            **kwargs: Keyword arguments
            
        Returns:
            Generated cache key
        """
        # Create deterministic key from arguments
        key_parts = [prefix]
        
        # Add positional args
        for arg in args:
            key_parts.append(str(arg))
        
        # Add sorted keyword args
        for k, v in sorted(kwargs.items()):
            key_parts.append(f"{k}={v}")
        
        key_string = "|".join(key_parts)
        
        # Hash long keys for consistency
        if len(key_string) > 250:
            key_hash = hashlib.md5(key_string.encode()).hexdigest()
            return f"{prefix}|{key_hash}"
        
        return key_string
    
    def get_stats(self) -> Dict[str, Any]:
        """Get cache statistics
        
        Returns:
            Dictionary with cache statistics
        """
        return self.store.get_stats()
    
    def _estimate_size(self, value: Any) -> int:
        """Estimate size of value in bytes
        
        Args:
            value: Value to size
            
        Returns:
            Estimated size in bytes
        """
        try:
            # Use pickle to estimate serialized size
            return len(pickle.dumps(value))
        except Exception:
            # Fallback estimation
            if isinstance(value, str):
                return len(value.encode('utf-8'))
            elif isinstance(value, (list, tuple)):
                return sum(self._estimate_size(item) for item in value)
            elif isinstance(value, dict):
                return sum(
                    self._estimate_size(k) + self._estimate_size(v)
                    for k, v in value.items()
                )
            else:
                # Conservative estimate
                return 1024