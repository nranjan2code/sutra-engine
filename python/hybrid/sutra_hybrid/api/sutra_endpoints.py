"""Custom Sutra API endpoints.

Endpoints with full explainability, audit trails, and multi-strategy reasoning.
"""

from datetime import datetime
from typing import List, Optional

from fastapi import APIRouter, Depends, HTTPException, Query
import logging

from ..engine import SutraAI
from .models import (
    AuditLogEntry,
    AuditLogResponse,
    ConceptInfo,
    ConfidenceBreakdown,
    HealthResponse,
    LearnRequest,
    LearnResponse,
    MultiStrategyRequest,
    MultiStrategyResponse,
    QueryRequest,
    QueryResponse,
    ReasoningPath,
    SemanticMatch,
    StatsResponse,
    StrategyResult,
)

# Router for Sutra-specific endpoints
router = APIRouter(prefix="/sutra", tags=["Sutra"])
logger = logging.getLogger(__name__)

# Global AI instance (will be injected)
_ai_instance: Optional[SutraAI] = None
_start_time: float = 0.0


def set_ai_instance(ai: SutraAI) -> None:
    """Set the global AI instance."""
    global _ai_instance, _start_time
    import time

    _ai_instance = ai
    _start_time = time.time()


def get_ai() -> SutraAI:
    """Dependency to get AI instance."""
    if _ai_instance is None:
        raise HTTPException(status_code=500, detail="AI instance not initialized")
    return _ai_instance


@router.post("/learn", response_model=LearnResponse)
async def learn(
    request: LearnRequest,
    ai: SutraAI = Depends(get_ai),
) -> LearnResponse:
    """Learn new knowledge from text.

    Args:
        request: Learning request with text to learn from
        ai: SutraAI instance (injected)

    Returns:
        Learning response with statistics

    Raises:
        HTTPException: If learning fails
    """
    try:
        # Learn via hybrid (embeddings -> storage)
        result = ai.learn(request.text)

        return LearnResponse(
            success=True,
            concepts_learned=1,
            associations_created=0,
            message="Successfully learned 1 concept",
        )

    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Learning failed: {str(e)}")


@router.post("/query", response_model=QueryResponse)
async def query(
    request: QueryRequest,
    ai: SutraAI = Depends(get_ai),
) -> QueryResponse:
    """Query the knowledge base with full explainability.

    Args:
        request: Query request with question and parameters
        ai: SutraAI instance (injected)

    Returns:
        Query response with answer, confidence, and explanation

    Raises:
        HTTPException: If query fails
    """
    try:
        # Query the AI
        result = ai.ask(
            query=request.query,
            semantic_boost=request.semantic_boost,
            num_paths=request.max_paths,
            # Note: max_depth not supported in current SutraAI.ask() implementation
        )

        # Convert reasoning paths (if present)
        converted_reasoning_paths: List[ReasoningPath] = []
        raw_paths_for_nlg: List[dict] = []
        if result.reasoning_paths:
            for path in result.reasoning_paths:
                # path is ReasoningPathDetail dataclass
                converted_reasoning_paths.append(
                    ReasoningPath(
                        path=[c for c in getattr(path, "concepts", [])],
                        confidence=getattr(path, "confidence", 0.0),
                    )
                )
                raw_paths_for_nlg.append({
                    "concepts": getattr(path, "concepts", []),
                    "concept_ids": getattr(path, "concept_ids", []),
                    "association_types": getattr(path, "association_types", []),
                    "confidence": getattr(path, "confidence", 0.0),
                    "explanation": getattr(path, "explanation", ""),
                })
        
        # Optional NLG post-processing (template or hybrid LLM)
        final_answer_text = result.answer
        nlg_metadata = {}
        # NLG disabled - using external NLG service (sutraworks-model)
        # TODO: Implement HTTP client to call external NLG service
        nlg_enabled = False
        
        if nlg_enabled:
            # NLG processing would go here
            pass

        # Convert semantic support if available
        semantic_support = None
        if result.semantic_support:
            # result.semantic_support is already list of dicts from engine
            try:
                semantic_support = [
                    SemanticMatch(concept=item.get("concept_id") or item.get("concept", ""),
                                  similarity=float(item.get("similarity", 0.0)))
                    for item in result.semantic_support
                ]
            except Exception:
                semantic_support = None

        # Build confidence breakdown
        confidence_breakdown = ConfidenceBreakdown(
            graph_confidence=getattr(result, "graph_confidence", 0.0),
            semantic_confidence=getattr(result, "semantic_confidence", 0.0),
            final_confidence=result.confidence,
        )

        return QueryResponse(
            answer=final_answer_text,
            confidence=result.confidence,
            confidence_breakdown=confidence_breakdown,
            reasoning_paths=converted_reasoning_paths,
            semantic_support=semantic_support,
            explanation=result.explanation or "",
            timestamp=(result.audit_trail.timestamp if result.audit_trail else datetime.utcnow().isoformat()),
        )

    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Query failed: {str(e)}")


@router.post("/multi-strategy", response_model=MultiStrategyResponse)
async def multi_strategy(
    request: MultiStrategyRequest,
    ai: SutraAI = Depends(get_ai),
) -> MultiStrategyResponse:
    """Compare multiple reasoning strategies.

    Args:
        request: Multi-strategy request with query
        ai: SutraAI instance (injected)

    Returns:
        Multi-strategy response with comparison and recommendation

    Raises:
        HTTPException: If comparison fails
    """
    try:
        # Run multi-strategy comparison
        result = ai.multi_strategy(
            query=request.query,
            max_depth=request.max_depth,
            max_paths=request.max_paths,
        )

        # Convert strategies
        strategies = []
        for strategy in result.strategies:
            reasoning_paths = [
                ReasoningPath(
                    path=[concept for concept, _ in path.path],
                    confidence=path.confidence,
                )
                for path in strategy.reasoning_paths
            ]

            strategies.append(
                StrategyResult(
                    strategy=strategy.strategy,
                    answer=strategy.answer,
                    confidence=strategy.confidence,
                    reasoning_paths=reasoning_paths,
                )
            )

        return MultiStrategyResponse(
            query=result.query,
            strategies=strategies,
            agreement_score=result.agreement_score,
            recommended_answer=result.recommended_answer,
            explanation=result.explanation,
            timestamp=result.timestamp.isoformat(),
        )

    except Exception as e:
        raise HTTPException(
            status_code=500, detail=f"Multi-strategy comparison failed: {str(e)}"
        )


@router.get("/health", response_model=HealthResponse)
async def health(ai: SutraAI = Depends(get_ai)) -> HealthResponse:
    """Health check endpoint.

    Args:
        ai: SutraAI instance (injected)

    Returns:
        Health status and statistics
    """
    import time

    uptime = time.time() - _start_time

    stats = ai.get_stats()

    return HealthResponse(
        status="healthy",
        version="1.0.0",
        uptime_seconds=uptime,
        total_concepts=stats.get("total_concepts", 0),
        total_associations=stats.get("total_associations", 0),
    )


@router.get("/audit", response_model=AuditLogResponse)
async def audit_log(
    limit: int = Query(default=100, ge=1, le=1000),
    ai: SutraAI = Depends(get_ai),
) -> AuditLogResponse:
    """Get audit logs for compliance.

    Args:
        limit: Maximum number of entries to return
        ai: SutraAI instance (injected)

    Returns:
        Audit log entries

    Raises:
        HTTPException: If retrieval fails
    """
    try:
        # Get audit trail from AI
        audit_trail = ai.get_audit_trail(limit=limit)

        # Convert to API format
        entries = [
            AuditLogEntry(
                timestamp=entry["timestamp"].isoformat(),
                operation=entry["operation"],
                input_data=entry["input"],
                output_data=entry["output"],
                confidence=entry.get("confidence"),
            )
            for entry in audit_trail
        ]

        return AuditLogResponse(
            total_entries=len(entries),
            entries=entries,
        )

    except Exception as e:
        raise HTTPException(
            status_code=500, detail=f"Failed to retrieve audit logs: {str(e)}"
        )


@router.get("/stats", response_model=StatsResponse)
async def stats(
    top_n: int = Query(default=10, ge=1, le=100),
    ai: SutraAI = Depends(get_ai),
) -> StatsResponse:
    """Get knowledge base statistics.

    Args:
        top_n: Number of top concepts to return
        ai: SutraAI instance (injected)

    Returns:
        Knowledge base statistics
    """
    stats = ai.get_stats()

    total_concepts = int(stats.get("total_concepts", 0))
    total_associations = int(stats.get("total_associations", 0))
    avg_strength = 0.0  # Not tracked in storage server yet

    top_concepts: List[ConceptInfo] = []  # Not available via gRPC yet

    return StatsResponse(
        total_concepts=total_concepts,
        total_associations=total_associations,
        avg_concept_strength=avg_strength,
        top_concepts=top_concepts,
    )


@router.post("/reset")
async def reset(ai: SutraAI = Depends(get_ai)):
    """Reset the knowledge base (for testing).

    Args:
        ai: SutraAI instance (injected)

    Returns:
        Success message
    """
    try:
        # This would need to be implemented in SutraAI
        # For now, return a message
        return {"message": "Reset not implemented - restart server to reset"}

    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Reset failed: {str(e)}")
