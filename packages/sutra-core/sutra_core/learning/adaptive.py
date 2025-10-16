"""
Adaptive Focus Learning for the Sutra AI system.

This module implements:
- Adaptive reinforcement based on concept difficulty
- Loss-Driven Adaptive Token Focusing (LATF)
- Dynamic learning strategies for different concept strengths
"""

import hashlib
import logging
from typing import Dict, Optional

from ..graph.concepts import Concept
from .associations import AssociationExtractor

logger = logging.getLogger(__name__)


class AdaptiveLearner:
    """
    Implements adaptive learning based on concept difficulty.

    Based on "LLM-Oriented Token-Adaptive Knowledge Distillation" (Oct 2025):
    - Difficult concepts (strength < 4.0): Strong reinforcement (1.15×)
    - Easy concepts (strength > 7.0): Minimal reinforcement (1.01×)
    - Moderate concepts: Standard reinforcement (1.02×)
    """

    # Adaptive reinforcement thresholds
    DIFFICULT_THRESHOLD = 4.0
    EASY_THRESHOLD = 7.0

    # Reinforcement multipliers
    DIFFICULT_MULTIPLIER = 1.15
    EASY_MULTIPLIER = 1.01
    STANDARD_MULTIPLIER = 1.02

    def __init__(
        self,
        storage,
        association_extractor: AssociationExtractor,
    ):
        """
        Initialize adaptive learner.

        Args:
            storage: RustStorageAdapter (single source of truth)
            association_extractor: Association extraction handler
        """
        self.storage = storage
        self.association_extractor = association_extractor

    def learn_adaptive(
        self,
        content: str,
        source: Optional[str] = None,
        category: Optional[str] = None,
        embedding: Optional[any] = None,
    ) -> str:
        """
        Learn new knowledge with adaptive focus.

        Spends more compute time on difficult concepts:
        - Stronger reinforcement for weak concepts
        - Deeper association extraction for new/difficult concepts
        - Minimal processing for well-established concepts

        Args:
            content: Knowledge to learn
            source: Source of knowledge
            category: Category/domain
            embedding: Pre-computed embedding vector

        Returns:
            Concept ID
        """
        import numpy as np

        # Create concept ID
        concept_id = hashlib.md5(content.encode()).hexdigest()[:16]

        existing_concept = (
            self.storage.get_concept(concept_id) if self.storage else None
        )

        if existing_concept:
            # Handle existing concept with adaptive reinforcement
            old_strength = existing_concept.strength

            # Standard access strengthening
            existing_concept.access()

            # Apply adaptive reinforcement
            self._apply_adaptive_reinforcement(existing_concept)

            # Update in storage
            if embedding is not None:
                embedding_array = np.array(embedding, dtype=np.float32)
                self.storage.add_concept(existing_concept, embedding_array)

            logger.debug(
                f"Strengthened concept: {content[:30]}... "
                f"({old_strength:.2f} → {existing_concept.strength:.2f})"
            )
            concept = existing_concept
        else:
            # Create new concept
            concept = Concept(
                id=concept_id,
                content=content,
                source=source,
                category=category,
            )

            # Store in Rust storage immediately
            if embedding is not None:
                embedding_array = np.array(embedding, dtype=np.float32)
                self.storage.add_concept(concept, embedding_array)

            logger.debug(f"Created new concept: {content[:30]}...")

        # Extract associations with adaptive depth
        extraction_depth = self._get_extraction_depth(concept)

        associations_created = self.association_extractor.extract_associations_adaptive(
            content, concept_id, depth=extraction_depth
        )

        logger.debug(
            f"Extracted {associations_created} associations "
            f"(depth={extraction_depth}) for concept: {content[:30]}..."
        )

        return concept_id

    def _apply_adaptive_reinforcement(self, concept: Concept) -> None:
        """
        Apply adaptive reinforcement based on concept strength.

        Args:
            concept: Concept to reinforce
        """
        if concept.strength < self.DIFFICULT_THRESHOLD:
            # Difficult concept: strong reinforcement
            concept.strength = min(10.0, concept.strength * self.DIFFICULT_MULTIPLIER)

        elif concept.strength > self.EASY_THRESHOLD:
            # Easy concept: minimal reinforcement
            concept.strength = min(10.0, concept.strength * self.EASY_MULTIPLIER)

        # Moderate concepts use standard 1.02× from access() method

    def _get_extraction_depth(self, concept: Concept) -> int:
        """
        Determine association extraction depth based on concept difficulty.

        Args:
            concept: Concept to analyze

        Returns:
            Extraction depth (1=normal, 2=deep)
        """
        if concept.strength < self.DIFFICULT_THRESHOLD:
            # New/weak concepts get deeper extraction
            return 2
        else:
            # Established concepts get standard extraction
            return 1

    def get_learning_stats(self) -> dict:
        """
        Get adaptive learning statistics.

        Returns:
            Dictionary with learning stats
        """
        if not self.storage:
            return {
                "total_concepts": 0,
                "difficult_concepts": 0,
                "moderate_concepts": 0,
                "easy_concepts": 0,
                "average_strength": 0.0,
            }

        all_concept_ids = self.storage.get_all_concept_ids()
        if not all_concept_ids:
            return {
                "total_concepts": 0,
                "difficult_concepts": 0,
                "moderate_concepts": 0,
                "easy_concepts": 0,
                "average_strength": 0.0,
            }

        strengths = []
        for concept_id in all_concept_ids:
            concept = self.storage.get_concept(concept_id)
            if concept:
                strengths.append(concept.strength)

        if not strengths:
            return {
                "total_concepts": len(all_concept_ids),
                "difficult_concepts": 0,
                "moderate_concepts": 0,
                "easy_concepts": 0,
                "average_strength": 0.0,
            }

        difficult = sum(1 for s in strengths if s < self.DIFFICULT_THRESHOLD)
        easy = sum(1 for s in strengths if s > self.EASY_THRESHOLD)
        moderate = len(strengths) - difficult - easy

        return {
            "total_concepts": len(strengths),
            "difficult_concepts": difficult,
            "moderate_concepts": moderate,
            "easy_concepts": easy,
            "average_strength": sum(strengths) / len(strengths),
        }
