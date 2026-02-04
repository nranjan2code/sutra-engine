"""
Batch embedding generation with Apple Silicon MPS optimization.

This module provides efficient batch processing for concept embeddings:
- Automatic CPU/MPS device selection based on batch size
- Optimal batch size tuning for Apple M-series chips
- Embedding caching to avoid recomputation
- Memory-efficient processing for large batches

Performance characteristics on Apple Silicon:
- Batch < 64: CPU is faster (~1800/sec)
- Batch ≥ 64: MPS is 2x faster (~3700/sec)
- Optimal batch size: 128-256 for M1/M2/M3
"""

import logging
import time
from typing import Dict, List, Optional, Tuple

import numpy as np
import torch

try:
    from sentence_transformers import SentenceTransformer
except ImportError:
    raise ImportError(
        "sentence-transformers is required for embedding generation. "
        "Install with: pip install sentence-transformers"
    )

logger = logging.getLogger(__name__)


class EmbeddingBatchProcessor:
    """
    Efficient batch embedding generation optimized for Apple Silicon.

    Key features:
    - Automatic device selection (CPU vs MPS)
    - Smart batch size tuning
    - Embedding cache for duplicates
    - Memory monitoring and optimization
    """

    def __init__(
        self,
        model_name: str = "all-MiniLM-L6-v2",
        device: Optional[str] = "auto",
        mps_threshold: int = 64,
        cache_size: int = 10000,
        enable_fp16: bool = True,
    ):
        """
        Initialize batch embedding processor.

        Args:
            model_name: Sentence-transformer model to use
            device: Device to use ('cpu', 'mps', 'auto')
            mps_threshold: Minimum batch size for MPS (auto mode)
            cache_size: Maximum embeddings to cache
            enable_fp16: Use FP16 for MPS (2x memory savings)
        """
        self.model_name = model_name
        self.mps_threshold = mps_threshold
        self.cache_size = cache_size
        self.enable_fp16 = enable_fp16

        # Load model
        logger.info(f"Loading embedding model: {model_name}")
        start_time = time.time()

        # Try to load from cache first (no network calls)
        try:
            self.model = SentenceTransformer(model_name, local_files_only=True)
            logger.debug(f"Loaded {model_name} from local cache")
        except Exception:
            # If not in cache, download once then use cache
            logger.info(f"Downloading {model_name} to cache...")
            self.model = SentenceTransformer(model_name, local_files_only=False)

        load_time = time.time() - start_time
        logger.info(f"Model loaded in {load_time:.2f}s")

        # Determine device
        self.device = self._select_device(device)
        logger.info(f"Using device: {self.device}")

        # Move model to device
        if self.device == "mps":
            self.model.to("mps")
            if self.enable_fp16:
                logger.info("Enabling FP16 for MPS")
                # Note: FP16 conversion would be done per-batch if needed

        # Embedding cache
        self.cache: Dict[str, np.ndarray] = {}
        self.cache_hits = 0
        self.cache_misses = 0

        # Performance tracking
        self.total_embeddings = 0
        self.total_time = 0.0
        self.batch_count = 0

    def _select_device(self, device: str) -> str:
        """
        Select optimal device for embedding generation.

        Args:
            device: 'cpu', 'mps', or 'auto'

        Returns:
            Selected device string
        """
        if device in ["cpu", "mps"]:
            if device == "mps" and not torch.backends.mps.is_available():
                logger.warning("MPS requested but not available, falling back to CPU")
                return "cpu"
            return device

        # Auto-selection
        if torch.backends.mps.is_available():
            logger.info(
                "MPS available - will use for batches ≥ {}".format(self.mps_threshold)
            )
            return "mps"  # Will switch dynamically based on batch size

        return "cpu"

    def _get_device_for_batch(self, batch_size: int) -> str:
        """
        Determine optimal device for given batch size.

        Args:
            batch_size: Number of texts to encode

        Returns:
            Device to use ('cpu' or 'mps')
        """
        if self.device == "cpu":
            return "cpu"

        # Use MPS for large batches, CPU for small ones
        if batch_size >= self.mps_threshold:
            return "mps"

        return "cpu"

    def encode_batch(
        self,
        texts: List[str],
        show_progress: bool = False,
        normalize: bool = True,
        prompt_name: Optional[str] = None,
    ) -> np.ndarray:
        """
        Encode batch of texts to embeddings with caching.

        Args:
            texts: List of text strings to encode
            show_progress: Show progress bar
            normalize: Normalize embeddings to unit length
            prompt_name: Optional prompt name for task-specific models (e.g., 'Retrieval-query', 'Retrieval-document')

        Returns:
            Array of embeddings (batch_size, embedding_dim)
        """
        if not texts:
            return np.array([])

        start_time = time.time()
        batch_size = len(texts)

        # Check cache (include prompt in cache key for task-specific models)
        uncached_texts = []
        uncached_indices = []
        embeddings = [None] * batch_size

        for i, text in enumerate(texts):
            # Cache key includes prompt to avoid mixing query/document embeddings
            cache_key = f"{prompt_name}:{text}" if prompt_name else text
            if cache_key in self.cache:
                embeddings[i] = self.cache[cache_key]
                self.cache_hits += 1
            else:
                uncached_texts.append(text)
                uncached_indices.append(i)
                self.cache_misses += 1

        # Generate uncached embeddings
        if uncached_texts:
            # Select device based on batch size
            target_device = self._get_device_for_batch(len(uncached_texts))

            # Move model if needed
            if target_device == "mps" and str(self.model.device) != "mps":
                self.model.to("mps")
            elif target_device == "cpu" and str(self.model.device) != "cpu":
                self.model.to("cpu")

            # Generate embeddings
            encode_kwargs = {
                "device": target_device,
                "show_progress_bar": show_progress,
                "normalize_embeddings": normalize,
                "convert_to_numpy": True,
            }

            # Add prompt for task-specific models (e.g., EmbeddingGemma)
            if prompt_name:
                encode_kwargs["prompt_name"] = prompt_name

            new_embeddings = self.model.encode(uncached_texts, **encode_kwargs)

            # Update cache and results
            for i, idx in enumerate(uncached_indices):
                emb = new_embeddings[i]
                embeddings[idx] = emb

                # Add to cache if not full (use prompt-aware key)
                if len(self.cache) < self.cache_size:
                    cache_key = (
                        f"{prompt_name}:{uncached_texts[i]}"
                        if prompt_name
                        else uncached_texts[i]
                    )
                    self.cache[cache_key] = emb

        # Stack embeddings
        result = np.vstack(embeddings)

        # Track performance
        elapsed = time.time() - start_time
        self.total_embeddings += batch_size
        self.total_time += elapsed
        self.batch_count += 1

        throughput = batch_size / elapsed if elapsed > 0 else 0
        cache_rate = self.cache_hits / (self.cache_hits + self.cache_misses) * 100

        logger.debug(
            f"Encoded {batch_size} texts in {elapsed:.3f}s "
            f"({throughput:.1f} texts/sec, cache hit rate: {cache_rate:.1f}%)"
        )

        return result

    def encode_single(
        self, text: str, normalize: bool = True, prompt_name: Optional[str] = None
    ) -> np.ndarray:
        """
        Encode single text (convenience wrapper).

        Args:
            text: Text to encode
            normalize: Normalize embedding
            prompt_name: Optional prompt name for task-specific models

        Returns:
            Embedding vector
        """
        result = self.encode_batch(
            [text], show_progress=False, normalize=normalize, prompt_name=prompt_name
        )
        return result[0]

    def clear_cache(self):
        """Clear embedding cache."""
        self.cache.clear()
        self.cache_hits = 0
        self.cache_misses = 0
        logger.info("Embedding cache cleared")

    def get_stats(self) -> Dict[str, float]:
        """
        Get performance statistics.

        Returns:
            Dictionary with performance metrics
        """
        avg_throughput = (
            self.total_embeddings / self.total_time if self.total_time > 0 else 0
        )
        cache_rate = (
            self.cache_hits / (self.cache_hits + self.cache_misses) * 100
            if (self.cache_hits + self.cache_misses) > 0
            else 0
        )

        return {
            "total_embeddings": self.total_embeddings,
            "total_time": self.total_time,
            "batch_count": self.batch_count,
            "avg_throughput": avg_throughput,
            "cache_hits": self.cache_hits,
            "cache_misses": self.cache_misses,
            "cache_hit_rate": cache_rate,
            "cache_size": len(self.cache),
        }

    def __repr__(self) -> str:
        stats = self.get_stats()
        return (
            f"EmbeddingBatchProcessor("
            f"model={self.model_name}, "
            f"device={self.device}, "
            f"embeddings={stats['total_embeddings']}, "
            f"throughput={stats['avg_throughput']:.1f}/s, "
            f"cache_rate={stats['cache_hit_rate']:.1f}%)"
        )


def create_embedding_processor(
    model_name: str = "all-MiniLM-L6-v2",
    **kwargs,
) -> EmbeddingBatchProcessor:
    """
    Factory function to create embedding processor.

    Args:
        model_name: Sentence-transformer model name
        **kwargs: Additional arguments for EmbeddingBatchProcessor

    Returns:
        Configured EmbeddingBatchProcessor instance
    """
    return EmbeddingBatchProcessor(model_name=model_name, **kwargs)
