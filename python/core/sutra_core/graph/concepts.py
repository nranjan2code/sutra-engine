"""
Core data structures for the Sutra AI system.

This module contains the fundamental building blocks:
- Concept: Represents a knowledge unit with adaptive strength
- Association: Weighted connections between concepts
- ReasoningStep: Single step in reasoning chain
- ReasoningPath: Complete reasoning from query to answer
"""

import time
from dataclasses import dataclass, field
from enum import Enum
from typing import List, Optional


class AssociationType(Enum):
    """Types of associations between concepts"""

    SEMANTIC = "semantic"  # Meaning-based connections
    CAUSAL = "causal"  # Cause and effect relationships
    TEMPORAL = "temporal"  # Time-based sequences
    HIERARCHICAL = "hierarchical"  # Parent-child relationships
    COMPOSITIONAL = "compositional"  # Part-whole relationships


@dataclass
class Concept:
    """A living knowledge concept with adaptive strength learning."""

    id: str
    content: str
    created: float = field(default_factory=time.time)

    # Learning dynamics
    access_count: int = 0
    strength: float = 1.0
    last_accessed: float = field(default_factory=time.time)

    # Metadata for explainability
    source: Optional[str] = None
    category: Optional[str] = None
    confidence: float = 1.0

    def access(self) -> None:
        """Access concept and strengthen it adaptively."""
        self.access_count += 1
        self.last_accessed = time.time()
        # Gradual strengthening: max 10.0 to prevent runaway growth
        self.strength = min(10.0, self.strength * 1.02)

    def to_dict(self) -> dict:
        """Serialize concept to dictionary."""
        return {
            "id": self.id,
            "content": self.content,
            "created": self.created,
            "access_count": self.access_count,
            "strength": self.strength,
            "last_accessed": self.last_accessed,
            "source": self.source,
            "category": self.category,
            "confidence": self.confidence,
        }

    @classmethod
    def from_dict(cls, data: dict) -> "Concept":
        """Deserialize concept from dictionary."""
        return cls(
            id=data["id"],
            content=data["content"],
            created=data.get("created", time.time()),
            access_count=data.get("access_count", 0),
            strength=data.get("strength", 1.0),
            last_accessed=data.get("last_accessed", time.time()),
            source=data.get("source"),
            category=data.get("category"),
            confidence=data.get("confidence", 1.0),
        )


@dataclass
class Association:
    """Weighted association between concepts with confidence scoring."""

    source_id: str
    target_id: str
    assoc_type: AssociationType
    weight: float = 1.0
    confidence: float = 1.0
    created: float = field(default_factory=time.time)
    last_used: float = field(default_factory=time.time)

    def strengthen(self, confidence_boost: float = 0.02) -> None:
        """Strengthen this association through use.
        Increases both weight (bounded) and confidence (bounded), updates last_used."""
        # Max weight of 5.0 to prevent dominance
        self.weight = min(5.0, self.weight * 1.1)
        # Increase confidence modestly; cap at 1.0
        self.confidence = min(1.0, self.confidence + confidence_boost)
        # Update usage timestamp
        self.last_used = time.time()

    def to_dict(self) -> dict:
        """Serialize association to dictionary."""
        return {
            "source_id": self.source_id,
            "target_id": self.target_id,
            "assoc_type": self.assoc_type.value,
            "weight": self.weight,
            "confidence": self.confidence,
            "created": self.created,
            "last_used": self.last_used,
        }

    @classmethod
    def from_dict(cls, data: dict) -> "Association":
        """Deserialize association from dictionary."""
        # Handle both integer (from Rust) and string (from JSON) enum values
        assoc_type_raw = data["assoc_type"]
        if isinstance(assoc_type_raw, int):
            # Map Rust integer to Python enum
            assoc_type_map = {
                0: AssociationType.SEMANTIC,
                1: AssociationType.CAUSAL,
                2: AssociationType.TEMPORAL,
                3: AssociationType.HIERARCHICAL,
                4: AssociationType.COMPOSITIONAL,
            }
            assoc_type = assoc_type_map.get(assoc_type_raw, AssociationType.SEMANTIC)
        else:
            # String value from JSON
            assoc_type = AssociationType(assoc_type_raw)

        return cls(
            source_id=data["source_id"],
            target_id=data["target_id"],
            assoc_type=assoc_type,
            weight=data.get("weight", 1.0),
            confidence=data.get("confidence", 1.0),
            created=data.get("created", time.time()),
            last_used=data.get("last_used", data.get("created", time.time())),
        )


@dataclass
class ReasoningStep:
    """A single step in an explainable reasoning chain."""

    source_concept: str
    relation: str
    target_concept: str
    confidence: float
    step_number: int
    # New: stable identifiers for overlap/diversity calculations
    source_id: Optional[str] = None
    target_id: Optional[str] = None

    def to_dict(self) -> dict:
        """Serialize reasoning step to dictionary."""
        return {
            "source_concept": self.source_concept,
            "relation": self.relation,
            "target_concept": self.target_concept,
            "confidence": self.confidence,
            "step_number": self.step_number,
            "source_id": self.source_id,
            "target_id": self.target_id,
        }


@dataclass
class ReasoningPath:
    """Complete explainable reasoning from query to answer."""

    query: str
    answer: str
    steps: List[ReasoningStep]
    confidence: float
    total_time: float

    def to_dict(self) -> dict:
        """Serialize reasoning path to dictionary."""
        return {
            "query": self.query,
            "answer": self.answer,
            "steps": [step.to_dict() for step in self.steps],
            "confidence": self.confidence,
            "total_time": self.total_time,
        }
