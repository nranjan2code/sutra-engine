"""
Sutra storage adapters.

Provides storage adapters for local (Rust), gRPC (deprecated), and TCP (preferred) deployments.
"""

from .rust_adapter import RustStorageAdapter
from .grpc_adapter import GrpcStorageAdapter
from .tcp_adapter import TcpStorageAdapter

__all__ = ["RustStorageAdapter", "GrpcStorageAdapter", "TcpStorageAdapter"]
