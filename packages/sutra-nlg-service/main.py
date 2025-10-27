#!/usr/bin/env python3
"""
Sutra NLG Service - Next Generation
Built on sutra-ml-base foundation with edition-aware grounded text generation
"""

import asyncio
import logging
import time
import os
from typing import List, Optional, Dict, Any

# Import the new ML foundation
from sutra_ml_base import (
    BaseMlService, ServiceConfig, HealthStatus,
    ModelLoader, LoaderConfig, ModelType,
    MetricsCollector, CacheManager, CacheConfig,
    EditionManager, setup_environment, setup_logging
)

try:
    import torch
    from fastapi import HTTPException
    from pydantic import BaseModel, Field
    HAS_TORCH = True
except ImportError:
    HAS_TORCH = False
    raise RuntimeError("PyTorch and FastAPI required for NLG service")

# Setup environment first
setup_environment()
logger = setup_logging("sutra-nlg-service")

# Request/Response Models (Enhanced with validation)
class GenerateRequest(BaseModel):
    """Request model for text generation"""
    prompt: str = Field(
        ..., 
        min_length=1,
        max_length=2048,  # Will be limited by edition
        description="Constrained prompt with grounding facts"
    )
    max_tokens: int = Field(
        default=150, 
        ge=1, 
        le=512,  # Will be limited by edition
        description="Maximum tokens to generate"
    )
    temperature: float = Field(
        default=0.3, 
        ge=0.0, 
        le=1.0, 
        description="Sampling temperature (0.0 = deterministic)"
    )
    top_p: float = Field(
        default=0.9,
        ge=0.0,
        le=1.0,
        description="Nucleus sampling parameter"
    )
    stop_sequences: List[str] = Field(
        default_factory=list,
        max_items=10,
        description="Stop generation at these sequences"
    )
    grounding_mode: str = Field(
        default="strict",
        pattern="^(strict|balanced|creative)$",
        description="Grounding enforcement level"
    )

class GenerateResponse(BaseModel):
    """Response model for text generation"""
    text: str = Field(..., description="Generated text")
    model: str = Field(..., description="Model used for generation")
    processing_time_ms: float = Field(..., description="Processing time in milliseconds")
    tokens_generated: int = Field(..., description="Number of tokens generated")
    edition: str = Field(..., description="Sutra edition used")
    grounding_applied: bool = Field(..., description="Whether grounding constraints were applied")
    cache_used: bool = Field(default=False, description="Whether cached result was used")


class SutraNlgService(BaseMlService):
    """Next-generation NLG service with ML foundation"""
    
    def __init__(self, config: ServiceConfig, edition_manager: Optional[EditionManager] = None):
        """Initialize NLG service
        
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
            max_memory_mb=int(self.edition_manager.get_cache_size_gb() * 512),  # Smaller cache for NLG
            max_items=5000,
            default_ttl_seconds=1800,  # 30 minutes for generated content
            persistent=self.edition_manager.supports_advanced_caching()
        )
        self.cache = CacheManager(cache_config)
        
        # Model configuration
        self.model_name = self._get_model_for_edition()
        
        logger.info(f"NLG service initialized for {self.edition_manager.edition.value} edition")
    
    def _get_model_for_edition(self) -> str:
        """Get appropriate model based on edition"""
        if self.edition_manager.edition.value == "simple":
            # Lightweight model for simple edition
            return "microsoft/DialoGPT-small"
        elif self.edition_manager.edition.value == "community":
            # Better model for community edition
            return "google/gemma-2-2b-it"
        else:  # enterprise
            # Advanced model for enterprise
            return "microsoft/DialoGPT-large"
    
    async def load_model(self) -> bool:
        """Load NLG model with edition-aware configuration"""
        try:
            logger.info(f"Loading NLG model: {self.model_name}")
            
            # Create loader configuration
            config = LoaderConfig(
                model_name=self.model_name,
                model_type=ModelType.GENERATIVE,
                device="cpu",  # CPU-optimized for all editions
                torch_dtype="float32",
                max_memory_gb=self.edition_manager.get_model_size_limit(),
                cache_dir="/tmp/.cache/huggingface",
                verify_model=True
            )
            
            # Load model using ML foundation
            self.model, self.tokenizer, self.loader = ModelLoader.load_model(config, self.edition_manager)
            
            # Configure tokenizer for generation
            if self.tokenizer.pad_token is None:
                self.tokenizer.pad_token = self.tokenizer.eos_token
            
            # Update service state
            model_info = self.loader.get_model_info()
            model_info["supports_generation"] = True
            model_info["max_sequence_length"] = self.edition_manager.get_sequence_length_limit()
            model_info["max_new_tokens"] = self._get_max_tokens_for_edition()
            
            self.set_model_loaded(model_info)
            
            logger.info(f"NLG model loaded successfully")
            return True
            
        except Exception as e:
            logger.error(f"Failed to load NLG model: {e}")
            self.set_health_status(HealthStatus.UNHEALTHY)
            return False
    
    def _get_max_tokens_for_edition(self) -> int:
        """Get maximum token generation limit for edition"""
        if self.edition_manager.edition.value == "simple":
            return 128
        elif self.edition_manager.edition.value == "community":
            return 256
        else:  # enterprise
            return 512
    
    async def process_request(self, request: GenerateRequest) -> GenerateResponse:
        """Process text generation request"""
        start_time = time.time()
        
        # Validate limits against edition
        max_tokens = min(request.max_tokens, self._get_max_tokens_for_edition())
        max_prompt = self.edition_manager.get_sequence_length_limit()
        
        if len(request.prompt) > max_prompt:
            raise HTTPException(
                status_code=413,
                detail=f"Prompt length exceeds edition limit of {max_prompt} characters"
            )
        
        # Check cache for existing generation
        cache_used = False
        if self.edition_manager.supports_advanced_caching():
            cache_key = self.cache.cache_key(
                "generation", request.prompt, request.temperature, 
                max_tokens, request.grounding_mode
            )
            cached_result = self.cache.get(cache_key)
            
            if cached_result is not None:
                logger.info("Using cached generation result")
                cached_result["cache_used"] = True
                cached_result["processing_time_ms"] = (time.time() - start_time) * 1000
                return GenerateResponse(**cached_result)
        
        # Apply grounding constraints based on mode
        processed_prompt = self._apply_grounding(request.prompt, request.grounding_mode)
        
        # Generate text
        generated_text, tokens_generated = self._generate_text_internal(
            processed_prompt,
            max_tokens,
            request.temperature,
            request.top_p,
            request.stop_sequences
        )
        
        processing_time = (time.time() - start_time) * 1000
        
        # Prepare response
        response_data = {
            "text": generated_text,
            "model": self.model_name,
            "processing_time_ms": processing_time,
            "tokens_generated": tokens_generated,
            "edition": self.edition_manager.edition.value,
            "grounding_applied": request.grounding_mode == "strict",
            "cache_used": cache_used
        }
        
        # Cache the result
        if self.edition_manager.supports_advanced_caching():
            cache_result = response_data.copy()
            cache_result.pop("processing_time_ms", None)  # Don't cache timing
            cache_result.pop("cache_used", None)
            self.cache.set(cache_key, cache_result, 1800)  # 30 min TTL
        
        return GenerateResponse(**response_data)
    
    def _apply_grounding(self, prompt: str, mode: str) -> str:
        """Apply grounding constraints to prompt"""
        if mode == "strict":
            # Strict grounding: Add explicit constraints
            grounding_prefix = (
                "INSTRUCTIONS: Answer based ONLY on the facts provided. "
                "If the facts don't contain the answer, say 'I don't know based on the provided information.'\n\n"
            )
            return grounding_prefix + prompt
        elif mode == "balanced":
            # Balanced: Light constraints
            grounding_prefix = "Please answer based on the information provided:\n\n"
            return grounding_prefix + prompt
        else:  # creative
            # Creative: Minimal constraints
            return prompt
    
    def _generate_text_internal(
        self, 
        prompt: str, 
        max_tokens: int,
        temperature: float,
        top_p: float,
        stop_sequences: List[str]
    ) -> tuple[str, int]:
        """Internal method to generate text"""
        if not self.model or not self.tokenizer:
            raise HTTPException(status_code=503, detail="Model not loaded")
        
        try:
            # Tokenize input
            max_length = min(1024, self.edition_manager.get_sequence_length_limit())
            
            inputs = self.tokenizer(
                prompt,
                return_tensors="pt",
                truncation=True,
                max_length=max_length,
                padding=False
            )
            input_length = inputs['input_ids'].shape[1]
            
            # Generate with controlled parameters
            with torch.no_grad():
                outputs = self.model.generate(
                    **inputs,
                    max_new_tokens=max_tokens,
                    temperature=temperature if temperature > 0 else 1.0,
                    do_sample=temperature > 0,
                    top_p=top_p,
                    top_k=50,
                    pad_token_id=self.tokenizer.pad_token_id,
                    eos_token_id=self.tokenizer.eos_token_id,
                    repetition_penalty=1.1,
                    no_repeat_ngram_size=3,  # Reduce repetition
                )
            
            # Decode only new tokens
            generated_text = self.tokenizer.decode(
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
            
        except Exception as e:
            logger.error(f"Text generation failed: {e}")
            raise HTTPException(status_code=500, detail=f"Generation failed: {str(e)}")
    
    def get_service_info(self) -> Dict[str, Any]:
        """Get NLG service information"""
        return {
            "description": "High-quality grounded text generation with edition-aware scaling",
            "supported_models": [
                "microsoft/DialoGPT-small",  # Simple
                "google/gemma-2-2b-it",     # Community  
                "microsoft/DialoGPT-large", # Enterprise
                "custom models (enterprise edition)"
            ],
            "features": {
                "grounded_generation": True,
                "caching": self.edition_manager.supports_advanced_caching(),
                "custom_models": self.edition_manager.supports_custom_models(),
                "prompt_validation": True,
                "stop_sequences": True
            },
            "limits": {
                "max_prompt_length": self.edition_manager.get_sequence_length_limit(),
                "max_generation_tokens": self._get_max_tokens_for_edition(),
                "cache_size_gb": self.edition_manager.get_cache_size_gb()
            },
            "model": {
                "name": self.model_name,
                "parameters": self.loader.get_model_info().get("parameters", 0) if self.loader else 0,
                "supports_streaming": False  # Not implemented yet
            }
        }
    
    def _setup_service_routes(self):
        """Setup NLG-specific routes"""
        
        @self.app.post("/generate", response_model=GenerateResponse)
        async def generate_text(request: GenerateRequest):
            """Generate grounded natural language text"""
            if not self._model_loaded:
                raise HTTPException(status_code=503, detail="Model not loaded")
            
            return await self.process_request(request)
        
        @self.app.post("/validate_prompt")
        async def validate_prompt(prompt: str):
            """Validate prompt for grounding constraints"""
            validation_result = {
                "valid": True,
                "issues": [],
                "suggestions": []
            }
            
            if len(prompt) > self.edition_manager.get_sequence_length_limit():
                validation_result["valid"] = False
                validation_result["issues"].append("Prompt too long for edition")
            
            if not prompt.strip():
                validation_result["valid"] = False
                validation_result["issues"].append("Empty prompt")
            
            # Check for grounding markers
            if "FACTS:" not in prompt and "CONTEXT:" not in prompt:
                validation_result["suggestions"].append("Consider adding FACTS: or CONTEXT: section for better grounding")
            
            return validation_result
        
        @self.app.get("/cache/stats")
        async def get_cache_stats():
            """Get generation cache statistics (if caching enabled)"""
            if not self.edition_manager.supports_advanced_caching():
                raise HTTPException(status_code=404, detail="Advanced caching not available in this edition")
            
            return self.cache.get_stats()


def main():
    """Main entry point for NLG service"""
    # Service configuration
    config = ServiceConfig(
        service_name="sutra-nlg-service",
        service_version="2.0.0",
        port=int(os.getenv("PORT", "8889")),
        workers=1,  # Single worker for model services
        enable_metrics=True,
        log_level=os.getenv("LOG_LEVEL", "INFO")
    )
    
    # Create and run service
    service = SutraNlgService(config)
    
    try:
        logger.info("Starting Sutra NLG Service...")
        service.run()
    except KeyboardInterrupt:
        logger.info("Service stopped by user")
    except Exception as e:
        logger.error(f"Service failed: {e}")
        raise


if __name__ == "__main__":
    main()

async def load_model():
    """Load the NLG model with production-grade error handling"""
    global model, tokenizer
    
    logger.info(f"[{instance_id}] Loading model: {model_name}")
    start_time = time.time()
    
    # Get HuggingFace token for gated models
    hf_token = os.getenv("HF_TOKEN")
    if hf_token:
        logger.info(f"[{instance_id}] HuggingFace token found, will use for authentication")
    else:
        logger.warning(f"[{instance_id}] No HuggingFace token found - gated models will fail")
    
    # Check for cached local model first
    local_model_path = f"/app/models/{model_name.split('/')[-1]}"
    use_local = os.path.isdir(local_model_path) and os.path.isfile(
        os.path.join(local_model_path, "config.json")
    )
    
    if use_local:
        logger.info(f"[{instance_id}] Using cached local model: {local_model_path}")
        model_path = local_model_path
    else:
        logger.info(f"[{instance_id}] Local model not found, will download from HuggingFace")
        model_path = model_name
    
    try:
        # Load tokenizer
        tokenizer = AutoTokenizer.from_pretrained(
            model_path,
            trust_remote_code=True,
            token=hf_token,  # Pass HF token for gated models
            cache_dir='/tmp/.cache/huggingface' if not use_local else None
        )
        
        # Ensure pad token is set
        if tokenizer.pad_token is None:
            tokenizer.pad_token = tokenizer.eos_token
        
        # Load model (CPU-optimized)
        model = AutoModelForCausalLM.from_pretrained(
            model_path,
            trust_remote_code=True,
            token=hf_token,  # Pass HF token for gated models
            cache_dir='/tmp/.cache/huggingface' if not use_local else None,
            torch_dtype=torch.float32,
            device_map="cpu",
            low_cpu_mem_usage=True
        )
        
        # Set to eval mode
        model.eval()
        
        load_time = time.time() - start_time
        logger.info(f"[{instance_id}] Model loaded successfully in {load_time:.2f}s")
        logger.info(f"[{instance_id}] Model source: {'cached local' if use_local else 'downloaded'}")
        logger.info(f"[{instance_id}] Device: {next(model.parameters()).device}")
        logger.info(f"[{instance_id}] Tokenizer vocab size: {len(tokenizer)}")
        
        return True
        
    except Exception as e:
        logger.error(f"[{instance_id}] Failed to load model: {e}", exc_info=True)
        return False

def generate_text(
    prompt: str, 
    max_tokens: int, 
    temperature: float, 
    stop_sequences: List[str]
) -> tuple[str, int, float]:
    """
    Generate text from prompt with grounding constraints
    
    Returns:
        tuple: (generated_text, tokens_generated, processing_time_ms)
    """
    if model is None or tokenizer is None:
        raise HTTPException(status_code=503, detail="Model not loaded")
    
    start_time = time.time()
    
    try:
        # Tokenize
        inputs = tokenizer(
            prompt, 
            return_tensors="pt", 
            truncation=True, 
            max_length=1024,
            padding=False
        )
        input_length = inputs['input_ids'].shape[1]
        
        # Generate with controlled parameters
        with torch.no_grad():
            outputs = model.generate(
                **inputs,
                max_new_tokens=max_tokens,
                temperature=temperature if temperature > 0 else 1.0,
                do_sample=temperature > 0,
                top_p=0.9,
                top_k=50,
                pad_token_id=tokenizer.pad_token_id,
                eos_token_id=tokenizer.eos_token_id,
                repetition_penalty=1.1,  # Reduce repetition
            )
        
        # Decode only new tokens
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
        processing_time = (time.time() - start_time) * 1000
        
        logger.info(
            f"[{instance_id}] Generated {tokens_generated} tokens in {processing_time:.2f}ms "
            f"(temp={temperature})"
        )
        
        return generated_text.strip(), tokens_generated, processing_time
        
    except Exception as e:
        logger.error(f"[{instance_id}] Generation failed: {e}", exc_info=True)
        raise HTTPException(status_code=500, detail=f"Generation failed: {str(e)}")

@app.on_event("startup")
async def startup_event():
    """Load model on startup with fail-fast validation"""
    logger.info(f"[{instance_id}] Starting Sutra NLG Service...")
    
    success = await load_model()
    if not success:
        logger.error(f"[{instance_id}] CRITICAL: Model loading failed")
        raise RuntimeError("Model loading failed - cannot start service")
    
    logger.info(f"[{instance_id}] Service ready for requests")

@app.get("/health", response_model=HealthResponse)
async def health():
    """
    Health check endpoint
    
    Returns 200 if model is loaded and ready
    """
    model_loaded = model is not None and tokenizer is not None
    device = str(next(model.parameters()).device) if model else "unknown"
    
    return HealthResponse(
        status="healthy" if model_loaded else "unhealthy",
        model_loaded=model_loaded,
        model_name=model_name,
        device=device,
        instance_id=instance_id
    )

@app.get("/info", response_model=InfoResponse)
async def info():
    """Model information endpoint"""
    if model is None:
        raise HTTPException(status_code=503, detail="Model not loaded")
    
    device = str(next(model.parameters()).device)
    
    return InfoResponse(
        model=model_name,
        device=device,
        max_tokens=300,
        default_temperature=0.3
    )

@app.get("/metrics", response_model=MetricsResponse)
async def get_metrics():
    """Service metrics endpoint"""
    uptime = time.time() - metrics["start_time"]
    avg_time = (
        metrics["total_time_ms"] / metrics["total_requests"] 
        if metrics["total_requests"] > 0 
        else 0.0
    )
    
    return MetricsResponse(
        total_requests=metrics["total_requests"],
        total_tokens_generated=metrics["total_tokens"],
        avg_generation_time_ms=avg_time,
        model_name=model_name,
        uptime_seconds=uptime
    )

@app.post("/generate", response_model=GenerateResponse)
async def generate(request: GenerateRequest):
    """
    Generate natural language from constrained prompt
    
    CRITICAL: Prompt should include FACTS section to constrain generation
    
    Example prompt format:
    ```
    FACTS:
    - Paris is the capital of France
    - The Eiffel Tower is in Paris
    
    QUESTION: Where is the Eiffel Tower?
    
    ANSWER (using ONLY the facts above):
    ```
    """
    if not request.prompt:
        raise HTTPException(status_code=400, detail="No prompt provided")
    
    try:
        text, tokens, processing_time = generate_text(
            request.prompt,
            request.max_tokens,
            request.temperature,
            request.stop_sequences
        )
        
        # Update metrics
        metrics["total_requests"] += 1
        metrics["total_tokens"] += tokens
        metrics["total_time_ms"] += processing_time
        
        return GenerateResponse(
            text=text,
            model=model_name,
            processing_time_ms=processing_time,
            tokens_generated=tokens
        )
        
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"[{instance_id}] Generation request failed: {e}", exc_info=True)
        raise HTTPException(status_code=500, detail=f"Generation failed: {str(e)}")

@app.get("/")
async def root():
    """Root endpoint with service information"""
    return {
        "service": "Sutra NLG Service",
        "version": "1.0.0",
        "model": model_name,
        "instance_id": instance_id,
        "status": "running" if model else "loading"
    }

if __name__ == "__main__":
    # Run service
    port = int(os.getenv("PORT", "8889"))
    uvicorn.run(
        app, 
        host="0.0.0.0", 
        port=port, 
        log_level="info",
        access_log=True
    )
