#!/usr/bin/env python3
"""
Test cross-session persistence to identify the reload problem.
"""

import time
import sys
import os

# Add packages to path
sys.path.append(os.path.join(os.path.dirname(__file__), 'packages', 'sutra-core'))
sys.path.append(os.path.join(os.path.dirname(__file__), 'packages', 'sutra-hybrid'))

from sutra_hybrid.engine import SutraAI

def test_persistence():
    print("🔍 TESTING CROSS-SESSION PERSISTENCE")
    print("=" * 50)
    
    # Session 1: Learn some concepts
    print("📝 SESSION 1: Learning concepts...")
    engine1 = SutraAI(storage_path="./knowledge", enable_semantic=True)
    
    initial_stats = engine1.engine.storage.stats()
    print(f"Initial storage stats: {initial_stats}")
    
    # Learn 3 unique concepts with timestamps
    timestamp = int(time.time())
    concepts = [
        f"Persistence Test Alpha - {timestamp}",
        f"Persistence Test Beta - {timestamp}", 
        f"Persistence Test Gamma - {timestamp}"
    ]
    
    learned_ids = []
    for i, concept in enumerate(concepts):
        result = engine1.learn(concept)
        learned_ids.append(result.concept_id)
        print(f"  Learned: {concept[:30]}... -> ID: {result.concept_id[:8]}")
    
    # Test immediate queries
    print("\n🔍 Testing immediate queries...")
    for concept in concepts:
        response = engine1.ask(f"What is {concept}?")
        print(f"  Query '{concept[:20]}...': confidence={response.confidence:.3f}")
    
    session1_stats = engine1.engine.storage.stats()
    print(f"\nSession 1 final stats: {session1_stats}")
    
    # Force save
    engine1.save()
    print("✅ Storage saved")
    
    # Clear the first engine
    del engine1
    print("🗑️ Engine 1 destroyed")
    
    # Wait a moment
    time.sleep(2)
    
    # Session 2: Create new engine and test persistence
    print("\n" + "=" * 50)
    print("📖 SESSION 2: Testing persistence...")
    engine2 = SutraAI(storage_path="./knowledge", enable_semantic=True)
    
    session2_stats = engine2.engine.storage.stats()
    print(f"Session 2 initial stats: {session2_stats}")
    
    # Compare concept counts
    count_diff = session2_stats['total_concepts'] - initial_stats['total_concepts']
    print(f"Concept count change: {count_diff} ({initial_stats['total_concepts']} -> {session2_stats['total_concepts']})")
    
    # Test queries for the learned concepts
    print("\n🔍 Testing persistence queries...")
    persistent_queries = 0
    for concept in concepts:
        response = engine2.ask(f"What is {concept}?")
        is_persistent = response.confidence > 0.5
        status = "✅" if is_persistent else "❌"
        print(f"  {status} Query '{concept[:20]}...': confidence={response.confidence:.3f}")
        if is_persistent:
            persistent_queries += 1
    
    persistence_rate = (persistent_queries / len(concepts)) * 100
    print(f"\n📊 PERSISTENCE RESULTS:")
    print(f"  Persistent queries: {persistent_queries}/{len(concepts)}")
    print(f"  Persistence rate: {persistence_rate:.1f}%")
    print(f"  Concept count stable: {'✅' if count_diff >= 0 else '❌'}")
    
    # Detailed analysis
    print(f"\n📋 DETAILED ANALYSIS:")
    print(f"  Initial concepts: {initial_stats['total_concepts']}")
    print(f"  After learning: {session1_stats['total_concepts']}")
    print(f"  After reload: {session2_stats['total_concepts']}")
    print(f"  Expected increase: {len(concepts)}")
    print(f"  Actual increase: {count_diff}")
    
    if persistence_rate < 100:
        print(f"\n💥 PERSISTENCE PROBLEM DETECTED!")
        print(f"  Issue: Only {persistence_rate:.1f}% of concepts persist across sessions")
        print(f"  Root cause: Likely dummy data loader overwriting real data")
    else:
        print(f"\n✅ PERSISTENCE WORKING CORRECTLY!")
    
    return {
        'persistence_rate': persistence_rate,
        'count_stable': count_diff >= 0,
        'concepts_learned': len(concepts),
        'concepts_persistent': persistent_queries
    }

if __name__ == "__main__":
    results = test_persistence()
    print("\n" + "=" * 50)
    print("📋 FINAL RESULTS")
    print("=" * 50)
    for key, value in results.items():
        print(f"  {key}: {value}")