"""
Sutra storage adapters.

Provides storage adapters for both local (Rust) and distributed (gRPC) deployments.
"""

from .rust_adapter import RustStorageAdapter
from .grpc_adapter import GrpcStorageAdapter

__all__ = ["RustStorageAdapter", "GrpcStorageAdapter"]
