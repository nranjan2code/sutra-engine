"""
Mass learning adapters for different data sources.

This module provides adapters for ingesting knowledge from various sources:
- Files (text, markdown, JSON, CSV)
- External data sources (for ingestion only - Sutra uses TCP binary protocol)
- Datasets (Wikipedia, academic papers)
- Streaming systems (Kafka, message queues)
"""

from .base import ChunkMetadata, LearningProgress, MassLearningAdapter
from .dataset_adapter import DatasetAdapter
from .file_adapter import FileAdapter

__all__ = [
    "MassLearningAdapter",
    "LearningProgress",
    "ChunkMetadata",
    "FileAdapter",
    "DatasetAdapter",
]
