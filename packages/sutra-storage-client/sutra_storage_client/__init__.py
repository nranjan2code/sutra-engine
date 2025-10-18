"""
Sutra Storage Client

Production-ready TCP client for Sutra Storage Server (msgpack protocol).
"""

from .client_tcp import StorageClient

__version__ = "2.0.0"
__all__ = ["StorageClient"]
