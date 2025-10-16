"""
Entity cache for fast entity lookups.

This module provides a simple interface for the reasoning engine
to access cached entity extractions without calling LLMs.
"""

import json
import logging
from pathlib import Path
from typing import Dict, List, Optional

logger = logging.getLogger(__name__)


class EntityCache:
    """
    Fast entity cache for reasoning engine.

    Reads from cache populated by background entity extraction service.
    Does NOT call LLMs - just reads cached data.
    """

    def __init__(self, storage_path: str = "./knowledge"):
        """
        Initialize entity cache.

        Args:
            storage_path: Path to shared storage directory
        """
        self.storage_path = Path(storage_path)
        self.entity_cache_path = self.storage_path / "entity_cache.json"
        self.processing_queue_path = self.storage_path / "processing_queue.json"

        # In-memory cache for fast access
        self._cache: Dict[str, List[Dict]] = {}
        self._load_cache()

    def _load_cache(self) -> None:
        """Load entity cache from disk into memory."""
        if not self.entity_cache_path.exists():
            logger.warning(f"Entity cache not found at {self.entity_cache_path}")
            return

        try:
            data = json.loads(self.entity_cache_path.read_text())
            # Extract just the entities (discard metadata)
            for concept_id, cache_entry in data.items():
                if isinstance(cache_entry, dict) and "entities" in cache_entry:
                    self._cache[concept_id] = cache_entry["entities"]
                else:
                    self._cache[concept_id] = cache_entry

            logger.info(f"Loaded {len(self._cache)} cached entity extractions")
        except Exception as e:
            logger.error(f"Error loading entity cache: {e}")

    def get(self, concept_id: str) -> Optional[List[Dict]]:
        """
        Get cached entities for a concept.

        Args:
            concept_id: Concept ID to look up

        Returns:
            List of entities or None if not in cache
            Entities format: [{"text": "Dogs", "type": "animal", "confidence": 0.95}, ...]
        """
        return self._cache.get(concept_id)

    def has(self, concept_id: str) -> bool:
        """Check if concept has cached entities."""
        return concept_id in self._cache

    def add_to_processing_queue(self, concept_id: str) -> None:
        """
        Add concept to processing queue for background extraction.

        This tells the background service to extract entities for this concept.
        """
        try:
            # Load current queue
            if self.processing_queue_path.exists():
                queue = json.loads(self.processing_queue_path.read_text())
            else:
                queue = []

            # Add if not already in queue
            if concept_id not in queue:
                queue.append(concept_id)
                self.processing_queue_path.write_text(json.dumps(queue, indent=2))
                logger.debug(f"Added {concept_id} to processing queue")

        except Exception as e:
            logger.error(f"Error adding to processing queue: {e}")

    def reload(self) -> None:
        """Reload cache from disk (for picking up background updates)."""
        self._cache.clear()
        self._load_cache()

    def stats(self) -> Dict:
        """Get cache statistics."""
        return {
            "cached_concepts": len(self._cache),
            "total_entities": sum(len(entities) for entities in self._cache.values()),
        }
