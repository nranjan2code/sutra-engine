#!/usr/bin/env python3
"""
Sutra Embedding Service - Simple Implementation
High-performance embedding service using transformers directly
Avoids sentence-transformers compatibility issues
"""

import asyncio
import logging
import os
import time
from typing import List, Optional, Dict, Any

# Set cache directories before importing transformers to avoid permission issues
os.environ['TRANSFORMERS_CACHE'] = '/tmp/.cache/huggingface'
os.environ['HF_HOME'] = '/tmp/.cache/huggingface'
os.environ['TORCH_HOME'] = '/tmp/.cache/torch'

import numpy as np
import torch
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from transformers import AutoTokenizer, AutoModel
import uvicorn

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

# Request/Response Models
class EmbeddingRequest(BaseModel):
    texts: List[str]
    normalize: bool = True

class EmbeddingResponse(BaseModel):
    embeddings: List[List[float]]
    dimension: int
    model: str
    processing_time_ms: float
    cached_count: int = 0

class HealthResponse(BaseModel):
    status: str
    model_loaded: bool
    dimension: int
    model_name: str

class InfoResponse(BaseModel):
    model: str
    dimension: int
    device: str
    max_batch_size: int

# Global model state
model = None
tokenizer = None
model_name = "nomic-ai/nomic-embed-text-v1.5"

app = FastAPI(title="Sutra Embedding Service", version="1.0.0")

def mean_pooling(model_output, attention_mask):
    """Mean pooling for sentence embeddings"""
    token_embeddings = model_output[0]  # First element contains all token embeddings
    input_mask_expanded = attention_mask.unsqueeze(-1).expand(token_embeddings.size()).float()
    return torch.sum(token_embeddings * input_mask_expanded, 1) / torch.clamp(input_mask_expanded.sum(1), min=1e-9)

async def load_model():
    """Load the embedding model"""
    global model, tokenizer
    
    logger.info(f"Loading model: {model_name} from Hugging Face")
    start_time = time.time()
    
    try:
        # Download model from Hugging Face with caching
        tokenizer = AutoTokenizer.from_pretrained(model_name, trust_remote_code=True, cache_dir='/tmp/.cache/huggingface')
        model = AutoModel.from_pretrained(model_name, trust_remote_code=True, cache_dir='/tmp/.cache/huggingface')
        
        # Set to eval mode
        model.eval()
        
        load_time = time.time() - start_time
        logger.info(f"Model loaded successfully in {load_time:.2f}s")
        logger.info(f"Model device: {next(model.parameters()).device}")
        
        return True
        
    except Exception as e:
        logger.error(f"Failed to load model: {e}")
        return False

def generate_embeddings(texts: List[str], normalize: bool = True) -> List[List[float]]:
    """Generate embeddings for a batch of texts"""
    if model is None or tokenizer is None:
        raise HTTPException(status_code=503, detail="Model not loaded")
    
    start_time = time.time()
    
    # Tokenize
    encoded_input = tokenizer(
        texts, 
        padding=True, 
        truncation=True, 
        return_tensors='pt',
        max_length=512
    )
    
    # Generate embeddings
    with torch.no_grad():
        model_output = model(**encoded_input)
        embeddings = mean_pooling(model_output, encoded_input['attention_mask'])
        
        # Normalize if requested
        if normalize:
            embeddings = torch.nn.functional.normalize(embeddings, p=2, dim=1)
    
    processing_time = (time.time() - start_time) * 1000
    
    # Convert to list
    embeddings_list = embeddings.cpu().numpy().tolist()
    
    logger.info(f"Generated {len(embeddings_list)} embeddings in {processing_time:.2f}ms")
    
    return embeddings_list, processing_time

@app.on_event("startup")
async def startup_event():
    """Load model on startup"""
    success = await load_model()
    if not success:
        logger.error("Failed to load model on startup")
        raise RuntimeError("Model loading failed")

@app.get("/health", response_model=HealthResponse)
async def health():
    """Health check endpoint"""
    model_loaded = model is not None and tokenizer is not None
    
    return HealthResponse(
        status="healthy" if model_loaded else "unhealthy",
        model_loaded=model_loaded,
        dimension=768,
        model_name=model_name
    )

@app.get("/info", response_model=InfoResponse)
async def info():
    """Model information endpoint"""
    if model is None:
        raise HTTPException(status_code=503, detail="Model not loaded")
        
    device = str(next(model.parameters()).device)
    
    return InfoResponse(
        model=model_name,
        dimension=768,
        device=device,
        max_batch_size=64
    )

@app.post("/embed", response_model=EmbeddingResponse)
async def embed(request: EmbeddingRequest):
    """Generate embeddings for texts"""
    if not request.texts:
        raise HTTPException(status_code=400, detail="No texts provided")
    
    if len(request.texts) > 64:
        raise HTTPException(status_code=400, detail="Too many texts (max 64)")
    
    try:
        embeddings, processing_time = generate_embeddings(request.texts, request.normalize)
        
        return EmbeddingResponse(
            embeddings=embeddings,
            dimension=768,
            model=model_name,
            processing_time_ms=processing_time
        )
        
    except Exception as e:
        logger.error(f"Embedding generation failed: {e}")
        raise HTTPException(status_code=500, detail=f"Embedding generation failed: {str(e)}")

@app.get("/")
async def root():
    """Root endpoint"""
    return {"service": "Sutra Embedding Service", "model": model_name, "status": "running"}

if __name__ == "__main__":
    uvicorn.run(app, host="0.0.0.0", port=8888, log_level="info")