"""
Storage client using custom binary protocol.

Replaces gRPC client while maintaining distributed architecture.
Based on working test_tcp_client.py implementation.
"""

import socket
import struct
from typing import Dict, List, Optional, Tuple
try:
    import msgpack
except ImportError:
    raise ImportError("msgpack package required: pip install msgpack")


class StorageClient:
    """
    TCP client for storage server using custom binary protocol.
    
    Based on working test_tcp_client.py implementation.
    """
    
    def __init__(self, server_address: str = "localhost:50051"):
        """
        Connect to storage server.
        
        Args:
            server_address: Address of storage server (host:port)
        """
        host, port = server_address.split(":")
        self.address = (host, int(port))
        self.socket = None
        self._connect()
    
    def _connect(self):
        """Establish TCP connection"""
        self.socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.socket.setsockopt(socket.IPPROTO_TCP, socket.TCP_NODELAY, 1)  # Low latency
        self.socket.connect(self.address)
    
    def _send_request(self, variant_name: str, data: dict) -> dict:
        """Send request and receive response.
        
        Rust enum format: {variant_name: variant_data}
        """
        # Pack request as enum: {variant_name: data}
        request = {variant_name: data}
        packed = msgpack.packb(request)
        
        # Send with length prefix
        self.socket.sendall(struct.pack(">I", len(packed)))
        self.socket.sendall(packed)
        
        # Receive response length
        length_bytes = self.socket.recv(4)
        if len(length_bytes) < 4:
            raise ConnectionError("Connection closed")
        length = struct.unpack(">I", length_bytes)[0]
        
        # Receive response
        response_bytes = b""
        while len(response_bytes) < length:
            chunk = self.socket.recv(min(4096, length - len(response_bytes)))
            if not chunk:
                raise ConnectionError("Connection closed")
            response_bytes += chunk
        
        return msgpack.unpackb(response_bytes, raw=False)
    
    def _recv_exactly(self, n: int) -> bytes:
        """Receive exactly n bytes"""
        data = b''
        while len(data) < n:
            chunk = self.socket.recv(n - len(data))
            if not chunk:
                raise ConnectionError("Connection closed by server")
            data += chunk
        return data
    
    def learn_concept(
        self,
        concept_id: str,
        content: str,
        embedding: Optional[List[float]] = None,
        strength: float = 1.0,
        confidence: float = 1.0,
    ) -> int:
        """Learn a concept with optional embedding"""
        response = self._send_request("LearnConcept", {
            "concept_id": concept_id,
            "content": content,
            "embedding": [float(x) for x in (embedding or [])],
            "strength": float(strength),
            "confidence": float(confidence),
        })
        
        if "Error" in response:
            raise RuntimeError(response["Error"]["message"])
        
        if "LearnConceptOk" in response:
            sequence_list = response["LearnConceptOk"]
            return sequence_list[0] if sequence_list else 0
        
        raise RuntimeError(f"Unexpected response: {response}")
    
    def learn_association(
        self,
        source_id: str,
        target_id: str,
        assoc_type: int = 0,
        confidence: float = 1.0,
    ) -> int:
        """Learn an association between concepts"""
        response = self._send_request("LearnAssociation", {
            "source_id": source_id,
            "target_id": target_id,
            "assoc_type": int(assoc_type),
            "confidence": float(confidence),
        })
        
        if "Error" in response:
            raise RuntimeError(response["Error"]["message"])
        
        if "LearnAssociationOk" in response:
            sequence_list = response["LearnAssociationOk"]
            return sequence_list[0] if sequence_list else 0
        
        raise RuntimeError(f"Unexpected response: {response}")
    
    def query_concept(self, concept_id: str) -> Optional[Dict]:
        """Query a concept by ID"""
        response = self._send_request("QueryConcept", {
            "concept_id": concept_id,
        })
        
        if "Error" in response:
            raise RuntimeError(response["Error"]["message"])
        
        result = response["QueryConceptOk"]
        if not result["found"]:
            return None
        
        return {
            "id": result["concept_id"],
            "content": result["content"],
            "strength": result["strength"],
            "confidence": result["confidence"],
        }
    
    def contains(self, concept_id: str) -> bool:
        """Check if concept exists"""
        result = self.query_concept(concept_id)
        return result is not None
    
    def get_neighbors(self, concept_id: str) -> List[str]:
        """Get neighboring concepts"""
        response = self._send_request("GetNeighbors", {
            "concept_id": concept_id,
        })
        
        if "Error" in response:
            raise RuntimeError(response["Error"]["message"])
        
        return response["GetNeighborsOk"]["neighbor_ids"]
    
    def find_path(
        self,
        start_id: str,
        end_id: str,
        max_depth: int = 6,
    ) -> Optional[List[str]]:
        """Find path between two concepts"""
        response = self._send_request("FindPath", {
            "start_id": start_id,
            "end_id": end_id,
            "max_depth": max_depth,
        })
        
        if "Error" in response:
            raise RuntimeError(response["Error"]["message"])
        
        result = response["FindPathOk"]
        if not result["found"]:
            return None
        
        return result["path"]
    
    def vector_search(
        self,
        query_vector: List[float],
        k: int = 10,
        ef_search: int = 50,
    ) -> List[Tuple[str, float]]:
        """Vector similarity search"""
        response = self._send_request("VectorSearch", {
            "query_vector": query_vector,
            "k": k,
            "ef_search": ef_search,
        })
        
        if "Error" in response:
            raise RuntimeError(response["Error"]["message"])
        
        return [(r[0], r[1]) for r in response["VectorSearchOk"]["results"]]
    
    def stats(self) -> Dict:
        """Get storage statistics"""
        response = self._send_request("GetStats", {})
        
        if "Error" in response:
            raise RuntimeError(response["Error"]["message"])
        
        return response["StatsOk"]
    
    def flush(self) -> None:
        """Force flush to disk"""
        response = self._send_request("Flush", {})
        
        if "Error" in response:
            raise RuntimeError(f"Flush failed: {response['Error']['message']}")
    
    def health_check(self) -> Dict:
        """Check server health"""
        response = self._send_request("HealthCheck", {})
        
        if "Error" in response:
            raise RuntimeError(response["Error"]["message"])
        
        return response["HealthCheckOk"]
    
    def close(self):
        """Close connection to server"""
        if self.socket:
            self.socket.close()
            self.socket = None
    
    def __enter__(self):
        return self
    
    def __exit__(self, *args):
        self.close()


__all__ = ["StorageClient"]
