#!/usr/bin/env python3
"""
Sutra NLG Service - Grounded Text Generation
Self-hosted small LLM for natural language generation with strict grounding
"""

import asyncio
import logging
import os
import time
from typing import List, Optional, Dict, Any

# Set cache directories before importing transformers
os.environ['TRANSFORMERS_CACHE'] = '/tmp/.cache/huggingface'
os.environ['HF_HOME'] = '/tmp/.cache/huggingface'
os.environ['TORCH_HOME'] = '/tmp/.cache/torch'

import torch
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel, Field
from transformers import AutoTokenizer, AutoModelForCausalLM
import uvicorn

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

# Request/Response Models
class GenerateRequest(BaseModel):
    prompt: str = Field(..., description="Constrained prompt with facts")
    max_tokens: int = Field(150, ge=1, le=300, description="Maximum tokens to generate")
    temperature: float = Field(0.3, ge=0.0, le=1.0, description="Sampling temperature")
    stop_sequences: List[str] = Field(default_factory=list, description="Stop generation at these sequences")

class GenerateResponse(BaseModel):
    text: str
    model: str
    processing_time_ms: float
    tokens_generated: int

class HealthResponse(BaseModel):
    status: str
    model_loaded: bool
    model_name: str
    device: str
    instance_id: str

class InfoResponse(BaseModel):
    model: str
    device: str
    max_tokens: int
    default_temperature: float

class MetricsResponse(BaseModel):
    total_requests: int
    total_tokens_generated: int
    avg_generation_time_ms: float
    model_name: str
    uptime_seconds: float

# Global model state
model = None
tokenizer = None
model_name = os.getenv("NLG_MODEL", "google/gemma-3-270m-it")
instance_id = os.getenv("INSTANCE_ID", "nlg-unknown")

# Metrics
metrics = {
    "total_requests": 0,
    "total_tokens": 0,
    "total_time_ms": 0,
    "start_time": time.time()
}

app = FastAPI(
    title="Sutra NLG Service",
    version="1.0.0",
    description="Self-hosted grounded natural language generation"
)

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
