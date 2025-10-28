#!/usr/bin/env python3
"""
Sutra Embedding Service - Lightweight Client (v2.0)
Uses centralized ML-Base service for efficient resource utilization and horizontal scaling.

This service acts as a lightweight proxy to the ML-Base service, providing:
- API compatibility with existing embedding endpoints
- Request validation and preprocessing
- Caching and batching optimizations
- Edition-aware feature controls
- Comprehensive monitoring and error handling
"""

import asyncio
import logging
import os
import time
import uuid
from typing import List, Optional, Dict, Any

import uvicorn
from fastapi import FastAPI, HTTPException, Depends
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel, Field

# Import ML-Base client
import sys
sys.path.append('/app')
from client import MLBaseClient, check_ml_base_health

# Import monitoring utilities
from monitoring import (
    setup_production_logging, performance_monitor, 
    handle_ml_errors, request_context, HealthCheckManager
)

# Setup environment first
setup_environment()
logger = setup_logging("sutra-embedding-service")

# Request/Response Models (Enhanced with validation)
class EmbeddingRequest(BaseModel):
    """Request model for embedding generation"""
    texts: List[str] = Field(
        ..., 
        min_items=1, 
        max_items=256,  # Will be limited by edition
        description="List of texts to embed"
    )
    normalize: bool = Field(
        default=True, 
        description="Whether to L2 normalize embeddings"
    )
    cache_ttl_seconds: Optional[int] = Field(
        default=3600,
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
    cached_count: int = Field(default=0, description="Number of cached results used")
    edition: str = Field(..., description="Sutra edition used")
    batch_size: int = Field(..., description="Actual batch size processed")


class SutraEmbeddingService(BaseMlService):
    """Next-generation embedding service with ML foundation"""
    
    def __init__(self, config: ServiceConfig, edition_manager: Optional[EditionManager] = None):
        """Initialize embedding service
        
        Args:
            config: Service configuration
            edition_manager: Edition manager (auto-created if None)
        """
        super().__init__(config, edition_manager)
        
        # ML-specific components
        self.model = None
        self.tokenizer = None
        self.loader = None
        
        # Performance components
        cache_config = CacheConfig(
            max_memory_mb=int(self.edition_manager.get_cache_size_gb() * 1024),
            max_items=10000,
            default_ttl_seconds=3600,
            persistent=self.edition_manager.supports_advanced_caching()
        )
        self.cache = CacheManager(cache_config)
        
        # Model configuration
        self.model_name = self._get_model_for_edition()
        self.embedding_dimension = 768  # Will be updated after model load
        
        logger.info(f"Embedding service initialized for {self.edition_manager.edition.value} edition")
    
    def _get_model_for_edition(self) -> str:
        """Get appropriate model based on edition"""
        if self.edition_manager.edition.value == "simple":
            # Smaller, faster model for simple edition
            return "nomic-ai/nomic-embed-text-v1.5"
        elif self.edition_manager.edition.value == "community":
            # Better quality model for community
            return "nomic-ai/nomic-embed-text-v1.5"
        else:  # enterprise
            # Best model for enterprise
            return "nomic-ai/nomic-embed-text-v1.5"  # Could be larger model
    
    async def load_model(self) -> bool:
        """Load embedding model with edition-aware configuration"""
        try:
            logger.info(f"Loading embedding model: {self.model_name}")
            
            # Create loader configuration
            config = LoaderConfig(
                model_name=self.model_name,
                model_type=ModelType.EMBEDDING,
                device="auto",
                torch_dtype="float32",  # Stable for embeddings
                max_memory_gb=self.edition_manager.get_model_size_limit(),
                cache_dir="/tmp/.cache/huggingface",
                verify_model=True
            )
            
            # Load model using ML foundation
            self.model, self.tokenizer, self.loader = ModelLoader.load_model(config, self.edition_manager)
            
            # Update service state
            model_info = self.loader.get_model_info()
            self.embedding_dimension = self._detect_embedding_dimension()
            
            # Update model info with embedding-specific data
            model_info["embedding_dimension"] = self.embedding_dimension
            model_info["supports_batch"] = True
            model_info["max_sequence_length"] = self.edition_manager.get_sequence_length_limit()
            
            self.set_model_loaded(model_info)
            
            logger.info(f"Embedding model loaded successfully (dimension: {self.embedding_dimension})")
            return True
            
        except Exception as e:
            logger.error(f"Failed to load embedding model: {e}")
            self.set_health_status(HealthStatus.UNHEALTHY)
            return False
    
    def _detect_embedding_dimension(self) -> int:
        """Detect embedding dimension by running test input"""
        try:
            test_text = "Test input for dimension detection"
            embeddings = self._generate_embeddings_internal([test_text], normalize=True)
            return len(embeddings[0])
        except Exception:
            logger.warning("Could not detect embedding dimension, using default 768")
            return 768
    
    async def process_request(self, request: EmbeddingRequest) -> EmbeddingResponse:
        """Process embedding generation request"""
        start_time = time.time()
        
        # Validate batch size against edition limits
        max_batch_size = self.edition_manager.get_batch_size_limit()
        if len(request.texts) > max_batch_size:
            raise HTTPException(
                status_code=413,
                detail=f"Batch size {len(request.texts)} exceeds edition limit of {max_batch_size}"
            )
        
        # Check cache for existing embeddings
        cached_embeddings = []
        texts_to_compute = []
        cache_keys = []
        
        for text in request.texts:
            if request.cache_ttl_seconds > 0:
                cache_key = self.cache.cache_key("embedding", text, self.model_name, request.normalize)
                cached_result = self.cache.get(cache_key)
                
                if cached_result is not None:
                    cached_embeddings.append((len(texts_to_compute), cached_result))
                    cache_keys.append(None)  # Placeholder
                else:
                    texts_to_compute.append(text)
                    cache_keys.append(cache_key)
            else:
                texts_to_compute.append(text)
                cache_keys.append(None)
        
        # Generate embeddings for non-cached texts
        new_embeddings = []
        if texts_to_compute:
            new_embeddings = self._generate_embeddings_internal(texts_to_compute, request.normalize)
            
            # Cache new embeddings
            if request.cache_ttl_seconds > 0:
                cache_idx = 0
                for i, cache_key in enumerate(cache_keys):
                    if cache_key is not None:
                        self.cache.set(cache_key, new_embeddings[cache_idx], request.cache_ttl_seconds)
                        cache_idx += 1
        
        # Combine cached and new embeddings in correct order
        final_embeddings = new_embeddings.copy()
        for cache_idx, cached_embedding in cached_embeddings:
            final_embeddings.insert(cache_idx, cached_embedding)
        
        processing_time = (time.time() - start_time) * 1000
        
        return EmbeddingResponse(
            embeddings=final_embeddings,
            dimension=self.embedding_dimension,
            model=self.model_name,
            processing_time_ms=processing_time,
            cached_count=len(cached_embeddings),
            edition=self.edition_manager.edition.value,
            batch_size=len(request.texts)
        )
    
    def _generate_embeddings_internal(self, texts: List[str], normalize: bool) -> List[List[float]]:
        """Internal method to generate embeddings"""
        if not self.model or not self.tokenizer:
            raise HTTPException(status_code=503, detail="Model not loaded")
        
        try:
            # Tokenize with sequence length limits
            max_length = min(512, self.edition_manager.get_sequence_length_limit())
            
            encoded_input = self.tokenizer(
                texts,
                padding=True,
                truncation=True,
                return_tensors='pt',
                max_length=max_length
            )
            
            # Generate embeddings
            with torch.no_grad():
                model_output = self.model(**encoded_input)
                
                # Mean pooling
                token_embeddings = model_output[0]
                input_mask_expanded = encoded_input['attention_mask'].unsqueeze(-1).expand(token_embeddings.size()).float()
                embeddings = torch.sum(token_embeddings * input_mask_expanded, 1) / torch.clamp(input_mask_expanded.sum(1), min=1e-9)
                
                # Normalize if requested
                if normalize:
                    embeddings = torch.nn.functional.normalize(embeddings, p=2, dim=1)
            
            # Convert to list
            return embeddings.cpu().numpy().tolist()
            
        except Exception as e:
            logger.error(f"Embedding generation failed: {e}")
            raise HTTPException(status_code=500, detail=f"Embedding generation failed: {str(e)}")
    
    def get_service_info(self) -> Dict[str, Any]:
        """Get embedding service information"""
        return {
            "description": "High-performance embedding service with edition-aware scaling",
            "supported_models": [
                "nomic-ai/nomic-embed-text-v1.5",
                "sentence-transformers/all-MiniLM-L6-v2",
                "custom models (community+ editions)"
            ],
            "features": {
                "caching": self.edition_manager.supports_advanced_caching(),
                "custom_models": self.edition_manager.supports_custom_models(),
                "batch_processing": True,
                "sequence_truncation": True
            },
            "limits": {
                "max_batch_size": self.edition_manager.get_batch_size_limit(),
                "max_sequence_length": self.edition_manager.get_sequence_length_limit(),
                "cache_size_gb": self.edition_manager.get_cache_size_gb()
            },
            "model": {
                "name": self.model_name,
                "dimension": self.embedding_dimension,
                "parameters": self.loader.get_model_info().get("parameters", 0) if self.loader else 0
            }
        }
    
    def _setup_service_routes(self):
        """Setup embedding-specific routes"""
        
        @self.app.post("/embed", response_model=EmbeddingResponse)
        async def generate_embeddings(request: EmbeddingRequest):
            """Generate embeddings for input texts"""
            if not self._model_loaded:
                raise HTTPException(status_code=503, detail="Model not loaded")
            
            return await self.process_request(request)
        
        @self.app.get("/cache/stats")
        async def get_cache_stats():
            """Get cache statistics (if caching enabled)"""
            if not self.edition_manager.supports_advanced_caching():
                raise HTTPException(status_code=404, detail="Advanced caching not available in this edition")
            
            return self.cache.get_stats()
        
        @self.app.delete("/cache")
        async def clear_cache():
            """Clear embedding cache (if caching enabled)"""
            if not self.edition_manager.supports_advanced_caching():
                raise HTTPException(status_code=404, detail="Advanced caching not available in this edition")
            
            success = self.cache.clear()
            return {"cache_cleared": success}


def main():
    """Main entry point for embedding service"""
    # Service configuration
    config = ServiceConfig(
        service_name="sutra-embedding-service",
        service_version="2.0.0",
        port=int(os.getenv("PORT", "8888")),
        workers=1,  # Single worker for model services
        enable_metrics=True,
        log_level=os.getenv("LOG_LEVEL", "INFO")
    )
    
    # Create and run service
    service = SutraEmbeddingService(config)
    
    try:
        logger.info("Starting Sutra Embedding Service...")
        service.run()
    except KeyboardInterrupt:
        logger.info("Service stopped by user")
    except Exception as e:
        logger.error(f"Service failed: {e}")
        raise


if __name__ == "__main__":
    main()