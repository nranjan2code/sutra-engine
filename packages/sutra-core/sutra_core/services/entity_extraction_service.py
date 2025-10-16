"""
Independent Entity Extraction Service.

This service runs independently from the main reasoning engine.
It monitors a processing queue and extracts entities using Ollama Cloud API.

Separation of Concerns:
- Reasoning Engine: Fast, real-time, uses cached entities
- This Service: Async, batch processing, updates cache
"""

import json
import logging
import os
import time
from pathlib import Path
from typing import Any, Dict, List, Optional, Union

from ollama import Client

logger = logging.getLogger(__name__)


class EntityExtractionService:
    """
    Background service for extracting entities using Ollama Cloud.

    Runs independently, monitors processing queue, and updates entity cache.
    """

    def __init__(
        self,
        storage_path: str = "./knowledge",
        ollama_api_key: Optional[str] = None,
        model: str = "gpt-oss:120b-cloud",
        batch_size: int = 10,
        poll_interval: float = 5.0,
    ):
        """
        Initialize entity extraction service.

        Args:
            storage_path: Path to shared storage directory
            ollama_api_key: Ollama Cloud API key (or use OLLAMA_API_KEY env var)
            model: Ollama Cloud model to use
            batch_size: Number of concepts to process in one batch
            poll_interval: Seconds between queue checks
        """
        self.storage_path = Path(storage_path)
        self.storage_path.mkdir(parents=True, exist_ok=True)

        self.batch_size = batch_size
        self.poll_interval = poll_interval

        # File paths
        self.entity_cache_path = self.storage_path / "entity_cache.json"
        self.processing_queue_path = self.storage_path / "processing_queue.json"
        self.concepts_path = self.storage_path / "concepts.json"

        # Initialize files if they don't exist
        self._init_storage()

        # Initialize Ollama client
        api_key = ollama_api_key or os.environ.get("OLLAMA_API_KEY")
        if not api_key:
            raise ValueError(
                "Ollama API key required. Set OLLAMA_API_KEY env var or pass ollama_api_key parameter."
            )

        self.ollama = Client(
            host="https://ollama.com", headers={"Authorization": f"Bearer {api_key}"}
        )
        self.model = model

        logger.info(f"Entity Extraction Service initialized with model: {model}")
        logger.info(f"Storage path: {storage_path}")

    def _init_storage(self) -> None:
        """Initialize storage files if they don't exist."""
        if not self.entity_cache_path.exists():
            self.entity_cache_path.write_text("{}")

        if not self.processing_queue_path.exists():
            self.processing_queue_path.write_text("[]")

        if not self.concepts_path.exists():
            self.concepts_path.write_text("{}")

    def _load_json(self, path: Path) -> Union[Dict[str, Any], List[Any]]:
        """Load JSON file safely."""
        try:
            return json.loads(path.read_text())
        except Exception as e:
            logger.error(f"Error loading {path}: {e}")
            return {} if "cache" in str(path) or "concepts" in str(path) else []

    def _save_json(self, path: Path, data: Union[Dict[str, Any], List[Any]]) -> None:
        """Save JSON file safely."""
        try:
            path.write_text(json.dumps(data, indent=2))
        except Exception as e:
            logger.error(f"Error saving {path}: {e}")

    def get_processing_queue(self) -> List[str]:
        """Get list of concept IDs needing entity extraction."""
        result = self._load_json(self.processing_queue_path)
        if not isinstance(result, list):
            logger.warning(f"Processing queue is not a list, returning empty list")
            return []
        return result

    def clear_from_queue(self, concept_ids: List[str]) -> None:
        """Remove processed concept IDs from queue."""
        """Remove processed concept IDs from queue."""
        queue = self.get_processing_queue()
        queue = [cid for cid in queue if cid not in concept_ids]
        self._save_json(self.processing_queue_path, queue)

    def get_concept(self, concept_id: str) -> Optional[Dict[str, Any]]:
        """Get concept content by ID."""
        concepts = self._load_json(self.concepts_path)
        if not isinstance(concepts, dict):
            logger.warning(f"Concepts file is not a dict, returning None")
            return None
        return concepts.get(concept_id)

    def extract_entities_with_ollama(self, content: str) -> List[Dict[str, Any]]:
        """
        Extract entities from content using Ollama Cloud.

        Args:
            content: Text to extract entities from

        Returns:
            List of entities: [{"text": "...", "type": "...", "confidence": 0.95}, ...]
        """
        system_prompt = """You are an entity extraction expert. Extract all important entities from the text.
        
For each entity, identify:
- text: The exact entity text
- type: Entity type (animal, person, technology, concept, object, event, location, etc.)
- confidence: Your confidence (0.0-1.0)

Return JSON format:
{"entities": [{"text": "Dogs", "type": "animal", "confidence": 0.95}, ...]}
"""

        try:
            response = self.ollama.chat(
                model=self.model,
                messages=[
                    {"role": "system", "content": system_prompt},
                    {"role": "user", "content": f'Extract entities from: "{content}"'},
                ],
                format="json",
                options={"temperature": 0.1},  # Low temperature for consistency
            )

            # Parse response
            result = json.loads(response["message"]["content"])
            return result.get("entities", [])

        except Exception as e:
            logger.error(f"Error extracting entities with Ollama: {e}")
            return []

    def process_batch(self, concept_ids: List[str]) -> int:
        """
        Process a batch of concepts and update entity cache.

        Args:
            concept_ids: List of concept IDs to process

        Returns:
            Number of concepts successfully processed
        """
        entity_cache_data = self._load_json(self.entity_cache_path)
        if not isinstance(entity_cache_data, dict):
            logger.error("Entity cache is not a dict, initializing empty dict")
            entity_cache_data = {}

        processed_count = 0

        for concept_id in concept_ids:
            # Get concept content
            concept = self.get_concept(concept_id)
            if not concept:
                logger.warning(f"Concept {concept_id} not found, skipping")
                continue

            content = concept.get("content", "")
            if not content:
                logger.warning(f"Concept {concept_id} has no content, skipping")
                continue

            # Extract entities
            logger.info(f"Extracting entities for: {content[:50]}...")
            entities = self.extract_entities_with_ollama(content)

            # Update cache
            entity_cache_data[concept_id] = {
                "entities": entities,
                "timestamp": time.time(),
                "model": self.model,
            }

            processed_count += 1
            logger.info(f"  → Found {len(entities)} entities")

        # Save updated cache
        self._save_json(self.entity_cache_path, entity_cache_data)

        # Remove from queue
        self.clear_from_queue(concept_ids)

        return processed_count

    def run_once(self) -> int:
        """
        Process one batch from the queue.

        Returns:
            Number of concepts processed
        """
        queue = self.get_processing_queue()

        if not queue:
            return 0

        # Take a batch
        batch = queue[: self.batch_size]
        logger.info(f"Processing batch of {len(batch)} concepts")

        return self.process_batch(batch)

    def run_forever(self) -> None:
        """
        Run service continuously, monitoring queue.

        This is the main loop for the background service.
        """
        logger.info("Entity Extraction Service starting...")
        logger.info(f"Polling every {self.poll_interval}s")

        try:
            while True:
                processed = self.run_once()

                if processed > 0:
                    logger.info(f"Processed {processed} concepts")

                # Sleep until next poll
                time.sleep(self.poll_interval)

        except KeyboardInterrupt:
            logger.info("Entity Extraction Service stopped by user")
        except Exception as e:
            logger.error(f"Entity Extraction Service error: {e}")
            raise


def main() -> None:
    """Run the entity extraction service as a standalone program."""
    import sys

    # Configure logging
    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
    )

    # Get storage path from command line or use default
    storage_path = sys.argv[1] if len(sys.argv) > 1 else "./knowledge"

    # Create and run service
    service = EntityExtractionService(storage_path=storage_path)
    service.run_forever()


if __name__ == "__main__":
    main()
