"""
Sutra ML-Base Service Client Library

Provides async HTTP client for connecting to the centralized ML-Base service.
Used by embedding and NLG services to offload inference to the shared ML platform.
"""

import asyncio
import logging
from typing import Any, Dict, List, Optional

import aiohttp
from pydantic import BaseModel

logger = logging.getLogger(__name__)


class MLBaseClient:
    """Async client for Sutra ML-Base service"""
    
    def __init__(self, base_url: str, timeout: float = 30.0):
        """Initialize client
        
        Args:
            base_url: Base URL of ML-Base service (e.g., "http://ml-base:8887")
            timeout: Request timeout in seconds
        """
        self.base_url = base_url.rstrip('/')
        self.timeout = aiohttp.ClientTimeout(total=timeout)
        self._session: Optional[aiohttp.ClientSession] = None
        
    async def __aenter__(self):
        """Async context manager entry"""
        self._session = aiohttp.ClientSession(timeout=self.timeout)
        return self
    
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        """Async context manager exit"""
        if self._session:
            await self._session.close()
    
    @property
    def session(self) -> aiohttp.ClientSession:
        """Get or create session"""
        if self._session is None:
            self._session = aiohttp.ClientSession(timeout=self.timeout)
        return self._session
    
    async def health(self) -> Dict[str, Any]:
        """Check service health"""
        async with self.session.get(f"{self.base_url}/health") as resp:
            resp.raise_for_status()
            return await resp.json()
    
    async def list_models(self) -> List[Dict[str, Any]]:
        """List available models"""
        async with self.session.get(f"{self.base_url}/models") as resp:
            resp.raise_for_status()
            return await resp.json()
    
    async def embed(
        self, 
        model_id: str, 
        texts: List[str], 
        normalize: bool = True,
        batch_size: Optional[int] = None
    ) -> Dict[str, Any]:
        """Generate embeddings
        
        Args:
            model_id: Model ID to use (e.g., "embedding-nomic-v1.5")
            texts: List of texts to embed
            normalize: Whether to L2 normalize embeddings
            batch_size: Override batch size
            
        Returns:
            Dict with embeddings, dimension, processing_time_ms, etc.
        """
        payload = {
            "model_id": model_id,
            "texts": texts,
            "normalize": normalize
        }
        
        if batch_size is not None:
            payload["batch_size"] = batch_size
        
        async with self.session.post(f"{self.base_url}/embed", json=payload) as resp:
            if resp.status != 200:
                error_text = await resp.text()
                logger.error(f"Embedding request failed: {resp.status} - {error_text}")
                resp.raise_for_status()
            
            return await resp.json()
    
    async def generate(
        self,
        model_id: str,
        prompt: str,
        max_tokens: int = 150,
        temperature: float = 0.3,
        top_p: float = 0.9,
        stop_sequences: Optional[List[str]] = None
    ) -> Dict[str, Any]:
        """Generate text
        
        Args:
            model_id: Model ID to use (e.g., "nlg-dialogpt-large")
            prompt: Generation prompt
            max_tokens: Maximum tokens to generate
            temperature: Sampling temperature (0.0 = deterministic)
            top_p: Nucleus sampling parameter
            stop_sequences: Stop generation at these sequences
            
        Returns:
            Dict with text, tokens_generated, processing_time_ms, etc.
        """
        payload = {
            "model_id": model_id,
            "prompt": prompt,
            "max_tokens": max_tokens,
            "temperature": temperature,
            "top_p": top_p,
            "stop_sequences": stop_sequences or []
        }
        
        async with self.session.post(f"{self.base_url}/generate", json=payload) as resp:
            if resp.status != 200:
                error_text = await resp.text()
                logger.error(f"Generation request failed: {resp.status} - {error_text}")
                resp.raise_for_status()
            
            return await resp.json()
    
    async def load_model(self, model_id: str, model_name: str, model_type: str) -> bool:
        """Load a new model
        
        Args:
            model_id: Unique model ID
            model_name: HuggingFace model name
            model_type: "embedding" or "nlg"
            
        Returns:
            True if loaded successfully
        """
        params = {
            "model_name": model_name,
            "model_type": model_type
        }
        
        async with self.session.post(f"{self.base_url}/models/{model_id}/load", params=params) as resp:
            if resp.status == 200:
                return True
            else:
                error_text = await resp.text()
                logger.error(f"Model load failed: {resp.status} - {error_text}")
                return False
    
    async def unload_model(self, model_id: str) -> bool:
        """Unload a model
        
        Args:
            model_id: Model ID to unload
            
        Returns:
            True if unloaded successfully
        """
        async with self.session.delete(f"{self.base_url}/models/{model_id}") as resp:
            return resp.status == 200
    
    async def close(self):
        """Close client session"""
        if self._session:
            await self._session.close()
            self._session = None


# Convenience functions for single requests

async def embed_texts(
    base_url: str,
    model_id: str, 
    texts: List[str],
    normalize: bool = True,
    timeout: float = 30.0
) -> List[List[float]]:
    """Convenience function to embed texts
    
    Args:
        base_url: ML-Base service URL
        model_id: Model ID to use
        texts: Texts to embed
        normalize: Whether to normalize embeddings
        timeout: Request timeout
        
    Returns:
        List of embeddings
    """
    async with MLBaseClient(base_url, timeout) as client:
        result = await client.embed(model_id, texts, normalize)
        return result["embeddings"]


async def generate_text(
    base_url: str,
    model_id: str,
    prompt: str,
    max_tokens: int = 150,
    temperature: float = 0.3,
    timeout: float = 30.0
) -> str:
    """Convenience function to generate text
    
    Args:
        base_url: ML-Base service URL
        model_id: Model ID to use
        prompt: Generation prompt
        max_tokens: Max tokens to generate
        temperature: Sampling temperature
        timeout: Request timeout
        
    Returns:
        Generated text
    """
    async with MLBaseClient(base_url, timeout) as client:
        result = await client.generate(model_id, prompt, max_tokens, temperature)
        return result["text"]


# Health check utility
async def check_ml_base_health(base_url: str, timeout: float = 5.0) -> bool:
    """Check if ML-Base service is healthy
    
    Args:
        base_url: ML-Base service URL
        timeout: Request timeout
        
    Returns:
        True if service is healthy
    """
    try:
        async with MLBaseClient(base_url, timeout) as client:
            health = await client.health()
            return health.get("status") == "healthy"
    except Exception as e:
        logger.warning(f"ML-Base health check failed: {e}")
        return False