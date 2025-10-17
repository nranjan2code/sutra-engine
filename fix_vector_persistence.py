#!/usr/bin/env python3
"""
PRODUCTION FIX: Vector Persistence via Python Layer
Since Rust rebuild has linking issues, we'll fix it at the Python level.
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

from sutra_hybrid.engine import SutraAI
from sutra_core.learning.adaptive import AdaptiveLearner

logger = logging.getLogger(__name__)

def create_vector_sync_storage():
    """
    PRODUCTION: Create a wrapper that properly syncs vectors between 
    Python embeddings and Rust storage on every save/load cycle.
    """
    
    class VectorSyncStorage:
        """Production wrapper that ensures vectors persist correctly."""
        
        def __init__(self, original_storage, embeddings_dict):
            self.original_storage = original_storage
            self.embeddings_dict = embeddings_dict
            self.vector_cache = {}
            
        def learn_concept(self, concept_id, content, embedding, strength, confidence):
            """Learn concept and cache vector for persistence."""
            # Store in original storage
            result = self.original_storage.learn_concept(
                concept_id, content, embedding, strength, confidence
            )
            
            # CRITICAL: Cache vector for persistence
            if embedding is not None:
                self.vector_cache[concept_id] = embedding.copy()
                logger.debug(f"Cached vector for {concept_id[:8]}... (dim={len(embedding)})")
            
            return result
            
        def vector_search(self, query_embedding, k=10):
            """Vector search with fallback to cached vectors."""
            try:
                # Try original storage first
                results = self.original_storage.vector_search(query_embedding, k)
                if results:
                    return results
            except Exception as e:
                logger.warning(f"Original vector search failed: {e}")
            
            # Fallback: use cached vectors with brute force search
            if not self.vector_cache:
                logger.warning("No cached vectors available for search")
                return []
            
            similarities = []
            for concept_id, vector in self.vector_cache.items():
                # Cosine similarity
                dot_product = np.dot(query_embedding, vector)
                norm_a = np.linalg.norm(query_embedding)
                norm_b = np.linalg.norm(vector)
                similarity = dot_product / (norm_a * norm_b) if (norm_a * norm_b) > 0 else 0.0
                similarities.append((concept_id, similarity))
            
            # Sort by similarity and return top k
            similarities.sort(key=lambda x: x[1], reverse=True)
            results = similarities[:k]
            logger.info(f"✅ Fallback vector search: {len(results)} results from cache")
            return results
        
        def save(self):
            """Save with vector persistence."""
            # Save original storage
            self.original_storage.save()
            
            # CRITICAL: Save cached vectors to ensure persistence
            if self.vector_cache:
                storage_path = Path(self.original_storage.storage_path)
                vector_cache_path = storage_path / "vector_cache.pkl"
                
                try:
                    with open(vector_cache_path, 'wb') as f:
                        pickle.dump(self.vector_cache, f)
                    logger.info(f"💾 Saved {len(self.vector_cache)} vectors to cache")
                except Exception as e:
                    logger.error(f"Failed to save vector cache: {e}")
        
        def load_vector_cache(self):
            """Load cached vectors on initialization."""
            storage_path = Path(self.original_storage.storage_path)
            vector_cache_path = storage_path / "vector_cache.pkl"
            
            if vector_cache_path.exists():
                try:
                    with open(vector_cache_path, 'rb') as f:
                        self.vector_cache = pickle.load(f)
                    logger.info(f"📂 Loaded {len(self.vector_cache)} vectors from cache")
                    
                    # Verify dimensions
                    if self.vector_cache:
                        sample_vector = next(iter(self.vector_cache.values()))
                        expected_dim = getattr(self.original_storage, 'vector_dimension', 768)
                        if len(sample_vector) != expected_dim:
                            logger.warning(f"Vector dimension mismatch: {len(sample_vector)} vs {expected_dim}")
                    
                except Exception as e:
                    logger.error(f"Failed to load vector cache: {e}")
                    self.vector_cache = {}
            else:
                logger.info("No vector cache found - starting fresh")
                self.vector_cache = {}
        
        def __getattr__(self, name):
            """Delegate all other methods to original storage."""
            return getattr(self.original_storage, name)
    
    return VectorSyncStorage

def patch_sutra_ai():
    """Patch SutraAI to use vector sync storage."""
    
    original_init = SutraAI.__init__
    
    def patched_init(self, storage_path="./knowledge", enable_semantic=True):
        # Call original init
        original_init(self, storage_path, enable_semantic)
        
        # CRITICAL: Wrap storage with vector sync
        VectorSyncStorage = create_vector_sync_storage()
        sync_storage = VectorSyncStorage(self.engine.storage, self._concept_embeddings)
        
        # Load cached vectors
        sync_storage.load_vector_cache()
        
        # Replace storage
        self.engine.storage = sync_storage
        
        # Replace query processor storage reference
        self.engine.query_processor.storage = sync_storage
        
        logger.info("✅ PRODUCTION: Vector sync storage activated")
    
    # Apply patch
    SutraAI.__init__ = patched_init
    logger.info("🔧 PRODUCTION PATCH: SutraAI patched with vector persistence fix")

def test_vector_persistence_fix():
    """Test the vector persistence fix."""
    print("🔧 PRODUCTION VECTOR PERSISTENCE FIX")
    print("=" * 50)
    
    # Apply patch
    patch_sutra_ai()
    
    # Clean start
    storage_path = "./knowledge/storage.dat"
    if os.path.exists(storage_path):
        os.remove(storage_path)
        print("🗑️ Cleaned storage")
    
    # Test learning
    print("\n📝 Testing learning with fix...")
    engine = SutraAI(storage_path="./knowledge", enable_semantic=True)
    
    test_content = f"Vector Fix Test - {int(time.time())}"
    result = engine.learn(test_content)
    print(f"✅ Learned: {test_content}")
    print(f"   ID: {result.concept_id}")
    
    # Test immediate search
    immediate_response = engine.ask(f"What is {test_content}?")
    print(f"🔍 Immediate query: {immediate_response.confidence:.3f}")
    
    # Save and close
    engine.save()
    engine.close()
    del engine
    print("💾 Saved and closed")
    
    # Test persistence
    print("\n🔄 Testing persistence...")
    engine2 = SutraAI(storage_path="./knowledge", enable_semantic=True)
    
    persistence_response = engine2.ask(f"What is {test_content}?")
    print(f"🔍 Persistence query: {persistence_response.confidence:.3f}")
    
    # Check concept retrieval
    concept_details = engine2.get_concept(result.concept_id)
    concept_found = concept_details is not None
    print(f"📋 Concept found by ID: {concept_found}")
    
    if persistence_response.confidence > 0.5:
        print("\n✅ VECTOR PERSISTENCE FIX SUCCESSFUL!")
        return True
    else:
        print(f"\n❌ Vector persistence still has issues")
        print(f"   Answer: {persistence_response.answer}")
        return False

if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO)
    success = test_vector_persistence_fix()
    if success:
        print("\n🎉 READY FOR PRODUCTION - Vector persistence fixed!")
    else:
        print("\n⚠️ Additional fixes needed")