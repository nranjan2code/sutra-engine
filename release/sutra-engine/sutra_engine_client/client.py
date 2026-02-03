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
        """
        options = {
            "generate_embedding": kwargs.get("generate_embedding", True),
            "extract_associations": kwargs.get("extract_associations", True),
            "confidence": kwargs.get("confidence", 1.0),
            "strength": kwargs.get("strength", 1.0),
            "min_association_confidence": 0.5,
            "max_associations_per_concept": 10
        }
        return self.learn_concept_v2(content, options=options)

    def search(self, query: str, limit: int = 5) -> List[Dict]:
        """
        Search for related knowledge using semantic vector search.
        """
        try:
            results = self._send_request({
                "TextSearch": {
                    "query": query,
                    "limit": limit
                }
            })
            if "TextSearchOk" in results:
                return [{"id": id_hex, "score": score} for id_hex, score in results["TextSearchOk"]["results"]]
            return []
        except Exception as e:
            logger.error(f"Search failed: {e}")
            return []
