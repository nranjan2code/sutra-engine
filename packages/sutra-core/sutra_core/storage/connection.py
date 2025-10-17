"""
Storage connection factory.

Determines whether to use:
1. Direct ConcurrentStorage (embedded mode)
2. StorageClient (server mode)

Based on environment variable SUTRA_STORAGE_MODE.
"""

import os
import logging
from typing import Union

logger = logging.getLogger(__name__)


def get_storage_backend(storage_path: str, **kwargs):
    """
    Get storage backend based on environment.
    
    Args:
        storage_path: Path to storage directory (or ignored if using server)
        **kwargs: Additional configuration
        
    Returns:
        Storage backend (ConcurrentStorage or StorageClient)
        
    Environment:
        SUTRA_STORAGE_MODE: "embedded" (default) or "server"
        SUTRA_STORAGE_SERVER: Server address (default: localhost:50051)
    """
    mode = os.environ.get("SUTRA_STORAGE_MODE", "embedded").lower()
    
    if mode == "server":
        # Use client to connect to standalone server
        try:
            from sutra_storage_client import StorageClient
            
            server_addr = os.environ.get("SUTRA_STORAGE_SERVER", "localhost:50051")
            logger.info(f"Connecting to storage server at {server_addr}")
            
            return StorageClient(server_address=server_addr)
            
        except ImportError:
            logger.error("sutra_storage_client not installed, falling back to embedded")
            mode = "embedded"
    
    if mode == "embedded":
        # Use embedded ConcurrentStorage
        try:
            from sutra_storage import ConcurrentStorage
            
            logger.info(f"Using embedded storage at {storage_path}")
            
            return ConcurrentStorage(
                storage_path,
                reconcile_interval_ms=kwargs.get("reconcile_interval_ms", 10),
                memory_threshold=kwargs.get("memory_threshold", 50000),
                vector_dimension=kwargs.get("vector_dimension", 768),
            )
            
        except ImportError:
            raise ImportError(
                "sutra_storage module not available. "
                "Build with: cd packages/sutra-storage && maturin develop"
            )
    
    raise ValueError(f"Unknown storage mode: {mode}")


# Backwards compatibility
def create_storage_adapter(storage_path: str, **kwargs):
    """Create RustStorageAdapter with automatic backend selection."""
    from .rust_adapter import RustStorageAdapter
    
    # Get backend
    backend = get_storage_backend(storage_path, **kwargs)
    
    # Wrap in adapter
    # Note: This requires modifying RustStorageAdapter to accept either backend
    adapter = RustStorageAdapter.__new__(RustStorageAdapter)
    adapter.storage_path = storage_path
    adapter.store = backend
    adapter.vector_dimension = kwargs.get("vector_dimension", 768)
    
    return adapter
