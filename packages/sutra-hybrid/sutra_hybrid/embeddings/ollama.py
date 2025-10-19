"""
Ollama embedding provider using granite-embedding:30m model.

Provides embeddings via Ollama API for the granite-embedding:30m model.
"""

import json
import logging
import os
from typing import List

import numpy as np
import requests

from .base import EmbeddingProvider

logger = logging.getLogger(__name__)


class OllamaEmbedding(EmbeddingProvider):
    """
    Ollama embedding provider using granite-embedding:30m.
    
    Connects to local Ollama service to generate embeddings using
    the granite-embedding:30m model, normalized to 768 dimensions.
    """

    def __init__(
        self,
        model_name: str = "nomic-embed-text",
        ollama_url: str = None,
    ):
        """
        Initialize Ollama embedding provider - PRODUCTION: nomic-embed-text only.

        Args:
            model_name: Name of the Ollama model (MUST be nomic-embed-text)
            ollama_url: URL of the Ollama API service

        Raises:
            ValueError: If model is not nomic-embed-text
            ConnectionError: If cannot connect to Ollama service
        """
        # PRODUCTION: Enforce nomic-embed-text for 768-dimensional embeddings
        if model_name != "nomic-embed-text":
            raise ValueError(
                f"PRODUCTION REQUIREMENT: Only nomic-embed-text (768-d) is supported. "
                f"Got: {model_name}. Update configuration."
            )
        
        self.model_name = model_name
        # Use environment variable or provided URL or default
        self.ollama_url = (
            ollama_url
            or os.getenv("SUTRA_OLLAMA_URL", "http://localhost:11434")
        )
        self.api_url = f"{self.ollama_url}/api"
        
        # Test connection and ensure model is available
        self._ensure_model_available()
        logger.info(f"✅ PRODUCTION: Initialized OllamaEmbedding with {model_name} (768-d)")

    def _ensure_model_available(self):
        """Ensure the model is available in Ollama."""
        try:
            # Check if model is available
            response = requests.get(f"{self.api_url}/tags", timeout=10)
            response.raise_for_status()
            
            models = response.json().get("models", [])
            available_models = [model["name"] for model in models]
            
            if self.model_name not in available_models:
                logger.info(f"Model {self.model_name} not found, pulling...")
                self._pull_model()
                
        except requests.RequestException as e:
            raise ConnectionError(f"Cannot connect to Ollama at {self.ollama_url}: {e}")

    def _pull_model(self):
        """Pull the model if not available."""
        try:
            response = requests.post(
                f"{self.api_url}/pull",
                json={"name": self.model_name},
                timeout=300,  # 5 minutes for model download
            )
            response.raise_for_status()
            logger.info(f"Successfully pulled model: {self.model_name}")
        except requests.RequestException as e:
            raise RuntimeError(f"Failed to pull model {self.model_name}: {e}")

    def encode(self, texts: List[str]) -> np.ndarray:
        """
        Encode texts into embeddings using Ollama.

        Args:
            texts: List of text strings to encode

        Returns:
            numpy array of shape (len(texts), 768) - standardized dimension
        """
        if not texts:
            return np.array([]).reshape(0, self.get_dimension())

        embeddings = []
        
        for text in texts:
            try:
                response = requests.post(
                    f"{self.api_url}/embeddings",
                    json={
                        "model": self.model_name,
                        "prompt": text,
                    },
                    timeout=30,
                )
                response.raise_for_status()
                
                result = response.json()
                raw_embedding = np.array(result["embedding"], dtype=np.float32)
                
                # PRODUCTION: Enforce 768-d, no normalization
                if len(raw_embedding) != 768:
                    raise ValueError(
                        f"PRODUCTION ERROR: Model {self.model_name} returned {len(raw_embedding)}-d "
                        f"embedding, expected 768-d. Ensure nomic-embed-text is configured correctly."
                    )
                
                # L2 normalize
                norm = np.linalg.norm(raw_embedding)
                if norm > 0:
                    embedding = raw_embedding / norm
                else:
                    embedding = raw_embedding
                    
                embeddings.append(embedding)
                
            except requests.RequestException as e:
                logger.error(f"Failed to get embedding for text: {text[:50]}... Error: {e}")
                # Return zero embedding as fallback
                embeddings.append(np.zeros(self.get_dimension(), dtype=np.float32))
                
        return np.array(embeddings)

    def get_dimension(self) -> int:
        """
        Get embedding dimension - PRODUCTION: strict 768-d requirement.

        Returns:
            768 (nomic-embed-text native dimension)
        """
        return 768

    def get_name(self) -> str:
        """
        Get provider name.

        Returns:
            Provider identifier string
        """
        return f"ollama-{self.model_name}"

    def similarity(self, embedding1: np.ndarray, embedding2: np.ndarray) -> float:
        """
        Calculate cosine similarity between two embeddings.

        Args:
            embedding1: First embedding vector
            embedding2: Second embedding vector

        Returns:
            Cosine similarity score between -1 and 1
        """
        # Ensure embeddings are normalized
        norm1 = np.linalg.norm(embedding1)
        norm2 = np.linalg.norm(embedding2)
        
        if norm1 == 0 or norm2 == 0:
            return 0.0
            
        normalized1 = embedding1 / norm1
        normalized2 = embedding2 / norm2
        
        return float(np.dot(normalized1, normalized2))