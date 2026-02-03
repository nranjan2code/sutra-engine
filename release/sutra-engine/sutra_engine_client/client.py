"""
Sutra Engine Python Client
Professional-grade client for the Sutra Memory reasoning engine.
"""

import logging
from typing import Dict, List, Optional, Any
from .base_client import HighPerformanceStorageClient

logger = logging.getLogger("sutra")

class SutraClient(HighPerformanceStorageClient):
    """
    Main client for the Sutra Engine.
    Exposes high-level methods for knowledge ingestion and search.
    """
    
    def __init__(self, host: str = "localhost", port: int = 50051):
        address = f"{host}:{port}"
        super().__init__(address)
        
    def learn(self, content: str, **kwargs) -> str:
        """
        Ingest knowledge into the engine.
        
        Args:
            content: The text to learn.
            generate_embedding: Whether to generate vectors (default: True).
            extract_associations: Whether to look for relationships (default: True).
        """
        options = {
            "generate_embedding": kwargs.get("generate_embedding", True),
            "extract_associations": kwargs.get("extract_associations", True),
            "confidence": kwargs.get("confidence", 1.0),
            "strength": kwargs.get("strength", 1.0)
        }
        return self.learn_concept_v2(content, options=options)

    def get(self, concept_id: str) -> Optional[Dict]:
        """Retrieve a concept by its ID."""
        return self.get_concept(concept_id)

    def search(self, query: str, limit: int = 5) -> List[Dict]:
        """
        Search for related knowledge.
        
        Returns a list of dicts with 'content', 'id', and 'confidence'.
        """
        try:
            results = self._send_request({
                "TextSearch": {
                    "query": query,
                    "limit": limit
                }
            })
            if "TextSearchOk" in results:
                return results["TextSearchOk"]["results"]
            return []
        except Exception as e:
            logger.error(f"Search failed: {e}")
            return []

    def get_neighbors(self, concept_id: str) -> List[Dict]:
        """Get concepts directly connected to the given ID."""
        try:
            response = self._send_request({
                "GetNeighbors": {
                    "concept_id": concept_id
                }
            })
            if "GetNeighborsOk" in response:
                return response["GetNeighborsOk"]["neighbors"]
            return []
        except Exception as e:
            logger.error(f"Failed to get neighbors: {e}")
            return []
