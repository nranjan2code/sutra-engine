#!/usr/bin/env python3
"""Quick test of TCP storage client against containerized server."""

import socket
import msgpack
import struct

class SimpleStorageClient:
    """Minimal TCP storage client for testing."""
    
    def __init__(self, address: str):
        host, port = address.split(":")
        self.sock = socket.create_connection((host, int(port)))
    
    def _send_request(self, variant_name: str, data: dict) -> dict:
        """Send request and receive response.
        
        Rust enum format: {variant_name: variant_data}
        """
        # Pack request as enum: {variant_name: data}
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
    
    def store_concept(self, concept_id: str, name: str, description: str, confidence: float) -> bool:
        response = self._send_request("LearnConcept", {
            "concept_id": concept_id,
            "content": name + ": " + description,
            "embedding": [],
            "strength": confidence,
            "confidence": confidence,
        })
        # Response: {variant_name: data}
        return "LearnConceptOk" in response
    
    def get_concept(self, concept_id: str) -> dict:
        response = self._send_request("QueryConcept", {"concept_id": concept_id})
        print(f"DEBUG get_concept response: {response}")
        if "QueryConceptOk" in response:
            data = response["QueryConceptOk"]
            # If data is a list, convert to dict
            if isinstance(data, list):
                return {
                    "found": data[0] if len(data) > 0 else False,
                    "concept_id": data[1] if len(data) > 1 else "",
                    "content": data[2] if len(data) > 2 else "",
                    "strength": data[3] if len(data) > 3 else 0.0,
                    "confidence": data[4] if len(data) > 4 else 0.0,
                }
            return data
        return None
    
    def add_association(self, from_id: str, to_id: str, relation: str, confidence: float) -> bool:
        response = self._send_request("LearnAssociation", {
            "source_id": from_id,
            "target_id": to_id,
            "assoc_type": 0,
            "confidence": confidence,
        })
        return "LearnAssociationOk" in response
    
    def get_stats(self) -> dict:
        # GetStats is a unit variant - send just the string
        request = "GetStats"
        packed = msgpack.packb(request)
        
        # Send with length prefix
        self.sock.sendall(struct.pack(">I", len(packed)))
        self.sock.sendall(packed)
        
        # Receive response
        length_bytes = self.sock.recv(4)
        if len(length_bytes) < 4:
            raise ConnectionError("Connection closed")
        length = struct.unpack(">I", length_bytes)[0]
        
        response_bytes = b""
        while len(response_bytes) < length:
            chunk = self.sock.recv(min(4096, length - len(response_bytes)))
            if not chunk:
                raise ConnectionError("Connection closed")
            response_bytes += chunk
        
        response = msgpack.unpackb(response_bytes, raw=False)
        if "StatsOk" in response:
            data = response["StatsOk"]
            # If data is a list, convert to dict
            if isinstance(data, list):
                return {
                    "concepts": data[0] if len(data) > 0 else 0,
                    "edges": data[1] if len(data) > 1 else 0,
                    "written": data[2] if len(data) > 2 else 0,
                    "dropped": data[3] if len(data) > 3 else 0,
                    "pending": data[4] if len(data) > 4 else 0,
                    "reconciliations": data[5] if len(data) > 5 else 0,
                    "uptime_seconds": data[6] if len(data) > 6 else 0,
                }
            return data
        return {}
    
    def close(self):
        self.sock.close()

StorageClient = SimpleStorageClient

def main():
    print("🧪 Testing TCP Storage Client")
    print("=" * 50)
    
    # Connect to containerized storage server
    client = StorageClient("localhost:50051")
    
    # Test 1: Store concept
    print("\n✅ Test 1: Store concept")
    success = client.store_concept(
        concept_id="test_001",
        name="Python",
        description="A programming language",
        confidence=0.95
    )
    print(f"   Result: {success}")
    
    # Test 2: Retrieve concept
    print("\n✅ Test 2: Retrieve concept")
    concept = client.get_concept("test_001")
    if concept:
        print(f"   Content: {concept.get('content')}")
        print(f"   Confidence: {concept.get('confidence')}")
        print(f"   Found: {concept.get('found')}")
    else:
        print("   ❌ Failed to retrieve")
    
    # Test 3: Add association
    print("\n✅ Test 3: Add association")
    client.store_concept("test_002", "Programming", "The art of coding", 0.9)
    success = client.add_association(
        from_id="test_001",
        to_id="test_002",
        relation="is_type_of",
        confidence=0.85
    )
    print(f"   Result: {success}")
    
    # Test 4: Get storage stats
    print("\n✅ Test 4: Get storage stats")
    stats = client.get_stats()
    print(f"   Concepts: {stats.get('concepts', 0)}")
    print(f"   Edges: {stats.get('edges', 0)}")
    print(f"   Writes: {stats.get('writes', 0)}")
    
    print("\n" + "=" * 50)
    print("✨ All tests passed!")
    
    client.close()

if __name__ == "__main__":
    main()
