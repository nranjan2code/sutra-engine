"""
Direct adapter for Rust storage engine (ReasoningStore only).

This adapter is a thin, production-grade bridge to the Rust ReasoningStore.
Rust is the single source of truth for concepts, associations, embeddings, and
indexes. No JSON sidecars, no dual formats.
"""

import logging
from pathlib import Path
from typing import Dict, List, Optional, Tuple

import numpy as np

try:
    import sutra_storage

    RUST_STORAGE_AVAILABLE = True
except ImportError:
    RUST_STORAGE_AVAILABLE = False
    sutra_storage = None

from ..graph.concepts import Association, AssociationType, Concept

logger = logging.getLogger(__name__)


class RustStorageAdapter:
    """
    Production adapter over sutra_storage.ReasoningStore.

    Public surface (used by ReasoningEngine):
    - add_concept(concept, embedding)
    - has_concept(concept_id) -> bool
    - get_concept(concept_id) -> Concept | None
    - get_all_concept_ids() -> List[str]
    - add_association(association)
    - get_neighbors(concept_id) -> List[str]
    - search_by_text(text) -> List[str]
    - find_paths(start_ids, target_ids, max_depth, num_paths) -> List[ReasoningPath]
    - delete_concept(concept_id) [stub until Rust deletion is exposed]
    - save()/stats()
    """

    def __init__(
        self,
        storage_path: str,
        vector_dimension: int = 768,
        use_compression: bool = True,  # kept for API compatibility; ReasoningStore manages compression
    ):
        if not RUST_STORAGE_AVAILABLE:
            raise ImportError(
                "sutra_storage module not available. "
                "Build with: cd packages/sutra-storage && maturin develop"
            )

        self.storage_path = Path(storage_path)
        self.storage_path.mkdir(parents=True, exist_ok=True)
        self.vector_dimension = vector_dimension

        # Initialize ReasoningStore (authoritative backend)
        try:
            self.store = sutra_storage.ReasoningStore(
                str(self.storage_path),
                vector_dimension=vector_dimension,
            )
            logger.info(
                f"ReasoningStore initialized at {storage_path} (dim={vector_dimension})"
            )
        except Exception as e:
            raise RuntimeError(f"Failed to initialize ReasoningStore: {e}")

    # ===== Concept Operations =====

    def has_concept(self, concept_id: str) -> bool:
        try:
            return self.store.get_concept(concept_id) is not None
        except Exception:
            return False

    def add_concept(self, concept: Concept, embedding: np.ndarray) -> None:
        """
        Add concept with its embedding into ReasoningStore.

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
        if not np.isfinite(embedding).all():
            raise ValueError("Embedding contains NaN or Inf values")

        # Build ReasoningStore concept dict
        concept_dict = {
            "id": concept.id,
            "content": concept.content,
            "created": int(concept.created),
            "last_accessed": int(concept.last_accessed),
            "access_count": int(concept.access_count),
            "strength": float(concept.strength),
            "confidence": float(concept.confidence),
            "source": concept.source,
            "category": concept.category,
        }

        try:
            self.store.put_concept(concept_dict, embedding.astype(np.float32))
        except Exception as e:
            raise RuntimeError(f"Failed to put concept in ReasoningStore: {e}")

        logger.debug(f"Added concept {concept.id[:8]}...")

    def get_concept(self, concept_id: str) -> Optional[Concept]:
        """Retrieve concept by ID from ReasoningStore."""
        data = self.store.get_concept(concept_id)
        if not data:
            return None
        return Concept.from_dict(data)

    def get_all_concept_ids(self) -> List[str]:
        """Get all concept IDs in storage."""
        return list(self.store.get_all_concept_ids())

    def delete_concept(self, concept_id: str) -> None:
        """
        Delete concept (stub until Rust exposes deletion API).
        For now, we log and return to avoid breaking rollback paths.
        """
        logger.warning(
            "delete_concept not yet supported in ReasoningStore; stubbed no-op"
        )

    # ===== Association Operations =====

    def add_association(self, association: Association) -> None:
        """Add association to ReasoningStore."""
        # Map Python enum value to Rust u8
        type_map = {
            "semantic": 0,
            "causal": 1,
            "temporal": 2,
            "hierarchical": 3,
            "compositional": 4,
        }
        assoc_dict = {
            "source_id": association.source_id,
            "target_id": association.target_id,
            "assoc_type": type_map.get(association.assoc_type.value, 0),
            "weight": float(association.weight),
            "confidence": float(association.confidence),
            "created": int(association.created),
            "last_used": int(association.last_used),
        }
        self.store.put_association(assoc_dict)
        logger.debug(
            f"Added association {association.source_id[:8]}... → {association.target_id[:8]}..."
        )

    def get_association(self, source_id: str, target_id: str) -> Optional[Association]:
        """Get association between two concepts (from ReasoningStore)."""
        # Fallback via listing from source (since direct API not exposed in Python binding)
        try:
            assocs = self.store.get_associations_from(source_id)
        except Exception:
            return None
        for a in assocs:
            if a.get("target_id") == target_id:
                return Association.from_dict(a)
        return None

    def get_neighbors(self, concept_id: str) -> List[str]:
        """Get neighboring concept IDs (undirected) from ReasoningStore.

        Combines outgoing neighbors from the store with incoming neighbors
        discovered via association scan to support undirected reasoning.
        """
        try:
            out = set(self.store.get_neighbors(concept_id))
        except Exception:
            out = set()

        # Fallback: include incoming edges by scanning associations
        try:
            items = self.store.get_all_associations()
            for a in items:
                src = a.get("source_id")
                tgt = a.get("target_id")
                if tgt == concept_id and src:
                    out.add(src)
                if src == concept_id and tgt:
                    out.add(tgt)
        except Exception:
            pass

        return list(out)

    def get_all_associations(self) -> List[Association]:
        """Retrieve all associations from ReasoningStore."""
        out: List[Association] = []
        try:
            items = self.store.get_all_associations()
            type_map = {
                0: AssociationType.SEMANTIC,
                1: AssociationType.CAUSAL,
                2: AssociationType.TEMPORAL,
                3: AssociationType.HIERARCHICAL,
                4: AssociationType.COMPOSITIONAL,
            }
            for a in items:
                assoc = Association(
                    source_id=a.get("source_id"),
                    target_id=a.get("target_id"),
                    assoc_type=type_map.get(
                        int(a.get("assoc_type", 0)), AssociationType.SEMANTIC
                    ),
                    confidence=float(a.get("confidence", 0.5)),
                )
                # Optionally set weight/created/last_used if present
                if hasattr(assoc, "weight") and "weight" in a:
                    assoc.weight = float(a.get("weight", 1.0))
                if hasattr(assoc, "created") and "created" in a:
                    try:
                        assoc.created = float(a.get("created", 0))
                    except Exception:
                        pass
                if hasattr(assoc, "last_used") and "last_used" in a:
                    try:
                        assoc.last_used = float(a.get("last_used", 0))
                    except Exception:
                        pass
                out.append(assoc)
        except Exception as e:
            logger.warning(f"Failed to get all associations: {e}")
        return out

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

        # Try native pathfinding first
        if hasattr(self.store, "find_paths"):
            try:
                logger.debug(
                    f"Using NATIVE find_paths: {len(start_ids)} starts → {len(target_ids)} targets (depth={max_depth})"
                )
                for s in start_ids:
                    for t in target_ids:
                        native_paths = self.store.find_paths(s, t, max_depth, num_paths)
                        logger.debug(
                            f"✓ Native find_paths returned {len(native_paths)} paths for {s[:8]}...→{t[:8]}..."
                        )
                        for p in native_paths:
                            # Convert native dict into ReasoningPath
                            steps = []
                            conf = float(p.get("confidence", 0.0))
                            concepts_seq = p.get("concepts", [])
                            for i in range(len(concepts_seq) - 1):
                                src = concepts_seq[i]
                                tgt = concepts_seq[i + 1]
                                assoc = self.get_association(src, tgt)
                                src_c = self.get_concept(src)
                                tgt_c = self.get_concept(tgt)
                                step_conf = assoc.confidence if assoc else 0.5
                                steps.append(
                                    ReasoningStep(
                                        source_concept=(
                                            (src_c.content[:50] + "...")
                                            if src_c
                                            else src
                                        ),
                                        relation=(
                                            assoc.assoc_type.value
                                            if assoc
                                            else "related"
                                        ),
                                        target_concept=(
                                            (tgt_c.content[:50] + "...")
                                            if tgt_c
                                            else tgt
                                        ),
                                        confidence=step_conf,
                                        step_number=i + 1,
                                        source_id=src,
                                        target_id=tgt,
                                    )
                                )
                            # PRODUCTION: Extract best answer from path (not just last concept)
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
                # If none found natively, fall back to BFS
            except Exception as e:
                logger.warning(f"Native find_paths failed, falling back: {e}")

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
        """Search concepts by text content via ReasoningStore."""
        if not text:
            return []
        return list(self.store.search_by_text(text.lower()))

    # ===== Batch Operations =====

    # Quantizer/vector training is managed inside ReasoningStore; not exposed here.

    # ===== Persistence =====

    def save(self) -> None:
        """Persist all changes to disk via ReasoningStore.flush()."""
        try:
            self.store.flush()
        except Exception as e:
            logger.error(f"Failed to flush ReasoningStore: {e}")
            raise

    # Metadata JSON removed — ReasoningStore is the sole source of truth.

    def _save_metadata(self) -> None:
        pass

    # ===== Statistics =====

    def stats(self) -> Dict:
        """Get storage statistics from ReasoningStore."""
        rust_stats = self.store.stats()
        return {
            "total_concepts": rust_stats.get("total_concepts", 0),
            "total_associations": rust_stats.get("total_associations", 0),
            "total_edges": rust_stats.get("total_edges", 0),
            "total_words": rust_stats.get("total_words", 0),
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
