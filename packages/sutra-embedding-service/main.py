#!/usr/bin/env python3
"""
Sutra Embedding Service - Lightweight Client (v2.0)
Uses centralized ML-Base service for efficient resource utilization and horizontal scaling.

This service acts as a lightweight proxy to the ML-Base service, providing:
- API compatibility with existing embedding endpoints
- Request validation and preprocessing  
- Edition-aware feature controls
- Comprehensive monitoring and error handling
- Local caching for performance optimization
"""

import asyncio
import logging
import os
import time
import uuid
from typing import List, Optional, Dict, Any

import uvicorn
from fastapi import FastAPI, HTTPException, BackgroundTasks
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel, Field
import aiohttp


# ================================
# Configuration
# ================================

# Service Configuration
SERVICE_NAME = "sutra-embedding-service"
SERVICE_VERSION = "2.0.0"
PORT = int(os.getenv("PORT", "8888"))

# ML-Base Service Configuration
ML_BASE_URL = os.getenv("ML_BASE_URL", "http://ml-base:8887")
ML_BASE_TIMEOUT = float(os.getenv("ML_BASE_TIMEOUT", "30.0"))

# Edition Configuration
SUTRA_EDITION = os.getenv("SUTRA_EDITION", "simple")

# Logging Configuration
LOG_LEVEL = os.getenv("LOG_LEVEL", "INFO")

# Setup logging
logging.basicConfig(
    level=getattr(logging, LOG_LEVEL.upper()),
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(SERVICE_NAME)

# ================================
# Edition Limits
# ================================

EDITION_LIMITS = {
    "simple": {
        "max_batch_size": 8,
        "max_text_length": 512,
        "cache_enabled": False,
        "rate_limit_per_minute": 100
    },
    "community": {
        "max_batch_size": 32,
        "max_text_length": 1024,
        "cache_enabled": True,
        "rate_limit_per_minute": 1000
    },
    "enterprise": {
        "max_batch_size": 128,
        "max_text_length": 2048,
        "cache_enabled": True,
        "rate_limit_per_minute": -1  # Unlimited
    }
}

# Get current edition limits
LIMITS = EDITION_LIMITS.get(SUTRA_EDITION, EDITION_LIMITS["simple"])

# ================================
# Request/Response Models
# ================================

class EmbeddingRequest(BaseModel):
    """Request model for embedding generation"""
    texts: List[str] = Field(
        ..., 
        min_items=1,
        description="List of texts to embed"
    )
    normalize: bool = Field(
        True, 
        description="Whether to L2 normalize embeddings"
    )
    model_id: Optional[str] = Field(
        None,
        description="Model ID to use (optional, uses default if not specified)"
    )
    cache_ttl_seconds: Optional[int] = Field(
        3600,
        ge=0,
        le=86400,
        description="Cache TTL in seconds (0 = no cache)"
    )

class EmbeddingResponse(BaseModel):
    """Response model for embedding generation"""
    embeddings: List[List[float]] = Field(..., description="Generated embeddings")
    dimension: int = Field(..., description="Embedding dimension")
    model: str = Field(..., description="Model used for generation")
    processing_time_ms: float = Field(..., description="Processing time in milliseconds")
    cached_count: int = Field(0, description="Number of cached results used")
    edition: str = Field(..., description="Sutra edition used")
    batch_size: int = Field(..., description="Actual batch size processed")

class HealthResponse(BaseModel):
    """Health check response"""
    status: str = Field(..., description="Service health status")
    ml_base_connected: bool = Field(..., description="ML-Base service connectivity")
    edition: str = Field(..., description="Current edition")
    uptime_seconds: float = Field(..., description="Service uptime")
    total_requests: int = Field(..., description="Total requests processed")

class InfoResponse(BaseModel):
    """Service information response"""
    service: str = Field(..., description="Service name")
    version: str = Field(..., description="Service version") 
    edition: str = Field(..., description="Edition")
    ml_base_url: str = Field(..., description="ML-Base service URL")
    limits: Dict[str, Any] = Field(..., description="Edition limits")
    available_models: List[str] = Field(..., description="Available models")

# ================================
# ML-Base Client
# ================================

class MLBaseClient:
    """Async client for ML-Base service"""
    
    def __init__(self, base_url: str, timeout: float = 30.0):
        self.base_url = base_url.rstrip('/')
        self.timeout = aiohttp.ClientTimeout(total=timeout)
        self.session: Optional[aiohttp.ClientSession] = None
    
    async def __aenter__(self):
        self.session = aiohttp.ClientSession(timeout=self.timeout)
        return self
    
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        if self.session:
            await self.session.close()
    
    async def health(self) -> bool:
        """Check ML-Base service health"""
        try:
            async with self.session.get(f"{self.base_url}/health") as resp:
                return resp.status == 200
        except Exception:
            return False
    
    async def list_models(self) -> List[Dict[str, Any]]:
        """List available models"""
        async with self.session.get(f"{self.base_url}/models") as resp:
            resp.raise_for_status()
            return await resp.json()
    
    async def embed(self, model_id: str, texts: List[str], normalize: bool = True) -> Dict[str, Any]:
        """Generate embeddings"""
        payload = {
            "model_id": model_id,
            "texts": texts,
            "normalize": normalize
        }
        
        async with self.session.post(f"{self.base_url}/embed", json=payload) as resp:
            resp.raise_for_status()
            return await resp.json()

# ================================
# Service Metrics
# ================================

class ServiceMetrics:
    """Track service metrics"""
    
    def __init__(self):
        self.start_time = time.time()
        self.request_count = 0
        self.error_count = 0
        self.total_processing_time = 0.0
        self.cache_hits = 0
    
    def record_request(self, processing_time: float, success: bool = True, cache_hit: bool = False):
        """Record request metrics"""
        self.request_count += 1
        self.total_processing_time += processing_time
        
        if not success:
            self.error_count += 1
        
        if cache_hit:
            self.cache_hits += 1
    
    def get_stats(self) -> Dict[str, Any]:
        """Get current statistics"""
        uptime = time.time() - self.start_time
        return {
            "uptime_seconds": uptime,
            "total_requests": self.request_count,
            "error_count": self.error_count,
            "error_rate": self.error_count / max(self.request_count, 1),
            "requests_per_second": self.request_count / max(uptime, 1),
            "average_response_time_ms": (
                self.total_processing_time / max(self.request_count, 1) * 1000
            ),
            "cache_hit_rate": self.cache_hits / max(self.request_count, 1)
        }

# Global metrics instance
metrics = ServiceMetrics()

# ================================
# Embedding Service
# ================================

class EmbeddingService:
    """Lightweight embedding service using ML-Base backend"""
    
    def __init__(self):
        self.ml_base_url = ML_BASE_URL
        self.default_model_id = self._get_default_model_id()
        self.cache = {}  # Simple in-memory cache
    
    def _get_default_model_id(self) -> str:
        """Get default model ID based on edition"""
        if SUTRA_EDITION == "simple":
            return "embedding-nomic-v1.5"  # Match ML-Base model ID
        elif SUTRA_EDITION == "community":
            return "embedding-nomic-v1.5"
        else:  # enterprise
            return "embedding-nomic-v1.5"
    
    def _validate_request(self, request: EmbeddingRequest) -> None:
        """Validate request against edition limits"""
        # Check batch size
        if len(request.texts) > LIMITS["max_batch_size"]:
            raise HTTPException(
                status_code=413,
                detail=f"Batch size {len(request.texts)} exceeds edition limit of {LIMITS['max_batch_size']}"
            )
        
        # Check text length
        max_length = LIMITS["max_text_length"]
        for i, text in enumerate(request.texts):
            if len(text) > max_length:
                raise HTTPException(
                    status_code=413,
                    detail=f"Text {i} length {len(text)} exceeds edition limit of {max_length}"
                )
    
    def _get_cache_key(self, model_id: str, texts: List[str], normalize: bool) -> str:
        """Generate cache key for request"""
        import hashlib
        content = f"{model_id}:{normalize}:{':'.join(texts)}"
        return hashlib.md5(content.encode()).hexdigest()
    
    async def embed(self, request: EmbeddingRequest) -> EmbeddingResponse:
        """Process embedding request"""
        start_time = time.time()
        request_id = str(uuid.uuid4())[:8]
        
        logger.info(f"[{request_id}] Embedding request: {len(request.texts)} texts")
        
        try:
            # Validate request
            self._validate_request(request)
            
            # Use provided model or default
            model_id = request.model_id or self.default_model_id
            
            # Check cache (if enabled)
            cache_key = None
            if LIMITS["cache_enabled"] and request.cache_ttl_seconds > 0:
                cache_key = self._get_cache_key(model_id, request.texts, request.normalize)
                if cache_key in self.cache:
                    cached_result = self.cache[cache_key]
                    if time.time() - cached_result["timestamp"] < request.cache_ttl_seconds:
                        # Cache hit
                        processing_time = time.time() - start_time
                        metrics.record_request(processing_time, success=True, cache_hit=True)
                        
                        cached_result["data"]["processing_time_ms"] = processing_time * 1000
                        cached_result["data"]["cached_count"] = len(request.texts)
                        
                        logger.info(f"[{request_id}] Cache hit: {processing_time*1000:.2f}ms")
                        return EmbeddingResponse(**cached_result["data"])
            
            # Call ML-Base service
            async with MLBaseClient(self.ml_base_url, ML_BASE_TIMEOUT) as client:
                result = await client.embed(model_id, request.texts, request.normalize)
            
            processing_time = time.time() - start_time
            
            # Prepare response
            response_data = {
                "embeddings": result["embeddings"],
                "dimension": result["dimension"],
                "model": result["model_used"],
                "processing_time_ms": processing_time * 1000,
                "cached_count": 0,
                "edition": SUTRA_EDITION,
                "batch_size": len(request.texts)
            }
            
            # Cache result (if enabled)
            if LIMITS["cache_enabled"] and cache_key and request.cache_ttl_seconds > 0:
                self.cache[cache_key] = {
                    "data": response_data.copy(),
                    "timestamp": time.time()
                }
                
                # Simple cache cleanup - remove oldest entries if cache too large
                if len(self.cache) > 1000:
                    oldest_key = min(self.cache.keys(), key=lambda k: self.cache[k]["timestamp"])
                    del self.cache[oldest_key]
            
            # Record metrics
            metrics.record_request(processing_time, success=True)
            
            logger.info(f"[{request_id}] Completed: {processing_time*1000:.2f}ms")
            return EmbeddingResponse(**response_data)
            
        except HTTPException:
            processing_time = time.time() - start_time
            metrics.record_request(processing_time, success=False)
            logger.error(f"[{request_id}] Client error: {processing_time*1000:.2f}ms")
            raise
            
        except Exception as e:
            processing_time = time.time() - start_time
            metrics.record_request(processing_time, success=False)
            logger.error(f"[{request_id}] Error: {e} ({processing_time*1000:.2f}ms)")
            raise HTTPException(status_code=503, detail=f"ML-Base service error: {str(e)}")
    
    async def get_available_models(self) -> List[str]:
        """Get list of available embedding models"""
        try:
            async with MLBaseClient(self.ml_base_url, 5.0) as client:
                models = await client.list_models()
                return [m["id"] for m in models if m["type"] == "embedding"]
        except Exception as e:
            logger.warning(f"Failed to get models from ML-Base: {e}")
            return [self.default_model_id]
    
    async def check_ml_base_health(self) -> bool:
        """Check ML-Base service connectivity"""
        try:
            async with MLBaseClient(self.ml_base_url, 5.0) as client:
                return await client.health()
        except Exception:
            return False

# ================================
# FastAPI Application
# ================================

# Create service instance
embedding_service = EmbeddingService()

# Create FastAPI app
app = FastAPI(
    title="Sutra Embedding Service",
    description="Lightweight embedding client using centralized ML-Base service",
    version=SERVICE_VERSION
)

# Add CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# ================================
# API Endpoints
# ================================

@app.get("/health", response_model=HealthResponse)
async def health():
    """Service health check"""
    ml_base_connected = await embedding_service.check_ml_base_health()
    stats = metrics.get_stats()
    
    return HealthResponse(
        status="healthy" if ml_base_connected else "degraded",
        ml_base_connected=ml_base_connected,
        edition=SUTRA_EDITION,
        uptime_seconds=stats["uptime_seconds"],
        total_requests=stats["total_requests"]
    )

@app.get("/info", response_model=InfoResponse)
async def info():
    """Service information"""
    available_models = await embedding_service.get_available_models()
    
    return InfoResponse(
        service=SERVICE_NAME,
        version=SERVICE_VERSION,
        edition=SUTRA_EDITION,
        ml_base_url=ML_BASE_URL,
        limits=LIMITS,
        available_models=available_models
    )

@app.post("/embed", response_model=EmbeddingResponse)
async def embed(request: EmbeddingRequest):
    """Generate embeddings for input texts"""
    return await embedding_service.embed(request)

@app.get("/metrics")
async def get_metrics():
    """Service metrics"""
    return metrics.get_stats()

@app.get("/cache/clear")
async def clear_cache():
    """Clear embedding cache (if caching enabled)"""
    if not LIMITS["cache_enabled"]:
        raise HTTPException(status_code=404, detail="Caching not available in this edition")
    
    cache_size = len(embedding_service.cache)
    embedding_service.cache.clear()
    
    return {
        "cache_cleared": True,
        "entries_removed": cache_size
    }

@app.get("/cache/stats")
async def cache_stats():
    """Cache statistics (if caching enabled)"""
    if not LIMITS["cache_enabled"]:
        raise HTTPException(status_code=404, detail="Caching not available in this edition")
    
    return {
        "cache_size": len(embedding_service.cache),
        "cache_enabled": True,
        "hit_rate": metrics.get_stats()["cache_hit_rate"]
    }

@app.get("/")
async def root():
    """Root endpoint"""
    return {
        "service": SERVICE_NAME,
        "version": SERVICE_VERSION,
        "description": "Lightweight embedding client using centralized ML-Base service",
        "edition": SUTRA_EDITION,
        "ml_base_url": ML_BASE_URL,
        "status": "running"
    }

def main():
    """Main entry point"""
    logger.info(f"Starting {SERVICE_NAME} v{SERVICE_VERSION}")
    logger.info(f"Edition: {SUTRA_EDITION}")
    logger.info(f"ML-Base URL: {ML_BASE_URL}")
    logger.info(f"Limits: {LIMITS}")
    
    uvicorn.run(
        app,
        host="0.0.0.0",
        port=PORT,
        log_level=LOG_LEVEL.lower(),
        access_log=True
    )

if __name__ == "__main__":
    main()