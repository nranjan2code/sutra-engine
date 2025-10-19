"""
Embedding providers for semantic understanding.

This module provides different embedding strategies:
- OllamaEmbedding: Using Ollama with granite-embedding:30m (768 dimensions)
- SemanticEmbedding: Using sentence-transformers with EmbeddingGemma (768 dimensions)
- TfidfEmbedding: Lightweight TF-IDF fallback (100 dimensions)
"""

from .base import EmbeddingProvider
from .ollama import OllamaEmbedding
from .semantic import SemanticEmbedding
from .tfidf import TfidfEmbedding

__all__ = ["EmbeddingProvider", "OllamaEmbedding", "SemanticEmbedding", "TfidfEmbedding"]
