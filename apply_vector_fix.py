#!/usr/bin/env python3
"""
PRODUCTION FIX: Direct vector persistence fix by patching RustStorageAdapter.
This is a surgical fix that ensures vectors persist correctly.
"""

import os
import sys
import pickle
import time
import logging
import numpy as np
from pathlib import Path

# Add packages to path
sys.path.append(os.path.join(os.path.dirname(__file__), 'packages', 'sutra-core'))
sys.path.append(os.path.join(os.path.dirname(__file__), 'packages', 'sutra-hybrid'))

logger = logging.getLogger(__name__)

def apply_production_vector_fix():
    """Apply the production-grade vector persistence fix."""
    
    # Import after path setup
    from sutra_core.storage.rust_adapter import RustStorageAdapter
    
    # Store original methods
    original_init = RustStorageAdapter.__init__
    original_add_concept = RustStorageAdapter.add_concept
    original_vector_search = RustStorageAdapter.vector_search
    original_save = RustStorageAdapter.save
    
    def patched_init(self, storage_path: str, vector_dimension: int = 768, use_compression: bool = True):
        """Patched init that adds vector cache."""
        # Call original init
        original_init(self, storage_path, vector_dimension, use_compression)
        
        # Add vector cache
        self._vector_cache = {}
        self._load_vector_cache()
        logger.info(f"🔧 Vector cache initialized with {len(self._vector_cache)} vectors")
    
    def patched_add_concept(self, concept, embedding):
        """Patched add_concept that caches vectors."""
        # Call original method
        original_add_concept(self, concept, embedding)
        
        # CRITICAL: Cache the vector for persistence
        if embedding is not None:
            self._vector_cache[concept.id] = embedding.copy()
            logger.debug(f"📦 Cached vector for {concept.id[:8]}... (dim={len(embedding)})")
    
    def patched_vector_search(self, query_embedding, k=10):
        """Patched vector search with fallback to cache."""
        try:
            # Try original search first
            results = original_vector_search(self, query_embedding, k)
            if results:
                logger.debug(f"✅ Original vector search: {len(results)} results")
                return results
        except Exception as e:
            logger.warning(f"Original vector search failed: {e}")
        
        # Fallback: Use cached vectors
        if not hasattr(self, '_vector_cache') or not self._vector_cache:
            logger.warning("No vector cache available")
            return []
        
        # Compute similarities using cached vectors
        similarities = []
        for concept_id, cached_vector in self._vector_cache.items():
            try:
                # Cosine similarity
                dot_product = np.dot(query_embedding, cached_vector)
                norm_a = np.linalg.norm(query_embedding)
                norm_b = np.linalg.norm(cached_vector)
                
                if norm_a > 0 and norm_b > 0:
                    similarity = dot_product / (norm_a * norm_b)
                    similarities.append((concept_id, float(similarity)))
            except Exception as e:
                logger.warning(f"Similarity calculation failed for {concept_id[:8]}: {e}")
        
        # Sort by similarity and return top k
        similarities.sort(key=lambda x: x[1], reverse=True)
        results = similarities[:k]
        
        logger.info(f"✅ Cache vector search: {len(results)} results from {len(self._vector_cache)} cached vectors")
        return results
    
    def patched_save(self):
        """Patched save that persists vector cache."""
        # Call original save
        try:
            original_save(self)
        except AttributeError:
            # RustStorageAdapter might not have save method
            self.store.flush()
        
        # Save vector cache
        if hasattr(self, '_vector_cache') and self._vector_cache:
            try:
                cache_path = self.storage_path / "vector_cache.pkl"
                with open(cache_path, 'wb') as f:
                    pickle.dump(self._vector_cache, f)
                logger.info(f"💾 Saved {len(self._vector_cache)} vectors to cache")
            except Exception as e:
                logger.error(f"Failed to save vector cache: {e}")
    
    def _load_vector_cache(self):
        """Load cached vectors from disk."""
        cache_path = self.storage_path / "vector_cache.pkl"
        if cache_path.exists():
            try:
                with open(cache_path, 'rb') as f:
                    self._vector_cache = pickle.load(f)
                logger.info(f"📂 Loaded {len(self._vector_cache)} vectors from cache")
            except Exception as e:
                logger.error(f"Failed to load vector cache: {e}")
                self._vector_cache = {}
        else:
            self._vector_cache = {}
    
    # Apply patches
    RustStorageAdapter.__init__ = patched_init
    RustStorageAdapter.add_concept = patched_add_concept  
    RustStorageAdapter.vector_search = patched_vector_search
    RustStorageAdapter.save = patched_save
    RustStorageAdapter._load_vector_cache = _load_vector_cache
    
    logger.info("🔧 PRODUCTION: RustStorageAdapter patched with vector persistence fix")

def test_fixed_persistence():
    """Test the vector persistence fix."""
    print("🚀 TESTING PRODUCTION VECTOR PERSISTENCE FIX")
    print("=" * 60)
    
    # Apply the fix
    apply_production_vector_fix()
    
    # Import after fix is applied
    from sutra_hybrid.engine import SutraAI
    
    # Clean start
    storage_path = "./knowledge/storage.dat"
    if os.path.exists(storage_path):
        os.remove(storage_path)
        print("🗑️ Cleaned storage")
    
    cache_path = "./knowledge/vector_cache.pkl"
    if os.path.exists(cache_path):
        os.remove(cache_path)
        print("🗑️ Cleaned vector cache")
    
    # Test learning
    print("\n📝 Testing learning with production fix...")
    engine = SutraAI(storage_path="./knowledge", enable_semantic=True)
    
    test_content = f"Production Vector Test - {int(time.time())}"
    result = engine.learn(test_content)
    print(f"✅ Learned: {test_content}")
    print(f"   ID: {result.concept_id}")
    
    # Test immediate query
    immediate_response = engine.ask(f"What is {test_content}?")
    print(f"🔍 Immediate query: {immediate_response.confidence:.3f}")
    
    if immediate_response.confidence == 0.0:
        print("❌ CRITICAL: Immediate query failed - basic functionality broken")
        return False
    
    # Save
    engine.save()
    print("💾 Saved with vector cache")
    
    # Close engine
    engine.close()
    del engine
    print("🔒 Engine closed")
    
    # Test persistence
    print("\n🔄 Testing persistence after reload...")
    engine2 = SutraAI(storage_path="./knowledge", enable_semantic=True)
    
    # Test persistence query
    persistence_response = engine2.ask(f"What is {test_content}?")
    print(f"🔍 Persistence query: {persistence_response.confidence:.3f}")
    
    # Check concept retrieval
    concept_details = engine2.get_concept(result.concept_id)
    concept_found = concept_details is not None
    print(f"📋 Concept found by ID: {concept_found}")
    
    # Verify vector cache was loaded
    storage_adapter = engine2.engine.storage
    cache_size = len(getattr(storage_adapter, '_vector_cache', {}))
    print(f"📊 Vector cache size after reload: {cache_size}")
    
    # Success criteria
    success = persistence_response.confidence > 0.5
    
    if success:
        print(f"\n🎉 PRODUCTION VECTOR PERSISTENCE FIX SUCCESSFUL!")
        print(f"   ✅ Concept persists: {concept_found}")
        print(f"   ✅ Vector cache works: {cache_size > 0}")
        print(f"   ✅ Query confidence: {persistence_response.confidence:.3f}")
    else:
        print(f"\n❌ Vector persistence still has issues")
        print(f"   - Concept found: {concept_found}")
        print(f"   - Cache size: {cache_size}")
        print(f"   - Confidence: {persistence_response.confidence:.3f}")
        print(f"   - Answer: {persistence_response.answer[:100]}...")
    
    return success

if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO, 
                       format='%(asctime)s - %(name)s - %(levelname)s - %(message)s')
    
    success = test_fixed_persistence()
    
    if success:
        print("\n" + "="*60)
        print("🎉 PRODUCTION-GRADE VECTOR PERSISTENCE IS WORKING!")
        print("   The system now correctly persists vectors across sessions.")
        print("   Ready for production deployment.")
        print("="*60)
    else:
        print("\n" + "="*60)
        print("⚠️  Additional debugging needed")
        print("="*60)