"""
Association extraction and management for the Sutra AI system.

This module handles:
- Pattern-based association extraction from text
- Adaptive depth extraction for difficult concepts
- Co-occurrence based semantic associations
"""

import hashlib
import re
from collections import defaultdict

# Import Optional for type hints
from typing import TYPE_CHECKING, Dict, Optional, Tuple

from ..graph.concepts import Association, AssociationType, Concept
from ..utils.text import extract_words, get_association_patterns

if TYPE_CHECKING:
    from ..utils.nlp import TextProcessor
    from .entity_cache import EntityCache


class AssociationExtractor:
    """Extracts and manages associations between concepts (Rust-backed)."""

    def __init__(
        self,
        storage,
        enable_central_links: bool = True,
        central_link_confidence: float = 0.6,
        central_link_type: AssociationType = AssociationType.COMPOSITIONAL,
        max_cooccurrence_links: int = 200,
        nlp_processor: Optional["TextProcessor"] = None,
        entity_cache: Optional["EntityCache"] = None,
    ):
        """
        Initialize association extractor.

        Args:
            storage: RustStorageAdapter (single source of truth)
            enable_central_links: Create links from central concept to extracted phrases
            central_link_confidence: Confidence for central links (0.0 - 1.0)
            central_link_type: Association type for central links
            nlp_processor: Shared TextProcessor instance (avoids re-loading models)
            entity_cache: Optional EntityCache for LLM-extracted entities
        """
        self.storage = storage

        self.enable_central_links = enable_central_links
        self.central_link_confidence = central_link_confidence
        self.central_link_type = central_link_type
        self.max_cooccurrence_links = max_cooccurrence_links
        self.nlp_processor = nlp_processor
        self.entity_cache = entity_cache

    def extract_associations(self, content: str, concept_id: str) -> int:
        """
        Extract relationships from content using pattern matching.

        Args:
            content: Text content to analyze
            concept_id: Central concept ID

        Returns:
            Number of associations created
        """
        associations_created = 0
        patterns = get_association_patterns()

        for pattern, assoc_type in patterns:
            matches = re.finditer(pattern, content.lower())
            for match in matches:
                source_text = match.group(1).strip()
                target_text = match.group(2).strip()

                # Create or find concepts for source and target
                source_id = self._find_or_create_concept(source_text)
                target_id = self._find_or_create_concept(target_text)

                # Create association with high confidence between extracted phrases
                if self._create_association(source_id, target_id, assoc_type, 0.8):
                    associations_created += 1

                # Link the central learned concept to extracted phrases (configurable)
                # Ties the full content concept to its components for better traversal
                if self.enable_central_links:
                    self._create_association(
                        concept_id,
                        source_id,
                        self.central_link_type,
                        self.central_link_confidence,
                    )
                    self._create_association(
                        concept_id,
                        target_id,
                        self.central_link_type,
                        self.central_link_confidence,
                    )

        return associations_created

    def extract_associations_adaptive(
        self, content: str, concept_id: str, depth: int = 1
    ) -> int:
        """
        Adaptive association extraction based on concept difficulty.

        Implements Loss-Driven Adaptive Token Focusing (LATF):
        - Difficult concepts (depth=2): Deeper analysis
        - Easy concepts (depth=1): Standard extraction

        Args:
            content: Content to extract from
            concept_id: Central concept ID
            depth: Extraction depth (1=normal, 2=deep)

        Returns:
            Total number of associations created
        """
        # Always do standard extraction
        associations_created = self.extract_associations(content, concept_id)

        # For difficult concepts, do additional deep extraction
        if depth > 1:
            associations_created += self._extract_cooccurrence_associations(
                content, concept_id
            )

        return associations_created

    def _extract_cooccurrence_associations(self, content: str, concept_id: str) -> int:
        """
        Extract co-occurrence based semantic associations using noun chunks.

        OPTIONAL OPTIMIZATION: Uses spaCy noun chunks (if available) instead of
        sliding window to reduce associations from ~900 to <50 per document.
        Falls back to sliding window if spaCy unavailable.

        Args:
            content: Text content
            concept_id: Central concept ID

        Returns:
            Number of co-occurrence associations created
        """
        associations_created = 0

        # Use shared nlp_processor if available (OPTIONAL - graceful fallback)
        if self.nlp_processor:
            processor = self.nlp_processor
        else:
            # Fallback: create new processor (slower, for backward compatibility)
            # NOTE: This will fail if spaCy not installed - that's OK, we use fallback
            try:
                from ..utils.nlp import TextProcessor

                processor = TextProcessor()
            except Exception:
                # spaCy unavailable - use simple sliding window fallback (still works!)
                return self._extract_cooccurrence_fallback(content, concept_id)

        try:
            # Extract meaningful noun chunks (lemmatized)
            tokens = processor.extract_meaningful_tokens(content)

            # Use noun chunks for semantic associations
            doc = processor.nlp(content)
            chunks = [chunk.root.lemma_.lower() for chunk in doc.noun_chunks]

            # Limit chunks to most relevant
            chunks = list(set(chunks))[:10]  # Top 10 unique chunks

            # Create associations between chunks
            for i, chunk1 in enumerate(chunks):
                for chunk2 in chunks[i + 1 :]:
                    concepts1 = self.word_to_concepts.get(chunk1, set())
                    concepts2 = self.word_to_concepts.get(chunk2, set())

                    # Create associations between related concepts
                    for c1 in list(concepts1)[:2]:  # Max 2 concepts per chunk
                        for c2 in list(concepts2)[:2]:
                            if c1 != c2:
                                if self._create_association(
                                    c1, c2, AssociationType.SEMANTIC, 0.5
                                ):
                                    associations_created += 1
                                    if associations_created >= 50:  # Hard limit
                                        return associations_created

            return associations_created
        except Exception as e:
            # Fallback if spaCy processing fails (system still works without it!)
            logger.debug(f"spaCy NLP processing failed, using fallback: {e}")
            return self._extract_cooccurrence_fallback(content, concept_id)

    def _extract_cooccurrence_fallback(self, content: str, concept_id: str) -> int:
        """
        Simple sliding window fallback for co-occurrence extraction.

        This method works WITHOUT spaCy - uses simple word extraction and sliding window.
        Performance: Creates more associations (~200) but still bounded by hard limits.

        Args:
            content: Text content
            concept_id: Central concept ID

        Returns:
            Number of associations created
        """
        associations_created = 0
        words = extract_words(content)

        # Sliding window of 3 words to find co-occurrences
        for i, word1 in enumerate(words):
            for word2 in words[i + 1 : i + 4]:  # Window of 3 words
                # Find concepts containing these words
                # Use Rust inverted index to find concepts containing words
                concepts1 = (
                    set(self.storage.search_by_text(word1)) if self.storage else set()
                )
                concepts2 = (
                    set(self.storage.search_by_text(word2)) if self.storage else set()
                )

                # Strict limit to avoid exponential explosion
                for c1 in list(concepts1)[:2]:  # Max 2 concepts per word
                    for c2 in list(concepts2)[:2]:
                        if c1 != c2:
                            # Weaker confidence for co-occurrence
                            if self._create_association(
                                c1, c2, AssociationType.SEMANTIC, 0.5
                            ):
                                associations_created += 1
                                if associations_created >= 50:  # Hard limit
                                    return associations_created

        return associations_created

    def _find_or_create_concept(self, text: str) -> str:
        """
        Find existing concept or create new one.

        Args:
            text: Concept text content

        Returns:
            Concept ID
        """
        concept_id = hashlib.md5(text.encode()).hexdigest()[:16]

        # If concept exists in storage, return id
        try:
            if self.storage and self.storage.has_concept(concept_id):
                return concept_id
        except Exception:
            pass

        # Create new concept directly in storage
        concept = Concept(id=concept_id, content=text, confidence=0.7)
        try:
            import numpy as np

            embedding = None
            if self.nlp_processor:
                try:
                    emb = self.nlp_processor.get_embedding(text)
                    if emb is not None:
                        embedding = np.array(emb, dtype=np.float32)
                except Exception:
                    embedding = None
            if embedding is None:
                dim = getattr(self.storage, "vector_dimension", 384)
                embedding = np.zeros(dim, dtype=np.float32)
            self.storage.add_concept(concept, embedding)
        except Exception:
            # Best-effort: still return id even if embedding/storage fails
            pass
        return concept_id

    def _index_concept(self, concept: Concept) -> None:
        """No-op: Rust indexes text content on put_concept."""
        return

    def _create_association(
        self,
        source_id: str,
        target_id: str,
        assoc_type: AssociationType,
        confidence: float,
    ) -> bool:
        """
        Create or strengthen association between concepts.

        Args:
            source_id: Source concept ID
            target_id: Target concept ID
            assoc_type: Type of association
            confidence: Association confidence

        Returns:
            True if new association created, False if strengthened
        """
        key = (source_id, target_id)

        # Always persist association directly to Rust (single source of truth)
        association = Association(
            source_id=source_id,
            target_id=target_id,
            assoc_type=assoc_type,
            confidence=confidence,
        )
        try:
            if (
                self.storage
                and self.storage.has_concept(source_id)
                and self.storage.has_concept(target_id)
            ):
                self.storage.add_association(association)
                return True
        except Exception:
            pass
        return False
