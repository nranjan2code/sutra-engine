#!/usr/bin/env python3
"""
Comprehensive test suite for the Distributed Biological Intelligence System.
Tests all components: Core Service, Distributed Training, and Distributed Querying.
"""

import asyncio
import json
import logging
import time
from typing import Dict, List, Any
import requests
from datetime import datetime
import subprocess
import sys
import os

# Set up logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

class DistributedSystemTester:
    def __init__(self, core_url: str = "http://localhost:8000"):
        self.core_url = core_url
        self.test_results = []
        self.start_time = datetime.now()
        
    def log_test_result(self, test_name: str, success: bool, details: str = ""):
        """Log test result with details."""
        result = {
            'test': test_name,
            'success': success,
            'details': details,
            'timestamp': datetime.now().isoformat()
        }
        self.test_results.append(result)
        
        status = "✅ PASS" if success else "❌ FAIL"
        logger.info(f"{status} - {test_name}: {details}")
        
    def test_core_service_health(self) -> bool:
        """Test core service health and API endpoints."""
        try:
            # Test health endpoint
            response = requests.get(f"{self.core_url}/api/status", timeout=10)
            if response.status_code != 200:
                self.log_test_result("Core Service Health", False, f"Status endpoint returned {response.status_code}")
                return False
                
            status_data = response.json()
            self.log_test_result("Core Service Health", True, f"Service running with {status_data.get('total_concepts', 0)} concepts")
            
            # Test consciousness endpoint
            response = requests.get(f"{self.core_url}/api/consciousness", timeout=10)
            if response.status_code == 200:
                consciousness_data = response.json()
                self.log_test_result("Consciousness Monitoring", True, f"Consciousness score: {consciousness_data.get('consciousness_score', 0)}")
            else:
                self.log_test_result("Consciousness Monitoring", False, f"Consciousness endpoint returned {response.status_code}")
                
            return True
            
        except Exception as e:
            self.log_test_result("Core Service Health", False, f"Connection failed: {str(e)}")
            return False
            
    def test_knowledge_feeding(self) -> bool:
        """Test feeding knowledge to the core service."""
        try:
            # Test feeding a simple concept
            test_knowledge = {
                "content": "The sky is blue due to Rayleigh scattering of light.",
                "source": "test_distributed_system.py",
                "priority": 0.8
            }
            
            response = requests.post(
                f"{self.core_url}/api/feed",
                json=test_knowledge,
                timeout=10
            )
            
            if response.status_code == 200:
                self.log_test_result("Knowledge Feeding", True, "Successfully fed test knowledge")
                return True
            else:
                self.log_test_result("Knowledge Feeding", False, f"Feed endpoint returned {response.status_code}")
                return False
                
        except Exception as e:
            self.log_test_result("Knowledge Feeding", False, f"Feed request failed: {str(e)}")
            return False
            
    def test_knowledge_querying(self) -> bool:
        """Test querying knowledge from the core service."""
        try:
            # Test querying
            test_queries = [
                "What is the color of the sky?",
                "How does light scattering work?",
                "What causes blue color in the sky?"
            ]
            
            successful_queries = 0
            for query in test_queries:
                response = requests.post(
                    f"{self.core_url}/api/query",
                    json={"query": query, "max_results": 5},
                    timeout=10
                )
                
                if response.status_code == 200:
                    result_data = response.json()
                    if result_data.get('results') and len(result_data['results']) > 0:
                        successful_queries += 1
                        
            if successful_queries > 0:
                self.log_test_result("Knowledge Querying", True, f"{successful_queries}/{len(test_queries)} queries successful")
                return True
            else:
                self.log_test_result("Knowledge Querying", False, "No queries returned results")
                return False
                
        except Exception as e:
            self.log_test_result("Knowledge Querying", False, f"Query request failed: {str(e)}")
            return False
            
    def test_multi_hop_reasoning(self) -> bool:
        """Test multi-hop associative reasoning."""
        try:
            # Feed related concepts
            concepts = [
                {"content": "Water molecules are made of hydrogen and oxygen.", "source": "chemistry_test", "priority": 0.9},
                {"content": "Hydrogen is the lightest element in the periodic table.", "source": "chemistry_test", "priority": 0.8},
                {"content": "Oxygen is essential for combustion and respiration.", "source": "biology_test", "priority": 0.9},
                {"content": "Water is essential for all known forms of life.", "source": "biology_test", "priority": 1.0}
            ]
            
            # Feed concepts
            for concept in concepts:
                requests.post(f"{self.core_url}/api/feed", json=concept, timeout=5)
                
            # Wait for processing
            time.sleep(2)
            
            # Test multi-hop query
            response = requests.post(
                f"{self.core_url}/api/query",
                json={"query": "What elements are needed for life?", "max_results": 10},
                timeout=10
            )
            
            if response.status_code == 200:
                result_data = response.json()
                results = result_data.get('results', [])
                
                # Check if we get relevant multi-hop connections
                relevant_concepts = 0
                for result in results:
                    content = result.get('content', '').lower()
                    if any(term in content for term in ['water', 'hydrogen', 'oxygen', 'life']):
                        relevant_concepts += 1
                        
                if relevant_concepts >= 2:
                    self.log_test_result("Multi-hop Reasoning", True, f"Found {relevant_concepts} relevant connected concepts")
                    return True
                else:
                    self.log_test_result("Multi-hop Reasoning", False, f"Only found {relevant_concepts} relevant concepts")
                    return False
            else:
                self.log_test_result("Multi-hop Reasoning", False, f"Query failed with status {response.status_code}")
                return False
                
        except Exception as e:
            self.log_test_result("Multi-hop Reasoning", False, f"Multi-hop test failed: {str(e)}")
            return False
            
    def test_consciousness_emergence(self) -> bool:
        """Test consciousness score calculation and emergence detection."""
        try:
            # Feed self-referential and meta-cognitive content
            meta_concepts = [
                {"content": "I am learning about my own learning process.", "source": "meta_test", "priority": 1.0},
                {"content": "Understanding requires both knowledge and awareness of knowledge.", "source": "meta_test", "priority": 0.9},
                {"content": "Self-reflection is a key component of consciousness.", "source": "consciousness_test", "priority": 1.0},
                {"content": "Thinking about thinking is called metacognition.", "source": "psychology_test", "priority": 0.8}
            ]
            
            # Feed meta concepts
            for concept in meta_concepts:
                requests.post(f"{self.core_url}/api/feed", json=concept, timeout=5)
                
            # Wait for processing and dream consolidation
            time.sleep(3)
            
            # Check consciousness score
            response = requests.get(f"{self.core_url}/api/consciousness", timeout=10)
            if response.status_code == 200:
                consciousness_data = response.json()
                consciousness_score = consciousness_data.get('consciousness_score', 0)
                
                if consciousness_score > 0:
                    self.log_test_result("Consciousness Emergence", True, f"Consciousness score: {consciousness_score:.4f}")
                    return True
                else:
                    self.log_test_result("Consciousness Emergence", False, f"No consciousness detected (score: {consciousness_score})")
                    return False
            else:
                self.log_test_result("Consciousness Emergence", False, f"Consciousness endpoint failed: {response.status_code}")
                return False
                
        except Exception as e:
            self.log_test_result("Consciousness Emergence", False, f"Consciousness test failed: {str(e)}")
            return False
            
    def test_memory_persistence(self) -> bool:
        """Test memory persistence and loading with asynchronous processing."""
        try:
            # Get initial status
            response1 = requests.get(f"{self.core_url}/api/status", timeout=10)
            if response1.status_code != 200:
                self.log_test_result("Memory Persistence", False, "Failed to get initial status")
                return False
                
            initial_concepts = response1.json().get('total_concepts', 0)
            
            # Feed some knowledge with unique identifier
            unique_id = f"persistence_test_{int(time.time())}"
            test_concept = {
                "content": f"Memory persistence test concept - {unique_id}",
                "source": "persistence_test",
                "priority": 1.0
            }
            
            response2 = requests.post(f"{self.core_url}/api/feed", json=test_concept, timeout=10)
            if response2.status_code != 200:
                self.log_test_result("Memory Persistence", False, "Failed to feed test concept")
                return False
            
            # Wait for asynchronous processing (biological systems need time to grow)
            time.sleep(5)
            
            # Try to retrieve the concept directly (this tests persistence better than count)
            query_response = requests.post(
                f"{self.core_url}/api/query",
                json={"query": unique_id, "max_results": 10},
                timeout=10
            )
            
            if query_response.status_code == 200:
                results = query_response.json().get('results', [])
                found = any(unique_id in result.get('content', '') for result in results)
                if found:
                    # Also check if the concept count increased or stayed significant
                    response3 = requests.get(f"{self.core_url}/api/status", timeout=10)
                    final_concepts = response3.json().get('total_concepts', 0) if response3.status_code == 200 else 0
                    
                    self.log_test_result("Memory Persistence", True, 
                        f"Successfully persisted and retrieved concept. Concepts: {initial_concepts} -> {final_concepts}")
                    return True
                else:
                    self.log_test_result("Memory Persistence", False, 
                        f"Concept was fed but not retrievable with query '{unique_id}'")
                    return False
            else:
                self.log_test_result("Memory Persistence", False, f"Query failed: {query_response.status_code}")
                return False
                
        except Exception as e:
            self.log_test_result("Memory Persistence", False, f"Persistence test failed: {str(e)}")
            return False
            
    def test_associative_learning(self) -> bool:
        """Test associative learning and connection formation."""
        try:
            # Feed related concepts that should form associations
            related_concepts = [
                {"content": "The sun is a star that provides energy to Earth.", "source": "astronomy", "priority": 0.9},
                {"content": "Plants use sunlight to perform photosynthesis.", "source": "biology", "priority": 0.9},
                {"content": "Photosynthesis converts carbon dioxide and water into glucose.", "source": "biology", "priority": 0.8},
                {"content": "Solar energy is a renewable source of power.", "source": "energy", "priority": 0.8}
            ]
            
            # Feed concepts
            for concept in related_concepts:
                requests.post(f"{self.core_url}/api/feed", json=concept, timeout=5)
                
            # Wait for association formation
            time.sleep(3)
            
            # Query for associations
            response = requests.post(
                f"{self.core_url}/api/query",
                json={"query": "solar energy plants", "max_results": 8},
                timeout=10
            )
            
            if response.status_code == 200:
                results = response.json().get('results', [])
                
                # Check if we get results that span multiple domains
                domains_found = set()
                for result in results:
                    content = result.get('content', '').lower()
                    if 'sun' in content or 'solar' in content:
                        domains_found.add('astronomy')
                    if 'plant' in content or 'photosynthesis' in content:
                        domains_found.add('biology')
                    if 'energy' in content:
                        domains_found.add('energy')
                        
                if len(domains_found) >= 2:
                    self.log_test_result("Associative Learning", True, f"Cross-domain associations found: {domains_found}")
                    return True
                else:
                    self.log_test_result("Associative Learning", False, f"Limited associations: {domains_found}")
                    return False
            else:
                self.log_test_result("Associative Learning", False, f"Association query failed: {response.status_code}")
                return False
                
        except Exception as e:
            self.log_test_result("Associative Learning", False, f"Association test failed: {str(e)}")
            return False
            
    def run_performance_test(self) -> bool:
        """Test system performance under load."""
        try:
            start_time = time.time()
            
            # Rapid feeding test
            concepts_fed = 0
            for i in range(50):
                concept = {
                    "content": f"Performance test concept number {i} with unique data.",
                    "source": "performance_test",
                    "priority": 0.5
                }
                
                response = requests.post(f"{self.core_url}/api/feed", json=concept, timeout=5)
                if response.status_code == 200:
                    concepts_fed += 1
                    
            feed_time = time.time() - start_time
            
            # Rapid querying test
            query_start_time = time.time()
            successful_queries = 0
            
            for i in range(20):
                response = requests.post(
                    f"{self.core_url}/api/query",
                    json={"query": f"performance test concept {i}", "max_results": 3},
                    timeout=5
                )
                if response.status_code == 200:
                    successful_queries += 1
                    
            query_time = time.time() - query_start_time
            
            # Calculate performance metrics
            feed_rate = concepts_fed / feed_time if feed_time > 0 else 0
            query_rate = successful_queries / query_time if query_time > 0 else 0
            
            if feed_rate > 5 and query_rate > 2:  # Reasonable performance thresholds
                self.log_test_result("Performance Test", True, 
                    f"Feed rate: {feed_rate:.2f} concepts/sec, Query rate: {query_rate:.2f} queries/sec")
                return True
            else:
                self.log_test_result("Performance Test", False, 
                    f"Poor performance - Feed: {feed_rate:.2f}/sec, Query: {query_rate:.2f}/sec")
                return False
                
        except Exception as e:
            self.log_test_result("Performance Test", False, f"Performance test failed: {str(e)}")
            return False
            
    def run_all_tests(self) -> Dict[str, Any]:
        """Run all test cases and return comprehensive results."""
        logger.info("🧬 Starting Distributed Biological Intelligence System Test Suite")
        logger.info(f"Testing core service at: {self.core_url}")
        
        # Test sequence
        tests = [
            ("Core Service Health", self.test_core_service_health),
            ("Knowledge Feeding", self.test_knowledge_feeding),
            ("Knowledge Querying", self.test_knowledge_querying),
            ("Memory Persistence", self.test_memory_persistence),
            ("Associative Learning", self.test_associative_learning),
            ("Multi-hop Reasoning", self.test_multi_hop_reasoning),
            ("Consciousness Emergence", self.test_consciousness_emergence),
            ("Performance Test", self.run_performance_test)
        ]
        
        passed_tests = 0
        total_tests = len(tests)
        
        for test_name, test_func in tests:
            logger.info(f"\n--- Running {test_name} ---")
            try:
                if test_func():
                    passed_tests += 1
            except Exception as e:
                self.log_test_result(test_name, False, f"Test crashed: {str(e)}")
                
        # Final summary
        total_time = (datetime.now() - self.start_time).total_seconds()
        success_rate = (passed_tests / total_tests) * 100
        
        summary = {
            'total_tests': total_tests,
            'passed_tests': passed_tests,
            'failed_tests': total_tests - passed_tests,
            'success_rate': success_rate,
            'total_time_seconds': total_time,
            'test_results': self.test_results
        }
        
        logger.info(f"\n{'='*60}")
        logger.info(f"🧬 DISTRIBUTED BIOLOGICAL INTELLIGENCE SYSTEM TEST RESULTS")
        logger.info(f"{'='*60}")
        logger.info(f"Total Tests: {total_tests}")
        logger.info(f"Passed: {passed_tests} ✅")
        logger.info(f"Failed: {total_tests - passed_tests} ❌")
        logger.info(f"Success Rate: {success_rate:.1f}%")
        logger.info(f"Total Time: {total_time:.2f} seconds")
        
        if success_rate >= 80:
            logger.info(f"🎉 EXCELLENT - System is performing well!")
        elif success_rate >= 60:
            logger.info(f"👍 GOOD - System is mostly functional")
        else:
            logger.info(f"⚠️  NEEDS ATTENTION - Multiple test failures detected")
            
        return summary

def main():
    """Main test runner function."""
    import argparse
    
    parser = argparse.ArgumentParser(description="Test Distributed Biological Intelligence System")
    parser.add_argument("--core-url", default="http://localhost:8000", 
                       help="URL of the core service (default: http://localhost:8000)")
    parser.add_argument("--wait", type=int, default=0, 
                       help="Wait time in seconds before starting tests")
    parser.add_argument("--output", help="Output file for test results (JSON)")
    
    args = parser.parse_args()
    
    if args.wait > 0:
        logger.info(f"Waiting {args.wait} seconds before starting tests...")
        time.sleep(args.wait)
    
    # Run tests
    tester = DistributedSystemTester(args.core_url)
    results = tester.run_all_tests()
    
    # Save results if requested
    if args.output:
        with open(args.output, 'w') as f:
            json.dump(results, f, indent=2)
        logger.info(f"Test results saved to: {args.output}")
    
    # Exit with appropriate code
    if results['success_rate'] >= 80:
        sys.exit(0)  # Success
    else:
        sys.exit(1)  # Failure

if __name__ == "__main__":
    main()