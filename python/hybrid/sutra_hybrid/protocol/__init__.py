"""
Sutra Binary Protocol (SBP) - High-performance internal communication.

This protocol is ONLY for internal communication between layers:
- Hybrid ↔ Core
- Core ↔ Storage

External API uses standard JSON (OpenAI-compatible).

Protocol Features:
- Binary encoding for zero-copy operations
- Versioned messages (backward compatibility)
- Type-safe operations
- Optimized for graph operations (concepts, associations, paths)
- 10-100x faster than JSON for internal ops

DO NOT expose this to external users.
"""

from .decoder import SBPDecoder
from .encoder import SBPEncoder
from .messages import (
    AssociationMessage,
    ConceptMessage,
    LearnMessage,
    MessageType,
    PathMessage,
    QueryMessage,
    ResultMessage,
    SBPMessage,
)

__all__ = [
    "MessageType",
    "SBPMessage",
    "LearnMessage",
    "QueryMessage",
    "ResultMessage",
    "ConceptMessage",
    "AssociationMessage",
    "PathMessage",
    "SBPEncoder",
    "SBPDecoder",
]

# Protocol version
PROTOCOL_VERSION = 1
