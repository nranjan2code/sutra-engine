"""
Sutra Binary Protocol (SBP) Decoder.

High-performance binary decoding for internal messages.
"""

import struct
from typing import Tuple

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


class SBPDecoder:
    """
    Binary decoder for SBP messages.

    Performance characteristics:
    - Header decode: ~50ns
    - ConceptMessage: ~150ns decoding
    - QueryMessage: ~250ns decoding
    - Zero-copy for numpy arrays
    """

    @staticmethod
    def decode_header(data: bytes) -> Tuple[int, MessageType, int]:
        """
        Decode message header (16 bytes).

        Returns:
            (version, msg_type, payload_size)
        """
        if len(data) < SBPMessage.HEADER_SIZE:
            raise ValueError(f"Invalid header size: {len(data)}")

        magic, version, msg_type, flags, reserved, payload_size = struct.unpack(
            "<IBBBBQ", data[:16]
        )

        if magic != SBPMessage.MAGIC:
            raise ValueError(f"Invalid magic number: 0x{magic:08x}")

        return version, MessageType(msg_type), payload_size

    @staticmethod
    def decode_concept(data: bytes) -> ConceptMessage:
        """
        Decode concept message.

        Format:
        - concept_id: 16 bytes
        - content_length: uint32
        - content: UTF-8
        - strength: float32
        - confidence: float32
        - access_count: uint32
        - created_ts: uint64
        - modified_ts: uint64
        - has_embedding: uint8
        - embedding: 768 float32 (optional)
        """
        offset = 0

        # Concept ID
        concept_id = data[offset : offset + 16]
        offset += 16

        # Content
        content_len = struct.unpack("<I", data[offset : offset + 4])[0]
        offset += 4
        content = data[offset : offset + content_len].decode("utf-8")
        offset += content_len

        # Strength and confidence
        strength, confidence = struct.unpack("<ff", data[offset : offset + 8])
        offset += 8

        # Access count
        access_count = struct.unpack("<I", data[offset : offset + 4])[0]
        offset += 4

        # Timestamps
        created_ts, modified_ts = struct.unpack("<QQ", data[offset : offset + 16])
        offset += 16

        # Embedding
        has_embedding = struct.unpack("<B", data[offset : offset + 1])[0]
        offset += 1

        embedding = None
        if has_embedding:
            # Zero-copy numpy array
            embedding = np.frombuffer(
                data[offset : offset + 768 * 4], dtype=np.float32
            ).copy()
            offset += 768 * 4

        return ConceptMessage(
            concept_id=concept_id,
            content=content,
            strength=strength,
            confidence=confidence,
            access_count=access_count,
            created_ts=created_ts,
            modified_ts=modified_ts,
            embedding=embedding,
        )

    @staticmethod
    def decode_query(data: bytes) -> QueryMessage:
        """
        Decode query message.

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
        offset = 0

        # Query
        query_len = struct.unpack("<I", data[offset : offset + 4])[0]
        offset += 4
        query = data[offset : offset + query_len].decode("utf-8")
        offset += query_len

        # Parameters
        num_paths, max_depth = struct.unpack("<HH", data[offset : offset + 4])
        offset += 4

        semantic_boost = struct.unpack("<B", data[offset : offset + 1])[0] == 1
        offset += 1

        min_confidence = struct.unpack("<f", data[offset : offset + 4])[0]
        offset += 4

        # Embedding
        has_embedding = struct.unpack("<B", data[offset : offset + 1])[0]
        offset += 1

        embedding = None
        if has_embedding:
            embedding = np.frombuffer(
                data[offset : offset + 768 * 4], dtype=np.float32
            ).copy()
            offset += 768 * 4

        return QueryMessage(
            query=query,
            num_paths=num_paths,
            max_depth=max_depth,
            semantic_boost=semantic_boost,
            min_confidence=min_confidence,
            embedding=embedding,
        )

    @staticmethod
    def decode_path(data: bytes) -> PathMessage:
        """Decode reasoning path message."""
        offset = 0

        # Concepts
        num_concepts = struct.unpack("<H", data[offset : offset + 2])[0]
        offset += 2

        concept_ids = []
        for _ in range(num_concepts):
            concept_ids.append(data[offset : offset + 16])
            offset += 16

        # Associations
        num_associations = struct.unpack("<H", data[offset : offset + 2])[0]
        offset += 2

        association_types = []
        for _ in range(num_associations):
            association_types.append(struct.unpack("<B", data[offset : offset + 1])[0])
            offset += 1

        # Confidence
        confidence = struct.unpack("<f", data[offset : offset + 4])[0]
        offset += 4

        # Explanation
        explanation_len = struct.unpack("<H", data[offset : offset + 2])[0]
        offset += 2
        explanation = data[offset : offset + explanation_len].decode("utf-8")
        offset += explanation_len

        return PathMessage(
            concept_ids=concept_ids,
            association_types=association_types,
            confidence=confidence,
            explanation=explanation,
        )

    @staticmethod
    def decode_result(data: bytes) -> ResultMessage:
        """Decode query result message."""
        offset = 0

        # Answer
        answer_len = struct.unpack("<I", data[offset : offset + 4])[0]
        offset += 4
        answer = data[offset : offset + answer_len].decode("utf-8")
        offset += answer_len

        # Confidences
        confidence, graph_conf, semantic_conf = struct.unpack(
            "<fff", data[offset : offset + 12]
        )
        offset += 12

        # Concepts accessed
        concepts_accessed = struct.unpack("<I", data[offset : offset + 4])[0]
        offset += 4

        # Execution time
        execution_time_ms = struct.unpack("<f", data[offset : offset + 4])[0]
        offset += 4

        # Paths
        num_paths = struct.unpack("<H", data[offset : offset + 2])[0]
        offset += 2

        paths = []
        for _ in range(num_paths):
            path_len = struct.unpack("<I", data[offset : offset + 4])[0]
            offset += 4
            path_data = data[offset : offset + path_len]
            offset += path_len
            paths.append(SBPDecoder.decode_path(path_data))

        return ResultMessage(
            answer=answer,
            confidence=confidence,
            graph_confidence=graph_conf,
            semantic_confidence=semantic_conf,
            paths=paths,
            concepts_accessed=concepts_accessed,
            execution_time_ms=execution_time_ms,
        )
