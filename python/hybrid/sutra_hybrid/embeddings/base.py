"""
Base interface for embedding providers.

Defines the common interface that all embedding providers must implement.
"""

from abc import ABC, abstractmethod
from typing import List

import numpy as np


class EmbeddingProvider(ABC):
    """Abstract base class for embedding providers."""

    @abstractmethod
    def encode(self, texts: List[str]) -> np.ndarray:
        """
        Encode texts into embeddings.

        Args:
            texts: List of text strings to encode

        Returns:
            numpy array of shape (len(texts), embedding_dim)
        """
        pass

    @abstractmethod
    def get_dimension(self) -> int:
        """
        Get the dimensionality of embeddings.

        Returns:
            Embedding dimension size
        """
        pass

    @abstractmethod
    def get_name(self) -> str:
        """
        Get the name/identifier of this embedding provider.

        Returns:
            Provider name string
        """
        pass

    def similarity(self, embedding1: np.ndarray, embedding2: np.ndarray) -> float:
        """
        Calculate cosine similarity between two embeddings.

        Args:
            embedding1: First embedding vector
            embedding2: Second embedding vector

        Returns:
            Similarity score between 0 and 1
        """
        # Normalize vectors
        norm1 = np.linalg.norm(embedding1)
        norm2 = np.linalg.norm(embedding2)

        if norm1 == 0 or norm2 == 0:
            return 0.0

        # Cosine similarity
        similarity = np.dot(embedding1, embedding2) / (norm1 * norm2)

        # Clip to [0, 1] range
        return float(np.clip(similarity, 0.0, 1.0))
