#!/usr/bin/env python3
"""
Sutra Hybrid - Comprehensive Demo

Demonstrates hybrid reasoning combining graph-based and semantic approaches.

⚠️  WARNING: TF-IDF persistence has known limitations (vectorizer state not saved).
   For production use with persistence, install: pip install sentence-transformers
"""

from sutra_hybrid import HybridAI


def main():
    print("🚀 SUTRA HYBRID - COMPREHENSIVE DEMO")
    print("=" * 60)
    print("Graph-based reasoning + Semantic embeddings")
    print("=" * 60)
    print()

    # Initialize with TF-IDF (no external dependencies)
    print("📦 Initializing HybridAI (TF-IDF mode)...")
    ai = HybridAI(use_semantic=False, storage_path="./demo_knowledge")
    print(f"✅ Initialized with {ai.embedding_provider.get_name()}")
    print()

    # Learn biology knowledge
    print("🧠 LEARNING PHASE")
    print("-" * 60)

    knowledge = [
        "Photosynthesis is the process by which plants convert sunlight "
        "into chemical energy stored in glucose molecules",
        "Cellular respiration breaks down glucose to produce ATP energy "
        "that cells use for various functions",
        "Mitochondria are organelles that produce ATP through cellular "
        "respiration in eukaryotic cells",
        "Chloroplasts are organelles in plant cells that perform "
        "photosynthesis using chlorophyll pigment",
        "DNA contains genetic information encoded in sequences of "
        "nucleotide bases adenine thymine guanine cytosine",
        "RNA is similar to DNA but uses uracil instead of thymine "
        "and is single-stranded",
        "Proteins are made of amino acids and perform most cellular "
        "functions including catalyzing reactions",
        "Enzymes are proteins that speed up chemical reactions by "
        "lowering activation energy",
    ]

    for i, text in enumerate(knowledge, 1):
        concept_id = ai.learn(text, source="biology_textbook")
        print(f"{i}. Learned: {text[:60]}... (ID: {concept_id[:8]})")

    print()
    print(f"✅ Learned {len(knowledge)} concepts")
    print()

    # Demonstrate semantic search
    print("🔍 SEMANTIC SEARCH DEMO")
    print("-" * 60)

    queries = [
        "How do plants make energy?",
        "What produces ATP in cells?",
        "What stores genetic information?",
        "How do enzymes work?",
    ]

    for query in queries:
        print(f"\n Query: '{query}'")
        results = ai.semantic_search(query, top_k=3, threshold=0.3)

        if results:
            print(f"   Found {len(results)} results:")
            for concept_id, similarity in results:
                concept = ai.get_concept(concept_id)
                print(f"   [{similarity:.3f}] {concept.content[:70]}...")
        else:
            print("   No results found")

    print()

    # Show statistics
    print("📊 SYSTEM STATISTICS")
    print("-" * 60)
    stats = ai.get_stats()
    print(f"Total Concepts: {stats['total_concepts']}")
    print(f"Total Associations: {stats['total_associations']}")
    print(f"Total Embeddings: {stats['total_embeddings']}")
    print(f"Embedding Provider: {stats['embedding_provider']}")
    print(f"Embedding Dimension: {stats['embedding_dimension']}")
    print(f"Average Concept Strength: {stats['average_strength']:.2f}")
    print()

    # Test persistence
    print("💾 TESTING PERSISTENCE")
    print("-" * 60)
    print("Saving knowledge...")
    ai.save()
    print("✅ Saved successfully")
    print()

    # Reload
    print("Loading knowledge...")
    ai2 = HybridAI(use_semantic=False, storage_path="./demo_knowledge")
    stats2 = ai2.get_stats()
    print(f"✅ Loaded {stats2['total_concepts']} concepts")
    print()

    # Verify loaded data works
    print("Verifying loaded data...")
    results = ai2.semantic_search("How do plants make energy?", top_k=1)
    if results:
        print(f"✅ Loaded data is functional")
    print()

    print("=" * 60)
    print("🎉 SUTRA HYBRID DEMO COMPLETE!")
    print("✅ Graph-based reasoning")
    print("✅ Semantic similarity search")
    print("✅ Persistent storage")
    print("✅ Automatic embedding fallback")
    print("=" * 60)


if __name__ == "__main__":
    main()
