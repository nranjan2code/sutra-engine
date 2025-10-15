#!/usr/bin/env python3
"""
Sutra AI - Advanced Reasoning Demo

This demo showcases the sophisticated AI reasoning capabilities that make
Sutra AI a genuine alternative to traditional LLMs:

- Natural language query processing
- Multi-path reasoning with consensus
- Explainable AI with confidence scoring
- Real-time learning and knowledge integration
- Advanced path-finding and inference
"""

import logging
import sys
from pathlib import Path

# Add sutra_core to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from sutra_core import ReasoningEngine

# Setup logging
logging.basicConfig(level=logging.INFO, format="%(message)s")
logger = logging.getLogger(__name__)


def setup_biological_knowledge(engine: ReasoningEngine) -> None:
    """Load comprehensive biological knowledge for AI reasoning demo."""

    print("🧬 Loading biological knowledge base...")

    # Core biological concepts
    knowledge_base = [
        "Photosynthesis is the process by which plants convert sunlight into chemical energy using chlorophyll",
        "Mitochondria are the powerhouses of cells that produce ATP through cellular respiration",
        "DNA stores genetic information in a double helix structure made of nucleotides",
        "Ribosomes are cellular structures that synthesize proteins by translating mRNA",
        "Chloroplasts contain chlorophyll and are responsible for photosynthesis in plant cells",
        "ATP provides immediate energy for cellular processes and is produced in mitochondria",
        "Enzymes are biological catalysts that speed up chemical reactions in living organisms",
        "Cell membranes control what enters and exits cells through selective permeability",
        "Nucleus controls cell activities and contains the cell's DNA and genetic material",
        "Cytoplasm is the gel-like substance that fills cells and contains organelles",
        # Relationships and processes
        "Sunlight causes photosynthesis in plant leaves and chloroplasts",
        "Glucose is a product of photosynthesis and provides energy for cellular respiration",
        "Oxygen is released during photosynthesis and required for cellular respiration",
        "Carbon dioxide is consumed during photosynthesis and produced during respiration",
        "Mitochondria use glucose and oxygen to produce ATP through cellular respiration",
        "DNA contains genes that code for proteins synthesized by ribosomes",
        "Enzymes lower activation energy and increase the rate of biochemical reactions",
        "Cell membrane is similar to a selective barrier that regulates molecular transport",
        "Nucleus is a control center that directs all cellular activities and reproduction",
        "Photosynthesis occurs before cellular respiration in the energy cycle of life",
        # Advanced relationships
        "Photosynthesis and cellular respiration are complementary processes in energy metabolism",
        "Chlorophyll absorbs light energy and converts it to chemical energy during photosynthesis",
        "ATP synthesis occurs in both chloroplasts during photosynthesis and mitochondria during respiration",
        "Genetic information flows from DNA to RNA to proteins in the central dogma of biology",
        "Enzyme specificity ensures that each enzyme catalyzes only specific biochemical reactions",
    ]

    # Learn all knowledge with source attribution
    learned_count = 0
    for knowledge in knowledge_base:
        engine.learn(knowledge, source="biology_textbook")
        learned_count += 1

    print(f"   ✅ Learned {learned_count} biological concepts and relationships")

    # Show system statistics
    stats = engine.get_system_stats()
    print(
        f"   📊 Total: {stats['system_info']['total_concepts']} concepts, "
        f"{stats['system_info']['total_associations']} associations"
    )


def demonstrate_ai_reasoning(engine: ReasoningEngine) -> None:
    """Demonstrate advanced AI reasoning capabilities."""

    print("\n" + "=" * 70)
    print("🧠 ADVANCED AI REASONING DEMONSTRATION")
    print("=" * 70)

    # Test queries of increasing complexity
    test_queries = [
        {
            "question": "What is photosynthesis?",
            "complexity": "Simple Definition",
            "expected": "Basic factual recall",
        },
        {
            "question": "How does photosynthesis relate to cellular respiration?",
            "complexity": "Relationship Analysis",
            "expected": "Multi-concept reasoning",
        },
        {
            "question": "Why is sunlight essential for life on Earth?",
            "complexity": "Causal Chain Reasoning",
            "expected": "Multi-step inference",
        },
        {
            "question": "What would happen if chloroplasts stopped working?",
            "complexity": "Hypothetical Reasoning",
            "expected": "Predictive inference",
        },
    ]

    for i, test in enumerate(test_queries, 1):
        print(f"\n🔍 Test {i}: {test['complexity']}")
        print(f"Question: {test['question']}")
        print(f"Expected: {test['expected']}")
        print("-" * 50)

        # Get AI reasoning result
        result = engine.ask(test["question"])

        print(f"🎯 Answer: {result.primary_answer}")
        print(f"📊 Confidence: {result.confidence:.2f}")
        print(f"🤝 Consensus: {result.consensus_strength:.2f}")
        print(f"💭 Reasoning: {result.reasoning_explanation}")

        # Show alternative answers if any
        if result.alternative_answers:
            print(
                f"🔄 Alternatives: {len(result.alternative_answers)} other possibilities"
            )
            for alt_answer, alt_confidence in result.alternative_answers[:2]:
                print(f"   • {alt_answer} (confidence: {alt_confidence:.2f})")

        print()


def demonstrate_explainable_ai(engine: ReasoningEngine) -> None:
    """Demonstrate explainable AI capabilities."""

    print("\n" + "=" * 70)
    print("🔍 EXPLAINABLE AI DEMONSTRATION")
    print("=" * 70)

    question = "How do plants make energy from sunlight?"

    print(f"📝 Question: {question}")
    print(f"🧠 Generating detailed reasoning explanation...")
    print("-" * 50)

    # Get detailed explanation
    explanation = engine.explain_reasoning(question, detailed=True)

    print(f"🎯 Final Answer: {explanation['answer']}")
    print(f"📊 Overall Confidence: {explanation['confidence']:.2f}")
    print(f"🤝 Consensus Strength: {explanation['consensus_strength']:.2f}")
    print()

    print("🧠 AI Reasoning Process:")
    print(explanation["reasoning_explanation"])
    print()

    # Show robustness analysis
    robustness = explanation["reasoning_robustness"]
    print("📊 Reasoning Robustness Analysis:")
    print(f"   • Overall Robustness Score: {robustness['robustness_score']:.2f}")
    print(f"   • Path Diversity: {robustness['path_diversity']:.2f}")
    print(f"   • Confidence Consistency: {robustness['confidence_consistency']:.2f}")
    print(f"   • Supporting Paths: {robustness['supporting_path_count']}")
    print(f"   • Alternative Answers: {robustness['alternative_answer_count']}")
    print()

    # Show detailed reasoning paths if available
    if "detailed_paths" in explanation:
        print("🛤️  Detailed Reasoning Paths:")
        for path in explanation["detailed_paths"][:2]:  # Show top 2 paths
            print(
                f"\n   Path {path['path_number']} (confidence: {path['confidence']:.2f}):"
            )
            for step in path["steps"]:
                print(
                    f"      Step {step['step']}: {step['from']} --[{step['relation']}]--> {step['to']} ({step['confidence']:.2f})"
                )


def demonstrate_real_time_learning(engine: ReasoningEngine) -> None:
    """Demonstrate real-time learning capabilities."""

    print("\n" + "=" * 70)
    print("🎓 REAL-TIME LEARNING DEMONSTRATION")
    print("=" * 70)

    # Test knowledge before learning
    question = "What is CRISPR gene editing?"

    print(f"❓ Before Learning: {question}")
    result_before = engine.ask(question)
    print(f"   Answer: {result_before.primary_answer}")
    print(f"   Confidence: {result_before.confidence:.2f}")
    print()

    # Learn new knowledge in real-time
    print("📚 Learning new information about CRISPR...")
    new_knowledge = [
        "CRISPR is a gene editing technology that allows precise modification of DNA sequences",
        "CRISPR uses guide RNA to target specific DNA locations and Cas9 enzyme to cut DNA",
        "CRISPR technology enables scientists to add, remove, or replace specific genes",
        "CRISPR gene editing has applications in treating genetic diseases and improving crops",
    ]

    for knowledge in new_knowledge:
        engine.learn(knowledge, source="recent_research")
        print(f"   ✅ {knowledge}")

    print()

    # Test knowledge after learning
    print(f"❓ After Learning: {question}")
    result_after = engine.ask(question)
    print(f"   Answer: {result_after.primary_answer}")
    print(f"   Confidence: {result_after.confidence:.2f}")
    print(f"   Improvement: {result_after.confidence - result_before.confidence:+.2f}")
    print()

    # Show system growth
    stats = engine.get_system_stats()
    print("📈 System Growth:")
    print(f"   • Learning Events: {stats['system_info']['learning_events']}")
    print(f"   • Total Queries: {stats['system_info']['total_queries']}")
    print(f"   • Cache Hit Rate: {stats['system_info']['cache_hit_rate']:.1%}")


def demonstrate_performance_features(engine: ReasoningEngine) -> None:
    """Demonstrate performance and optimization features."""

    print("\n" + "=" * 70)
    print("⚡ PERFORMANCE & OPTIMIZATION DEMONSTRATION")
    print("=" * 70)

    # Show system statistics
    stats = engine.get_system_stats()

    print("📊 Current System Status:")
    print(f"   • Concepts: {stats['system_info']['total_concepts']}")
    print(f"   • Associations: {stats['system_info']['total_associations']}")
    print(f"   • Queries Processed: {stats['system_info']['total_queries']}")
    print(f"   • Cache Hit Rate: {stats['system_info']['cache_hit_rate']:.1%}")
    print()

    # Test query performance with caching
    print("🚀 Testing Query Performance with Caching:")

    import time

    test_question = "What is the relationship between photosynthesis and energy?"

    # First query (no cache)
    start_time = time.time()
    result1 = engine.ask(test_question)
    first_query_time = time.time() - start_time

    # Second query (cached)
    start_time = time.time()
    result2 = engine.ask(test_question)
    second_query_time = time.time() - start_time

    print(f"   • First Query: {first_query_time*1000:.1f}ms (no cache)")
    print(f"   • Second Query: {second_query_time*1000:.1f}ms (cached)")
    print(f"   • Speedup: {first_query_time/max(second_query_time, 0.001):.1f}x faster")
    print()

    # Run performance optimization
    print("🔧 Running Performance Optimization:")
    optimizations = engine.optimize_performance()

    for metric, value in optimizations.items():
        if value > 0:
            print(f"   • {metric.replace('_', ' ').title()}: {value}")

    # Show association statistics
    assoc_stats = stats["association_stats"]
    print(f"\n🔗 Association Quality:")
    print(f"   • Average Confidence: {assoc_stats['average_confidence']:.2f}")
    print(f"   • By Type: {assoc_stats['by_type']}")


def main():
    """Run the complete AI reasoning demonstration."""

    print("🚀 SUTRA AI - ADVANCED REASONING SYSTEM")
    print("=" * 70)
    print("Demonstrating AI-level reasoning capabilities:")
    print("• Natural language understanding")
    print("• Multi-path reasoning with consensus")
    print("• Explainable AI with confidence scoring")
    print("• Real-time learning and knowledge integration")
    print("• Advanced path-finding and inference")
    print("=" * 70)

    # Initialize the AI reasoning engine
    print("\n🧠 Initializing Sutra AI Reasoning Engine...")
    engine = ReasoningEngine(enable_caching=True, max_cache_size=100)
    print("   ✅ Engine initialized with caching enabled")

    # Load knowledge base
    setup_biological_knowledge(engine)

    # Run demonstrations
    demonstrate_ai_reasoning(engine)
    demonstrate_explainable_ai(engine)
    demonstrate_real_time_learning(engine)
    demonstrate_performance_features(engine)

    # Final summary
    final_stats = engine.get_system_stats()

    print("\n" + "=" * 70)
    print("🎉 DEMONSTRATION COMPLETE!")
    print("=" * 70)
    print("📊 Final System Statistics:")
    print(f"   • Total Concepts: {final_stats['system_info']['total_concepts']}")
    print(
        f"   • Total Associations: {final_stats['system_info']['total_associations']}"
    )
    print(f"   • Queries Processed: {final_stats['system_info']['total_queries']}")
    print(f"   • Learning Events: {final_stats['system_info']['learning_events']}")
    print(f"   • Cache Hit Rate: {final_stats['system_info']['cache_hit_rate']:.1%}")

    learning_stats = final_stats["learning_stats"]
    print(f"\n🎓 Learning System Analysis:")
    print(
        f"   • Difficult Concepts: {learning_stats['difficult_concepts']} (need reinforcement)"
    )
    print(f"   • Moderate Concepts: {learning_stats['moderate_concepts']} (developing)")
    print(f"   • Easy Concepts: {learning_stats['easy_concepts']} (well established)")
    print(f"   • Average Strength: {learning_stats['average_strength']:.2f}")

    print("\n✅ This demonstrates genuine AI capabilities:")
    print("   • Complex reasoning beyond pattern matching")
    print("   • Explainable decision making with confidence")
    print("   • Real-time learning without retraining")
    print("   • Multi-path consensus for robustness")
    print("   • Performance optimization and caching")
    print("\n🚀 Sutra AI is now ready for production AI workloads!")


if __name__ == "__main__":
    main()
