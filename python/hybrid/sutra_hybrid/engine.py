"""
SutraAI - Hybrid service using TCP storage client (no gRPC).

All graph/storage operations are done via the TCP storage server.
Local component focuses on embeddings and orchestration only.
"""

import hashlib
import logging
import time
import uuid
from datetime import datetime
from typing import Any, Dict, List, Optional, Tuple

import numpy as np

# Import ReasoningEngine from sutra-core (proper architecture)
from sutra_core import ReasoningEngine
from sutra_core.reasoning.query import QueryProcessor
# Using EmbeddingServiceProvider for all embedding operations

from .embeddings import EmbeddingProvider, EmbeddingServiceProvider, SemanticEmbedding, TfidfEmbedding
from .explanation import ExplanationGenerator
from .results import (
    AuditTrail,
    ConfidenceBreakdown,
    ExplainableResult,
    LearnResult,
    MultiStrategyResult,
    ReasoningPathDetail,
)

logger = logging.getLogger(__name__)


class SutraAI:
    """
    Hybrid AI orchestrator (embeddings + storage via TCP).

    Responsibilities:
    - Generate embeddings for learn/query
    - Forward learn/graph ops to storage via TCP client
    - Build explainable results using available signals
    """

    def __init__(
        self,
        storage_server: str = "storage-server:50051",
        enable_semantic: bool = True,
    ) -> None:
        self.enable_semantic = enable_semantic
        # Use proper architecture: Hybrid -> Core -> Storage
        # Set environment variable for ReasoningEngine to use server mode
        import os
        os.environ["SUTRA_STORAGE_MODE"] = "server"
        os.environ["SUTRA_STORAGE_SERVER"] = storage_server
        
        # PRODUCTION: Initialize embedding service (ONLY option)
        # Create BEFORE ReasoningEngine to ensure proper initialization order
        import os
        service_url = os.getenv("SUTRA_EMBEDDING_SERVICE_URL", "http://sutra-embedding-service:8888")
        
        try:
            self.embedding_processor = EmbeddingServiceProvider(service_url=service_url)
            logger.info(f"✅ PRODUCTION: Created EmbeddingServiceProvider at {service_url}")
        except Exception as e:
            raise RuntimeError(
                f"PRODUCTION FAILURE: Cannot initialize embedding service at {service_url}. "
                f"Ensure service is running and healthy. Error: {e}"
            )
        
        # Initialize ReasoningEngine with embedding service processor
        # This ensures QueryProcessor gets the right embedding processor from the start
        self._core = ReasoningEngine()
        
        # Set the embedding processor BEFORE initialization completes
        self._core.embedding_processor = self.embedding_processor
        # Recreate QueryProcessor with the embedding service processor
        self._core.query_processor = QueryProcessor(
            self._core.storage,
            self._core.association_extractor,
            self._core.path_finder,
            self._core.mppa,
            embedding_processor=self.embedding_processor,
            nlp_processor=None,
        )
        logger.info("Recreated QueryProcessor with EmbeddingServiceProvider")
        self._embedding_provider = self._init_embeddings(enable_semantic)
        self._explainer = ExplanationGenerator()
        self._query_cache: Dict[str, ExplainableResult] = {}
        logger.info(
            "Initialized SutraAI (Hybrid->Core->Storage) with embeddings=%s",
            self._embedding_provider.get_name(),
        )

    def _init_embeddings(self, use_semantic: bool) -> EmbeddingProvider:
        # PRODUCTION: Use embedding service for consistency
        if not use_semantic:
            raise ValueError(
                "PRODUCTION REQUIREMENT: Semantic embeddings must be enabled. "
                "Set SUTRA_USE_SEMANTIC_EMBEDDINGS=true"
            )
        
        # Return the same embedding service provider for consistency
        logger.info("🔧 PRODUCTION: Using EmbeddingServiceProvider (nomic-embed-text-v1.5, 768-d)")
        return self.embedding_processor

    def _content_id(self, content: str) -> str:
        return hashlib.sha1(content.encode("utf-8")).hexdigest()[:16]

    # --------------------------- Learning ---------------------------------
    def learn(
        self,
        content: str,
        source: Optional[str] = None,
        category: Optional[str] = None,
        **metadata,
    ) -> LearnResult:
        start_time = time.time()

        # Learn via ReasoningEngine → TcpStorageAdapter → Storage Server
        # Storage server handles: embedding generation + association extraction + storage
        # This eliminates duplicate code and ensures consistency
        concept_id = self._core.learn(
            content=content,
            source=source,
            category=category,
        )

        exec_ms = (time.time() - start_time) * 1000
        logger.info(f"Learned concept {concept_id} in {exec_ms:.1f}ms (unified pipeline)")
        return LearnResult(
            concept_id=concept_id,
            timestamp=datetime.utcnow().isoformat() + "Z",
            concepts_created=1,
            associations_created=0,  # TODO: Get actual count from storage
            message="Knowledge learned successfully",
            source=source,
            category=category,
        )

    # ----------------------------- Query ---------------------------------
    def ask(
        self,
        query: str,
        explain: bool = True,
        semantic_boost: bool = True,
        num_paths: int = 5,
        min_confidence: float = 0.0,
    ) -> ExplainableResult:
        start_time = time.time()
        query_id = f"q_{uuid.uuid4().hex[:12]}"

        semantic_confidence = 0.0
        semantic_support: Optional[List[Dict[str, Any]]] = None
        matched_concept: Optional[str] = None

        # Use ReasoningEngine for proper reasoning
        core_result = self._core.ask(query, num_reasoning_paths=num_paths)
        
        # Extract answer and confidence from core result
        answer = core_result.primary_answer
        graph_confidence = core_result.confidence
        
        # Apply semantic boost if enabled
        if semantic_boost and self.enable_semantic:
            try:
                qvec = self._embedding_provider.encode([query])[0]
                # Use core's search capability
                search_results = self._core.search_concepts(query, limit=5)
                if search_results:
                    semantic_confidence = search_results[0].get('relevance_score', 0.0)
                    semantic_support = [
                        {"concept_id": result.get('id', ''), "similarity": result.get('relevance_score', 0.0)}
                        for result in search_results
                    ]
            except Exception as e:
                logger.warning(f"Semantic search failed: {e}")

        # Convert reasoning paths from core result
        reasoning_paths: Optional[List[ReasoningPathDetail]] = None
        if core_result.supporting_paths:
            reasoning_paths = []
            for path in core_result.supporting_paths:
                reasoning_paths.append(
                    ReasoningPathDetail(
                        concepts=[step.target_concept for step in path.steps],
                        concept_ids=[step.target_concept for step in path.steps],  # Simplified
                        association_types=[step.relation for step in path.steps],
                        confidence=path.confidence,
                        explanation=f"Graph reasoning path with {len(path.steps)} steps",
                    )
                )

        final_confidence = max(graph_confidence, semantic_confidence, min_confidence)
        exec_ms = (time.time() - start_time) * 1000

        confidence_breakdown = ConfidenceBreakdown(
            graph_confidence=graph_confidence,
            semantic_confidence=semantic_confidence,
            path_quality=graph_confidence,
            consensus_strength=core_result.consensus_strength,
            final_confidence=final_confidence,
        )

        audit_trail = AuditTrail(
            query_id=query_id,
            query=query,
            timestamp=datetime.utcnow().isoformat() + "Z",
            concepts_accessed=len(semantic_support or []),
            associations_traversed=0,
            execution_time_ms=exec_ms,
            reasoning_method="hybrid_graph_semantic" if semantic_boost else "graph_only",
            semantic_boost_used=semantic_boost and self.enable_semantic,
            paths_explored=len(reasoning_paths) if reasoning_paths else 0,
            storage_path="tcp://storage-server",
        )

        explanation_text = None
        if explain:
            explanation_text = self._explainer.generate(
                query=query,
                answer=answer,
                confidence=final_confidence,
                reasoning_paths=reasoning_paths or [],
                semantic_boost=semantic_boost and self.enable_semantic,
                semantic_contribution=semantic_confidence,
            )

        result = ExplainableResult(
            answer=answer,
            confidence=final_confidence,
            explanation=explanation_text,
            reasoning_paths=reasoning_paths,
            confidence_breakdown=confidence_breakdown,
            semantic_support=semantic_support,
            audit_trail=audit_trail,
            metadata={"query_id": query_id, "execution_time_ms": exec_ms},
        )

        self._query_cache[query_id] = result
        logger.info(
            f"Query {query_id} answered in {exec_ms:.1f}ms (confidence: {final_confidence:.2f})"
        )
        return result

    # --------------------------- Utilities --------------------------------
    def multi_strategy(self, query: str) -> MultiStrategyResult:
        # With gRPC-only architecture, we currently have one strategy (semantic)
        semantic = self.ask(query, explain=True, semantic_boost=True)
        return MultiStrategyResult(
            query=query,
            graph_only=semantic,  # Placeholder: graph handled by storage server
            semantic_enhanced=semantic,
            agreement_score=1.0,
            recommended_strategy="semantic_only",
            reasoning="Single available strategy in this service",
        )

    def get_audit_trail(self, limit: int = 10) -> List[Dict[str, Any]]:
        entries = []
        for qid, res in list(self._query_cache.items())[:limit]:
            if res.audit_trail:
                entries.append(
                    {
                        "query_id": qid,
                        "timestamp": res.audit_trail.timestamp,
                        "operation": "query",
                        "input": {"query": res.audit_trail.query},
                        "output": {"answer": res.answer},
                        "confidence": res.confidence,
                    }
                )
        return entries

    def get_stats(self) -> Dict[str, Any]:
        # Get stats from ReasoningEngine (proper architecture)
        s = self._core.get_system_stats()
        storage_stats = s.get("storage", {})
        return {
            "total_concepts": storage_stats.get("total_concepts", 0),
            "total_associations": storage_stats.get("total_associations", 0),
            "total_embeddings": 0,  # Will implement embedding tracking later
            "embedding_provider": self._embedding_provider.get_name(),
            "embedding_dimension": 768,  # Standard embedding dimension
            "average_strength": 1.0,  # Default value
            "semantic_enabled": self.enable_semantic,
            "version": "2.0.0",
            "cached_queries": len(self._query_cache),
        }

    def save(self) -> None:
        # Save via ReasoningEngine
        self._core.save()

    def close(self) -> None:
        try:
            self._core.close()
        except Exception:
            pass

    # learn method already defined above - removing duplicate

    # ask method already defined above - removing duplicate

    # _semantic_search method removed - using ReasoningEngine's search_concepts instead

    def explain(self, query_id: str) -> Optional[ExplainableResult]:
        """
        Get detailed explanation for a previous query.

        Args:
            query_id: ID from previous ask() call

        Returns:
            ExplainableResult with full explanation, or None if not found
        """
        return self._query_cache.get(query_id)

    def multi_strategy(self, query: str) -> MultiStrategyResult:
        """
        Compare different reasoning strategies.

        Runs query using:
        1. Pure graph reasoning (100% explainable, no embeddings)
        2. Semantic-enhanced reasoning (graph + embeddings boost)

        Args:
            query: Natural language question

        Returns:
            MultiStrategyResult showing comparison

        Example:
            >>> result = ai.multi_strategy("What causes climate change?")
            >>> print(result.agreement_score)
            >>> print(result.recommended_strategy)
        """
        # Strategy 1: Pure graph (no semantic)
        graph_result = self.ask(query, explain=True, semantic_boost=False)

        # Strategy 2: Semantic-enhanced
        if self.enable_semantic:
            semantic_result = self.ask(query, explain=True, semantic_boost=True)
        else:
            # If semantic not available, return same result
            semantic_result = graph_result

        # Calculate agreement using semantic similarity
        agreement = self._calculate_semantic_agreement(
            graph_result.answer, semantic_result.answer
        )

        # Determine recommendation
        if not self.enable_semantic:
            recommended = "graph_only"
            reasoning = "Semantic embeddings not available"
        elif agreement > 0.9:
            recommended = "semantic_enhanced"
            reasoning = "Strong agreement and higher confidence with semantic boost"
        elif agreement > 0.7:
            recommended = "semantic_enhanced"
            reasoning = "Moderate agreement, semantic provides additional context"
        else:
            recommended = "graph_only"
            reasoning = "Low agreement, prefer explainable graph reasoning"

        return MultiStrategyResult(
            query=query,
            graph_only=graph_result,
            semantic_enhanced=semantic_result,
            agreement_score=agreement,
            recommended_strategy=recommended,
            reasoning=reasoning,
        )

    def _calculate_semantic_agreement(self, answer1: str, answer2: str) -> float:
        """
        Calculate semantic agreement between two answers.

        Uses embeddings if available, falls back to word overlap.
        """
        if not self.enable_semantic:
            return self._word_overlap_similarity(answer1, answer2)

        try:
            # Use semantic embeddings for agreement
            emb1 = self._embedding_provider.encode([answer1])[0]
            emb2 = self._embedding_provider.encode([answer2])[0]

            similarity = self._embedding_provider.similarity(emb1, emb2)
            return float(similarity)

        except Exception as e:
            logger.warning(f"Semantic agreement calculation failed: {e}")
            return self._word_overlap_similarity(answer1, answer2)

    def _word_overlap_similarity(self, text1: str, text2: str) -> float:
        """Fallback: Simple word overlap for agreement calculation."""
        words1 = set(text1.lower().split())
        words2 = set(text2.lower().split())

        if not words1 or not words2:
            return 0.0

        overlap = len(words1 & words2)
        total = len(words1 | words2)

        return overlap / total if total > 0 else 0.0

    def get_audit_trail(self, limit: int = 10) -> List[Dict[str, Any]]:
        """
        Get recent audit trail entries.

        Args:
            limit: Maximum number of entries to return

        Returns:
            List of audit trail entries with timestamps, operations, and metadata
        """
        # Return cached query results as audit trail
        entries = []
        for query_id, result in list(self._query_cache.items())[:limit]:
            if result.audit_trail:
                entries.append(
                    {
                        "query_id": query_id,
                        "timestamp": result.audit_trail.timestamp,
                        "operation": "query",
                        "input": {"query": result.audit_trail.query},
                        "output": {"answer": result.answer},
                        "confidence": result.confidence,
                    }
                )
        return entries

    def get_concept(self, concept_id: str) -> Optional[Dict[str, Any]]:
        """
        Get detailed information about a specific concept.
        """
        return self._core.get_concept_info(concept_id)

    # get_stats method already defined above - removing duplicate

    # save method already defined above - removing duplicate

    # close method already defined above - removing duplicate

    # ======================== SEMANTIC QUERY METHODS ========================

    def find_semantic_path(
        self,
        start_query: str,
        end_query: str,
        semantic_filter: Optional[Dict] = None,
        max_depth: int = 5,
    ) -> Dict[str, Any]:
        """
        Find semantic path between concepts with explainable results.

        Args:
            start_query: Starting concept (natural language or ID)
            end_query: Ending concept (natural language or ID)
            semantic_filter: Optional semantic constraints
            max_depth: Maximum path depth

        Returns:
            Dictionary with paths and metadata
        """
        start_time = time.time()

        # For now, use queries as concept IDs directly
        # TODO: Add concept resolution from natural language
        start_id = self._content_id(start_query) if len(start_query) > 16 else start_query
        end_id = self._content_id(end_query) if len(end_query) > 16 else end_query

        # Call core engine
        paths = self._core.find_semantic_path(start_id, end_id, semantic_filter, max_depth)

        exec_ms = (time.time() - start_time) * 1000

        return {
            "start_query": start_query,
            "end_query": end_query,
            "paths": paths,
            "execution_time_ms": exec_ms,
            "filter_applied": semantic_filter is not None,
        }

    def find_temporal_chain(
        self,
        start_query: str,
        end_query: str,
        max_depth: int = 10,
        after: Optional[str] = None,
        before: Optional[str] = None,
    ) -> Dict[str, Any]:
        """
        Find temporal reasoning chain.

        Args:
            start_query: Starting concept
            end_query: Ending concept
            max_depth: Maximum chain depth
            after: Filter events after this date (ISO 8601)
            before: Filter events before this date (ISO 8601)

        Returns:
            Dictionary with temporal chains
        """
        start_time = time.time()

        start_id = self._content_id(start_query) if len(start_query) > 16 else start_query
        end_id = self._content_id(end_query) if len(end_query) > 16 else end_query

        chains = self._core.find_temporal_chain(start_id, end_id, max_depth, after, before)

        exec_ms = (time.time() - start_time) * 1000

        return {
            "start_query": start_query,
            "end_query": end_query,
            "chains": chains,
            "temporal_constraints": {"after": after, "before": before},
            "execution_time_ms": exec_ms,
        }

    def find_causal_chain(
        self,
        start_query: str,
        end_query: str,
        max_depth: int = 5,
    ) -> Dict[str, Any]:
        """
        Find causal reasoning chain.

        Args:
            start_query: Starting concept
            end_query: Ending concept
            max_depth: Maximum chain depth

        Returns:
            Dictionary with causal chains
        """
        start_time = time.time()

        start_id = self._content_id(start_query) if len(start_query) > 16 else start_query
        end_id = self._content_id(end_query) if len(end_query) > 16 else end_query

        chains = self._core.find_causal_chain(start_id, end_id, max_depth)

        exec_ms = (time.time() - start_time) * 1000

        return {
            "start_query": start_query,
            "end_query": end_query,
            "chains": chains,
            "execution_time_ms": exec_ms,
        }

    def find_contradictions(
        self,
        query: str,
        max_depth: int = 3,
    ) -> Dict[str, Any]:
        """
        Detect contradictions in knowledge base.

        Args:
            query: Concept to check (natural language or ID)
            max_depth: Search depth for contradictions

        Returns:
            Dictionary with contradictions
        """
        start_time = time.time()

        concept_id = self._content_id(query) if len(query) > 16 else query

        contradictions = self._core.find_contradictions(concept_id, max_depth)

        exec_ms = (time.time() - start_time) * 1000

        return {
            "query": query,
            "concept_id": concept_id,
            "contradictions": [
                {
                    "concept_id1": c[0],
                    "concept_id2": c[1],
                    "confidence": c[2],
                }
                for c in contradictions
            ],
            "count": len(contradictions),
            "execution_time_ms": exec_ms,
        }

    def query_semantic(
        self,
        semantic_filter: Dict,
        max_results: int = 100,
    ) -> Dict[str, Any]:
        """
        Query concepts by semantic filter.

        Args:
            semantic_filter: Semantic filter constraints
            max_results: Maximum number of results

        Returns:
            Dictionary with matching concepts
        """
        start_time = time.time()

        results = self._core.query_semantic(semantic_filter, max_results)

        exec_ms = (time.time() - start_time) * 1000

        return {
            "filter": semantic_filter,
            "results": results,
            "count": len(results),
            "execution_time_ms": exec_ms,
        }

    def _convert_paths(self, core_paths) -> List[ReasoningPathDetail]:
        """Convert core ReasoningPath objects to ReasoningPathDetail."""
        converted = []

        for path in core_paths:
            converted.append(
                ReasoningPathDetail(
                    concepts=[c.content for c in path.concepts],
                    concept_ids=[c.id for c in path.concepts],
                    association_types=[a.assoc_type.name for a in path.associations],
                    confidence=path.confidence,
                    explanation=(
                        path.explanation
                        if hasattr(path, "explanation")
                        else "Reasoning path"
                    ),
                )
            )

        return converted
