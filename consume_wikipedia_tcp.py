#!/usr/bin/env python3
"""
TCP-Optimized Wikipedia Dataset Consumer
=======================================

Production-ready consumer for Wikipedia datasets with explicit TCP storage support
and optimizations for distributed deployment environments.

Key improvements over base consumer:
- Explicit TCP storage configuration
- Connection health monitoring
- Bulk batch processing optimizations
- Memory management for large datasets
- Production-grade error handling
- Environment-specific deployment validation

Usage:
    export SUTRA_STORAGE_MODE=server
    export SUTRA_STORAGE_SERVER=localhost:50051
    python consume_wikipedia_tcp.py
"""

import os
import time
import logging
import psutil
from pathlib import Path
from typing import List, Dict, Any, Optional, Tuple
import gc

from sutra_core import ReasoningEngine
from sutra_core.adapters import DatasetAdapter

# Configure production logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(name)s - %(message)s'
)
logger = logging.getLogger(__name__)


class TcpWikipediaConsumer:
    """
    Production-ready Wikipedia dataset processor optimized for TCP storage.
    
    Features:
    - Explicit TCP storage validation
    - Connection health monitoring
    - Memory-optimized bulk processing
    - Production deployment checks
    - Comprehensive error handling and recovery
    """
    
    def __init__(self, 
                 dataset_path: str = "dataset/wikipedia.txt",
                 batch_size: int = 50,
                 memory_limit_mb: int = 2048):
        self.dataset_path = Path(dataset_path)
        self.batch_size = batch_size
        self.memory_limit_mb = memory_limit_mb
        
        # Validate production environment
        self._validate_environment()
        
        # Initialize TCP-configured engine
        self.engine = self._initialize_tcp_engine()
        
        # Performance tracking
        self.learned_articles = []
        self.start_time = None
        self.stats = {
            'total_articles': 0,
            'concepts_created': 0,
            'associations_created': 0,
            'processing_time': 0,
            'articles_per_second': 0,
            'bytes_processed': 0,
            'tcp_reconnections': 0,
            'memory_cleanups': 0
        }
        
        logger.info("TCP Wikipedia Consumer initialized successfully")
    
    def _validate_environment(self) -> None:
        """Validate required TCP deployment environment."""
        # Check storage mode
        storage_mode = os.environ.get("SUTRA_STORAGE_MODE")
        if storage_mode != "server":
            logger.warning(f"SUTRA_STORAGE_MODE={storage_mode}, should be 'server' for TCP")
            os.environ["SUTRA_STORAGE_MODE"] = "server"
        
        # Check storage server address
        storage_server = os.environ.get("SUTRA_STORAGE_SERVER")
        if not storage_server:
            logger.warning("SUTRA_STORAGE_SERVER not set, using default")
            os.environ["SUTRA_STORAGE_SERVER"] = "storage-server:50051"
        
        # Check dataset exists
        if not self.dataset_path.exists():
            raise FileNotFoundError(f"Wikipedia dataset not found: {self.dataset_path}")
        
        # Log environment
        logger.info("Environment validation:")
        logger.info(f"  Storage Mode: {os.environ.get('SUTRA_STORAGE_MODE')}")
        logger.info(f"  Storage Server: {os.environ.get('SUTRA_STORAGE_SERVER')}")
        logger.info(f"  Dataset: {self.dataset_path} ({self.dataset_path.stat().st_size:,} bytes)")
        logger.info(f"  Available Memory: {psutil.virtual_memory().available / 1024 / 1024:.0f} MB")
    
    def _initialize_tcp_engine(self) -> ReasoningEngine:
        """Initialize ReasoningEngine with TCP storage validation."""
        try:
            # Force TCP storage mode
            os.environ["SUTRA_STORAGE_MODE"] = "server"
            
            engine = ReasoningEngine(
                storage_path="./knowledge",  # Not used in server mode
                enable_caching=True,
                max_cache_size=1000,
                enable_batch_embeddings=True,  # Better for bulk learning
                enable_parallel_associations=True,  # Faster association extraction
                association_workers=4
            )
            
            # Validate TCP connection
            if hasattr(engine.storage, 'health_check'):
                health = engine.storage.health_check()
                if health.get('status') == 'unhealthy':
                    raise RuntimeError(f"TCP storage unhealthy: {health.get('error')}")
                logger.info(f"TCP storage health: {health}")
            
            # Log storage stats
            stats = engine.get_system_stats()
            storage_stats = stats.get('storage', {})
            logger.info(f"Connected to TCP storage: {storage_stats.get('total_concepts', 0)} concepts")
            
            return engine
            
        except Exception as e:
            logger.error(f"Failed to initialize TCP storage: {e}")
            raise RuntimeError(f"TCP storage initialization failed: {e}")
    
    def setup_adapter(self) -> DatasetAdapter:
        """Configure DatasetAdapter for optimal TCP performance."""
        
        def progress_callback(progress):
            """Enhanced progress tracking with TCP metrics."""
            elapsed = progress.elapsed_seconds
            articles_per_sec = progress.chunks_processed / elapsed if elapsed > 0 else 0
            
            # Memory monitoring
            memory_mb = psutil.Process().memory_info().rss / 1024 / 1024
            memory_percent = memory_mb / self.memory_limit_mb * 100
            
            print(f"\r🌐 TCP Progress: {progress.progress_percent:.1f}% | "
                  f"Articles: {progress.chunks_processed}/{progress.total_chunks} | "
                  f"Concepts: {progress.concepts_created} | "
                  f"Rate: {articles_per_sec:.1f}/sec | "
                  f"Memory: {memory_mb:.0f}MB ({memory_percent:.1f}%) | "
                  f"Elapsed: {elapsed:.1f}s", end='', flush=True)
        
        return DatasetAdapter(
            batch_size=self.batch_size,
            min_article_length=150,      # Slightly higher for quality
            max_article_length=6000,     # Smaller chunks for TCP efficiency  
            stream_buffer_size=32768,    # Larger buffer for network efficiency
            progress_callback=progress_callback
        )
    
    def consume_dataset_tcp(self, max_articles: Optional[int] = None) -> Dict[str, Any]:
        """
        Process Wikipedia dataset with TCP-optimized bulk learning.
        
        Args:
            max_articles: Optional limit for testing (None = process all)
        """
        logger.info(f"🚀 Starting TCP-optimized Wikipedia consumption...")
        if max_articles:
            logger.info(f"   Article limit: {max_articles:,}")
        
        adapter = self.setup_adapter()
        self.start_time = time.time()
        
        try:
            # Collect articles in batches for bulk learning
            batch_contents = []
            total_processed = 0
            
            # Create article generator with optional limit
            def article_generator():
                count = 0
                for content, metadata in adapter.get_chunks(str(self.dataset_path), category="encyclopedia"):
                    if max_articles and count >= max_articles:
                        break
                    count += 1
                    yield content, metadata
            
            for content, metadata in article_generator():
                # Add to batch
                batch_contents.append((content, "Wikipedia Dataset", "encyclopedia"))
                
                # Process batch when full
                if len(batch_contents) >= self.batch_size:
                    self._process_batch_tcp(batch_contents, total_processed)
                    total_processed += len(batch_contents)
                    batch_contents = []
                    
                    # Memory management
                    self._memory_cleanup_if_needed()
            
            # Process final partial batch
            if batch_contents:
                self._process_batch_tcp(batch_contents, total_processed)
                total_processed += len(batch_contents)
            
            # Calculate final stats
            end_time = time.time()
            total_time = end_time - self.start_time
            
            self.stats.update({
                'total_articles': total_processed,
                'processing_time': total_time,
                'articles_per_second': total_processed / total_time if total_time > 0 else 0,
            })
            
            print()  # New line after progress
            logger.info("✅ TCP Wikipedia consumption completed successfully!")
            
            # Final TCP storage flush
            if hasattr(self.engine.storage, 'save'):
                self.engine.storage.save()
                logger.info("💾 Flushed all data to TCP storage")
            
            return self.stats
            
        except Exception as e:
            logger.error(f"❌ TCP consumption error: {e}")
            raise
    
    def _process_batch_tcp(self, batch_contents: List[Tuple[str, str, str]], batch_start_idx: int) -> None:
        """Process a batch of articles with TCP-specific optimizations."""
        batch_number = (batch_start_idx // self.batch_size) + 1
        
        try:
            # Use ReasoningEngine's production batch learning
            concept_ids = self.engine.learn_batch(
                batch_contents,
                batch_size=self.batch_size,
                memory_cleanup_interval=3,  # More frequent cleanup for TCP
                fail_on_error=False         # Continue on individual failures
            )
            
            self.stats['concepts_created'] += len(concept_ids)
            
            # Log batch completion
            if batch_number % 10 == 0:
                elapsed = time.time() - self.start_time
                rate = (batch_start_idx + len(batch_contents)) / elapsed
                logger.info(f"📦 TCP Batch {batch_number} complete: {rate:.1f} articles/sec")
            
        except Exception as e:
            logger.error(f"💥 TCP Batch {batch_number} failed: {e}")
            # Continue processing other batches
    
    def _memory_cleanup_if_needed(self) -> None:
        """Force garbage collection if memory usage is high."""
        memory_mb = psutil.Process().memory_info().rss / 1024 / 1024
        memory_percent = memory_mb / self.memory_limit_mb * 100
        
        if memory_percent > 80:  # Above 80% of limit
            gc.collect()
            self.stats['memory_cleanups'] += 1
            new_memory_mb = psutil.Process().memory_info().rss / 1024 / 1024
            logger.info(f"🧹 Memory cleanup: {memory_mb:.0f}MB → {new_memory_mb:.0f}MB")
    
    def verify_tcp_learning(self, sample_queries: List[str]) -> Dict[str, Any]:
        """Verify learning worked by querying the TCP storage."""
        logger.info("🔍 Verifying TCP learning with sample queries...")
        
        verification_results = {
            'total_queries': len(sample_queries),
            'successful_queries': 0,
            'failed_queries': 0,
            'average_confidence': 0,
            'tcp_query_latency': 0,
            'query_results': []
        }
        
        total_confidence = 0
        total_latency = 0
        
        for query in sample_queries:
            try:
                start_time = time.time()
                result = self.engine.ask(query, num_reasoning_paths=3)
                latency = time.time() - start_time
                
                success = result.confidence > 0.3 and len(result.primary_answer) > 30
                
                if success:
                    verification_results['successful_queries'] += 1
                else:
                    verification_results['failed_queries'] += 1
                
                total_confidence += result.confidence
                total_latency += latency
                
                verification_results['query_results'].append({
                    'query': query,
                    'confidence': result.confidence,
                    'success': success,
                    'tcp_latency_ms': latency * 1000,
                    'answer_preview': result.primary_answer[:150] + "..." if len(result.primary_answer) > 150 else result.primary_answer
                })
                
            except Exception as e:
                logger.warning(f"TCP query failed for '{query}': {e}")
                verification_results['failed_queries'] += 1
                total_latency += 1.0  # Assume 1s for failed queries
        
        verification_results['average_confidence'] = (
            total_confidence / len(sample_queries) if sample_queries else 0
        )
        verification_results['tcp_query_latency'] = (
            total_latency / len(sample_queries) * 1000 if sample_queries else 0  # ms
        )
        
        success_rate = (
            verification_results['successful_queries'] / verification_results['total_queries'] * 100
            if verification_results['total_queries'] > 0 else 0
        )
        
        logger.info(f"📊 TCP Verification Results:")
        logger.info(f"  Success Rate: {success_rate:.1f}%")
        logger.info(f"  Average Confidence: {verification_results['average_confidence']:.2f}")
        logger.info(f"  Average TCP Query Latency: {verification_results['tcp_query_latency']:.1f}ms")
        
        return verification_results
    
    def print_tcp_report(self, stats: Dict[str, Any], verification: Dict[str, Any]):
        """Print comprehensive TCP deployment report."""
        print("\n" + "="*80)
        print("🌐 TCP WIKIPEDIA DATASET CONSUMPTION COMPLETE")
        print("="*80)
        
        # Processing stats
        print(f"\n📊 TCP PROCESSING STATISTICS:")
        print(f"  • Total Articles Processed: {stats['total_articles']:,}")
        print(f"  • Concepts Created: {stats['concepts_created']:,}")
        print(f"  • Processing Time: {stats['processing_time']:.1f} seconds")
        print(f"  • Articles per Second: {stats['articles_per_second']:.1f}")
        print(f"  • Memory Cleanups: {stats.get('memory_cleanups', 0)}")
        
        # TCP-specific metrics
        print(f"\n🌐 TCP PERFORMANCE METRICS:")
        success_rate = verification['successful_queries'] / verification['total_queries'] * 100
        print(f"  • Query Success Rate: {success_rate:.1f}% ({verification['successful_queries']}/{verification['total_queries']})")
        print(f"  • Average Query Confidence: {verification['average_confidence']:.2f}")
        print(f"  • Average TCP Query Latency: {verification['tcp_query_latency']:.1f}ms")
        
        # Storage stats from TCP server
        try:
            system_stats = self.engine.get_system_stats()
            storage_stats = system_stats.get('storage', {})
            print(f"\n💾 TCP STORAGE SERVER STATISTICS:")
            print(f"  • Total Concepts in Storage: {storage_stats.get('total_concepts', 'N/A'):,}")
            print(f"  • Total Associations: {storage_stats.get('total_associations', 'N/A'):,}")
            print(f"  • Written Operations: {storage_stats.get('written', 'N/A'):,}")
            print(f"  • Pending Operations: {storage_stats.get('pending', 'N/A'):,}")
        except Exception as e:
            logger.warning(f"Could not retrieve TCP storage stats: {e}")
        
        # Sample successful queries
        print(f"\n🎯 SAMPLE TCP QUERY RESULTS:")
        successful_queries = [r for r in verification['query_results'] if r['success']][:5]
        for result in successful_queries:
            print(f"  • {result['query']} (Confidence: {result['confidence']:.2f}, Latency: {result['tcp_latency_ms']:.0f}ms)")
            print(f"    → {result['answer_preview']}")
        
        print("\n" + "="*80)


def main():
    """Main execution function for TCP Wikipedia consumption."""
    # Common Wikipedia topics for verification
    sample_queries = [
        "What is artificial intelligence?",
        "What is the capital of France?",
        "What is democracy?",
        "What is climate change?", 
        "What is the solar system?",
        "What is evolution?",
        "What is mathematics?",
        "What is the internet?",
        "What is renewable energy?",
        "What is quantum physics?"
    ]
    
    consumer = TcpWikipediaConsumer()
    
    try:
        # 1. Consume dataset with TCP storage
        # For testing: limit to 200 articles
        # For production: set max_articles=None to process full dataset
        stats = consumer.consume_dataset_tcp(max_articles=200)
        
        # 2. Verify TCP learning
        verification = consumer.verify_tcp_learning(sample_queries)
        
        # 3. Print comprehensive report
        consumer.print_tcp_report(stats, verification)
        
        return True
        
    except Exception as e:
        logger.error(f"💥 TCP consumption critical error: {e}")
        return False


if __name__ == "__main__":
    success = main()
    exit(0 if success else 1)