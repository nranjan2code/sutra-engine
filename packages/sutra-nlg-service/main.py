#!/usr/bin/env python3
"""
Lightweight NLG Service - Production Grade
Uses ML-Base Service for all text generation operations

This is a major refactor from the original heavyweight NLG service.
The new architecture:
- Uses ML-Base Service for all text generation
- Lightweight client (~50MB vs 1.39GB)
- Better horizontal scaling
- Production monitoring and caching
"""

import asyncio
import hashlib
import json
import logging
import os
import time
from typing import Dict, List, Optional, Any
from datetime import datetime, timedelta
from dataclasses import dataclass

from fastapi import FastAPI, HTTPException, Depends, BackgroundTasks
from fastapi.responses import StreamingResponse
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel, Field
import uvicorn

# Import ML-Base client
import sys
sys.path.append(os.path.join(os.path.dirname(__file__), '..', 'sutra-ml-base-service'))
from client import MLBaseClient
from monitoring import PerformanceMonitor, HealthCheckManager, handle_ml_errors

# Environment Configuration
ML_BASE_URL = os.getenv("ML_BASE_URL", "http://ml-base-service:8887")
ML_BASE_TIMEOUT = float(os.getenv("ML_BASE_TIMEOUT", "60.0"))
ML_BASE_MAX_RETRIES = int(os.getenv("ML_BASE_MAX_RETRIES", "3"))
SUTRA_EDITION = os.getenv("SUTRA_EDITION", "simple")
PORT = int(os.getenv("PORT", "8003"))
NLG_CACHE_SIZE = int(os.getenv("NLG_CACHE_SIZE", "500"))
NLG_CACHE_TTL = int(os.getenv("NLG_CACHE_TTL", "1800"))

# Configure structured logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
    handlers=[
        logging.StreamHandler(),
        logging.FileHandler('/var/log/sutra/nlg-service.log', mode='a')
    ]
)
logger = logging.getLogger(__name__)


@dataclass
class GenerationRequest:
    """Request for text generation"""
    prompt: str
    max_tokens: int = 512
    temperature: float = 0.7
    top_p: float = 0.9
    stop_sequences: Optional[List[str]] = None
    stream: bool = False


@dataclass
class GenerationResponse:
    """Response from text generation"""
    text: str
    tokens_used: int
    generation_time: float
    model_used: str
    cached: bool = False


class NLGCache:
    """Production-grade caching for NLG operations"""
    
    def __init__(self, max_size: int = 1000, ttl_seconds: int = 3600):
        self.cache: Dict[str, Dict] = {}
        self.access_times: Dict[str, datetime] = {}
        self.max_size = max_size
        self.ttl = timedelta(seconds=ttl_seconds)
        self.hits = 0
        self.misses = 0
        
    def _generate_key(self, request: GenerationRequest) -> str:
        """Generate cache key from request parameters"""
        key_data = {
            'prompt': request.prompt,
            'max_tokens': request.max_tokens,
            'temperature': request.temperature,
            'top_p': request.top_p,
            'stop_sequences': request.stop_sequences
        }
        return hashlib.sha256(json.dumps(key_data, sort_keys=True).encode()).hexdigest()
    
    def get(self, request: GenerationRequest) -> Optional[GenerationResponse]:
        """Get cached response if available"""
        key = self._generate_key(request)
        
        if key not in self.cache:
            self.misses += 1
            return None
            
        # Check TTL
        if datetime.now() - self.access_times[key] > self.ttl:
            del self.cache[key]
            del self.access_times[key]
            self.misses += 1
            return None
            
        self.hits += 1
        self.access_times[key] = datetime.now()
        cached_data = self.cache[key]
        
        return GenerationResponse(
            text=cached_data['text'],
            tokens_used=cached_data['tokens_used'],
            generation_time=cached_data['generation_time'],
            model_used=cached_data['model_used'],
            cached=True
        )
    
    def set(self, request: GenerationRequest, response: GenerationResponse):
        """Cache response"""
        if request.stream:  # Don't cache streaming responses
            return
            
        key = self._generate_key(request)
        
        # Evict oldest if at capacity
        if len(self.cache) >= self.max_size:
            oldest_key = min(self.access_times.keys(), key=lambda k: self.access_times[k])
            del self.cache[oldest_key]
            del self.access_times[oldest_key]
        
        self.cache[key] = {
            'text': response.text,
            'tokens_used': response.tokens_used,
            'generation_time': response.generation_time,
            'model_used': response.model_used
        }
        self.access_times[key] = datetime.now()
    
    def get_stats(self) -> Dict[str, Any]:
        """Get cache statistics"""
        total_requests = self.hits + self.misses
        hit_rate = self.hits / total_requests if total_requests > 0 else 0
        
        return {
            'hits': self.hits,
            'misses': self.misses,
            'hit_rate': hit_rate,
            'cache_size': len(self.cache),
            'max_size': self.max_size
        }


class NLGService:
    """Lightweight NLG Service using ML-Base backend"""
    
    def __init__(self):
        self.monitor = PerformanceMonitor()
        self.health_manager = HealthCheckManager()
        self.cache = NLGCache(
            max_size=NLG_CACHE_SIZE,
            ttl_seconds=NLG_CACHE_TTL
        )
        
        # ML-Base client
        self.ml_client = None
        
        # Edition limits
        self.max_concurrent = self._get_edition_limits()
        self.active_generations = 0
        
        logger.info(f"NLG Service initialized for {SUTRA_EDITION} edition")
        logger.info(f"Max concurrent generations: {self.max_concurrent}")
    
    def _get_edition_limits(self) -> int:
        """Get concurrent generation limits by edition"""
        limits = {
            'simple': 5,
            'community': 20,
            'enterprise': 100
        }
        return limits.get(SUTRA_EDITION, 5)
    
    async def initialize(self):
        """Initialize ML-Base client connection"""
        try:
            self.ml_client = MLBaseClient(
                base_url=ML_BASE_URL,
                timeout=ML_BASE_TIMEOUT
            )
            
            # Test connection
            health = await self.ml_client.health()
            if not health.get('status') == 'healthy':
                raise Exception("ML-Base service health check failed")
                
            logger.info("Successfully connected to ML-Base service")
            
        except Exception as e:
            logger.error(f"Failed to initialize ML-Base client: {e}")
            raise
    
    async def cleanup(self):
        """Cleanup resources"""
        if self.ml_client:
            await self.ml_client.close()
    
    @handle_ml_errors(logger)
    async def generate_text(self, request: GenerationRequest) -> GenerationResponse:
        """Generate text using ML-Base service"""
        
        # Check concurrent limits
        if self.active_generations >= self.max_concurrent:
            raise HTTPException(
                status_code=429, 
                detail=f"Too many concurrent generations. Limit: {self.max_concurrent}"
            )
        
        # Check cache first
        cached_response = self.cache.get(request)
        if cached_response:
            logger.debug("Returning cached generation response")
            self.monitor.record_metric('nlg_cache_hits', 1)
            return cached_response
        
        self.monitor.record_metric('nlg_cache_misses', 1)
        
        try:
            self.active_generations += 1
            start_time = time.time()
            
            # Call ML-Base service
            result = await self.ml_client.generate(
                prompt=request.prompt,
                max_tokens=request.max_tokens,
                temperature=request.temperature,
                top_p=request.top_p,
                stop_sequences=request.stop_sequences
            )
            
            generation_time = time.time() - start_time
            
            response = GenerationResponse(
                text=result['generated_text'],
                tokens_used=result['tokens_used'],
                generation_time=generation_time,
                model_used=result['model_name']
            )
            
            # Cache the result
            self.cache.set(request, response)
            
            self.monitor.record_metric('nlg_generations_success', 1)
            self.monitor.record_metric('nlg_generation_time', generation_time)
            self.monitor.record_metric('nlg_tokens_generated', response.tokens_used)
            
            logger.info(f"Generated {response.tokens_used} tokens in {generation_time:.3f}s using {response.model_used}")
            
            return response
            
        except Exception as e:
            self.monitor.record_metric('nlg_generations_error', 1)
            logger.error(f"Generation failed: {e}")
            raise HTTPException(status_code=500, detail=f"Generation failed: {str(e)}")
            
        finally:
            self.active_generations -= 1
    
    async def generate_stream(self, request: GenerationRequest):
        """Stream text generation"""
        if self.active_generations >= self.max_concurrent:
            yield json.dumps({'error': f'Too many concurrent generations. Limit: {self.max_concurrent}'}) + '\n'
            return
        
        try:
            self.active_generations += 1
            start_time = time.time()
            
            async for chunk in self.ml_client.generate_stream(
                prompt=request.prompt,
                max_tokens=request.max_tokens,
                temperature=request.temperature,
                top_p=request.top_p,
                stop_sequences=request.stop_sequences
            ):
                yield json.dumps(chunk) + '\n'
            
            generation_time = time.time() - start_time
            self.monitor.record_metric('nlg_streaming_success', 1)
            self.monitor.record_metric('nlg_streaming_time', generation_time)
            
        except Exception as e:
            self.monitor.record_metric('nlg_streaming_error', 1)
            logger.error(f"Streaming generation failed: {e}")
            yield json.dumps({'error': f'Streaming failed: {str(e)}'}) + '\n'
            
        finally:
            self.active_generations -= 1
    
    def get_stats(self) -> Dict[str, Any]:
        """Get service statistics"""
        cache_stats = self.cache.get_stats()
        
        return {
            'service': 'nlg-service-v2',
            'version': '2.0.0',
            'edition': SUTRA_EDITION,
            'active_generations': self.active_generations,
            'max_concurrent': self.max_concurrent,
            'cache_stats': cache_stats,
            'ml_base_url': ML_BASE_URL,
            'uptime_seconds': time.time() - start_time
        }


# Pydantic models for API
class GenerateRequest(BaseModel):
    prompt: str = Field(..., description="Text prompt for generation")
    max_tokens: int = Field(default=512, ge=1, le=2048, description="Maximum tokens to generate")
    temperature: float = Field(default=0.7, ge=0.0, le=2.0, description="Sampling temperature")
    top_p: float = Field(default=0.9, ge=0.0, le=1.0, description="Top-p sampling parameter")
    stop_sequences: Optional[List[str]] = Field(default=None, description="Stop sequences")
    stream: bool = Field(default=False, description="Enable streaming response")


class GenerateResponse(BaseModel):
    text: str
    tokens_used: int
    generation_time: float
    model_used: str
    cached: bool = False


# FastAPI app
app = FastAPI(
    title="Sutra NLG Service v2",
    description="Lightweight Natural Language Generation service using ML-Base backend",
    version="2.0.0"
)

# CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Global service instance
nlg_service = None
start_time = time.time()


@app.on_event("startup")
async def startup_event():
    """Initialize service on startup"""
    global nlg_service
    nlg_service = NLGService()
    await nlg_service.initialize()


@app.on_event("shutdown")
async def shutdown_event():
    """Cleanup on shutdown"""
    if nlg_service:
        await nlg_service.cleanup()


def get_nlg_service() -> NLGService:
    """Dependency injection for NLG service"""
    if nlg_service is None:
        raise HTTPException(status_code=500, detail="Service not initialized")
    return nlg_service


@app.post("/generate", response_model=GenerateResponse)
async def generate_text(
    request: GenerateRequest,
    service: NLGService = Depends(get_nlg_service)
) -> GenerateResponse:
    """Generate text from prompt"""
    
    gen_request = GenerationRequest(
        prompt=request.prompt,
        max_tokens=request.max_tokens,
        temperature=request.temperature,
        top_p=request.top_p,
        stop_sequences=request.stop_sequences,
        stream=request.stream
    )
    
    response = await service.generate_text(gen_request)
    
    return GenerateResponse(
        text=response.text,
        tokens_used=response.tokens_used,
        generation_time=response.generation_time,
        model_used=response.model_used,
        cached=response.cached
    )


@app.post("/generate/stream")
async def generate_text_stream(
    request: GenerateRequest,
    service: NLGService = Depends(get_nlg_service)
):
    """Stream text generation"""
    
    gen_request = GenerationRequest(
        prompt=request.prompt,
        max_tokens=request.max_tokens,
        temperature=request.temperature,
        top_p=request.top_p,
        stop_sequences=request.stop_sequences,
        stream=True
    )
    
    return StreamingResponse(
        service.generate_stream(gen_request),
        media_type="application/x-ndjson"
    )


@app.get("/health")
async def health_check(service: NLGService = Depends(get_nlg_service)):
    """Health check endpoint"""
    try:
        # Check ML-Base service health
        ml_health = await service.ml_client.health_check()
        
        return {
            'healthy': True,
            'service': 'nlg-service-v2',
            'version': '2.0.0',
            'ml_base_healthy': ml_health.get('healthy', False),
            'active_generations': service.active_generations,
            'timestamp': datetime.now().isoformat()
        }
    except Exception as e:
        return {
            'healthy': False,
            'error': str(e),
            'timestamp': datetime.now().isoformat()
        }


@app.get("/stats")
async def get_statistics(service: NLGService = Depends(get_nlg_service)):
    """Get service statistics"""
    return service.get_stats()


@app.get("/cache/stats")
async def get_cache_stats(service: NLGService = Depends(get_nlg_service)):
    """Get cache statistics"""
    return service.cache.get_stats()


@app.delete("/cache")
async def clear_cache(service: NLGService = Depends(get_nlg_service)):
    """Clear generation cache"""
    old_size = len(service.cache.cache)
    service.cache.cache.clear()
    service.cache.access_times.clear()
    service.cache.hits = 0
    service.cache.misses = 0
    
    return {
        'message': 'Cache cleared',
        'entries_removed': old_size
    }


if __name__ == "__main__":
    uvicorn.run(
        "main_v2:app",
        host="0.0.0.0",
        port=8003,
        log_level="info",
        reload=False
    )