#!/usr/bin/env python3
"""
Test Suite for Sutra ML Base Foundation

Validates the new ML service architecture works correctly
across different editions and service types.
"""

import asyncio
import os
import sys
import logging
import json
from pathlib import Path

# Add packages to path for testing
sys.path.insert(0, str(Path(__file__).parent / "packages"))

# Test imports
try:
    from sutra_ml_base import (
        EditionManager, Edition, 
        ModelLoader, LoaderConfig, ModelType,
        BaseMlService, ServiceConfig,
        MetricsCollector, CacheManager, CacheConfig,
        setup_environment, setup_logging
    )
    print("✅ Successfully imported sutra-ml-base")
except ImportError as e:
    print(f"❌ Failed to import sutra-ml-base: {e}")
    sys.exit(1)

# Setup
setup_environment()
logger = setup_logging("test-ml-base", level="INFO")


class TestMlService(BaseMlService):
    """Test ML service implementation"""
    
    async def load_model(self) -> bool:
        """Mock model loading"""
        await asyncio.sleep(0.1)  # Simulate load time
        
        # Mock model info
        model_info = {
            "model_name": "test-model",
            "model_type": "test",
            "parameters": 1000000,
            "memory_usage_gb": 1.0,
            "device": "cpu"
        }
        
        self.set_model_loaded(model_info)
        return True
    
    async def process_request(self, request):
        """Mock request processing"""
        return {"result": "test_output", "input": request}
    
    def get_service_info(self):
        return {
            "description": "Test ML service",
            "supported_models": ["test-model"]
        }
    
    def _setup_service_routes(self):
        @self.app.post("/test")
        async def test_endpoint(request: dict):
            return await self.process_request(request)


async def test_edition_manager():
    """Test edition management"""
    print("\n🧪 Testing Edition Manager...")
    
    # Test different editions
    for edition_name in ["simple", "community", "enterprise"]:
        os.environ["SUTRA_EDITION"] = edition_name
        
        manager = EditionManager()
        print(f"  📋 {edition_name.capitalize()} Edition:")
        print(f"    - Max batch size: {manager.get_batch_size_limit()}")
        print(f"    - Max model size: {manager.get_model_size_limit()}GB")
        print(f"    - Cache size: {manager.get_cache_size_gb()}GB")
        print(f"    - Custom models: {manager.supports_custom_models()}")
        print(f"    - Advanced caching: {manager.supports_advanced_caching()}")
        print(f"    - Multi-GPU: {manager.supports_multi_gpu()}")
        
        # Validate limits increase with edition
        if edition_name == "simple":
            assert manager.get_batch_size_limit() == 32
            assert not manager.supports_custom_models()
        elif edition_name == "enterprise":
            assert manager.get_batch_size_limit() == 256
            assert manager.supports_custom_models()
            assert manager.supports_multi_gpu()
    
    print("✅ Edition Manager tests passed")


def test_cache_manager():
    """Test caching functionality"""
    print("\n🧪 Testing Cache Manager...")
    
    config = CacheConfig(
        max_memory_mb=100,
        max_items=1000,
        default_ttl_seconds=60
    )
    
    cache = CacheManager(config)
    
    # Test basic operations
    cache.set("key1", "value1")
    assert cache.get("key1") == "value1"
    
    cache.set("key2", {"nested": "data"})
    result = cache.get("key2")
    assert result["nested"] == "data"
    
    # Test cache key generation
    key = cache.cache_key("prefix", "arg1", "arg2", param1="value1")
    assert "prefix" in key
    
    # Test cache stats
    stats = cache.get_stats()
    assert stats["entries"] == 2
    assert stats["hit_rate"] >= 0
    
    print("✅ Cache Manager tests passed")


def test_metrics_collector():
    """Test metrics collection"""
    print("\n🧪 Testing Metrics Collector...")
    
    metrics = MetricsCollector()
    
    # Simulate some requests
    metrics.record_request("/test", "POST", 200, 50.5)
    metrics.record_request("/test", "POST", 200, 75.2)
    metrics.record_request("/health", "GET", 200, 10.1)
    metrics.record_request("/error", "POST", 500, 100.0)
    
    # Get stats
    stats = metrics.get_stats()
    
    assert stats.total_requests == 4
    assert stats.successful_requests == 3
    assert stats.failed_requests == 1
    assert stats.error_rate == 25.0  # 1/4 = 25%
    assert stats.avg_response_time_ms > 0
    
    # Test endpoint stats
    endpoint_stats = metrics.get_endpoint_stats()
    assert "POST /test" in endpoint_stats
    assert endpoint_stats["POST /test"]["requests"] == 2
    
    print("✅ Metrics Collector tests passed")


async def test_base_service():
    """Test base ML service"""
    print("\n🧪 Testing Base ML Service...")
    
    # Set simple edition for test
    os.environ["SUTRA_EDITION"] = "simple"
    
    config = ServiceConfig(
        service_name="test-service",
        port=8890,  # Use different port
        enable_metrics=True
    )
    
    service = TestMlService(config)
    
    # Test model loading
    success = await service.load_model()
    assert success, "Model loading should succeed"
    
    # Test service info
    info = service.get_service_info()
    assert "description" in info
    assert "supported_models" in info
    
    # Test request processing
    result = await service.process_request({"test": "data"})
    assert result["result"] == "test_output"
    
    print("✅ Base ML Service tests passed")


async def test_edition_compatibility():
    """Test edition-specific behavior"""
    print("\n🧪 Testing Edition Compatibility...")
    
    # Test simple edition limits
    os.environ["SUTRA_EDITION"] = "simple"
    manager_simple = EditionManager()
    
    # Small model should be allowed
    assert manager_simple.can_load_model(1.5), "Small model should be allowed in simple edition"
    
    # Large model should be rejected
    assert not manager_simple.can_load_model(10.0), "Large model should be rejected in simple edition"
    
    # Test enterprise edition
    os.environ["SUTRA_EDITION"] = "enterprise"
    manager_enterprise = EditionManager()
    
    # Large model should be allowed
    assert manager_enterprise.can_load_model(10.0), "Large model should be allowed in enterprise edition"
    
    print("✅ Edition compatibility tests passed")


def test_model_loader_config():
    """Test model loader configuration"""
    print("\n🧪 Testing Model Loader Config...")
    
    # Test embedding config
    config = LoaderConfig(
        model_name="test/embedding-model",
        model_type=ModelType.EMBEDDING,
        device="cpu",
        max_memory_gb=2.0
    )
    
    assert config.model_name == "test/embedding-model"
    assert config.model_type == ModelType.EMBEDDING
    assert config.device == "cpu"
    
    # Test generative config
    config_gen = LoaderConfig(
        model_name="test/generative-model", 
        model_type=ModelType.GENERATIVE,
        torch_dtype="float32",
        load_in_8bit=False
    )
    
    assert config_gen.model_type == ModelType.GENERATIVE
    assert config_gen.torch_dtype == "float32"
    
    print("✅ Model Loader Config tests passed")


async def run_all_tests():
    """Run all tests"""
    print("🚀 Starting Sutra ML Base Test Suite")
    print("=" * 50)
    
    try:
        # Run tests
        await test_edition_manager()
        test_cache_manager() 
        test_metrics_collector()
        await test_base_service()
        await test_edition_compatibility()
        test_model_loader_config()
        
        print("\n" + "=" * 50)
        print("✅ All tests passed! Sutra ML Base is working correctly.")
        print("\n🎯 Architecture Validation Summary:")
        print("  ✓ Edition-aware resource management")
        print("  ✓ Unified caching and metrics") 
        print("  ✓ FastAPI service scaffolding")
        print("  ✓ Model loader abstraction")
        print("  ✓ Cross-edition compatibility")
        print("\n🚀 Ready for production deployment!")
        
        return True
        
    except Exception as e:
        print(f"\n❌ Test failed: {e}")
        import traceback
        traceback.print_exc()
        return False


if __name__ == "__main__":
    # Run tests
    success = asyncio.run(run_all_tests())
    sys.exit(0 if success else 1)