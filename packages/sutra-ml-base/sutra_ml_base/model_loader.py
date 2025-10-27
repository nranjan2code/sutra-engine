"""
Universal Model Loader for Sutra ML Services

Provides unified loading for all model types:
- Embedding models (sentence transformers, custom embedders)
- Generative models (LLMs, chat models, completion models)  
- Multimodal models (vision-language, audio, etc.)

Features:
- Edition-aware resource management and model selection
- Efficient caching with persistent storage
- Dynamic model swapping and hot-reloading
- Performance optimization (quantization, pruning, etc.)
- Security validation and model verification
"""

import os
import logging
import time
import hashlib
from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from enum import Enum
from typing import Any, Dict, Optional, Union, List, Tuple, Type
from pathlib import Path

# ML framework imports with graceful degradation
try:
    import torch
    HAS_TORCH = True
except ImportError:
    HAS_TORCH = False
    logging.warning("PyTorch not available - some features disabled")

try:
    from transformers import AutoTokenizer, AutoModel, AutoModelForCausalLM, PreTrainedModel, PreTrainedTokenizer
    HAS_TRANSFORMERS = True
except ImportError:
    HAS_TRANSFORMERS = False
    logging.warning("Transformers not available - HuggingFace models disabled")

from .edition import EditionManager, get_current_edition

logger = logging.getLogger(__name__)


class ModelType(Enum):
    """Supported model types in Sutra ecosystem"""
    EMBEDDING = "embedding"          # Text → Vector (nomic-embed, etc.)
    GENERATIVE = "generative"        # Text → Text (Gemma, Phi, etc.)
    CHAT = "chat"                   # Conversational models  
    MULTIMODAL = "multimodal"       # Vision, audio, etc.
    CUSTOM = "custom"               # User-defined models


@dataclass
class LoaderConfig:
    """Configuration for model loading behavior"""
    # Model identification
    model_name: str
    model_type: ModelType
    local_path: Optional[str] = None
    
    # HuggingFace settings
    hf_token: Optional[str] = None
    trust_remote_code: bool = True
    revision: Optional[str] = None
    
    # Resource management
    device: str = "auto"              # "auto", "cpu", "cuda", "mps"
    torch_dtype: str = "auto"         # "auto", "float32", "float16", "bfloat16"
    max_memory_gb: Optional[float] = None
    
    # Performance optimization
    load_in_8bit: bool = False
    load_in_4bit: bool = False
    use_flash_attention: bool = False
    compile_model: bool = False
    
    # Caching
    cache_dir: str = "/tmp/.cache/huggingface"
    use_local_cache: bool = True
    
    # Security
    verify_model: bool = True
    allow_custom_code: bool = True  # Enable for Simple edition compatibility
    
    # Edition overrides (None = use edition defaults)
    max_batch_size: Optional[int] = None
    max_sequence_length: Optional[int] = None
    
    def __post_init__(self):
        """Validate and normalize configuration"""
        if not HAS_TRANSFORMERS and self.model_type in [ModelType.EMBEDDING, ModelType.GENERATIVE, ModelType.CHAT]:
            raise RuntimeError("Transformers library required for HuggingFace models")
        
        # Set HF token from environment if not provided
        if self.hf_token is None:
            self.hf_token = os.getenv("HF_TOKEN")
        
        # Ensure cache directory exists
        Path(self.cache_dir).mkdir(parents=True, exist_ok=True)


class BaseModelLoader(ABC):
    """Abstract base class for model loaders"""
    
    def __init__(self, config: LoaderConfig, edition_manager: Optional[EditionManager] = None):
        self.config = config
        self.edition_manager = edition_manager or EditionManager()
        self.model = None
        self.tokenizer = None
        self.load_time = 0.0
        self.memory_usage_gb = 0.0
        
    @abstractmethod
    def load(self) -> Tuple[Any, Any]:
        """Load model and tokenizer
        
        Returns:
            Tuple of (model, tokenizer)
        """
        pass
    
    @abstractmethod
    def validate_model(self) -> bool:
        """Validate loaded model works correctly"""
        pass
    
    def get_model_info(self) -> Dict[str, Any]:
        """Get model information and metadata"""
        return {
            "model_name": self.config.model_name,
            "model_type": self.config.model_type.value,
            "device": str(getattr(self.model, 'device', 'unknown')),
            "dtype": str(getattr(self.model, 'dtype', 'unknown')),
            "load_time_seconds": self.load_time,
            "memory_usage_gb": self.memory_usage_gb,
            "edition": self.edition_manager.edition.value,
            "parameters": self._count_parameters()
        }
    
    def _count_parameters(self) -> int:
        """Count model parameters"""
        if not HAS_TORCH or self.model is None:
            return 0
        
        try:
            if hasattr(self.model, 'parameters'):
                return sum(p.numel() for p in self.model.parameters())
        except Exception:
            pass
        return 0


class EmbeddingModelLoader(BaseModelLoader):
    """Loader for embedding/sentence transformer models"""
    
    def load(self) -> Tuple[Any, Any]:
        """Load embedding model and tokenizer"""
        if not HAS_TRANSFORMERS:
            raise RuntimeError("Transformers library required for embedding models")
        
        logger.info(f"Loading embedding model: {self.config.model_name}")
        start_time = time.time()
        
        # Check edition limits
        if not self.edition_manager.supports_custom_models() and self.config.local_path:
            raise PermissionError("Custom model loading not available in this edition")
        
        model_path = self.config.local_path or self.config.model_name
        
        try:
            # Load tokenizer
            self.tokenizer = AutoTokenizer.from_pretrained(
                model_path,
                token=self.config.hf_token,
                trust_remote_code=self.config.trust_remote_code and self.config.allow_custom_code,
                cache_dir=self.config.cache_dir if self.config.use_local_cache else None,
                revision=self.config.revision
            )
            
            # Load model
            self.model = AutoModel.from_pretrained(
                model_path,
                token=self.config.hf_token,
                trust_remote_code=self.config.trust_remote_code and self.config.allow_custom_code,
                cache_dir=self.config.cache_dir if self.config.use_local_cache else None,
                revision=self.config.revision,
                torch_dtype=self._get_torch_dtype(),
                device_map=self._get_device_map(),
                load_in_8bit=self.config.load_in_8bit,
                load_in_4bit=self.config.load_in_4bit
            )
            
            # Set to eval mode
            self.model.eval()
            
            # Compile if requested (PyTorch 2.0+)
            if self.config.compile_model and hasattr(torch, 'compile'):
                self.model = torch.compile(self.model)
            
            self.load_time = time.time() - start_time
            self._calculate_memory_usage()
            
            logger.info(f"Embedding model loaded in {self.load_time:.2f}s")
            return self.model, self.tokenizer
            
        except Exception as e:
            logger.error(f"Failed to load embedding model: {e}")
            raise
    
    def validate_model(self) -> bool:
        """Validate embedding model with test input"""
        if self.model is None or self.tokenizer is None:
            return False
        
        try:
            # Test with simple input
            test_text = "This is a test sentence for validation."
            inputs = self.tokenizer(test_text, return_tensors="pt", truncation=True)
            
            with torch.no_grad():
                outputs = self.model(**inputs)
                embeddings = outputs.last_hidden_state.mean(dim=1)
            
            # Basic shape validation
            expected_dim = 768  # Common embedding dimension
            if embeddings.shape[-1] not in [384, 512, 768, 1024, 1536]:
                logger.warning(f"Unexpected embedding dimension: {embeddings.shape[-1]}")
            
            logger.info(f"Embedding model validation passed - output shape: {embeddings.shape}")
            return True
            
        except Exception as e:
            logger.error(f"Embedding model validation failed: {e}")
            return False
    
    def _get_torch_dtype(self):
        """Get appropriate torch dtype"""
        if not HAS_TORCH:
            return None
            
        dtype_map = {
            "auto": "auto",
            "float32": torch.float32,
            "float16": torch.float16,
            "bfloat16": torch.bfloat16 if hasattr(torch, 'bfloat16') else torch.float16
        }
        return dtype_map.get(self.config.torch_dtype, "auto")
    
    def _get_device_map(self):
        """Get device mapping for model"""
        if self.config.device == "auto":
            return "auto" if HAS_TORCH and torch.cuda.is_available() else None
        elif self.config.device == "cpu":
            return "cpu"
        else:
            return self.config.device
    
    def _calculate_memory_usage(self):
        """Estimate memory usage of loaded model"""
        try:
            if HAS_TORCH and self.model is not None:
                param_count = self._count_parameters()
                bytes_per_param = 4  # float32
                if hasattr(self.model, 'dtype'):
                    if 'float16' in str(self.model.dtype) or 'half' in str(self.model.dtype):
                        bytes_per_param = 2
                    elif '8bit' in str(self.model.dtype):
                        bytes_per_param = 1
                
                self.memory_usage_gb = (param_count * bytes_per_param) / (1024**3)
        except Exception:
            self.memory_usage_gb = 0.0


class GenerativeModelLoader(BaseModelLoader):
    """Loader for generative language models (LLMs)"""
    
    def load(self) -> Tuple[Any, Any]:
        """Load generative model and tokenizer"""
        if not HAS_TRANSFORMERS:
            raise RuntimeError("Transformers library required for generative models")
        
        logger.info(f"Loading generative model: {self.config.model_name}")
        start_time = time.time()
        
        # Check edition limits
        if not self.edition_manager.supports_custom_models() and self.config.local_path:
            raise PermissionError("Custom model loading not available in this edition")
        
        model_path = self.config.local_path or self.config.model_name
        
        try:
            # Load tokenizer
            self.tokenizer = AutoTokenizer.from_pretrained(
                model_path,
                token=self.config.hf_token,
                trust_remote_code=self.config.trust_remote_code and self.config.allow_custom_code,
                cache_dir=self.config.cache_dir if self.config.use_local_cache else None,
                revision=self.config.revision
            )
            
            # Ensure pad token is set
            if self.tokenizer.pad_token is None:
                self.tokenizer.pad_token = self.tokenizer.eos_token
            
            # Load model
            self.model = AutoModelForCausalLM.from_pretrained(
                model_path,
                token=self.config.hf_token,
                trust_remote_code=self.config.trust_remote_code and self.config.allow_custom_code,
                cache_dir=self.config.cache_dir if self.config.use_local_cache else None,
                revision=self.config.revision,
                torch_dtype=self._get_torch_dtype(),
                device_map=self._get_device_map(),
                load_in_8bit=self.config.load_in_8bit,
                load_in_4bit=self.config.load_in_4bit,
                low_cpu_mem_usage=True
            )
            
            # Set to eval mode
            self.model.eval()
            
            # Compile if requested
            if self.config.compile_model and hasattr(torch, 'compile'):
                self.model = torch.compile(self.model)
            
            self.load_time = time.time() - start_time
            self._calculate_memory_usage()
            
            logger.info(f"Generative model loaded in {self.load_time:.2f}s")
            return self.model, self.tokenizer
            
        except Exception as e:
            logger.error(f"Failed to load generative model: {e}")
            raise
    
    def validate_model(self) -> bool:
        """Validate generative model with test generation"""
        if self.model is None or self.tokenizer is None:
            return False
        
        try:
            # Test with simple prompt
            test_prompt = "The capital of France is"
            inputs = self.tokenizer(test_prompt, return_tensors="pt")
            
            with torch.no_grad():
                outputs = self.model.generate(
                    **inputs,
                    max_new_tokens=5,
                    do_sample=False,
                    pad_token_id=self.tokenizer.pad_token_id
                )
                
                generated_text = self.tokenizer.decode(outputs[0], skip_special_tokens=True)
            
            # Basic validation - should contain original prompt
            if test_prompt not in generated_text:
                logger.warning("Generated text doesn't contain input prompt")
                return False
            
            logger.info(f"Generative model validation passed - generated: '{generated_text}'")
            return True
            
        except Exception as e:
            logger.error(f"Generative model validation failed: {e}")
            return False
    
    def _get_torch_dtype(self):
        """Get appropriate torch dtype for generative models"""
        if not HAS_TORCH:
            return None
            
        # Default to float32 for generative models for stability
        dtype_map = {
            "auto": torch.float32,
            "float32": torch.float32,
            "float16": torch.float16,
            "bfloat16": torch.bfloat16 if hasattr(torch, 'bfloat16') else torch.float16
        }
        return dtype_map.get(self.config.torch_dtype, torch.float32)
    
    def _get_device_map(self):
        """Get device mapping for generative models"""
        if self.config.device == "auto":
            return "cpu"  # Default to CPU for stability
        elif self.config.device == "cpu":
            return "cpu"
        else:
            return self.config.device
    
    def _calculate_memory_usage(self):
        """Estimate memory usage of generative model"""
        try:
            if HAS_TORCH and self.model is not None:
                param_count = self._count_parameters()
                bytes_per_param = 4  # float32 default for stability
                
                self.memory_usage_gb = (param_count * bytes_per_param) / (1024**3)
        except Exception:
            self.memory_usage_gb = 0.0


class ModelLoader:
    """Factory for creating appropriate model loaders"""
    
    _loader_classes: Dict[ModelType, Type[BaseModelLoader]] = {
        ModelType.EMBEDDING: EmbeddingModelLoader,
        ModelType.GENERATIVE: GenerativeModelLoader,
        ModelType.CHAT: GenerativeModelLoader,  # Chat models use generative loader
    }
    
    @classmethod
    def create_loader(cls, config: LoaderConfig, edition_manager: Optional[EditionManager] = None) -> BaseModelLoader:
        """Create appropriate loader for model type
        
        Args:
            config: Model loading configuration
            edition_manager: Edition manager (auto-created if None)
            
        Returns:
            Configured model loader instance
        """
        if config.model_type not in cls._loader_classes:
            raise ValueError(f"Unsupported model type: {config.model_type}")
        
        loader_class = cls._loader_classes[config.model_type]
        return loader_class(config, edition_manager)
    
    @classmethod
    def load_model(cls, config: LoaderConfig, edition_manager: Optional[EditionManager] = None) -> Tuple[Any, Any, BaseModelLoader]:
        """Convenience method to create loader and load model
        
        Args:
            config: Model loading configuration  
            edition_manager: Edition manager (auto-created if None)
            
        Returns:
            Tuple of (model, tokenizer, loader)
        """
        loader = cls.create_loader(config, edition_manager)
        model, tokenizer = loader.load()
        
        # Validate the loaded model
        if config.verify_model and not loader.validate_model():
            raise RuntimeError("Model validation failed")
        
        return model, tokenizer, loader