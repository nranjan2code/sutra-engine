"""
Embedding providers for semantic understanding.

This module provides different embedding strategies:
- EmbeddingServiceProvider: High-performance service using nomic-embed-text-v1.5 (768 dimensions)
- SemanticEmbedding: Using sentence-transformers with EmbeddingGemma (768 dimensions) 
- TfidfEmbedding: Lightweight TF-IDF fallback (100 dimensions)
"""

from .base import EmbeddingProvider
from .service import EmbeddingServiceProvider
from .semantic import SemanticEmbedding
from .tfidf import TfidfEmbedding

__all__ = ["EmbeddingProvider", "EmbeddingServiceProvider", "SemanticEmbedding", "TfidfEmbedding"]
