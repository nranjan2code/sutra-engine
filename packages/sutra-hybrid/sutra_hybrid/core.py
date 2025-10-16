"""
HybridAI - Combines graph-based reasoning with semantic embeddings.

This is the main class that integrates:
- Graph reasoning from sutra-core
- Semantic embeddings for similarity search
- Combined reasoning strategies
"""

import logging
from collections import defaultdict
from pathlib import Path
from typing import Dict, List, Optional, Tuple

import numpy as np
from sutra_core import AdaptiveLearner, AssociationExtractor, Concept

from .embeddings import EmbeddingProvider, SemanticEmbedding, TfidfEmbedding
from .storage import HybridStorage

logger = logging.getLogger(__name__)


class HybridAI:
    """
    Hybrid AI system combining graph reasoning with semantic embeddings.

    Features:
    - Graph-based associative reasoning
    - Semantic similarity search
    - Combined reasoning strategies
    - Automatic fallback to TF-IDF
    """

    def __init__(
        self,
        use_semantic: bool = True,
        storage_path: Optional[str] = None,
    ):
        """
        Initialize Hybrid AI system.

        Args:
            use_semantic: Try to use sentence-transformers
                (falls back to TF-IDF)
            storage_path: Path for persisting knowledge
        """
        # Initialize graph components from sutra-core
        self.concepts: Dict[str, Concept] = {}
        self.associations = {}
        self.word_to_concepts = defaultdict(set)
        self.concept_neighbors = defaultdict(set)

        # Initialize extractor and learner
        self.extractor = AssociationExtractor(
            self.concepts,
            self.word_to_concepts,
            self.concept_neighbors,
            self.associations,
        )
        self.learner = AdaptiveLearner(self.concepts, self.extractor)

        # Initialize embedding provider
        self.embedding_provider = self._init_embeddings(use_semantic)

        # Concept embeddings cache
        self.concept_embeddings: Dict[str, np.ndarray] = {}

        # Storage
        self.storage_path = (
            Path(storage_path) if storage_path else Path("./hybrid_knowledge")
        )
        self.storage_path.mkdir(exist_ok=True)
        self.storage = HybridStorage(self.storage_path)

        # Auto-load if data exists
        if self.storage.exists():
            self.load()

        logger.info(f"Initialized HybridAI with {self.embedding_provider.get_name()}")

    def _init_embeddings(self, use_semantic: bool) -> EmbeddingProvider:
        """Initialize embedding provider with fallback."""
        if use_semantic:
            try:
                return SemanticEmbedding()
            except ImportError:
                logger.warning("sentence-transformers not available, using TF-IDF")
                return TfidfEmbedding()
        else:
            return TfidfEmbedding()

    def learn(self, content: str, **kwargs) -> str:
        """
        Learn new knowledge with both graph and semantic understanding.

        Args:
            content: Knowledge to learn
            **kwargs: Additional arguments for adaptive learning

        Returns:
            Concept ID
        """
        # Learn using graph-based adaptive learning
        concept_id = self.learner.learn_adaptive(content, **kwargs)

        # Generate and store embedding
        embedding = self.embedding_provider.encode([content])[0]
        self.concept_embeddings[concept_id] = embedding

        return concept_id

    def learn_batch(
        self, contents: List[Tuple[str, Optional[str], Optional[str]]]
    ) -> List[str]:
        """
        Learn multiple knowledge items in batch for efficiency.

        Args:
            contents: List of (content, source, category) tuples

        Returns:
            List of concept IDs
        """
        if not contents:
            return []

        concept_ids = []
        embeddings_list = []

        # Learn all concepts first
        for content, source, category in contents:
            concept_id = self.learner.learn_adaptive(
                content, source=source, category=category
            )
            concept_ids.append(concept_id)

        # Batch generate embeddings
        content_texts = [c[0] for c in contents]
        embeddings = self.embedding_provider.encode(content_texts)

        # Store embeddings
        for concept_id, embedding in zip(concept_ids, embeddings):
            self.concept_embeddings[concept_id] = embedding

        logger.info(f"Batch learned {len(concept_ids)} concepts")

        return concept_ids

    def semantic_search(
        self, query: str, top_k: int = 5, threshold: float = 0.5
    ) -> List[Tuple[str, float]]:
        """
        Search for concepts using semantic similarity.

        Args:
            query: Search query
            top_k: Number of results to return
            threshold: Minimum similarity threshold

        Returns:
            List of (concept_id, similarity_score) tuples
        """
        if not self.concept_embeddings:
            return []

        # Encode query
        query_embedding = self.embedding_provider.encode([query])[0]

        # Calculate similarities
        similarities = []
        for concept_id, concept_embedding in self.concept_embeddings.items():
            similarity = self.embedding_provider.similarity(
                query_embedding, concept_embedding
            )
            if similarity >= threshold:
                similarities.append((concept_id, similarity))

        # Sort by similarity and return top_k
        similarities.sort(key=lambda x: x[1], reverse=True)
        return similarities[:top_k]

    def get_concept(self, concept_id: str) -> Optional[Concept]:
        """
        Get concept by ID.

        Args:
            concept_id: Concept identifier

        Returns:
            Concept object or None
        """
        return self.concepts.get(concept_id)

    def get_stats(self) -> dict:
        """
        Get system statistics.

        Returns:
            Dictionary with system stats
        """
        return {
            "total_concepts": len(self.concepts),
            "total_associations": len(self.associations),
            "total_embeddings": len(self.concept_embeddings),
            "embedding_provider": self.embedding_provider.get_name(),
            "embedding_dimension": self.embedding_provider.get_dimension(),
            **self.learner.get_learning_stats(),
        }

    def save(self) -> None:
        """
        Save all data to disk.
        """
        # Get vocabulary and vectorizer state if using TF-IDF
        vocabulary = None
        vectorizer_state = None

        if hasattr(self.embedding_provider, "get_vocabulary"):
            vocabulary = self.embedding_provider.get_vocabulary()  # Legacy

        if hasattr(self.embedding_provider, "get_state"):
            vectorizer_state = self.embedding_provider.get_state()
            if vectorizer_state:
                logger.info("Saving TF-IDF vectorizer state")

        self.storage.save(
            self.concepts,
            self.associations,
            self.concept_embeddings,
            self.embedding_provider.get_name(),
            vocabulary,
            vectorizer_state,
        )

    def load(self) -> None:
        """
        Load data from disk.
        """
        (
            concepts,
            associations,
            embeddings,
            provider_name,
            vocabulary,
            vectorizer_state,
        ) = self.storage.load()

        # Restore vectorizer state if available (preferred method)
        if vectorizer_state and hasattr(self.embedding_provider, "set_state"):
            success = self.embedding_provider.set_state(vectorizer_state)
            if success:
                logger.info("Restored TF-IDF vectorizer from pickled state")
            else:
                logger.warning("Failed to restore vectorizer, will refit on use")
        # Fallback to legacy vocabulary method
        elif vocabulary and hasattr(self.embedding_provider, "set_vocabulary"):
            logger.warning("Using legacy vocabulary restore (incomplete)")
            self.embedding_provider.set_vocabulary(vocabulary)

        # Restore data
        self.concepts.update(concepts)
        self.associations.update(associations)
        self.concept_embeddings.update(embeddings)

        # Rebuild indices
        for concept in self.concepts.values():
            self.extractor._index_concept(concept)

        # Rebuild neighbor indices
        for key in self.associations.keys():
            source_id, target_id = key
            self.concept_neighbors[source_id].add(target_id)
            self.concept_neighbors[target_id].add(source_id)

        logger.info(f"Loaded {len(concepts)} concepts from {provider_name} embeddings")
