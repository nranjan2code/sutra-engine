"""
gRPC Storage Adapter for distributed Sutra Storage.

Provides the same interface as RustStorageAdapter but communicates with
a remote storage server via gRPC instead of using local ConcurrentStorage.

This is the production deployment architecture:
- Storage Server: Centralized Rust gRPC server with ConcurrentStorage
- API/Hybrid Services: Python services using GrpcStorageAdapter
"""

import logging
import os
from pathlib import Path
from typing import Dict, List, Optional, Tuple

import numpy as np

try:
    from sutra_storage_client import StorageClient

    GRPC_CLIENT_AVAILABLE = True
except ImportError:
    GRPC_CLIENT_AVAILABLE = False
    StorageClient = None

from ..graph.concepts import Association, AssociationType, Concept

logger = logging.getLogger(__name__)


class GrpcStorageAdapter:
    """
    gRPC storage adapter for distributed deployments.

    Connects to a remote storage server via gRPC and provides the same
    interface as RustStorageAdapter for drop-in compatibility.
    """

    def __init__(
        self,
        server_address: Optional[str] = None,
        vector_dimension: int = 768,
        use_compression: bool = True,  # ignored, server-side setting
    ):
        """
        Initialize gRPC storage adapter.

        Args:
            server_address: Storage server address (host:port)
                Defaults to SUTRA_STORAGE_SERVER env var or "localhost:50051"
            vector_dimension: Expected embedding dimension (for validation)
            use_compression: Ignored, server-side setting

        Raises:
            ImportError: If sutra-storage-client is not installed
            RuntimeError: If cannot connect to storage server
        """
        if not GRPC_CLIENT_AVAILABLE:
            raise ImportError(
                "sutra-storage-client module not available. "
                "Install with: pip install -e packages/sutra-storage-client/"
            )

        # Get server address from parameter or environment
        if server_address is None:
            server_address = os.environ.get("SUTRA_STORAGE_SERVER", "localhost:50051")

        self.server_address = server_address
        self.vector_dimension = vector_dimension

        # Initialize gRPC client
        try:
            self.client = StorageClient(server_address)
            logger.info(f"gRPC storage connected to {server_address}")

            # Verify connection with health check
            health = self.client.health_check()
            logger.info(
                f"Storage server health: {health['status']} "
                f"(uptime: {health['uptime_seconds']}s)"
            )
        except Exception as e:
            raise RuntimeError(f"Failed to connect to storage server at {server_address}: {e}")

    # ===== Concept Operations =====

    def has_concept(self, concept_id: str) -> bool:
        """Check if concept exists in storage."""
        try:
            return self.client.contains(concept_id)
        except Exception as e:
            logger.warning(f"Failed to check concept existence: {e}")
            return False

    def add_concept(self, concept: Concept, embedding: np.ndarray) -> None:
        """
        Add concept with its embedding via gRPC.

        Args:
            concept: Concept object with all attributes
            embedding: Vector embedding (numpy array)

        Raises:
            ValueError: If embedding dimension doesn't match or embedding is invalid
            RuntimeError: If storage operation fails
        """
        # Validate embedding
        if not isinstance(embedding, np.ndarray):
            raise ValueError(f"Embedding must be numpy array, got {type(embedding)}")
        if embedding.shape[0] != self.vector_dimension:
            raise ValueError(
                f"Embedding dimension {embedding.shape[0]} doesn't match expected {self.vector_dimension}"
            )
        if not np.isfinite(embedding).all():
            raise ValueError("Embedding contains NaN or Inf values")

        try:
            self.client.learn_concept(
                concept.id,
                concept.content,
                embedding=embedding.astype(np.float32),
                strength=float(concept.strength),
                confidence=float(concept.confidence),
            )
            logger.debug(f"Learned concept {concept.id[:8]}... via gRPC")
        except Exception as e:
            raise RuntimeError(f"Failed to learn concept via gRPC: {e}")

    def get_concept(self, concept_id: str) -> Optional[Concept]:
        """Retrieve concept by ID from remote storage."""
        try:
            data = self.client.query_concept(concept_id)
            if not data:
                return None

            return Concept(
                id=data.get("id", concept_id),
                content=data.get("content", ""),
                strength=float(data.get("strength", 1.0)),
                confidence=float(data.get("confidence", 1.0)),
            )
        except Exception as e:
            logger.warning(f"Failed to query concept: {e}")
            return None

    def get_all_concept_ids(self) -> List[str]:
        """
        Get all concept IDs.

        Note: Not efficiently supported by gRPC protocol.
        Callers should track IDs externally for production.
        """
        logger.warning(
            "get_all_concept_ids requires full scan - not efficient for gRPC storage"
        )
        return []

    def delete_concept(self, concept_id: str) -> None:
        """
        Delete concept (not supported by gRPC storage yet).

        For now, we log and return to avoid breaking rollback paths.
        """
        logger.warning("delete_concept not yet supported via gRPC storage")

    # ===== Association Operations =====

    def add_association(self, association: Association) -> None:
        """Add association via gRPC."""
        # Map Python enum to Rust u8
        type_map = {
            "semantic": 0,
            "causal": 1,
            "temporal": 2,
            "hierarchical": 3,
            "compositional": 4,
        }

        try:
            self.client.learn_association(
                association.source_id,
                association.target_id,
                assoc_type=type_map.get(association.assoc_type.value, 0),
                confidence=float(association.confidence),
            )
            logger.debug(
                f"Learned association {association.source_id[:8]}... → {association.target_id[:8]}... via gRPC"
            )
        except Exception as e:
            logger.warning(f"Failed to learn association via gRPC: {e}")

    def get_association(self, source_id: str, target_id: str) -> Optional[Association]:
        """
        Get association between two concepts.

        Uses neighbor check as proxy (no direct association query in protocol).
        """
        try:
            neighbors = self.client.get_neighbors(source_id)
            if target_id in neighbors:
                return Association(
                    source_id=source_id,
                    target_id=target_id,
                    assoc_type=AssociationType.SEMANTIC,
                    confidence=0.5,
                )
        except Exception as e:
            logger.warning(f"Failed to get association: {e}")
        return None

    def get_neighbors(self, concept_id: str) -> List[str]:
        """Get neighboring concept IDs from remote storage."""
        try:
            return self.client.get_neighbors(concept_id)
        except Exception as e:
            logger.warning(f"Failed to get neighbors: {e}")
            return []

    def get_all_associations(self) -> List[Association]:
        """
        Retrieve all associations (not supported efficiently).

        Would require full graph scan over gRPC.
        """
        logger.warning("get_all_associations not supported via gRPC storage")
        return []

    def delete_association(self, source_id: str, target_id: str) -> None:
        """Delete association (not yet supported)."""
        logger.warning("delete_association not yet supported via gRPC storage")

    # ===== Path Finding =====

    def find_path(
        self,
        start_id: str,
        end_id: str,
        max_depth: int = 6,
    ) -> Optional[List[str]]:
        """
        Find path between concepts using server-side BFS.

        Args:
            start_id: Starting concept ID
            end_id: Target concept ID
            max_depth: Maximum path length

        Returns:
            List of concept IDs forming the path, or None if no path found
        """
        try:
            return self.client.find_path(start_id, end_id, max_depth=max_depth)
        except Exception as e:
            logger.warning(f"Failed to find path: {e}")
            return None

    # ===== Vector Search =====

    def vector_search(
        self,
        query_vector: np.ndarray,
        k: int = 10,
        ef_search: int = 50,
    ) -> List[Tuple[str, float]]:
        """
        Perform vector similarity search via remote HNSW index.

        Args:
            query_vector: Query embedding (numpy array)
            k: Number of neighbors to return
            ef_search: HNSW search parameter (higher = more accurate, slower)

        Returns:
            List of (concept_id, similarity_score) tuples
        """
        try:
            return self.client.vector_search(query_vector, k=k, ef_search=ef_search)
        except Exception as e:
            logger.warning(f"Failed to perform vector search: {e}")
            return []

    # ===== Persistence & Stats =====

    def save(self) -> None:
        """Flush pending writes to disk on server."""
        try:
            self.client.flush()
            logger.info("Flushed storage to disk via gRPC")
        except Exception as e:
            logger.warning(f"Failed to flush storage: {e}")

    def flush(self) -> None:
        """Alias for save()."""
        self.save()

    def stats(self) -> Dict:
        """Get storage statistics from server."""
        try:
            return self.client.stats()
        except Exception as e:
            logger.warning(f"Failed to get stats: {e}")
            return {"concepts": 0, "edges": 0}

    def close(self) -> None:
        """Close gRPC connection."""
        try:
            self.client.close()
            logger.info("Closed gRPC storage connection")
        except Exception:
            pass

    def __enter__(self):
        """Context manager entry."""
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit."""
        self.close()
        return False
