"""
Comprehensive Benchmarking Framework for Biological Training System

Tests performance, scalability, accuracy, and compares against traditional approaches.
Measures the revolutionary aspects: no gradients, associative learning, biological memory.
"""

import asyncio
import time
import psutil
import tracemalloc
import numpy as np
from typing import Dict, List, Any, Tuple, Optional
from dataclasses import dataclass, field
from collections import defaultdict
import json
import matplotlib.pyplot as plt
import seaborn as sns
from pathlib import Path

# Import the biological trainer
import sys
sys.path.append(str(Path(__file__).parent.parent))
from src.biological_trainer import BiologicalTrainer, MemoryType


@dataclass
class BenchmarkMetrics:
    """Comprehensive metrics for biological training evaluation"""
    # Performance metrics
    training_time: float = 0.0
    concepts_per_second: float = 0.0
    associations_per_second: float = 0.0
    
    # Memory metrics
    memory_usage_mb: float = 0.0
    peak_memory_mb: float = 0.0
    concept_memory_efficiency: float = 0.0  # concepts per MB
    
    # Learning metrics
    total_concepts: int = 0
    total_associations: int = 0
    unique_concepts: int = 0
    reinforcement_ratio: float = 0.0  # reinforced vs new
    
    # Biological metrics
    forgetting_rate: float = 0.0
    consolidation_events: int = 0
    memory_distribution: Dict[str, int] = field(default_factory=dict)
    average_concept_strength: float = 0.0
    
    # Network metrics
    average_degree: float = 0.0  # avg associations per concept
    clustering_coefficient: float = 0.0
    network_density: float = 0.0
    
    # Retrieval metrics
    retrieval_accuracy: float = 0.0
    retrieval_speed_ms: float = 0.0
    spreading_activation_reach: float = 0.0
    
    # Comparison metrics (vs traditional)
    speedup_vs_embedding: float = 0.0
    memory_savings_vs_embedding: float = 0.0
    accuracy_vs_tfidf: float = 0.0


class BiologicalBenchmark:
    """Main benchmarking system for biological training"""
    
    def __init__(self, output_dir: str = "benchmark_results"):
        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(exist_ok=True)
        self.metrics_history: List[BenchmarkMetrics] = []
        
    async def benchmark_learning_speed(self, 
                                       text_corpus: List[str],
                                       batch_sizes: List[int] = [1, 10, 50, 100]) -> Dict[str, Any]:
        """Test learning speed with different batch sizes"""
        results = {}
        
        for batch_size in batch_sizes:
            trainer = BiologicalTrainer()
            metrics = BenchmarkMetrics()
            
            # Start monitoring
            tracemalloc.start()
            process = psutil.Process()
            start_memory = process.memory_info().rss / 1024 / 1024  # MB
            
            # Train in batches
            start_time = time.time()
            total_concepts_before = len(trainer.memory_system.concepts)
            
            for i in range(0, len(text_corpus), batch_size):
                batch = text_corpus[i:i+batch_size]
                result = await trainer.train_from_stream(batch)
                
            training_time = time.time() - start_time
            
            # Collect metrics
            metrics.training_time = training_time
            metrics.total_concepts = len(trainer.memory_system.concepts)
            metrics.total_associations = len(trainer.memory_system.associations)
            metrics.concepts_per_second = metrics.total_concepts / training_time
            metrics.associations_per_second = metrics.total_associations / training_time
            
            # Memory metrics
            current_memory = process.memory_info().rss / 1024 / 1024
            metrics.memory_usage_mb = current_memory - start_memory
            metrics.peak_memory_mb = tracemalloc.get_traced_memory()[1] / 1024 / 1024
            metrics.concept_memory_efficiency = metrics.total_concepts / max(metrics.memory_usage_mb, 1)
            
            # Memory distribution
            memory_dist = defaultdict(int)
            for concept in trainer.memory_system.concepts.values():
                memory_dist[concept.memory_type.value] += 1
            metrics.memory_distribution = dict(memory_dist)
            
            tracemalloc.stop()
            
            results[f"batch_{batch_size}"] = {
                "metrics": metrics,
                "trainer": trainer
            }
            
        return results
    
    async def benchmark_biological_properties(self, trainer: BiologicalTrainer,
                                             test_duration_hours: float = 0.1) -> BenchmarkMetrics:
        """Test biological properties: forgetting, consolidation, memory tiers"""
        metrics = BenchmarkMetrics()
        initial_concepts = len(trainer.memory_system.concepts)
        
        # Simulate time passing for forgetting curves
        start_time = time.time()
        test_duration_seconds = test_duration_hours * 3600
        
        # Track concept lifecycle
        concept_strengths_over_time = []
        consolidation_events = []
        
        while time.time() - start_time < test_duration_seconds:
            # Force forgetting cycle
            trainer.memory_system.natural_forgetting()
            
            # Track strengths
            strengths = [c.strength for c in trainer.memory_system.concepts.values()]
            concept_strengths_over_time.append(strengths)
            
            # Check consolidations
            for concept in trainer.memory_system.concepts.values():
                trainer.memory_system._check_consolidation(concept.id)
            
            # Sleep briefly to simulate time
            await asyncio.sleep(0.01)
        
        # Calculate forgetting rate
        final_concepts = len(trainer.memory_system.concepts)
        metrics.forgetting_rate = (initial_concepts - final_concepts) / max(initial_concepts, 1)
        
        # Memory distribution after forgetting
        memory_dist = defaultdict(int)
        for concept in trainer.memory_system.concepts.values():
            memory_dist[concept.memory_type.value] += 1
        metrics.memory_distribution = dict(memory_dist)
        
        # Average strength
        if trainer.memory_system.concepts:
            metrics.average_concept_strength = np.mean([
                c.strength for c in trainer.memory_system.concepts.values()
            ])
        
        return metrics
    
    async def benchmark_retrieval_accuracy(self, trainer: BiologicalTrainer,
                                          test_queries: List[Tuple[str, List[str]]]) -> BenchmarkMetrics:
        """Test retrieval accuracy with known relevant documents"""
        metrics = BenchmarkMetrics()
        
        total_precision = 0
        total_recall = 0
        total_time = 0
        
        for query, expected_relevant in test_queries:
            start = time.time()
            results = trainer.query_knowledge(query, max_results=10, hops=2)
            retrieval_time = time.time() - start
            total_time += retrieval_time
            
            # Extract content from results
            retrieved_content = [r['content'] for r in results]
            
            # Calculate precision and recall
            relevant_retrieved = len(set(retrieved_content) & set(expected_relevant))
            precision = relevant_retrieved / len(results) if results else 0
            recall = relevant_retrieved / len(expected_relevant) if expected_relevant else 0
            
            total_precision += precision
            total_recall += recall
        
        num_queries = len(test_queries)
        metrics.retrieval_accuracy = (total_precision / num_queries + total_recall / num_queries) / 2
        metrics.retrieval_speed_ms = (total_time / num_queries) * 1000
        
        return metrics
    
    async def benchmark_network_properties(self, trainer: BiologicalTrainer) -> BenchmarkMetrics:
        """Analyze the associative network structure"""
        metrics = BenchmarkMetrics()
        
        if not trainer.memory_system.concepts:
            return metrics
        
        # Build adjacency information
        adjacency = defaultdict(set)
        for assoc in trainer.memory_system.associations:
            adjacency[assoc.source_id].add(assoc.target_id)
            adjacency[assoc.target_id].add(assoc.source_id)
        
        # Average degree
        degrees = [len(neighbors) for neighbors in adjacency.values()]
        metrics.average_degree = np.mean(degrees) if degrees else 0
        
        # Network density
        num_concepts = len(trainer.memory_system.concepts)
        max_edges = num_concepts * (num_concepts - 1) / 2
        actual_edges = len(trainer.memory_system.associations) / 2  # bidirectional
        metrics.network_density = actual_edges / max_edges if max_edges > 0 else 0
        
        # Clustering coefficient (simplified)
        clustering_coeffs = []
        for node, neighbors in adjacency.items():
            if len(neighbors) < 2:
                continue
            neighbors_list = list(neighbors)
            possible_triangles = len(neighbors) * (len(neighbors) - 1) / 2
            actual_triangles = 0
            for i in range(len(neighbors_list)):
                for j in range(i+1, len(neighbors_list)):
                    if neighbors_list[j] in adjacency.get(neighbors_list[i], set()):
                        actual_triangles += 1
            if possible_triangles > 0:
                clustering_coeffs.append(actual_triangles / possible_triangles)
        
        metrics.clustering_coefficient = np.mean(clustering_coeffs) if clustering_coeffs else 0
        
        return metrics
    
    async def benchmark_vs_traditional(self, text_corpus: List[str]) -> Dict[str, Any]:
        """Compare against traditional approaches (TF-IDF, embeddings)"""
        results = {}
        
        # Benchmark biological trainer
        print("Testing Biological Trainer...")
        bio_start = time.time()
        bio_trainer = BiologicalTrainer()
        bio_result = await bio_trainer.train_from_stream(text_corpus)
        bio_time = time.time() - bio_start
        bio_memory = psutil.Process().memory_info().rss / 1024 / 1024
        
        results['biological'] = {
            'time': bio_time,
            'memory_mb': bio_memory,
            'concepts': len(bio_trainer.memory_system.concepts),
            'associations': len(bio_trainer.memory_system.associations)
        }
        
        # Compare with TF-IDF
        try:
            from sklearn.feature_extraction.text import TfidfVectorizer
            print("Testing TF-IDF...")
            tfidf_start = time.time()
            vectorizer = TfidfVectorizer(max_features=10000)
            tfidf_matrix = vectorizer.fit_transform(text_corpus)
            tfidf_time = time.time() - tfidf_start
            tfidf_memory = psutil.Process().memory_info().rss / 1024 / 1024
            
            results['tfidf'] = {
                'time': tfidf_time,
                'memory_mb': tfidf_memory,
                'features': len(vectorizer.vocabulary_),
                'speedup': tfidf_time / bio_time if bio_time > 0 else 0
            }
        except ImportError:
            print("scikit-learn not installed, skipping TF-IDF comparison")
        
        # Compare with Word2Vec
        try:
            from gensim.models import Word2Vec
            print("Testing Word2Vec...")
            # Tokenize for Word2Vec
            tokenized = [text.split() for text in text_corpus]
            w2v_start = time.time()
            w2v_model = Word2Vec(tokenized, vector_size=100, window=5, min_count=1, workers=4)
            w2v_time = time.time() - w2v_start
            w2v_memory = psutil.Process().memory_info().rss / 1024 / 1024
            
            results['word2vec'] = {
                'time': w2v_time,
                'memory_mb': w2v_memory,
                'vocab_size': len(w2v_model.wv),
                'speedup': w2v_time / bio_time if bio_time > 0 else 0
            }
        except ImportError:
            print("gensim not installed, skipping Word2Vec comparison")
        
        return results
    
    async def stress_test_scalability(self, 
                                     corpus_sizes: List[int] = [100, 1000, 10000],
                                     text_length: int = 100) -> Dict[str, Any]:
        """Test scalability with increasing corpus sizes"""
        results = {}
        
        for size in corpus_sizes:
            print(f"Testing with {size} documents...")
            
            # Generate synthetic corpus
            corpus = [f"Document {i} contains text about topic {i % 10} " + 
                     " ".join([f"word{j}" for j in range(text_length)])
                     for i in range(size)]
            
            trainer = BiologicalTrainer()
            
            # Monitor resources
            process = psutil.Process()
            start_memory = process.memory_info().rss / 1024 / 1024
            
            start_time = time.time()
            result = await trainer.train_from_stream(corpus)
            training_time = time.time() - start_time
            
            end_memory = process.memory_info().rss / 1024 / 1024
            
            # Test retrieval on large network
            query_start = time.time()
            query_results = trainer.query_knowledge("topic word10", max_results=10)
            query_time = time.time() - query_start
            
            results[f"size_{size}"] = {
                'training_time': training_time,
                'memory_used_mb': end_memory - start_memory,
                'total_concepts': len(trainer.memory_system.concepts),
                'total_associations': len(trainer.memory_system.associations),
                'retrieval_time_ms': query_time * 1000,
                'throughput': size / training_time
            }
        
        return results
    
    def visualize_results(self, results: Dict[str, Any], save_path: Optional[str] = None):
        """Create comprehensive visualization of benchmark results"""
        fig, axes = plt.subplots(2, 3, figsize=(15, 10))
        
        # 1. Learning speed comparison
        if 'learning_speed' in results:
            ax = axes[0, 0]
            batch_sizes = []
            concepts_per_sec = []
            for batch_key, data in results['learning_speed'].items():
                batch_size = int(batch_key.split('_')[1])
                batch_sizes.append(batch_size)
                concepts_per_sec.append(data['metrics'].concepts_per_second)
            ax.bar(batch_sizes, concepts_per_sec, color='steelblue')
            ax.set_xlabel('Batch Size')
            ax.set_ylabel('Concepts/Second')
            ax.set_title('Learning Speed vs Batch Size')
        
        # 2. Memory distribution
        if 'biological_properties' in results:
            ax = axes[0, 1]
            memory_dist = results['biological_properties'].memory_distribution
            if memory_dist:
                ax.pie(memory_dist.values(), labels=memory_dist.keys(), autopct='%1.1f%%')
                ax.set_title('Memory Tier Distribution')
        
        # 3. Network properties
        if 'network_properties' in results:
            ax = axes[0, 2]
            metrics = results['network_properties']
            properties = ['Avg Degree', 'Clustering', 'Density']
            values = [
                metrics.average_degree,
                metrics.clustering_coefficient * 100,
                metrics.network_density * 100
            ]
            ax.bar(properties, values, color=['green', 'orange', 'red'])
            ax.set_ylabel('Value')
            ax.set_title('Network Properties')
        
        # 4. Scalability
        if 'scalability' in results:
            ax = axes[1, 0]
            sizes = []
            times = []
            for size_key, data in results['scalability'].items():
                size = int(size_key.split('_')[1])
                sizes.append(size)
                times.append(data['training_time'])
            ax.plot(sizes, times, marker='o', color='purple')
            ax.set_xlabel('Corpus Size')
            ax.set_ylabel('Training Time (s)')
            ax.set_title('Scalability Analysis')
            ax.set_xscale('log')
        
        # 5. Memory efficiency
        if 'scalability' in results:
            ax = axes[1, 1]
            sizes = []
            efficiency = []
            for size_key, data in results['scalability'].items():
                size = int(size_key.split('_')[1])
                sizes.append(size)
                eff = data['total_concepts'] / max(data['memory_used_mb'], 1)
                efficiency.append(eff)
            ax.plot(sizes, efficiency, marker='s', color='teal')
            ax.set_xlabel('Corpus Size')
            ax.set_ylabel('Concepts per MB')
            ax.set_title('Memory Efficiency')
            ax.set_xscale('log')
        
        # 6. Comparison with traditional methods
        if 'traditional_comparison' in results:
            ax = axes[1, 2]
            methods = []
            speedups = []
            trad_comp = results['traditional_comparison']
            
            if 'tfidf' in trad_comp:
                methods.append('TF-IDF')
                speedups.append(trad_comp['tfidf'].get('speedup', 1))
            if 'word2vec' in trad_comp:
                methods.append('Word2Vec')
                speedups.append(trad_comp['word2vec'].get('speedup', 1))
            
            if methods:
                ax.bar(methods, speedups, color='coral')
                ax.axhline(y=1, color='black', linestyle='--', label='Biological (baseline)')
                ax.set_ylabel('Speedup Factor')
                ax.set_title('Speed vs Traditional Methods')
                ax.legend()
        
        plt.suptitle('Biological Training System Benchmarks', fontsize=16)
        plt.tight_layout()
        
        if save_path:
            plt.savefig(save_path, dpi=300, bbox_inches='tight')
        plt.show()
        
        return fig
    
    async def run_full_benchmark(self, test_corpus: Optional[List[str]] = None) -> Dict[str, Any]:
        """Run complete benchmark suite"""
        print("="*50)
        print("BIOLOGICAL TRAINING SYSTEM BENCHMARK")
        print("="*50)
        
        # Use default corpus if not provided
        if test_corpus is None:
            test_corpus = [
                "Machine learning is a subset of artificial intelligence.",
                "Neural networks are inspired by biological brain structures.",
                "Deep learning uses multiple layers to learn complex patterns.",
                "Training requires large datasets and computational resources.",
                "Biological systems learn efficiently with minimal data.",
                "The brain uses associative memory for learning.",
                "Neurons form connections through synaptic plasticity.",
                "Memory consolidation happens during sleep.",
                "Forgetting is an important part of learning.",
                "Swarm intelligence emerges from simple agent interactions."
            ] * 10  # Repeat for larger corpus
        
        all_results = {}
        
        # 1. Learning speed
        print("\n1. Testing learning speed...")
        all_results['learning_speed'] = await self.benchmark_learning_speed(
            test_corpus, batch_sizes=[1, 5, 10, 20]
        )
        
        # 2. Biological properties
        print("\n2. Testing biological properties...")
        trainer = BiologicalTrainer()
        await trainer.train_from_stream(test_corpus[:50])
        all_results['biological_properties'] = await self.benchmark_biological_properties(
            trainer, test_duration_hours=0.01
        )
        
        # 3. Network properties
        print("\n3. Analyzing network properties...")
        all_results['network_properties'] = await self.benchmark_network_properties(trainer)
        
        # 4. Retrieval accuracy
        print("\n4. Testing retrieval accuracy...")
        test_queries = [
            ("machine learning", ["Machine learning is a subset of artificial intelligence."]),
            ("biological brain", ["Neural networks are inspired by biological brain structures."]),
            ("memory sleep", ["Memory consolidation happens during sleep."])
        ]
        all_results['retrieval_accuracy'] = await self.benchmark_retrieval_accuracy(
            trainer, test_queries
        )
        
        # 5. Scalability
        print("\n5. Testing scalability...")
        all_results['scalability'] = await self.stress_test_scalability(
            corpus_sizes=[100, 500, 1000]
        )
        
        # 6. Comparison with traditional methods
        print("\n6. Comparing with traditional methods...")
        all_results['traditional_comparison'] = await self.benchmark_vs_traditional(
            test_corpus[:50]
        )
        
        # Save results
        results_file = self.output_dir / "benchmark_results.json"
        with open(results_file, 'w') as f:
            # Convert metrics objects to dict for JSON serialization
            serializable_results = self._make_serializable(all_results)
            json.dump(serializable_results, f, indent=2)
        
        print(f"\n✅ Results saved to: {results_file}")
        
        # Generate report
        self._generate_report(all_results)
        
        return all_results
    
    def _make_serializable(self, obj):
        """Convert metrics objects to JSON-serializable format"""
        if isinstance(obj, BenchmarkMetrics):
            return obj.__dict__
        elif isinstance(obj, dict):
            return {k: self._make_serializable(v) for k, v in obj.items()}
        elif isinstance(obj, list):
            return [self._make_serializable(item) for item in obj]
        else:
            return obj
    
    def _generate_report(self, results: Dict[str, Any]):
        """Generate human-readable benchmark report"""
        report_path = self.output_dir / "benchmark_report.txt"
        
        with open(report_path, 'w') as f:
            f.write("="*60 + "\n")
            f.write("BIOLOGICAL TRAINING SYSTEM BENCHMARK REPORT\n")
            f.write("="*60 + "\n\n")
            
            # Learning speed
            if 'learning_speed' in results:
                f.write("1. LEARNING SPEED\n")
                f.write("-"*40 + "\n")
                for batch_key, data in results['learning_speed'].items():
                    metrics = data['metrics']
                    f.write(f"\n{batch_key}:\n")
                    f.write(f"  Concepts/second: {metrics.concepts_per_second:.2f}\n")
                    f.write(f"  Associations/second: {metrics.associations_per_second:.2f}\n")
                    f.write(f"  Memory efficiency: {metrics.concept_memory_efficiency:.2f} concepts/MB\n")
            
            # Biological properties
            if 'biological_properties' in results:
                f.write("\n2. BIOLOGICAL PROPERTIES\n")
                f.write("-"*40 + "\n")
                metrics = results['biological_properties']
                f.write(f"  Forgetting rate: {metrics.forgetting_rate:.2%}\n")
                f.write(f"  Average strength: {metrics.average_concept_strength:.3f}\n")
                f.write(f"  Memory distribution: {metrics.memory_distribution}\n")
            
            # Network properties
            if 'network_properties' in results:
                f.write("\n3. NETWORK PROPERTIES\n")
                f.write("-"*40 + "\n")
                metrics = results['network_properties']
                f.write(f"  Average degree: {metrics.average_degree:.2f}\n")
                f.write(f"  Clustering coefficient: {metrics.clustering_coefficient:.3f}\n")
                f.write(f"  Network density: {metrics.network_density:.4f}\n")
            
            # Retrieval
            if 'retrieval_accuracy' in results:
                f.write("\n4. RETRIEVAL PERFORMANCE\n")
                f.write("-"*40 + "\n")
                metrics = results['retrieval_accuracy']
                f.write(f"  Accuracy: {metrics.retrieval_accuracy:.2%}\n")
                f.write(f"  Speed: {metrics.retrieval_speed_ms:.2f}ms\n")
            
            # Scalability
            if 'scalability' in results:
                f.write("\n5. SCALABILITY\n")
                f.write("-"*40 + "\n")
                for size_key, data in results['scalability'].items():
                    f.write(f"\n{size_key}:\n")
                    f.write(f"  Training time: {data['training_time']:.3f}s\n")
                    f.write(f"  Throughput: {data['throughput']:.2f} docs/s\n")
                    f.write(f"  Memory: {data['memory_used_mb']:.2f} MB\n")
                    f.write(f"  Retrieval: {data['retrieval_time_ms']:.2f}ms\n")
            
            # Comparison
            if 'traditional_comparison' in results:
                f.write("\n6. VS TRADITIONAL METHODS\n")
                f.write("-"*40 + "\n")
                bio = results['traditional_comparison']['biological']
                f.write(f"\nBiological Trainer:\n")
                f.write(f"  Time: {bio['time']:.3f}s\n")
                f.write(f"  Concepts: {bio['concepts']}\n")
                f.write(f"  Associations: {bio['associations']}\n")
                
                if 'tfidf' in results['traditional_comparison']:
                    tfidf = results['traditional_comparison']['tfidf']
                    f.write(f"\nTF-IDF:\n")
                    f.write(f"  Time: {tfidf['time']:.3f}s\n")
                    f.write(f"  Speedup: {tfidf['speedup']:.2f}x\n")
                
                if 'word2vec' in results['traditional_comparison']:
                    w2v = results['traditional_comparison']['word2vec']
                    f.write(f"\nWord2Vec:\n")
                    f.write(f"  Time: {w2v['time']:.3f}s\n")
                    f.write(f"  Speedup: {w2v['speedup']:.2f}x\n")
            
            f.write("\n" + "="*60 + "\n")
            f.write("END OF REPORT\n")
        
        print(f"📊 Report saved to: {report_path}")


# Standalone benchmark runner
async def main():
    """Run the complete benchmark suite"""
    benchmark = BiologicalBenchmark(output_dir="benchmark_results")
    results = await benchmark.run_full_benchmark()
    
    # Visualize results
    benchmark.visualize_results(results, save_path="benchmark_results/benchmark_plots.png")
    
    print("\n✅ Benchmarking complete!")
    print("Check benchmark_results/ directory for detailed results.")


if __name__ == "__main__":
    asyncio.run(main())