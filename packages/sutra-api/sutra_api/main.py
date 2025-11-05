""" 
Minimal FastAPI application - Thin TCP Proxy.

This version uses only storage-client for TCP Binary Protocol communication.
NO local reasoning engine or heavy ML dependencies.

PRODUCTION SECURITY:
- httpOnly cookies for authentication (XSS protection)
- Security headers middleware (OWASP recommendations) 
- HTTPS enforcement in secure mode
- Rate limiting and request size limits
"""

import logging
import os
from contextlib import asynccontextmanager

from fastapi import Depends, FastAPI, HTTPException, Request, status
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse

from .config import settings
from .dependencies import (
    get_storage_client,
    get_uptime,
    init_dependencies,
    shutdown_dependencies,
)
from .exceptions import SutraError
from .models import (
    BatchLearnRequest,
    BatchLearnResponse,
    CausalChainRequest,
    CausalChainResponse,
    ConceptDetail,
    ContradictionRequest,
    ContradictionResponse,
    EditionFeaturesResponse,
    EditionLimitsResponse,
    EditionResponse,
    ErrorResponse,
    HealthResponse,
    LearnRequest,
    LearnResponse,
    SearchResult,
    SemanticPathRequest,
    SemanticPathResponse,
    SemanticQueryRequest,
    SemanticQueryResponse,
    SystemStats,
    TemporalChainRequest,
    TemporalChainResponse,
)
from .routes import auth_router, conversations_router, spaces_router, search_router, graph_router, users_router

# 🔥 PRODUCTION SECURITY: Import security middleware
from .security_middleware import (
    SecurityHeadersMiddleware,
    HTTPSRedirectMiddleware, 
    SecureCookieMiddleware,
)

# Import production monitoring (zero external dependencies)
try:
    from sutra_core.monitoring.metrics_middleware import add_production_monitoring
    MONITORING_AVAILABLE = True
except ImportError:
    MONITORING_AVAILABLE = False
    logger.warning("Production monitoring not available - install sutra-core")

# Configure logging
logging.basicConfig(level=settings.log_level, format=settings.log_format)
logger = logging.getLogger(__name__)


@asynccontextmanager
async def lifespan(app: FastAPI):
    """Lifespan context manager for startup and shutdown events."""
    # Startup
    logger.info(f"Starting {settings.api_title} v{settings.api_version}")
    logger.info(f"Storage server: {settings.storage_server}")
    
    # Initialize TCP storage client (Binary Protocol)
    init_dependencies(app)
    
    yield
    
    # Shutdown
    shutdown_dependencies(app)
    logger.info("Shutting down API service")


# Create FastAPI app
app = FastAPI(
    title=settings.api_title,
    version=settings.api_version,
    description="Production REST-to-TCP proxy for Sutra AI (Binary Protocol)",
    lifespan=lifespan,
)

# Add CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=settings.allow_origins,
    allow_credentials=settings.allow_credentials,
    allow_methods=settings.allow_methods,
    allow_headers=settings.allow_headers,
)

# 🔥 PRODUCTION SECURITY: Add security headers middleware
secure_mode = os.getenv("SUTRA_SECURE_MODE", "false").lower() == "true"
if secure_mode:
    logger.info("✅ Production security mode enabled")
    app.add_middleware(HTTPSRedirectMiddleware, enabled=True)
    app.add_middleware(SecureCookieMiddleware, secure=True, httponly=True, samesite="lax")
    app.add_middleware(
        SecurityHeadersMiddleware,
        strict_transport_security=True,
        content_security_policy=True,
        x_frame_options="DENY",
        x_content_type_options=True,
        x_xss_protection=True,
        referrer_policy="strict-origin-when-cross-origin",
        permissions_policy=True,
    )
else:
    logger.warning("⚠️  Development mode - security features disabled. Set SUTRA_SECURE_MODE=true for production.")
    # Still add security headers in dev, just less strict
    app.add_middleware(
        SecurityHeadersMiddleware,
        strict_transport_security=False,
        content_security_policy=True,
        x_frame_options="SAMEORIGIN",  # Allow in dev
    )

# Add production monitoring (internal metrics)
if MONITORING_AVAILABLE:
    app = add_production_monitoring(app, service_name="sutra-api", include_endpoints=True)
    logger.info("✅ Production monitoring enabled - zero external dependencies")

# Add rate limiting middleware  
from .middleware import RateLimitMiddleware

app.add_middleware(
    RateLimitMiddleware,
    default_limit=60,
    window_seconds=60,
    endpoint_limits={
        "/learn": settings.rate_limit_learn,
        "/learn/batch": settings.rate_limit_learn // 2,
        "/search": settings.rate_limit_search,
    },
)

# Include routers
app.include_router(auth_router)
app.include_router(users_router)
app.include_router(conversations_router)
app.include_router(spaces_router)
app.include_router(search_router)
app.include_router(graph_router)


# Exception handlers
@app.exception_handler(SutraError)
async def sutra_error_handler(request, exc: SutraError):
    """Handle Sutra-specific errors."""
    return JSONResponse(
        status_code=status.HTTP_400_BAD_REQUEST,
        content={
            "error": exc.__class__.__name__,
            "message": str(exc),
            "detail": None,
        },
    )


@app.exception_handler(Exception)
async def general_exception_handler(request, exc: Exception):
    """Handle unexpected errors."""
    logger.error(f"Unexpected error: {exc}", exc_info=True)
    return JSONResponse(
        status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
        content={
            "error": "InternalServerError",
            "message": "An unexpected error occurred",
            "detail": str(exc) if settings.log_level == "DEBUG" else None,
        },
    )


# Health check endpoint
@app.get("/health", response_model=HealthResponse, tags=["System"])
async def health_check(client=Depends(get_storage_client)):
    """
    Check API health status.
    
    Returns service status, version, uptime, and basic metrics.
    """
    # Get stats from storage server via TCP Binary Protocol
    try:
        stats = client.stats()
        concepts_loaded = stats.get("concepts", 0)
    except Exception as e:
        logger.warning(f"Failed to get storage stats: {e}")
        concepts_loaded = 0
    
    return HealthResponse(
        status="healthy",
        version=settings.api_version,
        uptime_seconds=get_uptime(),
        concepts_loaded=concepts_loaded,
    )


# Edition information endpoint
@app.get("/edition", response_model=EditionResponse, tags=["System"])
async def get_edition_info():
    """
    Get current edition information, limits, and features.
    
    Returns the active edition, rate limits, quotas, and available features.
    This information is useful for clients to adapt their behavior based on
    the deployment edition.
    """
    from datetime import datetime
    
    # Get edition limits from settings
    limits = settings.get_edition_limits()
    
    # Determine if license is valid (if using enterprise/community)
    license_valid = True
    license_expires = None
    
    if settings.edition in ["community", "enterprise"] and settings.license_key:
        try:
            # Validate license using feature flags
            from sutra_core.feature_flags import validate_license
            license_data = validate_license(settings.license_key)
            if license_data.get("expires"):
                license_expires = datetime.fromtimestamp(
                    license_data["expires"]
                ).isoformat()
        except Exception as e:
            logger.warning(f"License validation failed: {e}")
            license_valid = False
    
    # Determine features based on edition
    ha_enabled = settings.edition == "enterprise"
    grid_enabled = settings.edition == "enterprise"
    observability_enabled = settings.edition == "enterprise"
    multi_node = settings.edition == "enterprise"
    
    return EditionResponse(
        edition=settings.edition,
        limits=EditionLimitsResponse(
            learn_per_min=limits.learn_per_min,
            reason_per_min=limits.reason_per_min,
            max_concepts=limits.max_concepts,
            max_dataset_gb=limits.max_dataset_gb,
            ingest_workers=limits.ingest_workers,
        ),
        features=EditionFeaturesResponse(
            ha_enabled=ha_enabled,
            grid_enabled=grid_enabled,
            observability_enabled=observability_enabled,
            multi_node=multi_node,
        ),
        license_valid=license_valid,
        license_expires=license_expires,
        upgrade_url="https://sutra.ai/pricing",
    )


# Learning endpoint
@app.post(
    "/learn",
    response_model=LearnResponse,
    status_code=status.HTTP_201_CREATED,
    tags=["Learning"],
)
async def learn_knowledge(
    request: LearnRequest,
    client=Depends(get_storage_client)
):
    """
    Learn new knowledge using unified learning pipeline.
    
    Storage server handles: embedding generation + association extraction + storage.
    This ensures consistency across all services.
    """
    try:
        # Use unified learning pipeline (V2)
        concept_id = client.learn_concept_v2(
            content=request.content,
            options={
                "generate_embedding": True,
                "extract_associations": True,
                "strength": 1.0,
                "confidence": 1.0,
                "source": request.source,
            }
        )
        
        logger.info(f"✅ Learned concept {concept_id[:8]} via unified pipeline")
        
        return LearnResponse(
            concept_id=concept_id,
            message="Concept learned successfully via unified pipeline",
            concepts_created=1,
            associations_created=0,  # TODO: Get actual count from storage
        )
    except Exception as e:
        logger.error(f"Learning failed: {e}")
        raise HTTPException(
            status_code=500,
            detail=f"Failed to learn concept: {str(e)}"
        )


# Batch learning endpoint
@app.post(
    "/learn/batch",
    response_model=BatchLearnResponse,
    status_code=status.HTTP_201_CREATED,
    tags=["Learning"],
)
async def batch_learn(
    request: BatchLearnRequest,
    client=Depends(get_storage_client)
):
    """
    Learn multiple knowledge items in batch using optimized pipeline.
    
    Uses storage server's batch processing for better performance.
    Includes embedding generation and association extraction for all items.
    """
    try:
        # Extract contents from batch request
        contents = [item.content for item in request.items]
        
        # Use unified batch learning pipeline (V2)
        concept_ids = client.learn_batch_v2(
            contents=contents,
            options={
                "generate_embedding": True,
                "extract_associations": True,
                "strength": 1.0,
                "confidence": 1.0,
            }
        )
        
        logger.info(f"✅ Batch learned {len(concept_ids)} concepts via unified pipeline")
        
        return BatchLearnResponse(
            concept_ids=concept_ids,
            total_concepts=len(concept_ids),
            total_associations=0,  # TODO: Get actual count from storage
            message=f"Successfully learned {len(concept_ids)} concepts in batch",
        )
    except Exception as e:
        logger.error(f"Batch learning failed: {e}")
        raise HTTPException(
            status_code=500,
            detail=f"Failed to batch learn: {str(e)}"
        )


# System stats endpoint
@app.get("/stats", response_model=SystemStats, tags=["System"])
async def get_system_stats(client=Depends(get_storage_client)):
    """
    Get system statistics from storage server.
    """
    try:
        stats = client.stats()
        vectors_count = stats.get("vectors", 0)
        return SystemStats(
            total_concepts=stats.get("total_concepts", 0),
            total_associations=stats.get("total_edges", 0),
            total_embeddings=vectors_count,  # Use actual vectors count from storage
            embedding_provider="nomic-embed-text" if vectors_count > 0 else "none",
            embedding_dimension=768,  # nomic-embed-text produces 768-dim vectors
            average_strength=1.0,  # Default strength value
            memory_usage_mb=None,  # Optional field
        )
    except Exception as e:
        logger.error(f"Failed to get stats: {e}")
        raise HTTPException(
            status_code=500,
            detail=f"Failed to get stats: {str(e)}"
        )


# Vector search endpoint (proxy to storage server)
@app.post("/search/vector", response_model=list, tags=["Search"])
async def vector_search(
    query_embedding: list[float],
    k: int = 10,
    client=Depends(get_storage_client)
):
    """
    Perform vector similarity search via storage server.
    
    Note: Storage-client handles numpy conversion internally.
    """
    try:
        # Storage-client will handle numpy conversion
        import numpy as np
        results = client.vector_search(
            query_vector=np.array(query_embedding),
            k=k,
        )
        return [
            {"concept_id": cid, "similarity": sim}
            for cid, sim in results
        ]
    except Exception as e:
        logger.error(f"Vector search failed: {e}")
        raise HTTPException(
            status_code=500,
            detail=f"Vector search failed: {str(e)}"
        )


# ======================== SEMANTIC REASONING ENDPOINTS ========================

@app.post(
    "/sutra/semantic/path",
    response_model=SemanticPathResponse,
    tags=["Semantic Reasoning"],
)
async def semantic_path(
    request: SemanticPathRequest,
    client=Depends(get_storage_client)
):
    """
    Find semantic path between concepts with optional filter.
    
    Uses graph-based pathfinding with semantic pruning for efficient
    filtered path discovery.
    """
    try:
        # Convert filter to dict if provided
        filter_dict = None
        if request.filter:
            filter_dict = request.filter.dict(exclude_none=True)
        
        paths = client.find_path_semantic(
            start_id=request.start_query,
            end_id=request.end_query,
            max_depth=request.max_depth,
            semantic_filter=filter_dict,
        )
        
        return SemanticPathResponse(
            start_query=request.start_query,
            end_query=request.end_query,
            paths=paths,
            execution_time_ms=0.0,  # Client doesn't track this
            filter_applied=filter_dict is not None,
        )
    except Exception as e:
        logger.error(f"Semantic path failed: {e}")
        raise HTTPException(
            status_code=500,
            detail=f"Semantic path query failed: {str(e)}"
        )


@app.post(
    "/sutra/semantic/temporal",
    response_model=TemporalChainResponse,
    tags=["Semantic Reasoning"],
)
async def temporal_chain(
    request: TemporalChainRequest,
    client=Depends(get_storage_client)
):
    """
    Find temporal reasoning chain between concepts.
    
    Discovers temporal relationships and time-ordered event sequences.
    """
    try:
        chains = client.find_temporal_chain(
            start_id=request.start_query,
            end_id=request.end_query,
            max_depth=request.max_depth,
            after=request.after,
            before=request.before,
        )
        
        return TemporalChainResponse(
            start_query=request.start_query,
            end_query=request.end_query,
            chains=chains,
            temporal_constraints={"after": request.after, "before": request.before},
            execution_time_ms=0.0,
        )
    except Exception as e:
        logger.error(f"Temporal chain failed: {e}")
        raise HTTPException(
            status_code=500,
            detail=f"Temporal chain query failed: {str(e)}"
        )


@app.post(
    "/sutra/semantic/causal",
    response_model=CausalChainResponse,
    tags=["Semantic Reasoning"],
)
async def causal_chain(
    request: CausalChainRequest,
    client=Depends(get_storage_client)
):
    """
    Find causal reasoning chain between concepts.
    
    Identifies cause-and-effect relationships and reasoning chains.
    """
    try:
        chains = client.find_causal_chain(
            start_id=request.start_query,
            end_id=request.end_query,
            max_depth=request.max_depth,
        )
        
        return CausalChainResponse(
            start_query=request.start_query,
            end_query=request.end_query,
            chains=chains,
            execution_time_ms=0.0,
        )
    except Exception as e:
        logger.error(f"Causal chain failed: {e}")
        raise HTTPException(
            status_code=500,
            detail=f"Causal chain query failed: {str(e)}"
        )


@app.post(
    "/sutra/semantic/contradictions",
    response_model=ContradictionResponse,
    tags=["Semantic Reasoning"],
)
async def find_contradictions(
    request: ContradictionRequest,
    client=Depends(get_storage_client)
):
    """
    Detect contradictions in knowledge base.
    
    Identifies conflicting statements and logical inconsistencies.
    """
    try:
        contradictions = client.find_contradictions(
            concept_id=request.query,
            max_depth=request.max_depth,
        )
        
        # Format contradictions
        formatted_contradictions = [
            {
                "concept_id1": c[0],
                "concept_id2": c[1],
                "confidence": c[2],
            }
            for c in contradictions
        ]
        
        return ContradictionResponse(
            query=request.query,
            concept_id=request.query,  # Same for now
            contradictions=formatted_contradictions,
            count=len(formatted_contradictions),
            execution_time_ms=0.0,
        )
    except Exception as e:
        logger.error(f"Contradiction detection failed: {e}")
        raise HTTPException(
            status_code=500,
            detail=f"Contradiction detection failed: {str(e)}"
        )


@app.post(
    "/sutra/semantic/query",
    response_model=SemanticQueryResponse,
    tags=["Semantic Reasoning"],
)
async def semantic_query(
    request: SemanticQueryRequest,
    client=Depends(get_storage_client)
):
    """
    Query concepts by semantic filter.
    
    Search for concepts matching specific semantic criteria like type,
    domain, temporal constraints, etc.
    """
    try:
        filter_dict = request.filter.dict(exclude_none=True)
        
        results = client.query_by_semantic(
            semantic_filter=filter_dict,
            max_results=request.max_results,
        )
        
        return SemanticQueryResponse(
            filter=filter_dict,
            results=results,
            count=len(results),
            execution_time_ms=0.0,
        )
    except Exception as e:
        logger.error(f"Semantic query failed: {e}")
        raise HTTPException(
            status_code=500,
            detail=f"Semantic query failed: {str(e)}"
        )
