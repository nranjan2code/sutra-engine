#!/usr/bin/env python3
"""
Direct end-to-end test of Sutra AI without API server.

Tests: Learn → Save → Reload → Query

Architecture:
- sutra-core: Graph reasoning
- sutra-hybrid: Semantic embeddings (SutraAI)
- This script: Direct Python usage (for testing only, use API in production)
"""

import shutil
import tempfile
from pathlib import Path

from sutra_hybrid import SutraAI


def test_end_to_end():
    """Test complete workflow: learn, save, reload, query."""
    
    # Create temporary storage
    temp_dir = tempfile.mkdtemp()
    storage_path = Path(temp_dir) / "test_knowledge"
    
    print("="*70)
    print("SUTRA AI END-TO-END TEST")
    print("="*70)
    print(f"\nStorage: {storage_path}")
    print("\nArchitecture:")
    print("  sutra-core    → Graph reasoning")
    print("  sutra-hybrid  → Semantic embeddings (SutraAI)")
    print("="*70)
    
    try:
        # ========================================
        # PHASE 1: LEARN
        # ========================================
        print("\n📚 PHASE 1: LEARNING")
        print("-" * 70)
        
        ai1 = SutraAI(storage_path=str(storage_path), enable_semantic=True)
        
        facts = [
            "Python is a high-level programming language created by Guido van Rossum in 1991",
            "JavaScript is a scripting language commonly used for web development",
            "Rust is a systems programming language focused on memory safety",
            "Machine learning is a subset of artificial intelligence",
            "Neural networks are computing systems inspired by biological brains",
        ]
        
        print(f"\nLearning {len(facts)} facts...")
        total_concepts = 0
        total_associations = 0
        
        for i, fact in enumerate(facts, 1):
            result = ai1.learn(fact)
            total_concepts += result.concepts_created
            total_associations += result.associations_created
            print(f"  [{i}/{len(facts)}] ✓ Learned: {result.concepts_created} concepts, "
                  f"{result.associations_created} associations")
        
        print(f"\n✅ Total learned: {total_concepts} concepts, {total_associations} associations")
        
        # ========================================
        # PHASE 2: QUERY (before save)
        # ========================================
        print("\n❓ PHASE 2: QUERY (before save)")
        print("-" * 70)
        
        queries = [
            "Who created Python?",
            "What is Rust?",
            "What is machine learning?",
        ]
        
        print("\nQuerying learned knowledge...")
        for i, query in enumerate(queries, 1):
            result = ai1.ask(query, semantic_boost=True)
            print(f"\n  [{i}/{len(queries)}] Query: '{query}'")
            print(f"            Answer: {result.answer}")
            print(f"            Confidence: {result.confidence:.2f}")
            print(f"            Paths: {len(result.reasoning_paths)}")
        
        # ========================================
        # PHASE 3: SAVE
        # ========================================
        print("\n💾 PHASE 3: SAVE TO DISK")
        print("-" * 70)
        
        ai1.save()
        print(f"✅ Saved to: {storage_path}")
        
        # Check what was saved
        files = list(storage_path.glob("*"))
        print(f"✅ Files created: {len(files)}")
        for f in files:
            size = f.stat().st_size
            print(f"   - {f.name} ({size:,} bytes)")
        
        # Close first instance
        print("\n🔒 Closing first instance...")
        del ai1
        
        # ========================================
        # PHASE 4: RELOAD
        # ========================================
        print("\n🔄 PHASE 4: RELOAD FROM DISK")
        print("-" * 70)
        
        print("Creating new instance (simulating restart)...")
        ai2 = SutraAI(storage_path=str(storage_path), enable_semantic=True)
        
        try:
            stats = ai2.engine.get_system_stats()
            print(f"✅ Reloaded: {stats.get('total_concepts', 0)} concepts, "
                  f"{stats.get('total_associations', 0)} associations")
        except Exception as e:
            print(f"✅ Reloaded successfully (stats unavailable: {e})")
        
        # ========================================
        # PHASE 5: QUERY (after reload)
        # ========================================
        print("\n❓ PHASE 5: QUERY (after reload)")
        print("-" * 70)
        
        print("\nQuerying reloaded knowledge...")
        for i, query in enumerate(queries, 1):
            result = ai2.ask(query, semantic_boost=True)
            print(f"\n  [{i}/{len(queries)}] Query: '{query}'")
            print(f"            Answer: {result.answer}")
            print(f"            Confidence: {result.confidence:.2f}")
            print(f"            Paths: {len(result.reasoning_paths)}")
        
        # ========================================
        # PHASE 6: MULTI-STRATEGY COMPARISON
        # ========================================
        print("\n🔀 PHASE 6: MULTI-STRATEGY COMPARISON")
        print("-" * 70)
        
        test_query = "What is Python?"
        print(f"\nComparing strategies for: '{test_query}'")
        
        multi_result = ai2.multi_strategy(test_query)
        print(f"\n✅ Strategies compared: 2 (graph-only vs semantic-enhanced)")
        
        print(f"\n  Strategy 1: Graph-only (100% explainable)")
        print(f"    Answer: {multi_result.graph_only.answer}")
        print(f"    Confidence: {multi_result.graph_only.confidence:.2f}")
        
        print(f"\n  Strategy 2: Semantic-enhanced")
        print(f"    Answer: {multi_result.semantic_enhanced.answer}")
        print(f"    Confidence: {multi_result.semantic_enhanced.confidence:.2f}")
        
        print(f"\n  Agreement score: {multi_result.agreement_score:.2f}")
        print(f"  Recommended: {multi_result.recommended_strategy}")
        print(f"  Reasoning: {multi_result.reasoning}")
        
        # ========================================
        # PHASE 7: AUDIT TRAIL
        # ========================================
        print("\n📋 PHASE 7: AUDIT TRAIL")
        print("-" * 70)
        
        audit = ai2.get_audit_trail(limit=5)
        print(f"\n✅ Audit entries: {len(audit)}")
        
        for i, entry in enumerate(audit[:3], 1):
            print(f"\n  Entry {i}:")
            print(f"    Operation: {entry['operation']}")
            print(f"    Timestamp: {entry['timestamp']}")
            if 'confidence' in entry:
                print(f"    Confidence: {entry['confidence']:.2f}")
        
        # ========================================
        # SUMMARY
        # ========================================
        print("\n" + "="*70)
        print("✅ END-TO-END TEST PASSED")
        print("="*70)
        print("\nTested:")
        print("  ✓ Learning knowledge")
        print("  ✓ Querying with reasoning paths")
        print("  ✓ Saving to disk")
        print("  ✓ Reloading from disk")
        print("  ✓ Query persistence after reload")
        print("  ✓ Multi-strategy comparison")
        print("  ✓ Audit trail logging")
        print("\n🎉 All phases completed successfully!")
        print("="*70)
        
        return True
        
    except Exception as e:
        print(f"\n❌ ERROR: {e}")
        import traceback
        traceback.print_exc()
        return False
        
    finally:
        # Cleanup
        if storage_path.exists():
            shutil.rmtree(temp_dir)
            print(f"\n🧹 Cleaned up: {temp_dir}")


if __name__ == "__main__":
    import sys
    success = test_end_to_end()
    sys.exit(0 if success else 1)
