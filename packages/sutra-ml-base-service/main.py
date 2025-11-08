#!/usr/bin/env python3
"""
Sutra ML-Base Service - Centralized ML Inference Platform

This service provides horizontally scalable ML inference capabilities for all Sutra services,
enabling efficient resource utilization and granular scaling.

Key Features:
- Centralized model management and loading
- Multi-model inference (embeddings + NLG)  
- Horizontal scaling and load balancing
- Edition-aware resource limits and features
- Advanced caching and batching
- Comprehensive metrics and monitoring
"""

import asyncio
import logging
import os
import time
import uuid
from concurrent.futures import ThreadPoolExecutor
from contextlib import asynccontextmanager
from typing import Any, Dict, List, Optional, Union

import uvicorn
from fastapi import FastAPI, HTTPException, BackgroundTasks
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel, Field

# Import ML foundation
from sutra_ml_base import (
    BaseMlService, ServiceConfig, HealthStatus,
    ModelLoader, LoaderConfig, ModelType,
    MetricsCollector, CacheManager, CacheConfig,
    EditionManager, setup_environment, setup_logging
)

# ML dependencies  
try:
    import torch
    import numpy as np
    from transformers import AutoTokenizer, AutoModel, AutoModelForCausalLM
    HAS_TORCH = True
except ImportError:
    HAS_TORCH = False
    raise RuntimeError("PyTorch and transformers required for ML-Base service")

# Setup environment
setup_environment()
logger = setup_logging("sutra-ml-base-service")

# ================================
# Request/Response Models
# ================================

class ModelInfo(BaseModel):
    """Information about a loaded model"""
    id: str = Field(..., description="Model ID")
    type: str = Field(..., description="Model type (embedding|nlg)")
    name: str = Field(..., description="HuggingFace model name")
    status: str = Field(..., description="Model status (loading|loaded|error)")
    memory_mb: int = Field(..., description="Memory usage in MB")
    instances: int = Field(..., description="Number of loaded instances")
    dimension: Optional[int] = Field(None, description="Embedding dimension (if embedding model)")
    max_tokens: Optional[int] = Field(None, description="Max generation tokens (if NLG model)")
    device: str = Field(..., description="Device (cpu|cuda)")

class EmbedRequest(BaseModel):
    """Embedding generation request"""
    model_id: str = Field(..., description="Model ID to use")
    texts: List[str] = Field(..., min_items=1, max_items=256, description="Texts to embed")
    normalize: bool = Field(True, description="L2 normalize embeddings")
    batch_size: Optional[int] = Field(None, description="Override batch size")

class EmbedResponse(BaseModel):
    """Embedding generation response"""
    embeddings: List[List[float]] = Field(..., description="Generated embeddings")
    dimension: int = Field(..., description="Embedding dimension")
    processing_time_ms: float = Field(..., description="Processing time")
    model_used: str = Field(..., description="Model ID used")
    cache_hit: bool = Field(False, description="Whether result was cached")
    batch_size: int = Field(..., description="Actual batch size used")

class GenerateRequest(BaseModel):
    """Text generation request"""
    model_id: str = Field(..., description="Model ID to use")
    prompt: str = Field(..., min_length=1, max_length=4096, description="Generation prompt")
    max_tokens: int = Field(150, ge=1, le=1024, description="Max tokens to generate")
    temperature: float = Field(0.3, ge=0.0, le=1.0, description="Sampling temperature")
    top_p: float = Field(0.9, ge=0.0, le=1.0, description="Nucleus sampling")
    stop_sequences: List[str] = Field(default_factory=list, description="Stop sequences")

class GenerateResponse(BaseModel):
    """Text generation response"""  
    text: str = Field(..., description="Generated text")
    tokens_generated: int = Field(..., description="Number of tokens generated")
    processing_time_ms: float = Field(..., description="Processing time")
    model_used: str = Field(..., description="Model ID used")
    cache_hit: bool = Field(False, description="Whether result was cached")

class BatchRequest(BaseModel):
    """Batch processing request"""
    requests: List[Dict[str, Any]] = Field(..., description="List of individual requests")
    request_type: str = Field(..., pattern="^(embed|generate)$", description="Request type")

class BatchResponse(BaseModel):
    """Batch processing response"""
    results: List[Dict[str, Any]] = Field(..., description="Individual results")
    total_processing_time_ms: float = Field(..., description="Total processing time")
    batch_size: int = Field(..., description="Batch size processed")

class ServiceHealth(BaseModel):
    """Service health status"""
    status: str = Field(..., description="Overall service status")
    models_loaded: int = Field(..., description="Number of loaded models")
    total_memory_gb: float = Field(..., description="Total memory usage")
    average_latency_ms: float = Field(..., description="Average request latency")
    requests_per_second: float = Field(..., description="Current RPS")

class ServiceMetrics(BaseModel):
    """Detailed service metrics"""
    uptime_seconds: int = Field(..., description="Service uptime")
    total_requests: int = Field(..., description="Total requests processed")
    requests_by_type: Dict[str, int] = Field(..., description="Requests by type")
    model_utilization: Dict[str, float] = Field(..., description="Model utilization rates")
    cache_stats: Dict[str, Any] = Field(..., description="Cache statistics")
    error_rate: float = Field(..., description="Error rate (0-1)")

# ================================
# ML Model Manager  
# ================================

class MLModelManager:
    """Manages loading, unloading, and inference for multiple ML models"""
    
    def __init__(self, edition_manager: EditionManager, cache_manager: CacheManager):
        self.edition_manager = edition_manager
        self.cache_manager = cache_manager
        self.models: Dict[str, Dict[str, Any]] = {}
        self.load_lock = asyncio.Lock()
        self.inference_executor = ThreadPoolExecutor(max_workers=4)
        
        logger.info(f"ML Model Manager initialized for {edition_manager.edition.value} edition")
    
    async def load_model(self, model_id: str, model_name: str, model_type: str) -> bool:
        """Load a model with given ID and configuration"""
        async with self.load_lock:
            if model_id in self.models:
                logger.warning(f"Model {model_id} already loaded")
                return True
            
            try:
                logger.info(f"Loading model {model_id}: {model_name} ({model_type})")
                start_time = time.time()
                
                # Create loader config
                config = LoaderConfig(
                    model_name=model_name,
                    model_type=ModelType.EMBEDDING if model_type == "embedding" else ModelType.GENERATIVE,
                    device="cpu",  # Start with CPU, can be enhanced for GPU
                    torch_dtype="float32",
                    max_memory_gb=self.edition_manager.get_model_size_limit(),
                    cache_dir="/tmp/.cache/huggingface",
                    verify_model=True
                )
                
                # Load using ML foundation
                model, tokenizer, loader = ModelLoader.load_model(config, self.edition_manager)
                
                # Store model info
                model_info = {
                    "id": model_id,
                    "name": model_name, 
                    "type": model_type,
                    "model": model,
                    "tokenizer": tokenizer,
                    "loader": loader,
                    "status": "loaded",
                    "load_time": time.time() - start_time,
                    "memory_mb": self._estimate_model_memory(model),
                    "device": str(next(model.parameters()).device),
                    "instances": 1
                }
                
                # Add type-specific info
                if model_type == "embedding":
                    model_info["dimension"] = self._detect_embedding_dimension(model, tokenizer)
                elif model_type == "nlg":
                    model_info["max_tokens"] = self.edition_manager.get_sequence_length_limit()
                    # Ensure pad token
                    if tokenizer.pad_token is None:
                        tokenizer.pad_token = tokenizer.eos_token
                
                self.models[model_id] = model_info
                
                logger.info(f"Model {model_id} loaded successfully in {model_info['load_time']:.2f}s")
                return True
                
            except Exception as e:
                logger.error(f"Failed to load model {model_id}: {e}")
                if model_id in self.models:
                    self.models[model_id]["status"] = "error"
                return False
    
    def _estimate_model_memory(self, model) -> int:
        """Estimate model memory usage in MB"""
        try:
            param_size = sum(p.numel() * p.element_size() for p in model.parameters())
            buffer_size = sum(b.numel() * b.element_size() for b in model.buffers())
            return int((param_size + buffer_size) / (1024 * 1024))
        except:
            return 0
    
    def _detect_embedding_dimension(self, model, tokenizer) -> int:
        """Detect embedding dimension"""
        try:
            test_input = tokenizer("test", return_tensors="pt", padding=True, truncation=True)
            with torch.no_grad():
                output = model(**test_input)
                if hasattr(output, 'last_hidden_state'):
                    return output.last_hidden_state.shape[-1]
                elif hasattr(output, 'pooler_output'):
                    return output.pooler_output.shape[-1]
                else:
                    return output[0].shape[-1]
        except:
            return 768  # Default
    
    async def embed_texts(self, model_id: str, texts: List[str], normalize: bool = True) -> List[List[float]]:
        """Generate embeddings using specified model"""
        if model_id not in self.models or self.models[model_id]["status"] != "loaded":
            raise HTTPException(status_code=404, detail=f"Model {model_id} not loaded")
        
        model_info = self.models[model_id]
        if model_info["type"] != "embedding":
            raise HTTPException(status_code=400, detail=f"Model {model_id} is not an embedding model")
        
        # Run inference in thread pool to avoid blocking
        return await asyncio.get_event_loop().run_in_executor(
            self.inference_executor,
            self._embed_texts_sync,
            model_info, texts, normalize
        )
    
    def _embed_texts_sync(self, model_info: Dict, texts: List[str], normalize: bool) -> List[List[float]]:
        """Synchronous embedding generation with Matryoshka dimension support"""
        model = model_info["model"]
        tokenizer = model_info["tokenizer"]
        
        # Get Matryoshka dimension from environment (Phase 0: Scaling optimization)
        matryoshka_dim = int(os.getenv("MATRYOSHKA_DIM", "768"))
        
        # Tokenize
        max_length = min(512, self.edition_manager.get_sequence_length_limit())
        inputs = tokenizer(
            texts, 
            padding=True, 
            truncation=True, 
            return_tensors="pt",
            max_length=max_length
        )
        
        # Generate embeddings
        with torch.no_grad():
            outputs = model(**inputs)
            
            # Mean pooling
            if hasattr(outputs, 'last_hidden_state'):
                token_embeddings = outputs.last_hidden_state
            else:
                token_embeddings = outputs[0]
            
            attention_mask = inputs['attention_mask']
            input_mask_expanded = attention_mask.unsqueeze(-1).expand(token_embeddings.size()).float()
            embeddings = torch.sum(token_embeddings * input_mask_expanded, 1) / torch.clamp(input_mask_expanded.sum(1), min=1e-9)
            
            # Apply Matryoshka truncation if enabled (Phase 0: 3x performance improvement)
            if matryoshka_dim < 768:
                # Layer normalization before truncation for quality preservation
                embeddings = torch.nn.functional.layer_norm(
                    embeddings, 
                    normalized_shape=(embeddings.shape[1],)
                )
                # Truncate to desired dimensions
                embeddings = embeddings[:, :matryoshka_dim]
                logger.debug(f"Applied Matryoshka truncation: 768 → {matryoshka_dim} dimensions")
            
            # Normalize if requested
            if normalize:
                embeddings = torch.nn.functional.normalize(embeddings, p=2, dim=1)
        
        return embeddings.cpu().numpy().tolist()
    
    async def generate_text(
        self, 
        model_id: str, 
        prompt: str, 
        max_tokens: int, 
        temperature: float,
        top_p: float,
        stop_sequences: List[str]
    ) -> tuple[str, int]:
        """Generate text using specified model"""
        if model_id not in self.models or self.models[model_id]["status"] != "loaded":
            raise HTTPException(status_code=404, detail=f"Model {model_id} not loaded")
        
        model_info = self.models[model_id]
        if model_info["type"] != "nlg":
            raise HTTPException(status_code=400, detail=f"Model {model_id} is not a generation model")
        
        # Run inference in thread pool
        return await asyncio.get_event_loop().run_in_executor(
            self.inference_executor,
            self._generate_text_sync,
            model_info, prompt, max_tokens, temperature, top_p, stop_sequences
        )
    
    def _generate_text_sync(
        self, 
        model_info: Dict, 
        prompt: str, 
        max_tokens: int,
        temperature: float,
        top_p: float,
        stop_sequences: List[str]
    ) -> tuple[str, int]:
        """Synchronous text generation"""
        model = model_info["model"]
        tokenizer = model_info["tokenizer"]
        
        # Tokenize input
        max_length = min(1024, self.edition_manager.get_sequence_length_limit())
        inputs = tokenizer(
            prompt,
            return_tensors="pt",
            truncation=True,
            max_length=max_length,
            padding=False
        )
        input_length = inputs['input_ids'].shape[1]
        
        # Generate
        with torch.no_grad():
            outputs = model.generate(
                **inputs,
                max_new_tokens=max_tokens,
                temperature=temperature if temperature > 0 else 1.0,
                do_sample=temperature > 0,
                top_p=top_p,
                top_k=50,
                pad_token_id=tokenizer.pad_token_id,
                eos_token_id=tokenizer.eos_token_id,
                repetition_penalty=1.1
            )
        
        # Decode new tokens only
        generated_text = tokenizer.decode(
            outputs[0][input_length:],
            skip_special_tokens=True,
            clean_up_tokenization_spaces=True
        )
        
        # Apply stop sequences
        for stop in stop_sequences:
            if stop in generated_text:
                generated_text = generated_text.split(stop)[0]
                break
        
        tokens_generated = outputs.shape[1] - input_length
        return generated_text.strip(), tokens_generated
    
    def get_models(self) -> List[ModelInfo]:
        """Get list of all models"""
        return [
            ModelInfo(
                id=info["id"],
                type=info["type"],
                name=info["name"],
                status=info["status"],
                memory_mb=info["memory_mb"],
                instances=info["instances"],
                dimension=info.get("dimension"),
                max_tokens=info.get("max_tokens"),
                device=info["device"]
            )
            for info in self.models.values()
        ]
    
    async def unload_model(self, model_id: str) -> bool:
        """Unload a model"""
        async with self.load_lock:
            if model_id not in self.models:
                return False
            
            try:
                # Clear model from memory
                model_info = self.models[model_id]
                del model_info["model"]
                del model_info["tokenizer"]
                del model_info["loader"]
                
                # Remove from registry
                del self.models[model_id]
                
                # Force garbage collection
                torch.cuda.empty_cache() if torch.cuda.is_available() else None
                
                logger.info(f"Model {model_id} unloaded successfully")
                return True
            except Exception as e:
                logger.error(f"Failed to unload model {model_id}: {e}")
                return False

# ================================
# ML-Base Service 
# ================================

class MLBaseService:
    """Centralized ML inference service"""
    
    def __init__(self, config: ServiceConfig):
        self.config = config
        self.edition_manager = EditionManager()
        
        # Initialize cache
        cache_config = CacheConfig(
            max_memory_mb=int(self.edition_manager.get_cache_size_gb() * 1024),
            max_items=50000,  # Large cache for ML results
            default_ttl_seconds=7200,  # 2 hours
            persistent=self.edition_manager.supports_advanced_caching()
        )
        self.cache = CacheManager(cache_config)
        
        # Initialize model manager
        self.model_manager = MLModelManager(self.edition_manager, self.cache)
        
        # Metrics
        self.metrics = {
            "start_time": time.time(),
            "total_requests": 0,
            "embed_requests": 0,
            "generate_requests": 0,
            "cache_hits": 0,
            "total_time_ms": 0.0,
            "error_count": 0
        }
        
        logger.info(f"ML-Base service initialized for {self.edition_manager.edition.value} edition")
    
    async def startup(self):
        """Service startup - load default models"""
        logger.info("Starting ML-Base service...")
        
        # Load default models based on edition
        default_models = self._get_default_models()
        
        for model_id, model_name, model_type in default_models:
            success = await self.model_manager.load_model(model_id, model_name, model_type)
            if not success:
                logger.error(f"Failed to load default model {model_id}")
        
        logger.info(f"ML-Base service started with {len(self.model_manager.models)} models")
    
    def _get_default_models(self) -> List[tuple]:
        """Get default models for the edition"""
        if self.edition_manager.edition.value == "simple":
            return [
                ("embedding-nomic-v1.5", "nomic-ai/nomic-embed-text-v1.5", "embedding"),
                ("nlg-dialogpt-small", "microsoft/DialoGPT-small", "nlg")
            ]
        elif self.edition_manager.edition.value == "community":
            return [
                ("embedding-nomic-v1.5", "nomic-ai/nomic-embed-text-v1.5", "embedding"),
                ("nlg-gemma-2b", "google/gemma-2-2b-it", "nlg")
            ]
        else:  # enterprise
            return [
                ("embedding-nomic-v1.5", "nomic-ai/nomic-embed-text-v1.5", "embedding"),
                ("nlg-dialogpt-large", "microsoft/DialoGPT-large", "nlg")
            ]
    
    async def embed(self, request: EmbedRequest) -> EmbedResponse:
        """Process embedding request"""
        start_time = time.time()
        
        # Check cache first
        cache_key = self.cache.cache_key("embed", request.model_id, str(request.texts), request.normalize)
        cached_result = self.cache.get(cache_key)
        
        if cached_result:
            self.metrics["cache_hits"] += 1
            cached_result["cache_hit"] = True
            cached_result["processing_time_ms"] = (time.time() - start_time) * 1000
            return EmbedResponse(**cached_result)
        
        # Generate embeddings
        embeddings = await self.model_manager.embed_texts(
            request.model_id, 
            request.texts, 
            request.normalize
        )
        
        processing_time = (time.time() - start_time) * 1000
        
        # Update metrics
        self.metrics["total_requests"] += 1
        self.metrics["embed_requests"] += 1
        self.metrics["total_time_ms"] += processing_time
        
        # Prepare response
        response_data = {
            "embeddings": embeddings,
            "dimension": len(embeddings[0]) if embeddings else 0,
            "processing_time_ms": processing_time,
            "model_used": request.model_id,
            "cache_hit": False,
            "batch_size": len(request.texts)
        }
        
        # Cache result
        cache_result = response_data.copy()
        cache_result.pop("processing_time_ms")
        self.cache.set(cache_key, cache_result, 7200)  # 2 hour TTL
        
        return EmbedResponse(**response_data)
    
    async def generate(self, request: GenerateRequest) -> GenerateResponse:
        """Process generation request"""
        start_time = time.time()
        
        # Check cache first
        cache_key = self.cache.cache_key(
            "generate", request.model_id, request.prompt, 
            request.temperature, request.max_tokens
        )
        cached_result = self.cache.get(cache_key)
        
        if cached_result:
            self.metrics["cache_hits"] += 1  
            cached_result["cache_hit"] = True
            cached_result["processing_time_ms"] = (time.time() - start_time) * 1000
            return GenerateResponse(**cached_result)
        
        # Generate text
        text, tokens_generated = await self.model_manager.generate_text(
            request.model_id,
            request.prompt,
            request.max_tokens,
            request.temperature,
            request.top_p,
            request.stop_sequences
        )
        
        processing_time = (time.time() - start_time) * 1000
        
        # Update metrics  
        self.metrics["total_requests"] += 1
        self.metrics["generate_requests"] += 1
        self.metrics["total_time_ms"] += processing_time
        
        # Prepare response
        response_data = {
            "text": text,
            "tokens_generated": tokens_generated,
            "processing_time_ms": processing_time,
            "model_used": request.model_id,
            "cache_hit": False
        }
        
        # Cache result (only if deterministic)
        if request.temperature == 0:
            cache_result = response_data.copy()
            cache_result.pop("processing_time_ms")
            self.cache.set(cache_key, cache_result, 3600)  # 1 hour TTL
        
        return GenerateResponse(**response_data)
    
    def get_health(self) -> ServiceHealth:
        """Get service health"""
        uptime = time.time() - self.metrics["start_time"]
        avg_latency = (
            self.metrics["total_time_ms"] / self.metrics["total_requests"]
            if self.metrics["total_requests"] > 0 else 0
        )
        rps = self.metrics["total_requests"] / uptime if uptime > 0 else 0
        
        return ServiceHealth(
            status="healthy",
            models_loaded=len(self.model_manager.models),
            total_memory_gb=sum(m["memory_mb"] for m in self.model_manager.models.values()) / 1024,
            average_latency_ms=avg_latency,
            requests_per_second=rps
        )
    
    def get_metrics(self) -> ServiceMetrics:
        """Get detailed metrics"""
        uptime = time.time() - self.metrics["start_time"]
        
        return ServiceMetrics(
            uptime_seconds=int(uptime),
            total_requests=self.metrics["total_requests"],
            requests_by_type={
                "embedding": self.metrics["embed_requests"],
                "generation": self.metrics["generate_requests"]
            },
            model_utilization={
                model_id: 0.5  # Placeholder - would track actual usage
                for model_id in self.model_manager.models.keys()
            },
            cache_stats=self.cache.get_stats(),
            error_rate=self.metrics["error_count"] / max(self.metrics["total_requests"], 1)
        )


# ================================
# FastAPI Application
# ================================

@asynccontextmanager
async def lifespan(app: FastAPI):
    """Application lifespan manager"""
    # Startup
    await ml_service.startup()
    yield
    # Shutdown
    logger.info("ML-Base service shutting down...")

# Create service instance
config = ServiceConfig(
    service_name="sutra-ml-base-service",
    service_version="2.0.0",
    port=int(os.getenv("PORT", "8887")),
    workers=1,
    enable_metrics=True,
    log_level=os.getenv("LOG_LEVEL", "INFO")
)

ml_service = MLBaseService(config)

# Create FastAPI app
app = FastAPI(
    title="Sutra ML-Base Service",
    description="Centralized ML inference platform for horizontal scaling",
    version="2.0.0",
    lifespan=lifespan
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

@app.get("/health", response_model=ServiceHealth)
async def health():
    """Service health check"""
    return ml_service.get_health()

@app.get("/metrics", response_model=ServiceMetrics)
async def metrics():
    """Service metrics"""
    return ml_service.get_metrics()

@app.get("/models", response_model=List[ModelInfo])
async def list_models():
    """List all loaded models"""
    return ml_service.model_manager.get_models()

@app.post("/models/{model_id}/load")
async def load_model(model_id: str, model_name: str, model_type: str):
    """Load a new model"""
    if model_type not in ["embedding", "nlg"]:
        raise HTTPException(status_code=400, detail="model_type must be 'embedding' or 'nlg'")
    
    success = await ml_service.model_manager.load_model(model_id, model_name, model_type)
    if not success:
        raise HTTPException(status_code=500, detail=f"Failed to load model {model_id}")
    
    return {"message": f"Model {model_id} loaded successfully"}

@app.delete("/models/{model_id}")
async def unload_model(model_id: str):
    """Unload a model"""
    success = await ml_service.model_manager.unload_model(model_id)
    if not success:
        raise HTTPException(status_code=404, detail=f"Model {model_id} not found")
    
    return {"message": f"Model {model_id} unloaded successfully"}

@app.post("/embed", response_model=EmbedResponse)
async def embed(request: EmbedRequest):
    """Generate embeddings"""
    try:
        return await ml_service.embed(request)
    except Exception as e:
        ml_service.metrics["error_count"] += 1
        logger.error(f"Embedding request failed: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/generate", response_model=GenerateResponse)
async def generate(request: GenerateRequest):
    """Generate text"""
    try:
        return await ml_service.generate(request)
    except Exception as e:
        ml_service.metrics["error_count"] += 1
        logger.error(f"Generation request failed: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/")
async def root():
    """Root endpoint"""
    return {
        "service": "Sutra ML-Base Service",
        "version": "2.0.0",
        "description": "Centralized ML inference platform",
        "models_loaded": len(ml_service.model_manager.models),
        "edition": ml_service.edition_manager.edition.value
    }


def main():
    """Main entry point"""
    port = int(os.getenv("PORT", "8887"))
    uvicorn.run(
        "main:app",
        host="0.0.0.0",
        port=port,
        log_level="info",
        access_log=True,
        reload=False
    )


if __name__ == "__main__":
    main()