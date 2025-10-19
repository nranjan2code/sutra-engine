"""
Ollama-based NLP processor adapter for sutra-core.

Provides TextProcessor-compatible interface using Ollama embeddings
instead of spaCy/sentence-transformers.
"""

import logging
import os
from typing import List, Optional, Tuple

import numpy as np

from .embeddings import OllamaEmbedding

logger = logging.getLogger(__name__)


class OllamaNLPProcessor:
    """
    NLP processor adapter that uses Ollama embeddings.
    
    Implements the same interface as sutra-core's TextProcessor
    but uses Ollama for embeddings instead of sentence-transformers.
    
    Note: Only implements embedding functionality. Text processing features
    like tokenization, NER, etc. are not available without spaCy.
    """

    def __init__(
        self,
        model_name: str = "granite-embedding:30m",
        ollama_url: str = None,
    ):
        """
        Initialize Ollama NLP processor.

        Args:
            model_name: Ollama model name for embeddings
            ollama_url: Ollama service URL (uses environment variable if None)
        """
        self.ollama_embedding = OllamaEmbedding(
            model_name=model_name,
            ollama_url=ollama_url
        )
        self.model_name = model_name
        
        logger.info(f"Initialized OllamaNLPProcessor with {model_name}")

    def get_embedding(self, text: str) -> Optional[np.ndarray]:
        """
        Get vector embedding for text using Ollama.

        Args:
            text: Input text

        Returns:
            768-dimensional embedding array or None if text is empty
        """
        if not text or not text.strip():
            return None

        try:
            # Use Ollama embedding (returns 768-dim array)
            embeddings = self.ollama_embedding.encode([text])
            if len(embeddings) > 0:
                return embeddings[0]
            return None
        except Exception as e:
            logger.error(f"Failed to generate embedding: {e}")
            return None

    def get_embedding_dimension(self) -> int:
        """
        Get the dimensionality of embeddings.

        Returns:
            768 (standardized dimension)
        """
        return 768

    def get_embeddings_batch(self, texts: List[str]) -> Optional[np.ndarray]:
        """
        Get embeddings for multiple texts in batch.

        Args:
            texts: List of input texts

        Returns:
            numpy array of embeddings (N x 768)
        """
        if not texts:
            return None

        try:
            return self.ollama_embedding.encode(texts)
        except Exception as e:
            logger.error(f"Failed to generate batch embeddings: {e}")
            return None

    def similarity(self, text1: str, text2: str) -> float:
        """
        Compute semantic similarity between two texts.

        Args:
            text1: First text
            text2: Second text

        Returns:
            Similarity score (0.0 - 1.0)
        """
        if not text1 or not text2:
            return 0.0

        try:
            emb1 = self.get_embedding(text1)
            emb2 = self.get_embedding(text2)
            
            if emb1 is None or emb2 is None:
                return 0.0
                
            return float(self.ollama_embedding.similarity(emb1, emb2))
        except Exception as e:
            logger.error(f"Failed to compute similarity: {e}")
            return 0.0

    # Placeholder methods for TextProcessor compatibility
    # These would require spaCy for full implementation

    def extract_meaningful_tokens(self, text: str, min_length: int = 2, include_entities: bool = True) -> List[str]:
        """
        Basic tokenization fallback (without spaCy).
        
        Returns simple word tokens without lemmatization or entity recognition.
        """
        if not text or not text.strip():
            return []
        
        # Simple whitespace tokenization as fallback
        tokens = [
            word.lower().strip('.,!?;:"()[]{}')
            for word in text.split()
            if len(word.strip('.,!?;:"()[]{}')) >= min_length
        ]
        
        return [t for t in tokens if t]  # Remove empty strings

    def extract_entities(self, text: str) -> List[Tuple[str, str]]:
        """Not implemented without spaCy."""
        return []

    def extract_noun_chunks(self, text: str) -> List[str]:
        """Not implemented without spaCy."""
        return []

    def detect_negation(self, text: str) -> bool:
        """Simple negation detection without spaCy."""
        if not text:
            return False
        
        negation_words = {
            'not', 'no', 'never', 'nothing', 'nowhere', 'nobody', 
            'none', 'neither', 'nor', 'cannot', "can't", "won't", 
            "don't", "doesn't", "didn't", "isn't", "aren't", "wasn't", 
            "weren't", "haven't", "hasn't", "hadn't"
        }
        
        words = text.lower().split()
        return any(word in negation_words for word in words)

    def extract_subject_verb_object(self, text: str) -> List[Tuple[str, str, str, bool]]:
        """Not implemented without spaCy."""
        return []

    def extract_causal_relations(self, text: str) -> List[Tuple[str, str, bool]]:
        """Not implemented without spaCy."""
        return []