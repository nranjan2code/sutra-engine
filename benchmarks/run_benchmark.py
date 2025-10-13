#!/usr/bin/env python3
"""
Run comprehensive benchmarks for the Biological Training System

This script tests the revolutionary non-gradient, associative learning system
against traditional approaches and generates detailed performance reports.
"""

import asyncio
import sys
from pathlib import Path

# Add src to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from benchmark_framework import BiologicalBenchmark
from src.biological_trainer import BiologicalTrainer


def generate_test_corpus(size: int = 100) -> list:
    """Generate a diverse test corpus for benchmarking"""
    corpus = []
    
    # Scientific texts
    scientific = [
        "Machine learning algorithms are computational methods that enable computers to learn from data.",
        "Neural networks consist of interconnected nodes that process information in parallel.",
        "Deep learning architectures use multiple layers to extract hierarchical features.",
        "Convolutional neural networks excel at image recognition tasks.",
        "Recurrent neural networks are designed for sequential data processing.",
        "Transformer models have revolutionized natural language processing.",
        "Reinforcement learning agents learn through interaction with environments.",
        "Unsupervised learning discovers patterns without labeled data.",
        "Transfer learning leverages knowledge from pre-trained models.",
        "Federated learning enables distributed training while preserving privacy."
    ]
    
    # Biological memory texts
    biological = [
        "The hippocampus plays a crucial role in memory consolidation.",
        "Synaptic plasticity underlies learning and memory formation.",
        "Long-term potentiation strengthens synaptic connections.",
        "Memory consolidation occurs during sleep cycles.",
        "The amygdala processes emotional memories.",
        "Working memory has limited capacity but rapid access.",
        "Declarative memory stores facts and events.",
        "Procedural memory handles skills and habits.",
        "Memory retrieval strengthens neural pathways.",
        "Forgetting curves describe memory decay over time."
    ]
    
    # Technical descriptions
    technical = [
        "Gradient descent optimizes model parameters iteratively.",
        "Backpropagation computes gradients through the network.",
        "Batch normalization stabilizes training dynamics.",
        "Dropout prevents overfitting during training.",
        "Adam optimizer adapts learning rates per parameter.",
        "Cross-entropy loss measures classification error.",
        "Activation functions introduce non-linearity.",
        "Weight initialization affects convergence speed.",
        "Early stopping prevents overtraining.",
        "Regularization techniques reduce model complexity."
    ]
    
    # Combine and expand corpus
    base_texts = scientific + biological + technical
    
    # Expand to requested size
    while len(corpus) < size:
        for text in base_texts:
            if len(corpus) >= size:
                break
            # Add variations
            corpus.append(text)
            if len(corpus) < size:
                corpus.append(text.lower())
            if len(corpus) < size:
                corpus.append(text + " This is fundamental to understanding AI.")
    
    return corpus[:size]


async def run_quick_benchmark():
    """Run a quick benchmark for testing"""
    print("="*60)
    print("QUICK BIOLOGICAL TRAINING BENCHMARK")
    print("="*60)
    
    # Generate small test corpus
    corpus = generate_test_corpus(30)
    
    # Create benchmark
    benchmark = BiologicalBenchmark(output_dir="benchmark_results_quick")
    
    # Test learning speed
    print("\n📊 Testing learning speed...")
    speed_results = await benchmark.benchmark_learning_speed(
        corpus, batch_sizes=[1, 5, 10]
    )
    
    for batch_key, data in speed_results.items():
        metrics = data['metrics']
        print(f"\n{batch_key}:")
        print(f"  Concepts/sec: {metrics.concepts_per_second:.2f}")
        print(f"  Associations/sec: {metrics.associations_per_second:.2f}")
        print(f"  Memory: {metrics.memory_usage_mb:.2f} MB")
    
    # Test biological properties
    print("\n🧬 Testing biological properties...")
    trainer = BiologicalTrainer()
    await trainer.train_from_stream(corpus[:10])
    bio_metrics = await benchmark.benchmark_biological_properties(
        trainer, test_duration_hours=0.001  # Very short for quick test
    )
    
    print(f"  Forgetting rate: {bio_metrics.forgetting_rate:.2%}")
    print(f"  Avg strength: {bio_metrics.average_concept_strength:.3f}")
    print(f"  Memory distribution: {bio_metrics.memory_distribution}")
    
    # Test retrieval
    print("\n🔍 Testing retrieval...")
    test_queries = [
        ("machine learning", ["Machine learning algorithms are computational methods that enable computers to learn from data."]),
        ("memory consolidation", ["Memory consolidation occurs during sleep cycles."])
    ]
    retrieval_metrics = await benchmark.benchmark_retrieval_accuracy(
        trainer, test_queries
    )
    
    print(f"  Accuracy: {retrieval_metrics.retrieval_accuracy:.2%}")
    print(f"  Speed: {retrieval_metrics.retrieval_speed_ms:.2f}ms")
    
    print("\n✅ Quick benchmark complete!")


async def run_full_benchmark():
    """Run comprehensive benchmark suite"""
    print("="*60)
    print("COMPREHENSIVE BIOLOGICAL TRAINING BENCHMARK")
    print("="*60)
    
    # Generate large test corpus
    corpus = generate_test_corpus(500)
    
    # Create benchmark
    benchmark = BiologicalBenchmark(output_dir="benchmark_results_full")
    
    # Run full suite
    results = await benchmark.run_full_benchmark(corpus)
    
    # Generate visualizations (without displaying if no GUI)
    try:
        benchmark.visualize_results(
            results, 
            save_path="benchmark_results_full/benchmark_plots.png"
        )
    except Exception as e:
        print(f"Note: Visualization failed (probably no GUI): {e}")
    
    print("\n📈 BENCHMARK SUMMARY")
    print("-"*40)
    
    # Print key metrics
    if 'learning_speed' in results:
        batch_10 = results['learning_speed'].get('batch_10', {})
        if batch_10:
            metrics = batch_10.get('metrics')
            if metrics:
                print(f"Learning rate: {metrics.concepts_per_second:.0f} concepts/sec")
                print(f"Association formation: {metrics.associations_per_second:.0f} assoc/sec")
    
    if 'scalability' in results:
        size_1000 = results['scalability'].get('size_1000', {})
        if size_1000:
            print(f"1000-doc throughput: {size_1000.get('throughput', 0):.0f} docs/sec")
            print(f"1000-doc retrieval: {size_1000.get('retrieval_time_ms', 0):.2f}ms")
    
    if 'traditional_comparison' in results:
        bio = results['traditional_comparison'].get('biological', {})
        tfidf = results['traditional_comparison'].get('tfidf', {})
        if bio and tfidf:
            speedup = tfidf.get('speedup', 0)
            if speedup > 1:
                print(f"Faster than TF-IDF by: {speedup:.1f}x")
            else:
                print(f"TF-IDF faster by: {1/speedup:.1f}x")
    
    print("\n✅ Full benchmark complete!")
    print(f"📁 Results saved in: benchmark_results_full/")


async def run_stress_test():
    """Run stress test for scalability"""
    print("="*60)
    print("STRESS TEST: BIOLOGICAL TRAINING SCALABILITY")
    print("="*60)
    
    benchmark = BiologicalBenchmark(output_dir="benchmark_stress")
    
    # Test with increasing corpus sizes
    sizes = [100, 500, 1000, 5000]
    results = await benchmark.stress_test_scalability(
        corpus_sizes=sizes,
        text_length=50  # Shorter texts for stress test
    )
    
    print("\n📊 SCALABILITY RESULTS")
    print("-"*40)
    print(f"{'Size':<10} {'Time (s)':<12} {'Throughput':<15} {'Memory (MB)':<12} {'Concepts':<10}")
    print("-"*70)
    
    for size in sizes:
        data = results.get(f"size_{size}", {})
        if data:
            print(f"{size:<10} "
                  f"{data.get('training_time', 0):<12.3f} "
                  f"{data.get('throughput', 0):<15.1f} "
                  f"{data.get('memory_used_mb', 0):<12.2f} "
                  f"{data.get('total_concepts', 0):<10}")
    
    print("\n✅ Stress test complete!")


async def compare_with_traditional():
    """Direct comparison with traditional methods"""
    print("="*60)
    print("COMPARISON: BIOLOGICAL VS TRADITIONAL")
    print("="*60)
    
    corpus = generate_test_corpus(100)
    benchmark = BiologicalBenchmark(output_dir="benchmark_comparison")
    
    results = await benchmark.benchmark_vs_traditional(corpus)
    
    print("\n📊 PERFORMANCE COMPARISON")
    print("-"*40)
    
    bio = results.get('biological', {})
    print(f"\n🧬 Biological Trainer:")
    print(f"  Time: {bio.get('time', 0):.3f}s")
    print(f"  Concepts: {bio.get('concepts', 0)}")
    print(f"  Associations: {bio.get('associations', 0)}")
    print(f"  Memory: {bio.get('memory_mb', 0):.2f} MB")
    
    if 'tfidf' in results:
        tfidf = results.get('tfidf', {})
        print(f"\n📊 TF-IDF:")
        print(f"  Time: {tfidf.get('time', 0):.3f}s")
        print(f"  Features: {tfidf.get('features', 0)}")
        print(f"  Memory: {tfidf.get('memory_mb', 0):.2f} MB")
        speedup = tfidf.get('speedup', 1)
        if speedup > 1:
            print(f"  Biological is {speedup:.2f}x faster")
        else:
            print(f"  TF-IDF is {1/speedup:.2f}x faster")
    
    if 'word2vec' in results:
        w2v = results.get('word2vec', {})
        print(f"\n🔤 Word2Vec:")
        print(f"  Time: {w2v.get('time', 0):.3f}s")
        print(f"  Vocab: {w2v.get('vocab_size', 0)}")
        print(f"  Memory: {w2v.get('memory_mb', 0):.2f} MB")
        speedup = w2v.get('speedup', 1)
        if speedup > 1:
            print(f"  Biological is {speedup:.2f}x faster")
        else:
            print(f"  Word2Vec is {1/speedup:.2f}x faster")
    
    print("\n✅ Comparison complete!")


def main():
    """Main entry point"""
    import argparse
    
    parser = argparse.ArgumentParser(description='Benchmark Biological Training System')
    parser.add_argument('--mode', choices=['quick', 'full', 'stress', 'compare'], 
                        default='quick',
                        help='Benchmark mode to run')
    
    args = parser.parse_args()
    
    if args.mode == 'quick':
        asyncio.run(run_quick_benchmark())
    elif args.mode == 'full':
        asyncio.run(run_full_benchmark())
    elif args.mode == 'stress':
        asyncio.run(run_stress_test())
    elif args.mode == 'compare':
        asyncio.run(compare_with_traditional())


if __name__ == "__main__":
    main()