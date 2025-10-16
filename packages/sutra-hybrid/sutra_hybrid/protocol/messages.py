"""
Sutra Binary Protocol (SBP) Message Types.

Binary message format for efficient internal communication.
Not meant to be human-readable - optimized for performance.
"""

from dataclasses import dataclass
from enum import IntEnum
from typing import List, Optional

import numpy as np


class MessageType(IntEnum):
    """Message type identifiers (1 byte)."""

    # Learning operations
    LEARN = 0x01
    LEARN_BATCH = 0x02

    # Query operations
    QUERY = 0x10
    QUERY_MULTI_PATH = 0x11

    # Result operations
    RESULT = 0x20
    RESULT_MULTI_PATH = 0x21

    # Graph operations
    CONCEPT_READ = 0x30
    CONCEPT_WRITE = 0x31
    ASSOCIATION_READ = 0x32
    ASSOCIATION_WRITE = 0x33
    PATH_READ = 0x34

    # Storage operations
    STORAGE_SAVE = 0x40
    STORAGE_LOAD = 0x41
    STORAGE_STATS = 0x42

    # Vector operations
    VECTOR_SEARCH = 0x50
    VECTOR_INSERT = 0x51

    # Error
    ERROR = 0xFF


@dataclass
class SBPMessage:
    """
    Base message structure.

    Binary format:
    - Header: 16 bytes
      - Magic: 0x53425000 (SBP\0) [4 bytes]
      - Version: uint8 [1 byte]
      - Type: MessageType [1 byte]
      - Flags: uint8 [1 byte]
      - Reserved: uint8 [1 byte]
      - Payload length: uint64 [8 bytes]
    - Payload: variable
    """

    version: int
    msg_type: MessageType
    flags: int = 0
    payload: bytes = b""

    MAGIC = 0x53425000  # "SBP\0"
    HEADER_SIZE = 16


@dataclass
class LearnMessage:
    """
    Learn operation message.

    Optimized for concept learning with optional metadata.
    """

    content: str
    source: Optional[str] = None
    category: Optional[str] = None
    embedding: Optional[np.ndarray] = None
    metadata: Optional[dict] = None


@dataclass
class QueryMessage:
    """
    Query operation message.

    Optimized for graph traversal queries.
    """

    query: str
    num_paths: int = 5
    max_depth: int = 5
    semantic_boost: bool = True
    min_confidence: float = 0.5
    embedding: Optional[np.ndarray] = None


@dataclass
class ResultMessage:
    """
    Query result message.

    Compact representation of reasoning results.
    """

    answer: str
    confidence: float
    graph_confidence: float
    semantic_confidence: float
    paths: List["PathMessage"]
    concepts_accessed: int
    execution_time_ms: float
    metadata: Optional[dict] = None


@dataclass
class ConceptMessage:
    """
    Concept data message.

    Binary format optimized for concept data:
    - concept_id: 16 bytes (MD5)
    - content_length: uint32
    - content: variable UTF-8
    - strength: float32
    - confidence: float32
    - access_count: uint32
    - created_ts: uint64
    - modified_ts: uint64
    - embedding_present: bool
    - embedding: 768 float32 (if present)
    """

    concept_id: bytes  # 16 bytes MD5
    content: str
    strength: float
    confidence: float
    access_count: int
    created_ts: int
    modified_ts: int
    embedding: Optional[np.ndarray] = None

    @property
    def binary_size(self) -> int:
        """Calculate binary size for this concept."""
        base = 16 + 4 + len(self.content.encode("utf-8")) + 4 + 4 + 4 + 8 + 8 + 1
        if self.embedding is not None:
            base += 768 * 4  # 768 float32
        return base


@dataclass
class AssociationMessage:
    """
    Association data message.

    Binary format:
    - source_id: 16 bytes
    - target_id: 16 bytes
    - assoc_type: uint8
    - confidence: float32
    - weight: float32
    - created_ts: uint64
    - last_used_ts: uint64
    """

    source_id: bytes  # 16 bytes
    target_id: bytes  # 16 bytes
    assoc_type: int
    confidence: float
    weight: float
    created_ts: int
    last_used_ts: int

    BINARY_SIZE = 16 + 16 + 1 + 4 + 4 + 8 + 8  # 57 bytes


@dataclass
class PathMessage:
    """
    Reasoning path message.

    Compact representation of reasoning path:
    - num_concepts: uint16
    - concept_ids: array of 16-byte IDs
    - num_associations: uint16
    - association_types: array of uint8
    - confidence: float32
    - explanation_length: uint16
    - explanation: UTF-8 string
    """

    concept_ids: List[bytes]  # List of 16-byte IDs
    association_types: List[int]
    confidence: float
    explanation: str

    @property
    def binary_size(self) -> int:
        """Calculate binary size."""
        return (
            2  # num_concepts
            + len(self.concept_ids) * 16
            + 2  # num_associations
            + len(self.association_types)
            + 4  # confidence
            + 2  # explanation_length
            + len(self.explanation.encode("utf-8"))
        )


@dataclass
class VectorSearchMessage:
    """
    Vector search operation message.

    Optimized for HNSW/semantic search.
    """

    query_vector: np.ndarray  # 768 float32
    top_k: int
    threshold: float
    use_hnsw: bool = True


@dataclass
class ErrorMessage:
    """Error message."""

    error_code: int
    error_message: str
    stacktrace: Optional[str] = None
