"""
Result types for Sutra AI with full explainability and audit trails.

These are the ONLY result types users see - provides transparency.
"""

import json
from dataclasses import asdict, dataclass
from datetime import datetime
from typing import Any, Dict, List, Optional


@dataclass
class ReasoningPathDetail:
    """Details of a single reasoning path."""

    concepts: List[str]  # List of concept contents
    concept_ids: List[str]  # List of concept IDs
    association_types: List[str]  # Types of associations used
    confidence: float
    explanation: str

    def to_dict(self) -> dict:
        """Convert to dictionary."""
        return asdict(self)


@dataclass
class ConfidenceBreakdown:
    """Breakdown of how confidence was computed."""

    graph_confidence: float
    semantic_confidence: float
    path_quality: float
    consensus_strength: float
    final_confidence: float

    def to_dict(self) -> dict:
        """Convert to dictionary."""
        return asdict(self)


@dataclass
class AuditTrail:
    """Complete audit trail for regulatory compliance."""

    query_id: str
    query: str
    timestamp: str
    concepts_accessed: int
    associations_traversed: int
    execution_time_ms: float
    reasoning_method: str
    semantic_boost_used: bool
    paths_explored: int
    storage_path: str

    def to_dict(self) -> dict:
        """Convert to dictionary."""
        return asdict(self)

    def to_json(self) -> str:
        """Convert to JSON string for export."""
        return json.dumps(self.to_dict(), indent=2)


@dataclass
class ExplainableResult:
    """
    Main result type with full explainability.

    This is what users get - complete transparency into reasoning.
    Provides everything needed for:
    - Understanding the answer
    - Auditing the reasoning
    - Regulatory compliance
    - Debugging
    """

    # Core answer
    answer: str
    confidence: float

    # Explanation
    explanation: Optional[str] = None

    # Reasoning details
    reasoning_paths: Optional[List[ReasoningPathDetail]] = None
    confidence_breakdown: Optional[ConfidenceBreakdown] = None

    # Semantic support
    semantic_support: Optional[List[Dict[str, Any]]] = None

    # Audit trail
    audit_trail: Optional[AuditTrail] = None

    # Metadata
    metadata: Optional[Dict[str, Any]] = None

    def __str__(self) -> str:
        """Human-readable string representation."""
        return f"Answer: {self.answer}\nConfidence: {self.confidence:.2f}"

    def to_dict(self) -> dict:
        """Convert to dictionary."""
        result = {
            "answer": self.answer,
            "confidence": self.confidence,
        }

        if self.explanation:
            result["explanation"] = self.explanation

        if self.reasoning_paths:
            result["reasoning_paths"] = [p.to_dict() for p in self.reasoning_paths]

        if self.confidence_breakdown:
            result["confidence_breakdown"] = self.confidence_breakdown.to_dict()

        if self.semantic_support:
            result["semantic_support"] = self.semantic_support

        if self.audit_trail:
            result["audit_trail"] = self.audit_trail.to_dict()

        if self.metadata:
            result["metadata"] = self.metadata

        return result

    def to_json(self) -> str:
        """Convert to JSON string."""
        return json.dumps(self.to_dict(), indent=2)

    def show_explanation(self) -> None:
        """Print full explanation to console."""
        if self.explanation:
            print("=" * 70)
            print("EXPLANATION")
            print("=" * 70)
            print(self.explanation)
            print("=" * 70)
        else:
            print("No explanation available (explain=False)")

    def show_audit_trail(self) -> None:
        """Print complete audit trail for accountability."""
        if not self.audit_trail:
            print("No audit trail available")
            return

        print("=" * 70)
        print("AUDIT TRAIL")
        print("=" * 70)
        print(f"Query ID: {self.audit_trail.query_id}")
        print(f"Query: {self.audit_trail.query}")
        print(f"Timestamp: {self.audit_trail.timestamp}")
        print(f"Concepts accessed: {self.audit_trail.concepts_accessed}")
        print(f"Associations traversed: {self.audit_trail.associations_traversed}")
        print(f"Execution time: {self.audit_trail.execution_time_ms:.2f}ms")
        print(f"Reasoning method: {self.audit_trail.reasoning_method}")
        print(f"Semantic boost: {self.audit_trail.semantic_boost_used}")
        print(f"Paths explored: {self.audit_trail.paths_explored}")
        print("=" * 70)

    def show_confidence_breakdown(self) -> None:
        """Show how confidence was calculated."""
        if not self.confidence_breakdown:
            print("No confidence breakdown available")
            return

        print("=" * 70)
        print("CONFIDENCE BREAKDOWN")
        print("=" * 70)
        print(f"Graph reasoning: {self.confidence_breakdown.graph_confidence:.2f}")
        print(
            f"Semantic similarity: {self.confidence_breakdown.semantic_confidence:.2f}"
        )
        print(f"Path quality: {self.confidence_breakdown.path_quality:.2f}")
        print(f"Consensus strength: {self.confidence_breakdown.consensus_strength:.2f}")
        print(f"Final confidence: {self.confidence_breakdown.final_confidence:.2f}")
        print("=" * 70)

    def export_for_compliance(self) -> dict:
        """
        Export complete reasoning trace for regulatory compliance.

        Returns everything needed for audit/compliance including:
        - Full reasoning paths
        - Confidence calculations
        - Timing information
        - Data sources
        """
        return {
            "query": self.audit_trail.query if self.audit_trail else None,
            "answer": self.answer,
            "confidence": self.confidence,
            "reasoning_method": (
                self.audit_trail.reasoning_method if self.audit_trail else None
            ),
            "timestamp": self.audit_trail.timestamp if self.audit_trail else None,
            "execution_time_ms": (
                self.audit_trail.execution_time_ms if self.audit_trail else None
            ),
            "reasoning_paths": (
                [p.to_dict() for p in self.reasoning_paths]
                if self.reasoning_paths
                else []
            ),
            "confidence_breakdown": (
                self.confidence_breakdown.to_dict()
                if self.confidence_breakdown
                else None
            ),
            "audit_trail": self.audit_trail.to_dict() if self.audit_trail else None,
        }


@dataclass
class LearnResult:
    """Result from learning operation."""

    concept_id: str
    timestamp: str
    concepts_created: int
    associations_created: int
    message: str
    source: Optional[str] = None
    category: Optional[str] = None

    def to_dict(self) -> dict:
        """Convert to dictionary."""
        return asdict(self)


@dataclass
class MultiStrategyResult:
    """Result comparing multiple reasoning strategies."""

    query: str
    graph_only: ExplainableResult
    semantic_enhanced: ExplainableResult
    agreement_score: float
    recommended_strategy: str
    reasoning: str

    def to_dict(self) -> dict:
        """Convert to dictionary."""
        return {
            "query": self.query,
            "graph_only": self.graph_only.to_dict(),
            "semantic_enhanced": self.semantic_enhanced.to_dict(),
            "agreement_score": self.agreement_score,
            "recommended_strategy": self.recommended_strategy,
            "reasoning": self.reasoning,
        }
