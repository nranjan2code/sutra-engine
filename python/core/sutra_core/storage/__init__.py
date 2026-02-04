"""
Sutra storage adapter.

Uses TCP Binary Protocol (MessagePack) for high-performance communication
with the Rust storage server. This is the ONLY supported backend.

Architecture (Production):
    Hybrid → Core → TcpStorageAdapter → Storage Server (Rust)
"""

from .tcp_adapter import TcpStorageAdapter

__all__ = ["TcpStorageAdapter"]
