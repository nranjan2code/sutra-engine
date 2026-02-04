"""
Embedding Service Provider using dedicated Sutra Embedding Service.

High-performance embedding provider that connects to the dedicated
embedding service using nomic-embed-text-v1.5 for production deployments.
"""

import json
import logging
import os
import time
from typing import List, Optional

import numpy as np
import requests
from requests.adapters import HTTPAdapter
from urllib3.util.retry import Retry

from .base import EmbeddingProvider

logger = logging.getLogger(__name__)


class EmbeddingServiceProvider(EmbeddingProvider):
    """
    Production embedding provider using dedicated Sutra Embedding Service.
    
    Connects to high-performance embedding service for optimal throughput
    and reliability. Supports intelligent batching, caching, and monitoring.
    """

    def __init__(
        self,
        service_url: Optional[str] = None,
        timeout: int = 30,
        max_retries: int = 3,
        backoff_factor: float = 0.5,
    ):
        """
        Initialize embedding service provider.

        Args:
            service_url: URL of the embedding service
            timeout: Request timeout in seconds
            max_retries: Maximum number of retries on failure
            backoff_factor: Exponential backoff factor for retries

        Raises:
            ConnectionError: If cannot connect to embedding service
            ValueError: If service returns invalid response format
        """
        self.service_url = (
            service_url
            or os.getenv("SUTRA_EMBEDDING_SERVICE_URL", "http://sutra-embedding-service:8888")
        )
        self.timeout = timeout
        
        # Configure requests session with connection pooling and retries
        self.session = requests.Session()
        
        # Configure retry strategy
        retry_strategy = Retry(
            total=max_retries,
            backoff_factor=backoff_factor,
            status_forcelist=[429, 500, 502, 503, 504],
            allowed_methods=["GET", "POST"]
        )
        
        adapter = HTTPAdapter(
            max_retries=retry_strategy,
            pool_connections=10,
            pool_maxsize=20
        )
        
        self.session.mount("http://", adapter)
        self.session.mount("https://", adapter)
        
        # Validate service connection
        self._validate_service()
        logger.info(f"✅ PRODUCTION: Initialized EmbeddingServiceProvider with nomic-embed-text-v1.5")

    def _validate_service(self):
        """Validate service availability and model configuration."""
        try:
            # Check service health
            health_response = self.session.get(
                f"{self.service_url}/health",
                timeout=self.timeout
            )
            health_response.raise_for_status()
            
            health_data = health_response.json()
            
            if health_data.get("status") != "healthy":
                raise ConnectionError(f"Embedding service unhealthy: {health_data}")
            
            # Check if it's a lightweight client (ML-Base architecture) or standalone service
            if health_data.get("ml_base_connected") is not None:
                # Lightweight client - check ML-Base connectivity
                if not health_data.get("ml_base_connected", False):
                    raise ConnectionError("ML-Base service not connected")
            elif health_data.get("model_loaded") is False:
                # Standalone service - check model loaded (only fail if explicitly False)
                # External services may not report this field, so treat missing as OK
                raise ConnectionError("Embedding model not loaded in service")
            
                # Check service info for basic health
            info_response = self.session.get(
                f"{self.service_url}/info",
                timeout=self.timeout
            )
            info_response.raise_for_status()
            
            info_data = info_response.json()
            logger.info(f"Embedding service info: {info_data}")
            
            # Validate dimensions with a test embedding request
            test_response = self.session.post(
                f"{self.service_url}/embed",
                json={"texts": ["test validation"]},
                timeout=self.timeout
            )
            test_response.raise_for_status()
            
            test_data = test_response.json()
            # Try both 'dimension' (internal) and 'dimensions' (external service)
            dimension = test_data.get("dimension") or test_data.get("dimensions", 0)
            model = test_data.get("model", "")
            
            # Get expected dimension from environment (Phase 0: Matryoshka support)
            expected_dim = int(os.getenv("SUTRA_VECTOR_DIMENSION", "768"))
            
            if dimension != expected_dim:
                raise ValueError(
                    f"Service produces {dimension}-d embeddings, expected {expected_dim}-d"
                )
            
            if "nomic-embed-text-v1.5" not in model:
                logger.warning(
                    f"Expected nomic-embed-text-v1.5, service reports: {model}"
                )
            
            logger.info(f"Validated embedding service: dimension={dimension}, model={model}")
            
        except requests.RequestException as e:
            raise ConnectionError(f"Cannot connect to embedding service at {self.service_url}: {e}")

    def encode_single(self, text: str, prompt_name: Optional[str] = None) -> np.ndarray:
        """
        Encode a single text into embedding (convenience wrapper for QueryProcessor).
        
        Args:
            text: Text string to encode
            prompt_name: Optional prompt name (ignored, for compatibility)
        
        Returns:
            numpy array of shape (768,) - single embedding vector
        """
        result = self.encode([text])
        return result[0] if len(result) > 0 else np.zeros(768, dtype=np.float32)
    
    def encode(self, texts: List[str]) -> np.ndarray:
        """
        Encode texts into embeddings using the embedding service.

        Args:
            texts: List of text strings to encode

        Returns:
            numpy array of shape (len(texts), 768) - production 768-d embeddings

        Raises:
            RuntimeError: If embedding generation fails
            ValueError: If service returns invalid dimensions
        """
        if not texts:
            return np.array([]).reshape(0, 768)

        start_time = time.time()
        
        try:
            # Prepare request
            request_data = {
                "texts": texts,
                "normalize": True  # L2 normalize for consistency
            }
            
            # Make request to embedding service
            response = self.session.post(
                f"{self.service_url}/embed",
                json=request_data,
                timeout=self.timeout
            )
            response.raise_for_status()
            
            # Parse response
            result = response.json()
            
            # Validate response format
            if "embeddings" not in result:
                raise ValueError(f"Invalid response format: missing 'embeddings' field")
            
            # Get expected dimension from environment (Phase 0: Matryoshka support)
            expected_dim = int(os.getenv("SUTRA_VECTOR_DIMENSION", "768"))
            
            if result.get("dimension") != expected_dim:
                raise ValueError(
                    f"Service returned {result.get('dimension')}-d embeddings, expected {expected_dim}-d"
                )
            
            embeddings = result["embeddings"]
            
            # Validate embedding count
            if len(embeddings) != len(texts):
                raise ValueError(
                    f"Expected {len(texts)} embeddings, got {len(embeddings)}"
                )
            
            # Convert to numpy array
            embeddings_array = np.array(embeddings, dtype=np.float32)
            
            # Validate dimensions
            expected_dim = int(os.getenv("SUTRA_VECTOR_DIMENSION", "768"))
            if embeddings_array.shape[1] != expected_dim:
                raise ValueError(
                    f"Expected {expected_dim}-dimensional embeddings, got {embeddings_array.shape[1]}-d"
                )
            
            # Log performance metrics
            processing_time = time.time() - start_time
            cached_count = result.get("cached_count", 0)
            service_time = result.get("processing_time_ms", 0)
            
            logger.debug(
                f"Generated {len(embeddings)} embeddings in {processing_time*1000:.1f}ms "
                f"(service: {service_time:.1f}ms, cached: {cached_count})"
            )
            
            return embeddings_array
            
        except requests.RequestException as e:
            logger.error(f"Embedding service request failed: {e}")
            raise RuntimeError(f"Failed to get embeddings from service: {e}")
        
        except (KeyError, ValueError, TypeError) as e:
            logger.error(f"Invalid embedding service response: {e}")
            raise ValueError(f"Invalid response from embedding service: {e}")

    def get_dimension(self) -> int:
        """
        Get embedding dimension - PRODUCTION: strict 768-d requirement.

        Returns:
            768 (nomic-embed-text-v1.5 native dimension)
        """
        return 768

    def get_name(self) -> str:
        """
        Get provider name.

        Returns:
            Provider identifier string
        """
        return "embedding-service-nomic-v1.5"

    def similarity(self, embedding1: np.ndarray, embedding2: np.ndarray) -> float:
        """
        Calculate cosine similarity between two embeddings.

        Args:
            embedding1: First embedding vector
            embedding2: Second embedding vector

        Returns:
            Cosine similarity score between -1 and 1
        """
        # Ensure embeddings are normalized (service should already do this)
        norm1 = np.linalg.norm(embedding1)
        norm2 = np.linalg.norm(embedding2)
        
        if norm1 == 0 or norm2 == 0:
            return 0.0
            
        normalized1 = embedding1 / norm1
        normalized2 = embedding2 / norm2
        
        return float(np.dot(normalized1, normalized2))

    def health_check(self) -> bool:
        """
        Check if embedding service is healthy and responsive.

        Returns:
            True if service is healthy, False otherwise
        """
        try:
            response = self.session.get(
                f"{self.service_url}/health",
                timeout=5  # Short timeout for health checks
            )
            
            if not response.ok:
                return False
            
            health_data = response.json()
            return (
                health_data.get("status") == "healthy" and 
                health_data.get("model_loaded", False)
            )
            
        except Exception as e:
            logger.warning(f"Health check failed: {e}")
            return False

    def get_metrics(self) -> dict:
        """
        Get service metrics for monitoring.

        Returns:
            Dictionary with service metrics, empty dict if unavailable
        """
        try:
            response = self.session.get(
                f"{self.service_url}/metrics",
                timeout=5
            )
            
            if response.ok:
                # Parse Prometheus metrics (basic implementation)
                metrics_text = response.text
                
                # Extract basic metrics
                metrics = {}
                for line in metrics_text.split('\n'):
                    if 'embedding_requests_total' in line and 'status="success"' in line:
                        parts = line.split()
                        if len(parts) >= 2:
                            metrics['total_requests'] = float(parts[1])
                    elif 'embedding_cache_hits_total' in line:
                        parts = line.split()
                        if len(parts) >= 2:
                            metrics['cache_hits'] = float(parts[1])
                
                return metrics
            
        except Exception as e:
            logger.debug(f"Failed to get metrics: {e}")
        
        return {}

    def __del__(self):
        """Clean up session on destruction."""
        if hasattr(self, 'session'):
            self.session.close()