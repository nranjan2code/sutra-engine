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
from .nlp_adapter import OllamaNLPProcessor

from .embeddings import EmbeddingProvider, OllamaEmbedding, SemanticEmbedding, TfidfEmbedding
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
        
        # PRODUCTION: Strict nomic-embed-text NLP processor, NO FALLBACKS
        # Create BEFORE ReasoningEngine to ensure proper initialization order
        import os
        model_name = os.getenv("SUTRA_EMBEDDING_MODEL", "nomic-embed-text")
        try:
            self.ollama_nlp = OllamaNLPProcessor(model_name=model_name)
            logger.info(f"✅ PRODUCTION: Created Ollama NLP processor with {model_name}")
        except Exception as e:
            raise RuntimeError(
                f"PRODUCTION FAILURE: Cannot initialize Ollama NLP processor with {model_name}. "
                f"Ensure Ollama is running and nomic-embed-text is loaded. Error: {e}"
            )
        
        # Initialize ReasoningEngine with pre-created OllamaNLPProcessor
        # This ensures QueryProcessor gets the right NLP processor from the start
        self._core = ReasoningEngine(use_rust_storage=True)
        
        # Set the NLP processor BEFORE initialization completes
        self._core.nlp_processor = self.ollama_nlp
        # Recreate QueryProcessor with the correct NLP processor
        self._core.query_processor = QueryProcessor(
            self._core.storage,
            self._core.association_extractor,
            self._core.path_finder,
            self._core.mppa,
            embedding_processor=None,  # Will use nlp_processor fallback (which is now OllamaNLP)
            nlp_processor=self.ollama_nlp,
        )
        logger.info("Recreated QueryProcessor with OllamaNLPProcessor")
        self._embedding_provider = self._init_embeddings(enable_semantic)
        self._explainer = ExplanationGenerator()
        self._query_cache: Dict[str, ExplainableResult] = {}
        logger.info(
            "Initialized SutraAI (Hybrid->Core->Storage) with embeddings=%s",
            self._embedding_provider.get_name(),
        )

    def _init_embeddings(self, use_semantic: bool) -> EmbeddingProvider:
        # PRODUCTION: Strict nomic-embed-text requirement, NO FALLBACKS
        import os
        model_name = os.getenv("SUTRA_EMBEDDING_MODEL", "nomic-embed-text")
        
        if model_name != "nomic-embed-text":
            raise ValueError(
                f"PRODUCTION REQUIREMENT: Only nomic-embed-text (768-d) is supported. "
                f"Set SUTRA_EMBEDDING_MODEL=nomic-embed-text. Current: {model_name}"
            )
        
        if not use_semantic:
            raise ValueError(
                "PRODUCTION REQUIREMENT: Semantic embeddings must be enabled. "
                "Set SUTRA_USE_SEMANTIC_EMBEDDINGS=true"
            )
        
        try:
            logger.info(f"🔧 PRODUCTION: Initializing Ollama with {model_name} (768-d, NO FALLBACKS)")
            return OllamaEmbedding(model_name=model_name)
        except (ConnectionError, RuntimeError) as e:
            raise RuntimeError(
                f"PRODUCTION FAILURE: Cannot initialize nomic-embed-text embeddings. "
                f"Ensure Ollama is running with nomic-embed-text model loaded. Error: {e}"
            )

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
