"""
Minimal FastAPI application - Thin gRPC Proxy.

This version uses only storage-client for gRPC communication.
NO local reasoning engine or heavy ML dependencies.
"""

import logging
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
    ConceptDetail,
    ErrorResponse,
    HealthResponse,
    LearnRequest,
    LearnResponse,
    SearchResult,
    SystemStats,
)

# Configure logging
logging.basicConfig(level=settings.log_level, format=settings.log_format)
logger = logging.getLogger(__name__)


@asynccontextmanager
async def lifespan(app: FastAPI):
    """Lifespan context manager for startup and shutdown events."""
    # Startup
    logger.info(f"Starting {settings.api_title} v{settings.api_version}")
    logger.info(f"Storage server: {settings.storage_server}")
    
    # Initialize storage client
    init_dependencies(app)
    
    yield
    
    # Shutdown
    shutdown_dependencies(app)
    logger.info("Shutting down API service")


# Create FastAPI app
app = FastAPI(
    title=settings.api_title,
    version=settings.api_version,
    description="Lightweight REST-to-gRPC proxy for Sutra AI",
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
    # Get stats from storage server via gRPC
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
