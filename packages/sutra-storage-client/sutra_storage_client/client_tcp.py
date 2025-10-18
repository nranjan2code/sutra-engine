"""TCP Storage Client - Msgpack Protocol.

Replaces gRPC client with custom TCP protocol.
"""

import socket
import struct
import msgpack
from typing import Optional, Dict, List, Any, Tuple


class StorageClient:
    """TCP storage client using msgpack serialization."""
    
    def __init__(self, server_address: str = "localhost:50051"):
        """Connect to storage server.
        
        Args:
            server_address: host:port (e.g., "localhost:50051")
        """
        host, port = server_address.split(":")
        self.sock = socket.create_connection((host, int(port)))
        self.sock.setsockopt(socket.IPPROTO_TCP, socket.TCP_NODELAY, 1)
    
    def _send_request(self, variant_name: str, data: Any) -> Dict:
        """Send request and receive response.
        
        Args:
            variant_name: Rust enum variant name
            data: Variant data (dict or None for unit variants)
        
        Returns:
            Response dictionary
        """
        # Pack request as enum: {variant_name: data} or just variant_name for unit
        if data is None:
            request = variant_name  # Unit variant
        else:
            request = {variant_name: data}
        packed = msgpack.packb(request)
        
        # Send with length prefix
        self.sock.sendall(struct.pack(">I", len(packed)))
        self.sock.sendall(packed)
        
        # Receive response length
        length_bytes = self.sock.recv(4)
        if len(length_bytes) < 4:
            raise ConnectionError("Connection closed")
        length = struct.unpack(">I", length_bytes)[0]
        
        # Receive response
        response_bytes = b""
        while len(response_bytes) < length:
            chunk = self.sock.recv(min(4096, length - len(response_bytes)))
            if not chunk:
                raise ConnectionError("Connection closed")
            response_bytes += chunk
        
        return msgpack.unpackb(response_bytes, raw=False)
    
    def learn_concept(
        self,
        concept_id: str,
        content: str,
        embedding: Optional[List[float]] = None,
        strength: float = 1.0,
        confidence: float = 1.0,
    ) -> int:
        """Learn a concept with optional embedding."""
        response = self._send_request("LearnConcept", {
            "concept_id": concept_id,
            "content": content,
            "embedding": embedding or [],
            "strength": strength,
            "confidence": confidence,
        })
        if "LearnConceptOk" in response:
            data = response["LearnConceptOk"]
            if isinstance(data, dict) and "sequence" in data:
                return data["sequence"]
        return 0
    
    def learn_association(
        self,
        source_id: str,
        target_id: str,
        assoc_type: int = 0,
        confidence: float = 1.0,
    ) -> int:
        """Learn an association between concepts."""
        response = self._send_request("LearnAssociation", {
            "source_id": source_id,
            "target_id": target_id,
            "assoc_type": assoc_type,
            "confidence": confidence,
        })
        if "LearnAssociationOk" in response:
            data = response["LearnAssociationOk"]
            if isinstance(data, dict) and "sequence" in data:
                return data["sequence"]
        return 0
    
    def query_concept(self, concept_id: str) -> Optional[Dict]:
        """Query a concept by ID."""
        response = self._send_request("QueryConcept", {"concept_id": concept_id})
        if "QueryConceptOk" in response:
            data = response["QueryConceptOk"]
            if isinstance(data, list) and len(data) >= 5:
                return {
                    "id": data[1],
                    "content": data[2],
                    "strength": data[3],
                    "confidence": data[4],
                }
        return None
    
    def contains(self, concept_id: str) -> bool:
        """Check if concept exists."""
        result = self.query_concept(concept_id)
        return result is not None
    
    def get_neighbors(self, concept_id: str) -> List[str]:
        """Get neighboring concepts."""
        response = self._send_request("GetNeighbors", {"concept_id": concept_id})
        if "GetNeighborsOk" in response:
            data = response["GetNeighborsOk"]
            if isinstance(data, dict) and "neighbor_ids" in data:
                return data["neighbor_ids"]
        return []
    
    def find_path(
        self,
        start_id: str,
        end_id: str,
        max_depth: int = 6,
    ) -> Optional[List[str]]:
        """Find path between two concepts."""
        response = self._send_request("FindPath", {
            "start_id": start_id,
            "end_id": end_id,
            "max_depth": max_depth,
        })
        if "FindPathOk" in response:
            data = response["FindPathOk"]
            if isinstance(data, dict):
                if data.get("found") and "path" in data:
                    return data["path"]
            elif isinstance(data, list) and len(data) >= 2:
                if data[0] and len(data) > 1:
                    return data[1]
        return None
    
    def vector_search(
        self,
        query_vector: List[float],
        k: int = 10,
        ef_search: int = 50,
    ) -> List[Tuple[str, float]]:
        """Vector similarity search."""
        response = self._send_request("VectorSearch", {
            "query_vector": query_vector,
            "k": k,
            "ef_search": ef_search,
        })
        if "VectorSearchOk" in response:
            data = response["VectorSearchOk"]
            if isinstance(data, dict) and "results" in data:
                return data["results"]
        return []
    
    def stats(self) -> Dict:
        """Get storage statistics."""
        response = self._send_request("GetStats", None)
        if "StatsOk" in response:
            data = response["StatsOk"]
            if isinstance(data, list) and len(data) >= 7:
                return {
                    "concepts": data[0],
                    "edges": data[1],
                    "written": data[2],
                    "dropped": data[3],
                    "pending": data[4],
                    "reconciliations": data[5],
                    "uptime_seconds": data[6],
                }
            elif isinstance(data, dict):
                return data
        return {}
    
    def flush(self) -> None:
        """Force flush to disk."""
        response = self._send_request("Flush", None)
        if "FlushOk" not in response:
            if "Error" in response:
                raise RuntimeError(f"Flush failed: {response['Error'].get('message', 'Unknown error')}")
            raise RuntimeError("Flush failed")
    
    def health_check(self) -> Dict:
        """Check server health."""
        response = self._send_request("HealthCheck", None)
        if "HealthCheckOk" in response:
            data = response["HealthCheckOk"]
            if isinstance(data, list) and len(data) >= 3:
                return {
                    "healthy": data[0],
                    "status": data[1],
                    "uptime_seconds": data[2],
                }
            elif isinstance(data, dict):
                return data
        return {"healthy": False, "status": "unknown"}
    
    def close(self):
        """Close connection to server."""
        self.sock.close()
    
    def __enter__(self):
        return self
    
    def __exit__(self, *args):
        self.close()
