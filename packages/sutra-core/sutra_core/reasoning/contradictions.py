"""
Contradiction Detection and Resolution System.

Handles conflicting information in the knowledge graph:
- Semantic conflict detection
- Source reliability scoring
- Temporal versioning
- Probabilistic truth maintenance
- User feedback integration
"""

import logging
import time
from dataclasses import dataclass, field
from enum import Enum
from typing import Dict, List, Optional, Set, Tuple

from ..graph.concepts import Concept

logger = logging.getLogger(__name__)


class ConflictType(Enum):
    """Types of contradictions."""
    
    DIRECT = "direct"                 # A says X, B says not-X
    SEMANTIC = "semantic"             # Semantically opposite
    TEMPORAL = "temporal"             # Time-based conflict
    QUANTITATIVE = "quantitative"     # Numerical disagreement
    LOGICAL = "logical"               # Logical inconsistency


class ResolutionStrategy(Enum):
    """Strategies for resolving contradictions."""
    
    RECENCY = "recency"               # Prefer newer information
    CONFIDENCE = "confidence"         # Prefer higher confidence
    SOURCE = "source"                 # Prefer reliable sources
    CONSENSUS = "consensus"           # Prefer majority view
    USER = "user"                     # Let user decide
    PROBABILISTIC = "probabilistic"   # Maintain multiple versions


@dataclass
class Contradiction:
    """A detected contradiction between concepts."""
    
    concept_a: str  # Concept ID
    concept_b: str  # Concept ID
    conflict_type: ConflictType
    confidence: float  # Confidence that this is a real contradiction
    detected_at: float = field(default_factory=time.time)
    resolved: bool = False
    resolution: Optional[str] = None
    
    def __repr__(self) -> str:
        status = "resolved" if self.resolved else "unresolved"
        return f"Contradiction({self.concept_a[:8]} ⚔️  {self.concept_b[:8]}, {status})"


@dataclass
class SourceReliability:
    """Reliability score for a knowledge source."""
    
    source_id: str
    reliability: float  # 0.0 - 1.0
    num_correct: int = 0
    num_incorrect: int = 0
    num_contradicted: int = 0
    last_updated: float = field(default_factory=time.time)
    
    def update_score(self):
        """Recalculate reliability based on history."""
        total = self.num_correct + self.num_incorrect + self.num_contradicted
        if total == 0:
            self.reliability = 0.5  # Neutral
        else:
            # Weighted: correct=+1, incorrect=-1, contradicted=-0.5
            score = (
                self.num_correct 
                - self.num_incorrect 
                - (self.num_contradicted * 0.5)
            )
            self.reliability = max(0.0, min(1.0, 0.5 + (score / total) * 0.5))
        
        self.last_updated = time.time()


class ContradictionResolver:
    """
    System for detecting and resolving contradictions in knowledge.
    
    Features:
    - Semantic contradiction detection
    - Multi-strategy resolution
    - Source reliability tracking
    - Temporal versioning
    - Probabilistic truth maintenance
    """
    
    def __init__(
        self,
        concepts: Dict[str, Concept],
        default_strategy: ResolutionStrategy = ResolutionStrategy.CONFIDENCE,
    ):
        """
        Initialize contradiction resolver.
        
        Args:
            concepts: Dictionary of all concepts
            default_strategy: Default resolution strategy
        """
        self.concepts = concepts
        self.default_strategy = default_strategy
        
        # Track contradictions
        self.contradictions: List[Contradiction] = []
        self.contradiction_pairs: Set[Tuple[str, str]] = set()
        
        # Source reliability tracking
        self.source_reliability: Dict[str, SourceReliability] = {}
        
        # Negation keywords for direct contradictions
        self.negation_keywords = {
            "not", "no", "never", "none", "nothing", "neither",
            "cannot", "can't", "won't", "don't", "doesn't", "didn't",
            "isn't", "aren't", "wasn't", "weren't"
        }
        
        # Opposite pairs for semantic contradictions
        self.opposite_pairs = {
            ("hot", "cold"), ("big", "small"), ("fast", "slow"),
            ("high", "low"), ("increase", "decrease"), ("grow", "shrink"),
            ("alive", "dead"), ("true", "false"), ("yes", "no"),
            ("good", "bad"), ("up", "down"), ("in", "out"),
        }
        
        # Flatten opposite pairs for quick lookup
        self.opposites = {}
        for a, b in self.opposite_pairs:
            self.opposites[a] = b
            self.opposites[b] = a
    
    def detect_contradictions(
        self,
        new_concept_id: str,
        check_all: bool = False,
    ) -> List[Contradiction]:
        """
        Detect contradictions involving a concept.
        
        Args:
            new_concept_id: Concept to check for contradictions
            check_all: If True, check against all concepts
            
        Returns:
            List of detected contradictions
        """
        if new_concept_id not in self.concepts:
            return []
        
        new_concept = self.concepts[new_concept_id]
        detected = []
        
        # Get concepts to check against
        if check_all:
            to_check = list(self.concepts.keys())
        else:
            # Check only recent concepts (last 100)
            to_check = list(self.concepts.keys())[-100:]
        
        for other_id in to_check:
            if other_id == new_concept_id:
                continue
            
            # Skip if already detected
            if self._is_known_contradiction(new_concept_id, other_id):
                continue
            
            other_concept = self.concepts[other_id]
            
            # Check for contradictions
            contradiction = self._check_contradiction(
                new_concept, new_concept_id,
                other_concept, other_id
            )
            
            if contradiction:
                detected.append(contradiction)
                self.contradictions.append(contradiction)
                self.contradiction_pairs.add((new_concept_id, other_id))
                self.contradiction_pairs.add((other_id, new_concept_id))
                
                logger.warning(f"Detected contradiction: {contradiction}")
        
        return detected
    
    def _is_known_contradiction(self, id1: str, id2: str) -> bool:
        """Check if contradiction already known."""
        return (id1, id2) in self.contradiction_pairs
    
    def _check_contradiction(
        self,
        concept1: Concept,
        id1: str,
        concept2: Concept,
        id2: str,
    ) -> Optional[Contradiction]:
        """
        Check if two concepts contradict each other.
        
        Returns:
            Contradiction if found, None otherwise
        """
        content1_lower = concept1.content.lower()
        content2_lower = concept2.content.lower()
        
        # Check for direct negation
        if self._has_direct_negation(content1_lower, content2_lower):
            return Contradiction(
                concept_a=id1,
                concept_b=id2,
                conflict_type=ConflictType.DIRECT,
                confidence=0.9,
            )
        
        # Check for semantic opposites
        if self._has_semantic_opposite(content1_lower, content2_lower):
            return Contradiction(
                concept_a=id1,
                concept_b=id2,
                conflict_type=ConflictType.SEMANTIC,
                confidence=0.7,
            )
        
        # Check for quantitative conflicts
        quantitative = self._has_quantitative_conflict(content1_lower, content2_lower)
        if quantitative:
            return Contradiction(
                concept_a=id1,
                concept_b=id2,
                conflict_type=ConflictType.QUANTITATIVE,
                confidence=0.8,
            )
        
        return None
    
    def _has_direct_negation(self, content1: str, content2: str) -> bool:
        """Check for direct negation (X vs not-X)."""
        # Check if one contains negation and texts are otherwise similar
        words1 = set(content1.split())
        words2 = set(content2.split())
        
        negations1 = words1 & self.negation_keywords
        negations2 = words2 & self.negation_keywords
        
        # One has negation, other doesn't
        if bool(negations1) != bool(negations2):
            # Remove negations and compare
            words1_clean = words1 - self.negation_keywords
            words2_clean = words2 - self.negation_keywords
            
            # Check similarity of remaining words
            overlap = len(words1_clean & words2_clean)
            total = len(words1_clean | words2_clean)
            
            if total > 0 and overlap / total > 0.6:
                return True
        
        return False
    
    def _has_semantic_opposite(self, content1: str, content2: str) -> bool:
        """Check for semantic opposites."""
        words1 = set(content1.split())
        words2 = set(content2.split())
        
        # Check if any opposite pairs present
        for word in words1:
            opposite = self.opposites.get(word)
            if opposite and opposite in words2:
                return True
        
        return False
    
    def _has_quantitative_conflict(self, content1: str, content2: str) -> bool:
        """Check for conflicting numbers/quantities."""
        import re
        
        # Extract numbers
        nums1 = re.findall(r'\d+\.?\d*', content1)
        nums2 = re.findall(r'\d+\.?\d*', content2)
        
        if not nums1 or not nums2:
            return False
        
        # Check if they're describing the same thing with different numbers
        # Remove numbers and compare text
        text1_no_nums = re.sub(r'\d+\.?\d*', 'NUM', content1)
        text2_no_nums = re.sub(r'\d+\.?\d*', 'NUM', content2)
        
        if text1_no_nums == text2_no_nums:
            # Same structure, different numbers = conflict
            return nums1 != nums2
        
        return False
    
    def resolve_contradiction(
        self,
        contradiction: Contradiction,
        strategy: Optional[ResolutionStrategy] = None,
    ) -> str:
        """
        Resolve a contradiction using specified strategy.
        
        Args:
            contradiction: The contradiction to resolve
            strategy: Resolution strategy (uses default if None)
            
        Returns:
            ID of the winning concept
        """
        if contradiction.resolved:
            return contradiction.resolution
        
        strategy = strategy or self.default_strategy
        
        concept_a = self.concepts[contradiction.concept_a]
        concept_b = self.concepts[contradiction.concept_b]
        
        if strategy == ResolutionStrategy.RECENCY:
            # Prefer newer
            winner = (
                contradiction.concept_a 
                if concept_a.created > concept_b.created
                else contradiction.concept_b
            )
        
        elif strategy == ResolutionStrategy.CONFIDENCE:
            # Prefer higher confidence
            winner = (
                contradiction.concept_a
                if concept_a.confidence > concept_b.confidence
                else contradiction.concept_b
            )
        
        elif strategy == ResolutionStrategy.SOURCE:
            # Prefer more reliable source
            reliability_a = self._get_source_reliability(concept_a.source)
            reliability_b = self._get_source_reliability(concept_b.source)
            winner = (
                contradiction.concept_a
                if reliability_a > reliability_b
                else contradiction.concept_b
            )
        
        elif strategy == ResolutionStrategy.PROBABILISTIC:
            # Don't resolve - maintain both with probabilities
            winner = "both"
        
        else:
            # Default to confidence
            winner = (
                contradiction.concept_a
                if concept_a.confidence >= concept_b.confidence
                else contradiction.concept_b
            )
        
        contradiction.resolved = True
        contradiction.resolution = winner
        
        logger.info(f"Resolved {contradiction} using {strategy.value}: winner={winner[:8]}")
        return winner
    
    def _get_source_reliability(self, source: Optional[str]) -> float:
        """Get reliability score for a source."""
        if not source:
            return 0.5  # Neutral for unknown source
        
        if source not in self.source_reliability:
            self.source_reliability[source] = SourceReliability(
                source_id=source,
                reliability=0.5
            )
        
        return self.source_reliability[source].reliability
    
    def update_source_reliability(
        self,
        source: str,
        correct: bool,
        contradicted: bool = False,
    ):
        """
        Update reliability score for a source.
        
        Args:
            source: Source identifier
            correct: Whether the information was correct
            contradicted: Whether it was contradicted
        """
        if source not in self.source_reliability:
            self.source_reliability[source] = SourceReliability(
                source_id=source,
                reliability=0.5
            )
        
        reliability = self.source_reliability[source]
        
        if contradicted:
            reliability.num_contradicted += 1
        elif correct:
            reliability.num_correct += 1
        else:
            reliability.num_incorrect += 1
        
        reliability.update_score()
        
        logger.debug(
            f"Updated {source} reliability: {reliability.reliability:.2f} "
            f"(+{reliability.num_correct}/-{reliability.num_incorrect})"
        )
    
    def get_statistics(self) -> Dict:
        """Get contradiction statistics."""
        return {
            "total_contradictions": len(self.contradictions),
            "resolved": sum(1 for c in self.contradictions if c.resolved),
            "unresolved": sum(1 for c in self.contradictions if not c.resolved),
            "by_type": {
                conflict_type.value: sum(
                    1 for c in self.contradictions 
                    if c.conflict_type == conflict_type
                )
                for conflict_type in ConflictType
            },
            "tracked_sources": len(self.source_reliability),
            "avg_source_reliability": (
                sum(s.reliability for s in self.source_reliability.values()) /
                len(self.source_reliability)
                if self.source_reliability else 0.5
            ),
        }
    
    def get_unresolved_contradictions(self) -> List[Contradiction]:
        """Get all unresolved contradictions."""
        return [c for c in self.contradictions if not c.resolved]
    
    def mark_concept_incorrect(self, concept_id: str):
        """Mark a concept as incorrect (for training source reliability)."""
        if concept_id not in self.concepts:
            return
        
        concept = self.concepts[concept_id]
        if concept.source:
            self.update_source_reliability(concept.source, correct=False)
    
    def mark_concept_correct(self, concept_id: str):
        """Mark a concept as correct (for training source reliability)."""
        if concept_id not in self.concepts:
            return
        
        concept = self.concepts[concept_id]
        if concept.source:
            self.update_source_reliability(concept.source, correct=True)
