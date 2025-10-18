"""
Dependency injection for custom binary protocol storage client.

Replaced gRPC with custom binary protocol for better performance.
Storage remains a separate service (distributed architecture maintained).
"""

import logging
import os
import time

from fastapi import FastAPI, Request

logger = logging.getLogger(__name__)

# Track service start time
_start_time: float = time.time()
_storage_client = None


def init_dependencies(app: FastAPI) -> None:
    """
    Initialize storage client using custom binary protocol.
    
    Args:
        app: FastAPI application instance
    """
    global _storage_client
    
    logger.info("Initializing storage client (custom binary protocol)...")
    
    try:
        # Import TCP storage client
        from sutra_storage_client import StorageClient
        
        server_address = os.environ.get("SUTRA_STORAGE_SERVER", "storage-server:50051")
        logger.info(f"Connecting to storage server at {server_address}")
        
        # Create storage client
        _storage_client = StorageClient(server_address)
        logger.info("Successfully connected to storage server")
        
        # Store in app state
        app.state.storage_client = _storage_client
        
    except Exception as e:
        logger.error(f"Failed to initialize storage client: {e}")
        raise RuntimeError(f"Storage client initialization failed: {e}")


def shutdown_dependencies(app: FastAPI) -> None:
    """
    Clean up dependencies during shutdown.
    
    Args:
        app: FastAPI application instance
    """
    if hasattr(app.state, "storage_client"):
        try:
            # Flush any pending writes
            app.state.storage_client.flush()
            logger.info("Flushed storage and closed connection")
        except Exception as e:
            logger.warning(f"Error flushing storage: {e}")
        
        delattr(app.state, "storage_client")


def get_storage_client(request: Request):
    """
    Dependency to get storage client from request state.
    
    Args:
        request: FastAPI request containing app.state
    
    Returns:
        StorageClient instance
    """
    return request.app.state.storage_client


def get_uptime() -> float:
    """
    Get service uptime in seconds.
    
    Returns:
        Uptime in seconds
    """
    return time.time() - _start_time
