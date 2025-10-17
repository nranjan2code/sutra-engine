"""
Lightweight dependency injection for gRPC-based API.

This version uses only the storage-client for gRPC communication.
No local reasoning engine or heavy ML dependencies.
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
    Initialize gRPC storage client.
    
    Args:
        app: FastAPI application instance
    """
    global _storage_client
    
    logger.info("Initializing gRPC storage client...")
    
    try:
        from sutra_storage_client import StorageClient
        
        server_address = os.environ.get("SUTRA_STORAGE_SERVER", "storage-server:50051")
        _storage_client = StorageClient(server_address)
        
        # Verify connection
        health = _storage_client.health_check()
        logger.info(
            f"Connected to storage server at {server_address} "
            f"(status: {health['status']}, uptime: {health['uptime_seconds']}s)"
        )
        
        # Get initial stats
        stats = _storage_client.stats()
        logger.info(
            f"Storage contains {stats.get('concepts', 0)} concepts, "
            f"{stats.get('edges', 0)} edges"
        )
        
        # Store in app state
        app.state.storage_client = _storage_client
        
    except Exception as e:
        logger.error(f"Failed to initialize storage client: {e}")
        raise RuntimeError(f"Storage server connection required but failed: {e}")


def shutdown_dependencies(app: FastAPI) -> None:
    """
    Clean up dependencies during shutdown.
    
    Args:
        app: FastAPI application instance
    """
    if hasattr(app.state, "storage_client"):
        try:
            app.state.storage_client.close()
            logger.info("Closed storage client connection")
        except Exception as e:
            logger.warning(f"Error closing storage client: {e}")
        
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
