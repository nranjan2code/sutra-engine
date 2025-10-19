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
        
        # Defaults for unified learning
        self.default_learn_options = {
            "generate_embedding": True,
            "embedding_model": None,
            "extract_associations": True,
            "min_association_confidence": 0.5,
            "max_associations_per_concept": 10,
            "strength": 1.0,
            "confidence": 1.0,
        }
    
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
    
    def learn_concept_v2(
        self,
        content: str,
        options: Optional[dict] = None,
    ) -> str:
        """Learn a concept via unified learning pipeline (server-side embeddings+associations)."""
        opts = dict(self.default_learn_options)
        if options:
            opts.update({k: v for k, v in options.items() if v is not None})
        response = self._send_request("LearnConceptV2", {
            "content": content,
            "options": opts,
        })
        
        if "Error" in response:
            raise RuntimeError(response["Error"]["message"])
        
        if "LearnConceptV2Ok" in response:
            result = response["LearnConceptV2Ok"]
            # Handle both list format [concept_id] and dict format {concept_id: str}
            if isinstance(result, list) and len(result) > 0:
                return result[0]  # Storage server returns list format
            elif isinstance(result, dict) and "concept_id" in result:
                return result["concept_id"]
        raise RuntimeError(f"Unexpected response: {response}")
    
    def learn_batch_v2(
        self,
        contents: List[str],
        options: Optional[dict] = None,
    ) -> List[str]:
        """Batch learn via unified pipeline."""
        opts = dict(self.default_learn_options)
        if options:
            opts.update({k: v for k, v in options.items() if v is not None})
        response = self._send_request("LearnBatch", {
            "contents": contents,
            "options": opts,
        })
        if "Error" in response:
            raise RuntimeError(response["Error"]["message"])
        if "LearnBatchOk" in response:
            result = response["LearnBatchOk"]
            if isinstance(result, dict) and "concept_ids" in result:
                return result["concept_ids"]
        raise RuntimeError(f"Unexpected response: {response}")
    
    # Legacy explicit learn retained for compatibility
    def learn_concept(
        self,
        concept_id: str,
        content: str,
        embedding: Optional[List[float]] = None,
        strength: float = 1.0,
        confidence: float = 1.0,
    ) -> int:
        """Learn a concept with optional embedding (legacy)."""
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
        
        if "QueryConceptOk" in response:
            result = response["QueryConceptOk"]
            # Handle list format: [found, concept_id, content, strength, confidence]
            if isinstance(result, list):
                if len(result) >= 5 and result[0]:  # found = True
                    return {
                        "id": result[1],
                        "content": result[2],
                        "strength": result[3],
                        "confidence": result[4],
                    }
                return None  # found = False or not enough data
            # Handle dict format
            elif isinstance(result, dict):
                if not result.get("found", False):
                    return None
                return {
                    "id": result["concept_id"],
                    "content": result["content"],
                    "strength": result["strength"],
                    "confidence": result["confidence"],
                }
        
        return None
    
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
        
        result = response.get("GetNeighborsOk", {})
        if isinstance(result, dict):
            return result.get("neighbor_ids", [])
        return []
    
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
    
    def get_association(
        self,
        source_id: str,
        target_id: str,
    ) -> Optional[Dict]:
        """Get association between two concepts"""
        # TODO: Implement GetAssociation in storage server
        # For now, return stub to prevent errors
        try:
            response = self._send_request("GetAssociation", {
                "source_id": source_id,
                "target_id": target_id,
            })
            
            if "Error" in response:
                return None
            
            if "GetAssociationOk" in response:
                result = response["GetAssociationOk"]
                if isinstance(result, dict) and result.get("found", False):
                    return {
                        "source_id": result.get("source_id", source_id),
                        "target_id": result.get("target_id", target_id),
                        "type": result.get("assoc_type", 0),
                        "confidence": result.get("confidence", 1.0),
                    }
            return None
        except Exception:
            # Server doesn't support this yet, return None
            return None
    
    def get_all_concept_ids(self) -> List[str]:
        """Get all concept IDs"""
        # TODO: Implement GetAllConceptIds in storage server
        # For now, return stub to prevent errors
        try:
            # GetAllConceptIds is a unit variant - send just the string
            request = "GetAllConceptIds"
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
            
            response = msgpack.unpackb(response_bytes, raw=False)
            
            if "Error" in response:
                return []
            
            if "GetAllConceptIdsOk" in response:
                result = response["GetAllConceptIdsOk"]
                if isinstance(result, list):
                    return result
                elif isinstance(result, dict) and "concept_ids" in result:
                    return result["concept_ids"]
            return []
        except Exception:
            # Server doesn't support this yet, return empty list
            return []
    
    def vector_search(
        self,
        query_vector: List[float],
        k: int = 10,
        ef_search: int = 50,
    ) -> List[Tuple[str, float]]:
        """Vector similarity search"""
        # Convert numpy array to list if needed
        if hasattr(query_vector, 'tolist'):
            query_vector = query_vector.tolist()
        elif hasattr(query_vector, '__iter__'):
            query_vector = [float(x) for x in query_vector]
        
        response = self._send_request("VectorSearch", {
            "query_vector": query_vector,
            "k": k,
            "ef_search": ef_search,
        })
        
        if "Error" in response:
            raise RuntimeError(response["Error"]["message"])
        
        if "VectorSearchOk" in response:
            data = response["VectorSearchOk"]
            # Handle nested list format: [[['id', score], ['id', score]]]
            if isinstance(data, list):
                if len(data) > 0 and isinstance(data[0], list):
                    # Flatten one level: [['id', score], ['id', score]]
                    results = data[0] if len(data) > 0 else []
                    return [(r[0], r[1]) for r in results if len(r) >= 2]
                else:
                    # Direct format: [['id', score], ['id', score]]
                    return [(r[0], r[1]) for r in data if len(r) >= 2]
            # If data is a dict with "results" key, use that
            elif isinstance(data, dict) and "results" in data:
                return [(r[0], r[1]) for r in data["results"]]
        
        return []
    
    def stats(self) -> Dict:
        """Get storage statistics"""
        # GetStats is a unit variant - send just the string
        request = "GetStats"
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
        
        response = msgpack.unpackb(response_bytes, raw=False)
        
        if "Error" in response:
            raise RuntimeError(response["Error"]["message"])
        
        if "StatsOk" in response:
            data = response["StatsOk"]
            # If data is a list, convert to dict
            if isinstance(data, list):
                return {
                    "total_concepts": data[0] if len(data) > 0 else 0,
                    "total_edges": data[1] if len(data) > 1 else 0,
                    "writes": data[2] if len(data) > 2 else 0,
                    "drops": data[3] if len(data) > 3 else 0,
                    "pending": data[4] if len(data) > 4 else 0,
                    "reconciliations": data[5] if len(data) > 5 else 0,
                    "uptime_seconds": data[6] if len(data) > 6 else 0,
                }
            return data
        
        return {}
    
    def flush(self) -> None:
        """Force flush to disk"""
        # Flush is a unit variant - send just the string
        request = "Flush"
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
        
        response = msgpack.unpackb(response_bytes, raw=False)
        
        if "Error" in response:
            raise RuntimeError(f"Flush failed: {response['Error']['message']}")
    
    def health_check(self) -> Dict:
        """Check server health"""
        # HealthCheck is a unit variant - send just the string
        request = "HealthCheck"
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
        
        response = msgpack.unpackb(response_bytes, raw=False)
        
        if "Error" in response:
            raise RuntimeError(response["Error"]["message"])
        
        return response.get("HealthCheckOk", {})
    
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
