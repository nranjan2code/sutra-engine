"""
Semantic embedding provider using sentence-transformers.

Provides high-quality semantic embeddings using the sentence-transformers
library. Falls back gracefully if the library is not available.
"""

import logging
from typing import List

import numpy as np

from .base import EmbeddingProvider

logger = logging.getLogger(__name__)


class SemanticEmbedding(EmbeddingProvider):
    """
    Semantic embedding provider using sentence-transformers.

    Uses the 'all-MiniLM-L6-v2' model which provides:
    - 384-dimensional embeddings
    - ~22MB model size
    - Good balance of quality and speed
    - CPU-friendly operation
    """

    def __init__(self, model_name: str = "all-MiniLM-L6-v2"):
        """
        Initialize semantic embedding provider.

        Args:
            model_name: Name of the sentence-transformers model to use

        Raises:
            ImportError: If sentence-transformers is not installed
        """
        try:
            from sentence_transformers import SentenceTransformer

            self.model = SentenceTransformer(model_name)
            self.model_name = model_name
            logger.info(f"Initialized SemanticEmbedding with model: {model_name}")
        except ImportError:
            raise ImportError(
                "sentence-transformers is required for semantic embeddings. "
                "Install with: pip install sentence-transformers"
            )

    def encode(self, texts: List[str]) -> np.ndarray:
        """
        Encode texts into semantic embeddings.

        Args:
            texts: List of text strings to encode

        Returns:
            numpy array of shape (len(texts), 384)
        """
        if not texts:
            return np.array([]).reshape(0, self.get_dimension())

        # Encode using sentence-transformers
        embeddings = self.model.encode(
            texts,
            convert_to_numpy=True,
            show_progress_bar=False,
            normalize_embeddings=True,
        )

        return embeddings

    def get_dimension(self) -> int:
        """
        Get embedding dimension.

        Returns:
            384 (dimension of all-MiniLM-L6-v2)
        """
        return 384

    def get_name(self) -> str:
        """
        Get provider name.

        Returns:
            Provider identifier string
        """
        return f"semantic-{self.model_name}"
