"""
Persistence module for hybrid AI system.

Handles saving and loading of:
- Concepts and associations (from sutra-core)
- Embeddings and embedding metadata
"""

import json
import logging
from pathlib import Path
from typing import Dict, Optional

import numpy as np
from sutra_core import Concept

logger = logging.getLogger(__name__)


class HybridStorage:
    """DEPRECATED: JSON persistence is removed. Use Rust storage instead."""

    def __init__(self, storage_path: Path):
        """
        Initialize storage handler.

        Args:
            storage_path: Directory for storing knowledge
        """
        self.storage_path = storage_path
        self.storage_path.mkdir(parents=True, exist_ok=True)

    def save(
        self,
        concepts: Dict[str, Concept],
        associations: dict,
        embeddings: Dict[str, np.ndarray],
        embedding_provider_name: str,
        vocabulary: dict = None,
        vectorizer_state: Optional[bytes] = None,
    ) -> None:
        raise RuntimeError(
            "HybridStorage.save is removed. Use sutra_core.storage.RustStorageAdapter"
        )

    def load(self) -> tuple:
        raise RuntimeError(
            "HybridStorage.load is removed. Use sutra_core.storage.RustStorageAdapter"
        )

    def exists(self) -> bool:
        """DEPRECATED: always returns False."""
        return False
