"""
Adapter for ConcurrentStorage (next-gen lock-free storage).

This is a thin compatibility wrapper that bridges the old ReasoningEngine API
to the new ConcurrentStorage primitives. ConcurrentStorage handles:
- Lock-free writes with write-ahead log
- Background reconciliation (10ms intervals)
- Memory-mapped zero-copy reads
- Single-file persistence (storage.dat)
"""

import logging
from pathlib import Path
from typing import Dict, List, Optional, Tuple

import numpy as np

try:
    from sutra_storage import ConcurrentStorage

    RUST_STORAGE_AVAILABLE = True
except ImportError:
    RUST_STORAGE_AVAILABLE = False
    ConcurrentStorage = None

from ..graph.concepts import Association, AssociationType, Concept

logger = logging.getLogger(__name__)


class RustStorageAdapter:
    """
    Compatibility adapter over ConcurrentStorage.

    Provides the old ReasoningEngine API while using the new storage backend.
    Most operations are now simplified since ConcurrentStorage handles them internally.
    """

    def __init__(
        self,
        storage_path: str,
        vector_dimension: int = 768,
        use_compression: bool = True,  # ignored, ConcurrentStorage manages this
    ):
        if not RUST_STORAGE_AVAILABLE:
            raise ImportError(
                "sutra_storage module not available. "
                "Build with: cd packages/sutra-storage && maturin develop"
            )

        self.storage_path = Path(storage_path)
        self.storage_path.mkdir(parents=True, exist_ok=True)
        self.vector_dimension = vector_dimension

        # Initialize ConcurrentStorage (new engine)
        try:
            self.store = ConcurrentStorage(
                str(self.storage_path),
                reconcile_interval_ms=10,  # Fast reconciliation
                memory_threshold=50000,  # Flush every 50K concepts
            )
            logger.info(
                f"ConcurrentStorage initialized at {storage_path} (dim={vector_dimension})"
            )
        except Exception as e:
            raise RuntimeError(f"Failed to initialize ConcurrentStorage: {e}")

    # ===== Concept Operations =====

    def has_concept(self, concept_id: str) -> bool:
        try:
            return self.store.contains(concept_id)
        except Exception:
            return False

    def add_concept(self, concept: Concept, embedding: np.ndarray) -> None:
        """
        Add concept with its embedding using ConcurrentStorage.

        Args:
            concept: Concept object with all attributes
            embedding: Vector embedding (numpy array)

        Raises:
            ValueError: If embedding dimension doesn't match or embedding is invalid
            RuntimeError: If storage operation fails
        """
        # Validate embedding
        if not isinstance(embedding, np.ndarray):
            raise ValueError(f"Embedding must be numpy array, got {type(embedding)}")
        if embedding.shape[0] != self.vector_dimension:
            raise ValueError(
                f"Embedding dimension {embedding.shape[0]} doesn't match expected {self.vector_dimension}"
            )
        # Fix numpy array boolean ambiguity by using bool() conversion
        if not bool(np.isfinite(embedding).all()):
            raise ValueError("Embedding contains NaN or Inf values")

        try:
            # ConcurrentStorage.learn_concept(id, content, embedding, strength, confidence)
            self.store.learn_concept(
                concept.id,
                concept.content,
                embedding=embedding.astype(np.float32),
                strength=float(concept.strength),
                confidence=float(concept.confidence),
            )
            logger.debug(f"Learned concept {concept.id[:8]}... via ConcurrentStorage")
        except Exception as e:
            raise RuntimeError(f"Failed to learn concept in ConcurrentStorage: {e}")

    def get_concept(self, concept_id: str) -> Optional[Concept]:
        """Retrieve concept by ID from ConcurrentStorage."""
        data = self.store.query_concept(concept_id)
        if not data:
            return None

        # Convert ConcurrentStorage dict to Concept object
        # ConcurrentStorage returns: {id, content, strength, confidence}
        return Concept(
            id=data.get("id", concept_id),
            content=data.get("content", ""),
            strength=float(data.get("strength", 1.0)),
            confidence=float(data.get("confidence", 1.0)),
        )

    def get_all_concept_ids(self) -> List[str]:
        """Get all concept IDs - requires maintaining index externally or using stats."""
        # ConcurrentStorage doesn't expose get_all_concept_ids directly
        # For now, return empty - caller must track IDs
        logger.warning(
            "get_all_concept_ids not supported by ConcurrentStorage - tracking must be external"
        )
        return []

    def delete_concept(self, concept_id: str) -> None:
        """
        Delete concept (stub - not supported by ConcurrentStorage).
        For now, we log and return to avoid breaking rollback paths.
        """
        logger.warning(
            "delete_concept not yet supported in ConcurrentStorage; stubbed no-op"
        )

    # ===== Association Operations =====

    def add_association(self, association: Association) -> None:
        """Add association using ConcurrentStorage."""
        # Map Python enum value to Rust u8
        type_map = {
            "semantic": 0,
            "causal": 1,
            "temporal": 2,
            "hierarchical": 3,
            "compositional": 4,
        }

        try:
            # ConcurrentStorage.learn_association(source_id, target_id, assoc_type, confidence)
            self.store.learn_association(
                association.source_id,
                association.target_id,
                assoc_type=type_map.get(association.assoc_type.value, 0),
                confidence=float(association.confidence),
            )
            logger.debug(
                f"Learned association {association.source_id[:8]}... → {association.target_id[:8]}..."
            )
        except Exception as e:
            logger.warning(f"Failed to learn association: {e}")

    def get_association(self, source_id: str, target_id: str) -> Optional[Association]:
        """Get association between two concepts (not directly supported by ConcurrentStorage)."""
        # ConcurrentStorage doesn't expose direct association query
        # Check if target is in source's neighbors
        try:
            neighbors = self.store.get_neighbors(source_id)
            if target_id in neighbors:
                # Return a generic association
                return Association(
                    source_id=source_id,
                    target_id=target_id,
                    assoc_type=AssociationType.SEMANTIC,
                    confidence=0.5,
                )
        except Exception:
            pass
        return None

    def get_neighbors(self, concept_id: str) -> List[str]:
        """Get neighboring concept IDs from ConcurrentStorage."""
        try:
            return self.store.get_neighbors(concept_id)
        except Exception as e:
            logger.warning(f"Failed to get neighbors: {e}")
            return []

    def get_all_associations(self) -> List[Association]:
        """Retrieve all associations (not supported by ConcurrentStorage)."""
        # ConcurrentStorage doesn't expose get_all_associations
        logger.warning("get_all_associations not supported by ConcurrentStorage")
        return []

    # ===== Atomic Learn (concept + associations) =====
    def learn_atomic(
        self, concept: Concept, embedding: np.ndarray, associations: List[Association]
    ) -> None:
        """Atomically add a concept and its associations.
        If the native store exposes learn_atomic, use it. Otherwise fallback to sequential ops.
        """
        # Native fast-path
        if hasattr(self.store, "learn_atomic"):
            try:
                logger.debug(
                    f"Using NATIVE learn_atomic for {concept.id[:8]}... with {len(associations)} associations"
                )
                # Prepare dict payloads expected by native binding
                concept_dict = concept.to_dict()
                assoc_dicts = [a.to_dict() for a in associations]
                self.store.learn_atomic(
                    concept_dict, assoc_dicts, embedding.astype(np.float32)
                )
                logger.debug(f"✓ Native learn_atomic succeeded")
                return
            except Exception as e:
                logger.warning(f"Native learn_atomic failed, falling back: {e}")

        # Fallback: sequential (still durable due to WAL+flush on save())
        logger.debug(f"Using FALLBACK sequential ops for {concept.id[:8]}...")
        self.add_concept(concept, embedding)
        for a in associations:
            try:
                self.add_association(a)
            except Exception as e:
                logger.warning(
                    f"Failed to add association during learn_atomic fallback: {e}"
                )

    # ------------------------------------------------------------------
    # Reasoning helpers (BFS over Rust graph) until native Rust pathfinding is exposed
    # ------------------------------------------------------------------

    def _extract_best_answer_from_path(
        self,
        concepts_seq: List[str],
        query: str = "",
    ) -> str:
        """
        PRODUCTION: Extract the best answer from a reasoning path.

        Intelligently selects the most relevant concept based on:
        - Query term overlap (prefer concepts containing query words)
        - Content completeness (prefer full sentences over fragments)
        - Concept confidence and strength
        - Position preference (earlier concepts often more relevant)

        Args:
            concepts_seq: Ordered list of concept IDs in reasoning path
            query: Original query (for relevance scoring)

        Returns:
            Best answer string extracted from path
        """
        if not concepts_seq:
            return ""

        # Extract query words for relevance scoring
        from ..utils.text import extract_words

        query_words = set(extract_words(query.lower())) if query else set()

        # Score each concept in the path
        scored_concepts = []
        for idx, concept_id in enumerate(concepts_seq):
            concept = self.get_concept(concept_id)
            if not concept:
                continue

            content = concept.content
            content_words = set(extract_words(content.lower()))

            # Factor 1: Query relevance (word overlap)
            if query_words:
                overlap = len(query_words & content_words)
                query_relevance = overlap / len(query_words) if query_words else 0.0
            else:
                query_relevance = 0.5  # Neutral if no query

            # Factor 2: Content completeness
            # Full sentences (>5 words) STRONGLY preferred over fragments
            word_count = len(content.split())
            if word_count <= 2:
                # Single/double words get heavy penalty
                completeness_score = 0.1
            elif word_count <= 4:
                # Short phrases get moderate penalty
                completeness_score = 0.4
            else:
                # Full sentences (5+ words) get full score
                completeness_score = min(word_count / 10.0, 1.0)

            # Factor 3: Concept quality (confidence × strength)
            quality_score = concept.confidence * min(concept.strength / 5.0, 1.0)

            # Factor 4: Position preference
            # Earlier concepts (closer to query) often more relevant
            # But not too aggressive - middle concepts can be good too
            position_score = 1.0 - (idx / max(len(concepts_seq), 1)) * 0.3

            # Combined score with weighted factors
            final_score = (
                query_relevance * 0.35  # 35% weight on query match
                + completeness_score * 0.35  # 35% weight on completeness (increased!)
                + quality_score * 0.20  # 20% weight on quality
                + position_score * 0.10  # 10% weight on position
            )

            scored_concepts.append((concept_id, content, final_score))

        if not scored_concepts:
            # Fallback: return last concept's content
            last_concept = self.get_concept(concepts_seq[-1])
            return last_concept.content if last_concept else concepts_seq[-1]

        # Return content of highest-scoring concept
        best_concept_id, best_content, best_score = max(
            scored_concepts, key=lambda x: x[2]
        )
        logger.debug(
            f"Selected best answer: '{best_content[:50]}...' "
            f"(score: {best_score:.2f}, from {len(scored_concepts)} candidates)"
        )
        return best_content

    def find_paths(
        self,
        start_ids: List[str],
        target_ids: List[str],
        max_depth: int = 5,
        num_paths: int = 3,
        query: str = "",
    ) -> List["ReasoningPath"]:
        from ..graph.concepts import ReasoningPath, ReasoningStep

        targets = set(target_ids)
        paths: List[ReasoningPath] = []

        # Try ConcurrentStorage.find_path (single path)
        try:
            logger.debug(
                f"Using ConcurrentStorage find_path: {len(start_ids)} starts → {len(target_ids)} targets (depth={max_depth})"
            )
            for s in start_ids:
                for t in target_ids:
                    # ConcurrentStorage.find_path returns a single path or None
                    path_result = self.store.find_path(s, t, max_depth=max_depth)
                    if path_result:
                        # path_result is List[str] of concept IDs
                        concepts_seq = path_result
                        logger.debug(
                            f"✓ find_path returned {len(concepts_seq)} concepts for {s[:8]}...→{t[:8]}..."
                        )

                        # Build reasoning steps
                        steps = []
                        conf = 1.0
                        for i in range(len(concepts_seq) - 1):
                            src = concepts_seq[i]
                            tgt = concepts_seq[i + 1]
                            src_c = self.get_concept(src)
                            tgt_c = self.get_concept(tgt)
                            step_conf = 0.8  # Default confidence
                            conf *= step_conf
                            steps.append(
                                ReasoningStep(
                                    source_concept=(
                                        (src_c.content[:50] + "...") if src_c else src
                                    ),
                                    relation="related",
                                    target_concept=(
                                        (tgt_c.content[:50] + "...") if tgt_c else tgt
                                    ),
                                    confidence=step_conf,
                                    step_number=i + 1,
                                    source_id=src,
                                    target_id=tgt,
                                )
                            )

                        # Extract best answer from path
                        best_answer = self._extract_best_answer_from_path(
                            concepts_seq, query
                        )
                        paths.append(
                            ReasoningPath(
                                query=query,
                                answer=best_answer,
                                steps=steps,
                                confidence=conf,
                                total_time=0.0,
                            )
                        )
                        if len(paths) >= num_paths:
                            return paths
        except Exception as e:
            logger.warning(
                f"ConcurrentStorage find_path failed, falling back to BFS: {e}"
            )

        for start in start_ids:
            # BFS queue: (node, depth, path)
            from collections import deque

            queue = deque([(start, 0, [start])])
            seen = set([start])

            while queue and len(paths) < num_paths:
                node, depth, path = queue.popleft()
                if depth >= max_depth:
                    continue

                try:
                    neighbors = self.get_neighbors(node)
                except Exception:
                    neighbors = []

                for nb in neighbors:
                    if nb in path:
                        continue
                    new_path = path + [nb]
                    if nb in targets:
                        # Build ReasoningPath with simple confidence from edges
                        steps = []
                        conf = 1.0
                        for i in range(len(new_path) - 1):
                            s = new_path[i]
                            t = new_path[i + 1]
                            assoc = self.get_association(s, t)
                            src_c = self.get_concept(s)
                            tgt_c = self.get_concept(t)
                            step_conf = assoc.confidence if assoc else 0.5
                            conf = conf * step_conf
                            steps.append(
                                ReasoningStep(
                                    source_concept=(
                                        (src_c.content[:50] + "...") if src_c else s
                                    ),
                                    relation=(
                                        assoc.assoc_type.value if assoc else "related"
                                    ),
                                    target_concept=(
                                        (tgt_c.content[:50] + "...") if tgt_c else t
                                    ),
                                    confidence=step_conf,
                                    step_number=i + 1,
                                    source_id=s,
                                    target_id=t,
                                )
                            )
                        # PRODUCTION: Extract best answer from path (not just last concept)
                        best_answer = self._extract_best_answer_from_path(
                            new_path, query
                        )
                        paths.append(
                            ReasoningPath(
                                query=query,
                                answer=best_answer,
                                steps=steps,
                                confidence=conf,
                                total_time=0.0,
                            )
                        )
                        if len(paths) >= num_paths:
                            break
                    else:
                        if nb not in seen:
                            seen.add(nb)
                            queue.append((nb, depth + 1, new_path))
        return paths

    # ===== Vector Operations =====

    # Note: Vector operations are internal to ReasoningStore. Expose only if needed.

    # ===== Search Operations =====

    def search_by_text(self, text: str) -> List[str]:
        """Search concepts by text content (not supported by ConcurrentStorage)."""
        # ConcurrentStorage doesn't expose text search
        logger.warning("search_by_text not supported by ConcurrentStorage")
        return []

    def vector_search(
        self, query_embedding: np.ndarray, k: int = 10
    ) -> List[Tuple[str, float]]:
        """
        Vector similarity search using native Rust HNSW.

        Args:
            query_embedding: Query vector (numpy array)
            k: Number of nearest neighbors to return

        Returns:
            List of (concept_id, similarity_score) tuples
        """
        try:
            # Ensure numpy array (Rust binding expects PyReadonlyArray1<f32>)
            if not isinstance(query_embedding, np.ndarray):
                query_embedding = np.array(query_embedding, dtype=np.float32)
            
            # Convert to float32 if needed
            if query_embedding.dtype != np.float32:
                query_embedding = query_embedding.astype(np.float32)

            # Call Rust HNSW search (pass numpy array directly)
            results = self.store.vector_search(query_embedding, k=k)

            logger.debug(f"Vector search returned {len(results)} results")
            return results
        except Exception as e:
            logger.error(f"Vector search failed: {e}")
            return []

    # ===== Persistence =====

    def save(self) -> None:
        """Persist all changes to disk via ConcurrentStorage.flush()."""
        try:
            self.store.flush()
            logger.debug("ConcurrentStorage flushed to disk")
        except Exception as e:
            logger.error(f"Failed to flush ConcurrentStorage: {e}")
            raise

    # ===== Statistics =====

    def stats(self) -> Dict:
        """Get storage statistics from ConcurrentStorage."""
        rust_stats = self.store.stats()
        # ConcurrentStorage returns: written, dropped, pending, reconciliations,
        # entries_processed, concepts, edges, sequence
        return {
            "total_concepts": rust_stats.get("concepts", 0),
            "total_associations": rust_stats.get("edges", 0),
            "total_edges": rust_stats.get("edges", 0),
            "written": rust_stats.get("written", 0),
            "dropped": rust_stats.get("dropped", 0),
            "pending": rust_stats.get("pending", 0),
            "reconciliations": rust_stats.get("reconciliations", 0),
            "vector_dimension": self.vector_dimension,
            "storage_path": str(self.storage_path),
        }

    # ===== Context Manager =====

    def __enter__(self):
        """Enter context manager."""
        return self

    def __exit__(self, *args):
        """Exit context manager - auto-save."""
        self.save()

    def __repr__(self) -> str:
        """String representation."""
        stats = self.stats()
        return (
            f"RustStorageAdapter("
            f"path='{self.storage_path}', "
            f"concepts={stats['total_concepts']}"
            f")"
        )
