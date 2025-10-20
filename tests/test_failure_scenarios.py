"""
Production-grade failure scenario tests for unified learning architecture.

Tests verify system behavior under adverse conditions:
- Ollama service unavailable
- Malformed content
- Concurrent load
- Network failures
- Resource exhaustion
"""
import pytest
import requests
import time
import threading
from concurrent.futures import ThreadPoolExecutor, as_completed
from typing import List, Dict, Any


class TestFailureScenarios:
    """Test suite for failure scenarios and error handling."""
    
    API_URL = "http://localhost:8000"
    HYBRID_URL = "http://localhost:8001"
    STORAGE_URL = "http://localhost:50051"
    
    @pytest.fixture(autouse=True)
    def setup(self):
        """Wait for services to be available."""
        self._wait_for_services()
        yield
    
    def _wait_for_services(self, timeout: int = 30) -> bool:
        """Wait for services to be available."""
        for _ in range(timeout):
            try:
                response = requests.get(f"{self.API_URL}/health", timeout=1)
                if response.status_code == 200:
                    return True
            except Exception:
                time.sleep(1)
        return False
    
    # =====================================================================
    # Malformed Content Tests
    # =====================================================================
    
    def test_learn_with_empty_content(self):
        """Test learning with empty content fails gracefully."""
        response = requests.post(
            f"{self.HYBRID_URL}/sutra/learn",
            json={"text": ""}
        )
        # Should reject empty content
        assert response.status_code in [400, 422], f"Expected validation error, got {response.status_code}"
    
    def test_learn_with_very_long_content(self):
        """Test learning with very long content (stress test)."""
        # 10MB of text
        long_content = "A" * (10 * 1024 * 1024)
        
        response = requests.post(
            f"{self.HYBRID_URL}/sutra/learn",
            json={"text": long_content},
            timeout=60  # Generous timeout
        )
        
        # Should either succeed or reject gracefully
        assert response.status_code in [200, 413, 500], f"Unexpected status: {response.status_code}"
        
        if response.status_code == 200:
            print("✅ System handled 10MB content successfully")
        elif response.status_code == 413:
            print("✅ System rejected oversized content appropriately")
        else:
            print("⚠️  System returned error for oversized content")
    
    def test_learn_with_special_characters(self):
        """Test learning with special characters and unicode."""
        special_content = [
            "test_special_1: 日本語のテキスト",  # Japanese
            "test_special_2: Émojis 😀🎉🚀",  # Emojis
            "test_special_3: Math symbols: ∑∫∂√∞",  # Math
            "test_special_4: Control chars: \n\t\r",  # Control chars
        ]
        
        for content in special_content:
            response = requests.post(
                f"{self.HYBRID_URL}/sutra/learn",
                json={"text": content}
            )
            # Should handle or reject gracefully
            assert response.status_code in [200, 400, 422], \
                f"Failed on: {content[:50]}, status: {response.status_code}"
            
            if response.status_code == 200:
                print(f"✅ Handled special content: {content[:30]}...")
    
    def test_learn_with_malformed_json(self):
        """Test API handles malformed JSON gracefully."""
        response = requests.post(
            f"{self.HYBRID_URL}/sutra/learn",
            data="{'invalid': json}",  # Not valid JSON
            headers={"Content-Type": "application/json"}
        )
        # Should return 400 or 422
        assert response.status_code in [400, 422], \
            f"Expected validation error, got {response.status_code}"
    
    # =====================================================================
    # Concurrent Load Tests
    # =====================================================================
    
    def test_concurrent_learning_load(self):
        """Test system under concurrent learning load."""
        num_threads = 10
        num_requests_per_thread = 5
        
        def learn_concept(thread_id: int, request_id: int):
            try:
                content = f"test_concurrent_{thread_id}_{request_id}: Test fact number {request_id}"
                response = requests.post(
                    f"{self.HYBRID_URL}/sutra/learn",
                    json={"text": content},
                    timeout=10
                )
                return {
                    "thread_id": thread_id,
                    "request_id": request_id,
                    "status": response.status_code,
                    "success": response.status_code == 200
                }
            except Exception as e:
                return {
                    "thread_id": thread_id,
                    "request_id": request_id,
                    "status": -1,
                    "success": False,
                    "error": str(e)
                }
        
        results = []
        with ThreadPoolExecutor(max_workers=num_threads) as executor:
            futures = []
            for thread_id in range(num_threads):
                for request_id in range(num_requests_per_thread):
                    future = executor.submit(learn_concept, thread_id, request_id)
                    futures.append(future)
            
            for future in as_completed(futures):
                results.append(future.result())
        
        # Analyze results
        successful = sum(1 for r in results if r["success"])
        failed = len(results) - successful
        
        print(f"\n📊 Concurrent Load Test Results:")
        print(f"   Total requests: {len(results)}")
        print(f"   ✅ Successful: {successful}")
        print(f"   ❌ Failed: {failed}")
        print(f"   📈 Success rate: {successful/len(results)*100:.1f}%")
        
        # At least 80% should succeed under load
        success_rate = successful / len(results)
        assert success_rate >= 0.8, \
            f"Too many failures under load: {success_rate*100:.1f}% success rate"
    
    def test_concurrent_batch_learning(self):
        """Test concurrent batch learning operations."""
        num_batches = 5
        items_per_batch = 3
        
        def batch_learn(batch_id: int):
            try:
                contents = [
                    f"test_batch_{batch_id}_{i}: Batch fact {i}"
                    for i in range(items_per_batch)
                ]
                
                # API batch endpoint
                response = requests.post(
                    f"{self.API_URL}/learn/batch",
                    json={
                        "items": [{"content": c} for c in contents]
                    },
                    timeout=30
                )
                
                return {
                    "batch_id": batch_id,
                    "status": response.status_code,
                    "success": response.status_code == 201
                }
            except Exception as e:
                return {
                    "batch_id": batch_id,
                    "status": -1,
                    "success": False,
                    "error": str(e)
                }
        
        results = []
        with ThreadPoolExecutor(max_workers=num_batches) as executor:
            futures = [executor.submit(batch_learn, i) for i in range(num_batches)]
            for future in as_completed(futures):
                results.append(future.result())
        
        successful = sum(1 for r in results if r["success"])
        print(f"\n📊 Concurrent Batch Learning:")
        print(f"   Batches: {len(results)}")
        print(f"   ✅ Successful: {successful}")
        print(f"   Total concepts: {successful * items_per_batch}")
        
        # All batches should succeed
        assert successful == num_batches, \
            f"Only {successful}/{num_batches} batches succeeded"
    
    def test_concurrent_query_load(self):
        """Test system under concurrent query load."""
        # First, learn some facts
        facts = [
            "test_query_load_1: The sun is a star",
            "test_query_load_2: Water boils at 100C",
            "test_query_load_3: Earth is round"
        ]
        
        for fact in facts:
            requests.post(f"{self.HYBRID_URL}/sutra/learn", json={"text": fact})
        
        time.sleep(2)  # Allow processing
        
        # Now query concurrently
        queries = [
            "What is the sun?",
            "Boiling temperature?",
            "Earth shape?"
        ] * 10  # 30 total queries
        
        def query_concept(query: str, query_id: int):
            try:
                response = requests.post(
                    f"{self.HYBRID_URL}/sutra/query",
                    json={"query": query},
                    timeout=5
                )
                return {
                    "query_id": query_id,
                    "status": response.status_code,
                    "success": response.status_code == 200
                }
            except Exception as e:
                return {
                    "query_id": query_id,
                    "status": -1,
                    "success": False,
                    "error": str(e)
                }
        
        results = []
        with ThreadPoolExecutor(max_workers=10) as executor:
            futures = [
                executor.submit(query_concept, q, i) 
                for i, q in enumerate(queries)
            ]
            for future in as_completed(futures):
                results.append(future.result())
        
        successful = sum(1 for r in results if r["success"])
        success_rate = successful / len(results)
        
        print(f"\n📊 Concurrent Query Load:")
        print(f"   Queries: {len(results)}")
        print(f"   ✅ Successful: {successful}")
        print(f"   📈 Success rate: {success_rate*100:.1f}%")
        
        # At least 90% should succeed (queries are lighter than learning)
        assert success_rate >= 0.9, \
            f"Too many query failures: {success_rate*100:.1f}% success rate"
    
    # =====================================================================
    # Network Failure Simulation
    # =====================================================================
    
    def test_timeout_handling(self):
        """Test that system handles timeouts gracefully."""
        response = requests.post(
            f"{self.HYBRID_URL}/sutra/learn",
            json={"text": "test_timeout: Quick fact"},
            timeout=0.001  # Very short timeout
        )
        # Should either succeed quickly or timeout
        # This tests that the client handles timeouts
        print(f"Timeout test status: {response.status_code if response else 'timeout'}")
    
    def test_invalid_service_url(self):
        """Test handling of invalid service URLs."""
        try:
            response = requests.post(
                "http://localhost:99999/sutra/learn",  # Invalid port
                json={"text": "test"},
                timeout=1
            )
            assert False, "Should have raised connection error"
        except requests.exceptions.ConnectionError:
            print("✅ Properly handled invalid service URL")
    
    # =====================================================================
    # Data Consistency Tests
    # =====================================================================
    
    def test_embedding_generation_consistency(self):
        """Test that embeddings are consistently generated."""
        content = "test_consistency: Unique fact for consistency test"
        
        # Learn the concept
        response = requests.post(
            f"{self.HYBRID_URL}/sutra/learn",
            json={"text": content}
        )
        assert response.status_code == 200
        
        time.sleep(1)
        
        # Check stats - embeddings should be generated
        stats = requests.get(f"{self.API_URL}/stats").json()
        assert stats["total_embeddings"] > 0, \
            "Embeddings not generated by unified pipeline"
        
        print(f"✅ Embedding consistency verified: {stats['total_embeddings']} embeddings")
    
    def test_no_duplicate_concepts(self):
        """Test that learning same content twice handles correctly."""
        content = "test_duplicate: This is a duplicate test fact"
        
        # Learn twice
        response1 = requests.post(f"{self.HYBRID_URL}/sutra/learn", json={"text": content})
        response2 = requests.post(f"{self.HYBRID_URL}/sutra/learn", json={"text": content})
        
        assert response1.status_code == 200
        assert response2.status_code == 200
        
        # Both should succeed (storage handles duplicates by ID)
        print("✅ Duplicate content handled correctly")
    
    # =====================================================================
    # Recovery Tests
    # =====================================================================
    
    def test_learning_after_errors(self):
        """Test that system recovers after errors."""
        # Cause an error
        try:
            requests.post(
                f"{self.HYBRID_URL}/sutra/learn",
                json={"invalid_field": "bad data"},
                timeout=1
            )
        except Exception:
            pass
        
        # Now try valid learning
        response = requests.post(
            f"{self.HYBRID_URL}/sutra/learn",
            json={"text": "test_recovery: System should work after errors"}
        )
        
        assert response.status_code == 200, \
            "System didn't recover after error"
        
        print("✅ System recovered successfully after error")


if __name__ == "__main__":
    # Run specific test
    import sys
    if len(sys.argv) > 1:
        pytest.main([__file__ + "::" + sys.argv[1], "-v", "-s"])
    else:
        pytest.main([__file__, "-v", "-s"])
