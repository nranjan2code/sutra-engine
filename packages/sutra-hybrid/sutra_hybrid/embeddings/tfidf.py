"""
TF-IDF based embedding provider as a lightweight fallback.

Provides TF-IDF vectorization when sentence-transformers is not available
or when lightweight embeddings are preferred.
"""

import logging
import pickle
from typing import List, Optional

import numpy as np
from sklearn.feature_extraction.text import TfidfVectorizer

from .base import EmbeddingProvider

logger = logging.getLogger(__name__)


class TfidfEmbedding(EmbeddingProvider):
    """
    TF-IDF embedding provider for lightweight semantic analysis.

    Provides:
    - 100-dimensional embeddings (configurable)
    - No external model downloads required
    - Fast CPU-only operation
    - Good for keyword-based similarity
    """

    def __init__(self, max_features: int = 100, **kwargs):
        """
        Initialize TF-IDF embedding provider.

        Args:
            max_features: Maximum number of features (embedding dimension)
            **kwargs: Additional arguments for TfidfVectorizer
        """
        self.max_features = max_features
        self.vectorizer = TfidfVectorizer(
            max_features=max_features,
            stop_words="english",
            ngram_range=(1, 2),  # Unigrams and bigrams
            **kwargs,
        )
        self.is_fitted = False
        logger.info(f"Initialized TfidfEmbedding with {max_features} features")

    def fit(self, texts: List[str]) -> None:
        """
        Fit the TF-IDF vectorizer on a corpus.

        Args:
            texts: List of texts to fit the vectorizer
        """
        if texts:
            self.vectorizer.fit(texts)
            self.is_fitted = True
            logger.info(f"Fitted TF-IDF on {len(texts)} documents")

    def encode(self, texts: List[str]) -> np.ndarray:
        """
        Encode texts into TF-IDF embeddings.

        Args:
            texts: List of text strings to encode

        Returns:
            numpy array of shape (len(texts), max_features)
        """
        if not texts:
            return np.array([]).reshape(0, self.get_dimension())

        # Fit if not already fitted
        if not self.is_fitted:
            self.fit(texts)

        # Transform texts to TF-IDF vectors
        embeddings = self.vectorizer.transform(texts).toarray()

        # Normalize to unit vectors (for cosine similarity)
        norms = np.linalg.norm(embeddings, axis=1, keepdims=True)
        norms[norms == 0] = 1  # Avoid division by zero
        embeddings = embeddings / norms

        return embeddings

    def get_dimension(self) -> int:
        """
        Get embedding dimension.

        Returns:
            max_features specified in constructor
        """
        return self.max_features

    def get_name(self) -> str:
        """
        Get provider name.

        Returns:
            Provider identifier string
        """
        return f"tfidf-{self.max_features}d"

    def get_vocabulary(self) -> dict:
        """
        Get the learned vocabulary.

        Returns:
            Dictionary mapping terms to indices (converted to int)
        """
        if self.is_fitted and hasattr(self.vectorizer, "vocabulary_"):
            # Convert numpy int64 to regular int for JSON serialization
            return {k: int(v) for k, v in self.vectorizer.vocabulary_.items()}
        return {}

    def set_vocabulary(self, vocabulary: dict) -> None:
        """
        Set the vocabulary (for loading from disk).

        DEPRECATED: Use get_state() and set_state() for proper persistence.

        Args:
            vocabulary: Dictionary mapping terms to indices
        """
        if vocabulary:
            self.vectorizer.vocabulary = vocabulary
            self.is_fitted = True

    def get_state(self) -> Optional[bytes]:
        """
        Serialize the complete vectorizer state using pickle.

        This preserves the entire sklearn TfidfVectorizer including:
        - Vocabulary
        - IDF values
        - All internal parameters

        Returns:
            Pickled bytes of the vectorizer if fitted, None otherwise
        """
        if self.is_fitted:
            try:
                return pickle.dumps(self.vectorizer)
            except Exception as e:
                logger.error(f"Failed to serialize vectorizer: {e}")
                return None
        return None

    def set_state(self, state_bytes: bytes) -> bool:
        """
        Restore the vectorizer from pickled state.

        Args:
            state_bytes: Pickled bytes from get_state()

        Returns:
            True if successfully restored, False otherwise
        """
        if not state_bytes:
            return False

        try:
            self.vectorizer = pickle.loads(state_bytes)
            self.is_fitted = True
            # Update max_features in case it changed
            if hasattr(self.vectorizer, "max_features"):
                self.max_features = self.vectorizer.max_features or self.max_features
            logger.info("Successfully restored vectorizer from pickled state")
            return True
        except Exception as e:
            logger.error(f"Failed to restore vectorizer: {e}")
            return False
