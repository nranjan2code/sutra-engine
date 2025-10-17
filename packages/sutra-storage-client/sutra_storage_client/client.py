"""
Client for standalone storage server.

This replaces the direct ConcurrentStorage import with gRPC calls to the server.
"""

import grpc
import numpy as np
from typing import Dict, List, Optional, Tuple

# Generated from proto files
from . import storage_pb2
from . import storage_pb2_grpc


class StorageClient:
    """
    Client for connecting to standalone storage server.
    
    Drop-in replacement for ConcurrentStorage with network communication.
    """
    
    def __init__(self, server_address: str = "localhost:50051"):
        """
        Connect to storage server.
        
        Args:
            server_address: Address of storage server (host:port)
        """
        self.channel = grpc.insecure_channel(server_address)
        self.stub = storage_pb2_grpc.StorageServiceStub(self.channel)
        
    def learn_concept(
        self,
        concept_id: str,
        content: str,
        embedding: Optional[np.ndarray] = None,
        strength: float = 1.0,
        confidence: float = 1.0,
    ) -> int:
        """Learn a concept with optional embedding."""
        request = storage_pb2.LearnConceptRequest(
            concept_id=concept_id,
            content=content,
            embedding=embedding.tolist() if embedding is not None else [],
            strength=strength,
            confidence=confidence,
        )
        response = self.stub.LearnConcept(request)
        return response.sequence
    
    def learn_association(
        self,
        source_id: str,
        target_id: str,
        assoc_type: int = 0,
        confidence: float = 1.0,
    ) -> int:
        """Learn an association between concepts."""
        request = storage_pb2.LearnAssociationRequest(
            source_id=source_id,
            target_id=target_id,
            assoc_type=assoc_type,
            confidence=confidence,
        )
        response = self.stub.LearnAssociation(request)
        return response.sequence
    
    def query_concept(self, concept_id: str) -> Optional[Dict]:
        """Query a concept by ID."""
        request = storage_pb2.QueryConceptRequest(concept_id=concept_id)
        response = self.stub.QueryConcept(request)
        
        if not response.found:
            return None
        
        return {
            "id": response.concept_id,
            "content": response.content,
            "strength": response.strength,
            "confidence": response.confidence,
        }
    
    def contains(self, concept_id: str) -> bool:
        """Check if concept exists."""
        result = self.query_concept(concept_id)
        return result is not None
    
    def get_neighbors(self, concept_id: str) -> List[str]:
        """Get neighboring concepts."""
        request = storage_pb2.GetNeighborsRequest(concept_id=concept_id)
        response = self.stub.GetNeighbors(request)
        return list(response.neighbor_ids)
    
    def find_path(
        self,
        start_id: str,
        end_id: str,
        max_depth: int = 6,
    ) -> Optional[List[str]]:
        """Find path between two concepts."""
        request = storage_pb2.FindPathRequest(
            start_id=start_id,
            end_id=end_id,
            max_depth=max_depth,
        )
        response = self.stub.FindPath(request)
        
        if not response.found:
            return None
        
        return list(response.path)
    
    def vector_search(
        self,
        query_vector: np.ndarray,
        k: int = 10,
        ef_search: int = 50,
    ) -> List[Tuple[str, float]]:
        """Vector similarity search."""
        request = storage_pb2.VectorSearchRequest(
            query_vector=query_vector.tolist(),
            k=k,
            ef_search=ef_search,
        )
        response = self.stub.VectorSearch(request)
        
        return [(match.concept_id, match.similarity) for match in response.results]
    
    def stats(self) -> Dict:
        """Get storage statistics."""
        request = storage_pb2.StatsRequest()
        response = self.stub.GetStats(request)
        
        return {
            "concepts": response.concepts,
            "edges": response.edges,
            "written": response.written,
            "dropped": response.dropped,
            "pending": response.pending,
            "reconciliations": response.reconciliations,
        }
    
    def flush(self) -> None:
        """Force flush to disk."""
        request = storage_pb2.FlushRequest()
        response = self.stub.Flush(request)
        if not response.success:
            raise RuntimeError(f"Flush failed: {response.message}")
    
    def health_check(self) -> Dict:
        """Check server health."""
        request = storage_pb2.HealthCheckRequest()
        response = self.stub.HealthCheck(request)
        return {
            "status": response.status,
            "uptime_seconds": response.uptime_seconds,
        }
    
    def close(self):
        """Close connection to server."""
        self.channel.close()
    
    def __enter__(self):
        return self
    
    def __exit__(self, *args):
        self.close()
