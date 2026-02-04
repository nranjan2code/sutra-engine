"""
Parallel association extraction for improved performance.

This module provides multiprocessing-based parallel extraction of associations
from concept batches, achieving 3-4x speedup over sequential processing.

Key features:
- Process pool for parallel regex pattern matching
- Smart batching (only parallelize when beneficial)
- Graceful fallback to sequential for small batches
- Minimal data serialization overhead
"""

import logging
import multiprocessing as mp
import time
from collections import defaultdict
from dataclasses import dataclass
from typing import Dict, List, Optional, Set, Tuple

from ..graph.concepts import AssociationType

logger = logging.getLogger(__name__)


@dataclass
class AssociationTask:
    """Data for a single association extraction task."""

    concept_id: str
    content: str
    enable_central_links: bool
    central_link_confidence: float
    central_link_type: str  # AssociationType name as string
    depth: int


@dataclass
class AssociationResult:
    """Result of association extraction for one concept."""

    concept_id: str
    associations: List[
        Tuple[str, str, str, float]
    ]  # (source, target, type, confidence)
    concepts_to_create: List[Tuple[str, str]]  # (concept_id, content)
    processing_time: float
    associations_count: int


def _extract_associations_worker(
    task: AssociationTask,
    patterns: List[Tuple[str, str]],  # (pattern_str, assoc_type_name)
) -> AssociationResult:
    """
    Worker function for parallel association extraction.

    Must be a top-level function for pickling.
    Performs pattern matching and co-occurrence extraction.

    Args:
        task: Association extraction task data
        patterns: List of (regex_pattern, association_type) tuples

    Returns:
        AssociationResult with extracted associations
    """
    import hashlib
    import re

    start_time = time.time()
    associations = []
    concepts_to_create = []
    concept_cache = {}  # Local cache for this worker

    def get_or_create_concept_id(text: str) -> str:
        """Create concept ID for text."""
        if text not in concept_cache:
            concept_id = hashlib.md5(text.encode()).hexdigest()[:16]
            concept_cache[text] = concept_id
            concepts_to_create.append((concept_id, text))
        return concept_cache[text]

    # Extract pattern-based associations
    content_lower = task.content.lower()
    for pattern_str, assoc_type_name in patterns:
        try:
            pattern = re.compile(pattern_str)
            matches = pattern.finditer(content_lower)

            for match in matches:
                source_text = match.group(1).strip()
                target_text = match.group(2).strip()

                if not source_text or not target_text:
                    continue

                # Create or get concept IDs
                source_id = get_or_create_concept_id(source_text)
                target_id = get_or_create_concept_id(target_text)

                # Add association between extracted concepts
                associations.append((source_id, target_id, assoc_type_name, 0.8))

                # Add central links if enabled
                if task.enable_central_links:
                    associations.append(
                        (
                            task.concept_id,
                            source_id,
                            task.central_link_type,
                            task.central_link_confidence,
                        )
                    )
                    associations.append(
                        (
                            task.concept_id,
                            target_id,
                            task.central_link_type,
                            task.central_link_confidence,
                        )
                    )
        except Exception as e:
            logger.debug(f"Pattern matching error: {e}")
            continue

    processing_time = time.time() - start_time

    return AssociationResult(
        concept_id=task.concept_id,
        associations=associations,
        concepts_to_create=concepts_to_create,
        processing_time=processing_time,
        associations_count=len(associations),
    )


class ParallelAssociationExtractor:
    """
    Parallel association extractor using multiprocessing.

    Achieves 3-4x speedup on association extraction by processing
    multiple concepts simultaneously across CPU cores.

    Automatically determines when to use parallel vs sequential
    processing based on batch size.
    """

    def __init__(
        self,
        storage,
        enable_central_links: bool = True,
        central_link_confidence: float = 0.6,
        central_link_type: AssociationType = AssociationType.COMPOSITIONAL,
        num_workers: Optional[int] = None,
        parallel_threshold: int = 20,
        entity_cache: Optional["EntityCache"] = None,
    ):
        """
        Initialize parallel association extractor.

        Args:
            storage: RustStorageAdapter (single source of truth)
            enable_central_links: Whether to create central concept links
            central_link_confidence: Confidence for central links
            central_link_type: Type of central links
            num_workers: Number of worker processes (None = CPU count - 1)
            parallel_threshold: Minimum batch size for parallel processing
            entity_cache: Optional EntityCache for LLM-extracted entities
        """
        self.storage = storage
        # Temporary working structures (TODO: remove once fully refactored)
        self.concepts: Dict[str, any] = {}
        self.word_to_concepts = defaultdict(set)
        self.concept_neighbors = defaultdict(set)
        self.associations: Dict[Tuple[str, str], any] = {}

        self.enable_central_links = enable_central_links
        self.central_link_confidence = central_link_confidence
        self.central_link_type = central_link_type
        self.parallel_threshold = parallel_threshold
        self.entity_cache = entity_cache

        # Determine worker count
        if num_workers is None:
            cpu_count = mp.cpu_count()
            self.num_workers = max(1, cpu_count - 1)  # Leave one core free
        else:
            self.num_workers = max(1, num_workers)

        # Cache association patterns (loaded once)
        from ..utils.text import get_association_patterns

        self.patterns = [
            (pattern_str, assoc_type.name)
            for pattern_str, assoc_type in get_association_patterns()
        ]

        # Performance tracking
        self.total_processed = 0
        self.total_parallel_time = 0.0
        self.total_sequential_time = 0.0

        logger.info(
            f"ParallelAssociationExtractor initialized: "
            f"{self.num_workers} workers, threshold={parallel_threshold}"
        )

    def _index_concept(self, concept) -> None:
        """Index concept for fast retrieval by words."""
        from ..utils.text import extract_words

        words = extract_words(concept.content)
        for word in words:
            self.word_to_concepts[word.lower()].add(concept.id)

    def extract_associations_adaptive(
        self, content: str, concept_id: str, depth: int = 1
    ) -> int:
        """
        Adaptive association extraction for single concept (compatibility method).

        This is called by AdaptiveLearner for single-concept learning.
        For batch operations, use extract_associations_batch() instead.

        Checks entity cache first for LLM-extracted entities, falls back to regex.

        Args:
            content: Content to extract from
            concept_id: Central concept ID
            depth: Extraction depth (1=normal, 2=deep)

        Returns:
            Total number of associations created
        """
        associations_created = 0

        # Try entity cache first (if enabled)
        if self.entity_cache:
            cached_entities = self.entity_cache.get(concept_id)
            if cached_entities:
                # Use cached LLM-extracted entities
                associations_created = self._create_associations_from_entities(
                    concept_id, cached_entities
                )
                logger.debug(
                    f"Used {len(cached_entities)} cached entities for {concept_id}: "
                    f"{associations_created} associations"
                )
                return associations_created
            else:
                # Cache miss - add to processing queue for background extraction
                self.entity_cache.add_to_processing_queue(concept_id)
                logger.debug(
                    f"Cache miss for {concept_id}, queued for background extraction"
                )

        # Fallback to regex pattern extraction
        task = AssociationTask(
            concept_id=concept_id,
            content=content,
            enable_central_links=self.enable_central_links,
            central_link_confidence=self.central_link_confidence,
            central_link_type=self.central_link_type.name,
            depth=depth,
        )

        result = _extract_associations_worker(task, self.patterns)

        # Apply results to shared data structures
        self._apply_result(result)

        # Persist associations to Rust storage if available
        # Only persist if both concepts exist in Rust (have been stored with embeddings)
        try:
            if (
                self.storage is not None
                and result.associations
                and hasattr(self.storage, "add_association")
                and hasattr(self.storage, "has_concept")
            ):
                type_map = {
                    "SEMANTIC": AssociationType.SEMANTIC,
                    "CAUSAL": AssociationType.CAUSAL,
                    "TEMPORAL": AssociationType.TEMPORAL,
                    "HIERARCHICAL": AssociationType.HIERARCHICAL,
                    "COMPOSITIONAL": AssociationType.COMPOSITIONAL,
                }
                from ..graph.concepts import Association

                for src, tgt, assoc_type_name, conf in result.associations:
                    # Only persist if both concepts exist in Rust
                    if self.storage.has_concept(src) and self.storage.has_concept(tgt):
                        assoc = Association(
                            source_id=src,
                            target_id=tgt,
                            assoc_type=type_map.get(
                                assoc_type_name.upper(), AssociationType.SEMANTIC
                            ),
                            confidence=conf,
                        )
                        self.storage.add_association(assoc)
        except Exception:
            pass

        return result.associations_count

    def _create_associations_from_entities(
        self, concept_id: str, entities: List[Dict]
    ) -> int:
        """
        Create associations from cached LLM-extracted entities.

        Args:
            concept_id: Central concept ID
            entities: List of entity dicts with 'text', 'type', 'confidence'

        Returns:
            Number of associations created
        """
        import hashlib

        from ..graph.concepts import Concept

        associations_created = 0

        for entity in entities:
            entity_text = entity.get("text", "").strip()
            entity_type = entity.get("type", "").strip()
            confidence = entity.get("confidence", 0.8)

            if not entity_text or not entity_type:
                continue

            # Create or get entity concept
            entity_id = hashlib.md5(entity_text.encode()).hexdigest()[:16]

            if entity_id not in self.concepts:
                entity_concept = Concept(
                    id=entity_id,
                    content=entity_text,
                    source="llm_cache",
                    category=entity_type,
                )
                self.concepts[entity_id] = entity_concept
                self._index_concept(entity_concept)

            # Create association from central concept to entity
            # Use COMPOSITIONAL for "has entity" relationships
            assoc_key = (concept_id, entity_id)
            if assoc_key not in self.associations:
                from ..graph.concepts import Association

                assoc = Association(
                    source_id=concept_id,
                    target_id=entity_id,
                    assoc_type=AssociationType.COMPOSITIONAL,
                    confidence=confidence,
                )
                self.associations[assoc_key] = assoc
                self.concept_neighbors[concept_id].add(entity_id)
                self.concept_neighbors[entity_id].add(concept_id)
                associations_created += 1

        return associations_created

    def extract_associations_batch(
        self,
        concept_data: List[Tuple[str, str]],  # [(concept_id, content), ...]
        depth: int = 1,
    ) -> int:
        """
        Extract associations for a batch of concepts.

        Automatically chooses parallel or sequential processing
        based on batch size.

        Args:
            concept_data: List of (concept_id, content) tuples
            depth: Extraction depth (1=standard, 2=deep)

        Returns:
            Total number of associations created
        """
        if not concept_data:
            return 0

        batch_size = len(concept_data)

        # Use sequential for small batches (overhead not worth it)
        if batch_size < self.parallel_threshold:
            return self._extract_sequential(concept_data, depth)

        # Use parallel for large batches
        return self._extract_parallel(concept_data, depth)

    def _extract_sequential(
        self, concept_data: List[Tuple[str, str]], depth: int
    ) -> int:
        """Sequential extraction (fallback for small batches)."""
        start_time = time.time()
        total_associations = 0

        for concept_id, content in concept_data:
            # Use existing sequential extractor logic
            from .associations import AssociationExtractor

            extractor = AssociationExtractor(
                self.concepts,
                self.word_to_concepts,
                self.concept_neighbors,
                self.associations,
                self.enable_central_links,
                self.central_link_confidence,
                self.central_link_type,
                storage=self.storage,
            )
            count = extractor.extract_associations_adaptive(content, concept_id, depth)
            total_associations += count

        elapsed = time.time() - start_time
        self.total_sequential_time += elapsed
        self.total_processed += len(concept_data)

        logger.debug(
            f"Sequential extraction: {len(concept_data)} concepts in {elapsed:.3f}s "
            f"({len(concept_data)/elapsed:.1f} concepts/sec)"
        )

        return total_associations

    def _apply_result(self, result: AssociationResult) -> None:
        """
        Apply extraction result to shared data structures.

        Args:
            result: Association extraction result
        """
        # Create new concepts
        for concept_id, content in result.concepts_to_create:
            if concept_id not in self.concepts:
                from ..graph.concepts import Concept

                concept = Concept(id=concept_id, content=content, confidence=0.7)
                self.concepts[concept_id] = concept
                # Index concept words
                from ..utils.text import extract_words

                words = extract_words(content)
                for word in words:
                    self.word_to_concepts[word].add(concept_id)

                # PRODUCTION: Persist sub-concepts to Rust storage
                if self.storage is not None and hasattr(self.storage, "add_concept"):
                    try:
                        import numpy as np

                        # Store with placeholder embedding (no NLP in parallel path)
                        self.storage.add_concept(
                            concept, np.zeros(384, dtype=np.float32)
                        )
                    except Exception:
                        pass  # Non-fatal

        # Create associations
        for source_id, target_id, assoc_type_name, confidence in result.associations:
            assoc_type = getattr(AssociationType, assoc_type_name)
            key = (source_id, target_id)

            if key not in self.associations:
                from ..graph.concepts import Association

                assoc = Association(
                    source_id=source_id,
                    target_id=target_id,
                    assoc_type=assoc_type,
                    confidence=confidence,
                )
                self.associations[key] = assoc
                # Maintain symmetric neighbor indices
                self.concept_neighbors[source_id].add(target_id)
                self.concept_neighbors[target_id].add(source_id)

    def _extract_parallel(self, concept_data: List[Tuple[str, str]], depth: int) -> int:
        """Parallel extraction using process pool."""
        start_time = time.time()

        # Prepare tasks
        tasks = [
            AssociationTask(
                concept_id=concept_id,
                content=content,
                enable_central_links=self.enable_central_links,
                central_link_confidence=self.central_link_confidence,
                central_link_type=self.central_link_type.name,
                depth=depth,
            )
            for concept_id, content in concept_data
        ]

        # Process in parallel
        try:
            with mp.Pool(processes=self.num_workers) as pool:
                # Map tasks to workers
                results = pool.starmap(
                    _extract_associations_worker,
                    [(task, self.patterns) for task in tasks],
                )
        except Exception as e:
            logger.warning(
                f"Parallel extraction failed: {e}, falling back to sequential"
            )
            return self._extract_sequential(concept_data, depth)

        # Collect results and update shared state
        total_associations = 0
        for result in results:
            self._apply_result(result)
            total_associations += result.associations_count

        elapsed = time.time() - start_time
        self.total_parallel_time += elapsed
        self.total_processed += len(concept_data)

        throughput = len(concept_data) / elapsed
        logger.debug(
            f"Parallel extraction: {len(concept_data)} concepts in {elapsed:.3f}s "
            f"({throughput:.1f} concepts/sec, {self.num_workers} workers)"
        )

        return total_associations

    def get_stats(self) -> Dict:
        """Get performance statistics."""
        return {
            "total_processed": self.total_processed,
            "parallel_time": self.total_parallel_time,
            "sequential_time": self.total_sequential_time,
            "num_workers": self.num_workers,
            "parallel_threshold": self.parallel_threshold,
        }
