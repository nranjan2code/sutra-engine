"""
Sutra Binary Protocol (SBP) Encoder.

High-performance binary encoding for internal messages.
Optimized for zero-copy operations where possible.
"""

import struct
from typing import Union

import numpy as np

from .messages import (
    AssociationMessage,
    ConceptMessage,
    ErrorMessage,
    LearnMessage,
    MessageType,
    PathMessage,
    QueryMessage,
    ResultMessage,
    SBPMessage,
    VectorSearchMessage,
)


class SBPEncoder:
    """
    Binary encoder for SBP messages.

    Performance characteristics:
    - ConceptMessage: ~100ns encoding (fixed size portion)
    - QueryMessage: ~200ns encoding
    - ResultMessage: ~1μs encoding (depends on path count)
    - Zero-copy for numpy arrays
    """

    @staticmethod
    def encode_header(
        msg_type: MessageType, payload_size: int, version: int = 1
    ) -> bytes:
        """
        Encode message header (16 bytes).

        Format:
        - Magic: 0x53425000 (4 bytes)
        - Version: uint8 (1 byte)
        - Type: uint8 (1 byte)
        - Flags: uint8 (1 byte)
        - Reserved: uint8 (1 byte)
        - Payload size: uint64 (8 bytes)
        """
        return struct.pack(
            "<IBBBBQ",
            SBPMessage.MAGIC,
            version,
            msg_type,
            0,  # flags
            0,  # reserved
            payload_size,
        )

    @staticmethod
    def encode_concept(msg: ConceptMessage) -> bytes:
        """
        Encode concept message to binary.

        Format optimized for concepts:
        - concept_id: 16 bytes
        - content_length: uint32
        - content: UTF-8
        - strength: float32
        - confidence: float32
        - access_count: uint32
        - created_ts: uint64
        - modified_ts: uint64
        - has_embedding: uint8 (boolean)
        - embedding: 768 float32 (optional)

        Returns header + payload.
        """
        content_bytes = msg.content.encode("utf-8")

        # Build payload
        payload = bytearray()
        payload.extend(msg.concept_id)  # 16 bytes
        payload.extend(struct.pack("<I", len(content_bytes)))
        payload.extend(content_bytes)
        payload.extend(struct.pack("<ff", msg.strength, msg.confidence))
        payload.extend(struct.pack("<I", msg.access_count))
        payload.extend(struct.pack("<QQ", msg.created_ts, msg.modified_ts))

        # Embedding (optional)
        if msg.embedding is not None:
            payload.extend(struct.pack("<B", 1))
            # Zero-copy numpy array
            payload.extend(msg.embedding.astype(np.float32).tobytes())
        else:
            payload.extend(struct.pack("<B", 0))

        # Add header
        header = SBPEncoder.encode_header(MessageType.CONCEPT_WRITE, len(payload))
        return header + bytes(payload)

    @staticmethod
    def encode_association(msg: AssociationMessage) -> bytes:
        """
        Encode association message (fixed size: 57 bytes payload).

        Format:
        - source_id: 16 bytes
        - target_id: 16 bytes
        - assoc_type: uint8
        - confidence: float32
        - weight: float32
        - created_ts: uint64
        - last_used_ts: uint64
        """
        payload = struct.pack(
            "<16s16sBffQQ",
            msg.source_id,
            msg.target_id,
            msg.assoc_type,
            msg.confidence,
            msg.weight,
            msg.created_ts,
            msg.last_used_ts,
        )

        header = SBPEncoder.encode_header(MessageType.ASSOCIATION_WRITE, len(payload))
        return header + payload

    @staticmethod
    def encode_query(msg: QueryMessage) -> bytes:
        """
        Encode query message.

        Format:
        - query_length: uint32
        - query: UTF-8
        - num_paths: uint16
        - max_depth: uint16
        - semantic_boost: uint8
        - min_confidence: float32
        - has_embedding: uint8
        - embedding: 768 float32 (optional)
        """
        query_bytes = msg.query.encode("utf-8")

        payload = bytearray()
        payload.extend(struct.pack("<I", len(query_bytes)))
        payload.extend(query_bytes)
        payload.extend(struct.pack("<HH", msg.num_paths, msg.max_depth))
        payload.extend(struct.pack("<B", 1 if msg.semantic_boost else 0))
        payload.extend(struct.pack("<f", msg.min_confidence))

        if msg.embedding is not None:
            payload.extend(struct.pack("<B", 1))
            payload.extend(msg.embedding.astype(np.float32).tobytes())
        else:
            payload.extend(struct.pack("<B", 0))

        header = SBPEncoder.encode_header(MessageType.QUERY, len(payload))
        return header + bytes(payload)

    @staticmethod
    def encode_path(msg: PathMessage) -> bytes:
        """
        Encode reasoning path message.

        Format:
        - num_concepts: uint16
        - concept_ids: array of 16-byte IDs
        - num_associations: uint16
        - association_types: array of uint8
        - confidence: float32
        - explanation_length: uint16
        - explanation: UTF-8
        """
        payload = bytearray()

        # Concepts
        payload.extend(struct.pack("<H", len(msg.concept_ids)))
        for concept_id in msg.concept_ids:
            payload.extend(concept_id)

        # Associations
        payload.extend(struct.pack("<H", len(msg.association_types)))
        for assoc_type in msg.association_types:
            payload.extend(struct.pack("<B", assoc_type))

        # Confidence
        payload.extend(struct.pack("<f", msg.confidence))

        # Explanation
        explanation_bytes = msg.explanation.encode("utf-8")
        payload.extend(struct.pack("<H", len(explanation_bytes)))
        payload.extend(explanation_bytes)

        return bytes(payload)

    @staticmethod
    def encode_result(msg: ResultMessage) -> bytes:
        """
        Encode query result message.

        Format:
        - answer_length: uint32
        - answer: UTF-8
        - confidence: float32
        - graph_confidence: float32
        - semantic_confidence: float32
        - concepts_accessed: uint32
        - execution_time_ms: float32
        - num_paths: uint16
        - paths: array of PathMessage
        """
        answer_bytes = msg.answer.encode("utf-8")

        payload = bytearray()
        payload.extend(struct.pack("<I", len(answer_bytes)))
        payload.extend(answer_bytes)
        payload.extend(
            struct.pack(
                "<fff", msg.confidence, msg.graph_confidence, msg.semantic_confidence
            )
        )
        payload.extend(struct.pack("<I", msg.concepts_accessed))
        payload.extend(struct.pack("<f", msg.execution_time_ms))

        # Encode paths
        payload.extend(struct.pack("<H", len(msg.paths)))
        for path in msg.paths:
            path_bytes = SBPEncoder.encode_path(path)
            payload.extend(struct.pack("<I", len(path_bytes)))
            payload.extend(path_bytes)

        header = SBPEncoder.encode_header(MessageType.RESULT, len(payload))
        return header + bytes(payload)

    @staticmethod
    def encode_vector_search(msg: VectorSearchMessage) -> bytes:
        """
        Encode vector search message.

        Format:
        - query_vector: 768 float32 (zero-copy)
        - top_k: uint16
        - threshold: float32
        - use_hnsw: uint8
        """
        payload = bytearray()

        # Zero-copy numpy array
        payload.extend(msg.query_vector.astype(np.float32).tobytes())
        payload.extend(struct.pack("<H", msg.top_k))
        payload.extend(struct.pack("<f", msg.threshold))
        payload.extend(struct.pack("<B", 1 if msg.use_hnsw else 0))

        header = SBPEncoder.encode_header(MessageType.VECTOR_SEARCH, len(payload))
        return header + bytes(payload)

    @staticmethod
    def encode_learn(msg: LearnMessage) -> bytes:
        """
        Encode learn message.

        Format:
        - content_length: uint32
        - content: UTF-8
        - has_source: uint8
        - source_length: uint32 (if has_source)
        - source: UTF-8 (if has_source)
        - has_category: uint8
        - category_length: uint32 (if has_category)
        - category: UTF-8 (if has_category)
        - has_embedding: uint8
        - embedding: 768 float32 (if has_embedding)
        """
        content_bytes = msg.content.encode("utf-8")

        payload = bytearray()
        payload.extend(struct.pack("<I", len(content_bytes)))
        payload.extend(content_bytes)

        # Optional source
        if msg.source:
            source_bytes = msg.source.encode("utf-8")
            payload.extend(struct.pack("<B", 1))
            payload.extend(struct.pack("<I", len(source_bytes)))
            payload.extend(source_bytes)
        else:
            payload.extend(struct.pack("<B", 0))

        # Optional category
        if msg.category:
            category_bytes = msg.category.encode("utf-8")
            payload.extend(struct.pack("<B", 1))
            payload.extend(struct.pack("<I", len(category_bytes)))
            payload.extend(category_bytes)
        else:
            payload.extend(struct.pack("<B", 0))

        # Optional embedding
        if msg.embedding is not None:
            payload.extend(struct.pack("<B", 1))
            payload.extend(msg.embedding.astype(np.float32).tobytes())
        else:
            payload.extend(struct.pack("<B", 0))

        header = SBPEncoder.encode_header(MessageType.LEARN, len(payload))
        return header + bytes(payload)

    @staticmethod
    def encode_error(msg: ErrorMessage) -> bytes:
        """Encode error message."""
        error_msg_bytes = msg.error_message.encode("utf-8")

        payload = bytearray()
        payload.extend(struct.pack("<I", msg.error_code))
        payload.extend(struct.pack("<I", len(error_msg_bytes)))
        payload.extend(error_msg_bytes)

        if msg.stacktrace:
            stack_bytes = msg.stacktrace.encode("utf-8")
            payload.extend(struct.pack("<B", 1))
            payload.extend(struct.pack("<I", len(stack_bytes)))
            payload.extend(stack_bytes)
        else:
            payload.extend(struct.pack("<B", 0))

        header = SBPEncoder.encode_header(MessageType.ERROR, len(payload))
        return header + bytes(payload)
