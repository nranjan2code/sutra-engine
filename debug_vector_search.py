#!/usr/bin/env python3
"""
Debug vector search functionality specifically.
"""

import os
import sys
import time
import numpy as np

# Add packages to path
sys.path.append(os.path.join(os.path.dirname(__file__), 'packages', 'sutra-core'))
sys.path.append(os.path.join(os.path.dirname(__file__), 'packages', 'sutra-hybrid'))

from sutra_hybrid.engine import SutraAI

def debug_vector_search():
    """Debug vector search step by step."""
    print("🔍 VECTOR SEARCH DEBUG")
    print("=" * 50)
    
    # Clean start
    storage_path = "./knowledge/storage.dat"
    if os.path.exists(storage_path):
        os.remove(storage_path)
        print("🗑️ Cleaned storage")
    
    # Learn a concept
    print("\n📝 Learning concept...")
    engine = SutraAI(storage_path="./knowledge", enable_semantic=True)
    
    test_content = f"Vector Debug Test - {int(time.time())}"
    result = engine.learn(test_content)
    print(f"✅ Learned: {test_content}")
    print(f"   ID: {result.concept_id}")
    
    # Check if embedding was stored
    has_embedding = result.concept_id in engine._concept_embeddings
    print(f"📊 Concept has embedding: {has_embedding}")
    if has_embedding:
        embedding_dim = len(engine._concept_embeddings[result.concept_id])
        print(f"   Embedding dimension: {embedding_dim}")
    
    # Test direct vector search on storage
    print("\n🔍 Testing direct vector search...")
    storage = engine.engine.storage
    
    # Generate query embedding
    query_text = f"What is {test_content}?"
    print(f"Query: {query_text}")
    
    try:
        # Get embedding processor
        from sutra_core.learning.adaptive import AdaptiveLearner
        processor = AdaptiveLearner._get_embedding_processor()
        query_embedding = processor.encode_single(query_text, prompt_name="Retrieval-query")
        print(f"✅ Generated query embedding: dim={len(query_embedding)}")
        
        # Try vector search
        vector_results = storage.vector_search(query_embedding, k=5)
        print(f"🎯 Vector search results: {len(vector_results)} matches")
        
        for i, (concept_id, similarity) in enumerate(vector_results):
            concept = storage.get_concept(concept_id)
            content_preview = concept.content[:50] if concept else "None"
            print(f"   {i+1}. ID: {concept_id[:8]}... | Similarity: {similarity:.3f} | Content: {content_preview}...")
        
        # Test with exact content
        content_embedding = processor.encode_single(test_content, prompt_name="Retrieval-document")
        content_results = storage.vector_search(content_embedding, k=5)
        print(f"🎯 Content-based search results: {len(content_results)} matches")
        
        for i, (concept_id, similarity) in enumerate(content_results):
            concept = storage.get_concept(concept_id)
            content_preview = concept.content[:50] if concept else "None"
            print(f"   {i+1}. ID: {concept_id[:8]}... | Similarity: {similarity:.3f} | Content: {content_preview}...")
    
    except Exception as e:
        print(f"❌ Vector search failed: {e}")
    
    # Test full query processing
    print("\n🔍 Testing full query processing...")
    try:
        # Get the query processor directly
        query_processor = engine.engine.query_processor
        
        # Test concept finding
        relevant_concepts = query_processor._find_relevant_concepts(query_text.lower(), 5)
        print(f"📋 Relevant concepts found: {len(relevant_concepts)}")
        
        for i, (concept_id, score) in enumerate(relevant_concepts):
            concept = storage.get_concept(concept_id)
            content_preview = concept.content[:50] if concept else "None"
            print(f"   {i+1}. ID: {concept_id[:8]}... | Score: {score:.3f} | Content: {content_preview}...")
    
    except Exception as e:
        print(f"❌ Query processing failed: {e}")
    
    # Save and test persistence
    print("\n💾 Testing persistence...")
    engine.save()
    engine.close()
    del engine
    print("🗑️ Engine destroyed")
    
    # Reload and test
    print("\n🔄 Reloading engine...")
    engine2 = SutraAI(storage_path="./knowledge", enable_semantic=True)
    
    # Check storage stats
    stats = engine2.engine.storage.stats()
    print(f"📊 Reloaded stats: {stats}")
    
    # Check embeddings
    embedding_count = len(engine2._concept_embeddings)
    print(f"📊 Embeddings loaded: {embedding_count}")
    
    # Test concept retrieval
    concept_details = engine2.get_concept(result.concept_id)
    if concept_details:
        print(f"✅ Concept found by ID: {concept_details['content'][:50]}...")
    else:
        print(f"❌ Concept NOT found by ID: {result.concept_id}")
    
    # Test vector search after reload
    print("\n🔍 Testing vector search after reload...")
    try:
        processor2 = AdaptiveLearner._get_embedding_processor()
        query_embedding2 = processor2.encode_single(query_text, prompt_name="Retrieval-query")
        
        vector_results2 = engine2.engine.storage.vector_search(query_embedding2, k=5)
        print(f"🎯 Post-reload vector search: {len(vector_results2)} matches")
        
        for i, (concept_id, similarity) in enumerate(vector_results2):
            concept = engine2.engine.storage.get_concept(concept_id)
            content_preview = concept.content[:50] if concept else "None"
            print(f"   {i+1}. ID: {concept_id[:8]}... | Similarity: {similarity:.3f} | Content: {content_preview}...")
    
    except Exception as e:
        print(f"❌ Post-reload vector search failed: {e}")
    
    # Final query test
    print(f"\n🔍 Final query test...")
    response = engine2.ask(query_text)
    print(f"   Query: {query_text}")
    print(f"   Confidence: {response.confidence:.3f}")
    print(f"   Answer: {response.answer[:100]}...")
    
    return {
        'concept_stored': concept_details is not None,
        'embeddings_loaded': embedding_count > 0,
        'vector_search_works': len(vector_results2) > 0 if 'vector_results2' in locals() else False,
        'final_confidence': response.confidence
    }

if __name__ == "__main__":
    results = debug_vector_search()
    print("\n📋 SUMMARY")
    print("=" * 50)
    for key, value in results.items():
        print(f"  {key}: {value}")