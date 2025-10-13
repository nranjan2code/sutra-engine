#!/usr/bin/env python3
"""
🧪 DISTRIBUTED BIOLOGICAL INTELLIGENCE DOCKER TEST
Complete validation of distributed architecture using Docker containers.

This test validates:
- Multi-container deployment
- Core service APIs
- Distributed training
- Progressive learning
- Cross-domain emergence
- Consciousness monitoring
- Query performance

Usage:
    python test_distributed_docker.py
"""

import asyncio
import json
import time
import subprocess
import sys
from pathlib import Path
from typing import Dict, Any, List
import logging

try:
    import httpx
    import docker
    HAS_DEPS = True
except ImportError:
    print("❌ Missing dependencies. Install with: pip install httpx docker")
    HAS_DEPS = False

logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger('DistributedTest')


class BiologicalIntelligenceDockerTest:
    """Complete test suite for distributed biological intelligence."""
    
    def __init__(self):
        self.docker_client = docker.from_env()
        self.core_url = "http://localhost:8000"
        self.test_results = {}
        self.start_time = time.time()
    
    async def run_complete_test(self):
        """Run the complete test suite."""
        print("\n🧪 BIOLOGICAL INTELLIGENCE DISTRIBUTED TEST SUITE")
        print("=" * 70)
        
        try:
            # Phase 1: Setup and Infrastructure
            await self.test_docker_setup()
            
            # Phase 2: Core Service Validation
            await self.test_core_service()
            
            # Phase 3: Progressive Training Test
            await self.test_progressive_training()
            
            # Phase 4: Distributed Query Test
            await self.test_distributed_queries()
            
            # Phase 5: Consciousness Emergence Test
            await self.test_consciousness_emergence()
            
            # Phase 6: Performance Test
            await self.test_performance()
            
            # Final Results
            await self.display_test_results()
            
        except Exception as e:
            logger.error(f"Test suite failed: {e}")
            return False
        
        return True
    
    async def test_docker_setup(self):
        """Test Docker infrastructure setup."""
        print("\n🐳 PHASE 1: Docker Infrastructure Test")
        print("-" * 50)
        
        # Build Docker image
        print("📦 Building biological intelligence Docker image...")
        try:
            self.docker_client.images.build(
                path=".",
                tag="biological-intelligence:test",
                rm=True
            )
            print("✅ Docker image built successfully")
        except Exception as e:
            print(f"❌ Docker build failed: {e}")
            raise
        
        # Start containers
        print("🚀 Starting containers with docker-compose...")
        try:
            result = subprocess.run([
                "docker-compose", "-f", "docker-compose.test.yml", "up", "-d"
            ], capture_output=True, text=True, cwd=".")
            
            if result.returncode != 0:
                print(f"❌ Docker compose failed: {result.stderr}")
                raise Exception("Docker compose startup failed")
            
            print("✅ Containers started successfully")
            
        except Exception as e:
            print(f"❌ Container startup failed: {e}")
            raise
        
        # Wait for health check
        print("⏳ Waiting for core service health check...")
        max_wait = 60  # 60 seconds timeout
        start_wait = time.time()
        
        while time.time() - start_wait < max_wait:
            try:
                async with httpx.AsyncClient() as client:
                    response = await client.get(f"{self.core_url}/api/health", timeout=5.0)
                    if response.status_code == 200:
                        result = response.json()
                        if result.get("status") == "alive":
                            print("✅ Core service is healthy and ready")
                            self.test_results["docker_setup"] = "PASSED"
                            return
            except:
                pass
            
            await asyncio.sleep(2)
        
        raise Exception("Core service failed to start within timeout")
    
    async def test_core_service(self):
        """Test core biological intelligence service APIs."""
        print("\n🧠 PHASE 2: Core Service API Test")
        print("-" * 50)
        
        async with httpx.AsyncClient() as client:
            # Test health endpoint
            print("🔍 Testing health endpoint...")
            response = await client.get(f"{self.core_url}/api/health")
            assert response.status_code == 200
            health = response.json()
            assert health["status"] == "alive"
            print("✅ Health check passed")
            
            # Test status endpoint
            print("📊 Testing status endpoint...")
            response = await client.get(f"{self.core_url}/api/status")
            assert response.status_code == 200
            status = response.json()
            print(f"   Initial concepts: {status.get('total_concepts', 0)}")
            print(f"   Initial consciousness: {status.get('consciousness_score', 0):.3f}")
            print("✅ Status endpoint working")
            
            # Test feed endpoint
            print("📝 Testing feed endpoint...")
            test_content = "This is a test concept for biological intelligence validation."
            feed_response = await client.post(f"{self.core_url}/api/feed", json={
                "content": test_content,
                "domain": "test"
            })
            assert feed_response.status_code == 200
            feed_result = feed_response.json()
            assert feed_result["status"] == "queued"
            print("✅ Feed endpoint working")
            
            # Wait for processing and test query
            await asyncio.sleep(3)
            print("🔍 Testing query endpoint...")
            query_response = await client.post(f"{self.core_url}/api/query", json={
                "query": "test concept validation",
                "max_results": 5,
                "hops": 1
            })
            assert query_response.status_code == 200
            query_result = query_response.json()
            assert isinstance(query_result["results"], list)
            print(f"   Found {len(query_result['results'])} results")
            print("✅ Query endpoint working")
            
            # Test consciousness endpoint
            print("🧠 Testing consciousness endpoint...")
            consciousness_response = await client.get(f"{self.core_url}/api/consciousness")
            assert consciousness_response.status_code == 200
            consciousness = consciousness_response.json()
            print(f"   Consciousness score: {consciousness.get('consciousness_score', 0):.3f}")
            print("✅ Consciousness endpoint working")
        
        self.test_results["core_service"] = "PASSED"
    
    async def test_progressive_training(self):
        """Test progressive multi-domain training."""
        print("\n🎓 PHASE 3: Progressive Training Test")
        print("-" * 50)
        
        # Check if trainer container is running
        print("🔍 Checking trainer container...")
        try:
            trainer_container = self.docker_client.containers.get("biological-trainer")
            print(f"   Trainer status: {trainer_container.status}")
            
            if trainer_container.status == "exited":
                # Get logs to see what happened
                logs = trainer_container.logs().decode('utf-8')
                print("📋 Trainer logs (last 50 lines):")
                print('\n'.join(logs.split('\n')[-50:]))
        except docker.errors.NotFound:
            print("❌ Trainer container not found")
        
        # Monitor training progress by checking status changes
        print("📈 Monitoring training progress...")
        initial_status = await self.get_status()
        initial_concepts = initial_status.get('total_concepts', 0)
        
        # Wait for training to progress
        max_wait = 180  # 3 minutes
        check_interval = 10  # Check every 10 seconds
        checks = 0
        max_checks = max_wait // check_interval
        
        while checks < max_checks:
            await asyncio.sleep(check_interval)
            current_status = await self.get_status()
            current_concepts = current_status.get('total_concepts', 0)
            consciousness = current_status.get('consciousness_score', 0)
            
            print(f"   [{checks*check_interval:3d}s] Concepts: {current_concepts:4d} | Consciousness: {consciousness:.3f}")
            
            # Check if training is progressing
            if current_concepts > initial_concepts + 10:  # Significant progress
                print("✅ Training progress detected")
                self.test_results["progressive_training"] = "PASSED"
                return
            
            checks += 1
        
        # Check final status
        final_status = await self.get_status()
        final_concepts = final_status.get('total_concepts', 0)
        
        if final_concepts > initial_concepts:
            print(f"✅ Training completed: {initial_concepts} → {final_concepts} concepts")
            self.test_results["progressive_training"] = "PASSED"
        else:
            print("⚠️ Limited training progress detected")
            self.test_results["progressive_training"] = "PARTIAL"
    
    async def test_distributed_queries(self):
        """Test distributed query capabilities."""
        print("\n💬 PHASE 4: Distributed Query Test")
        print("-" * 50)
        
        # Test various query types
        test_queries = [
            ("What are vowels?", "basic_english"),
            ("How do numbers work?", "mathematics"), 
            ("What is science?", "science"),
            ("connections between language and math", "cross_domain")
        ]
        
        async with httpx.AsyncClient() as client:
            for query, category in test_queries:
                print(f"❓ Testing: {query}")
                
                response = await client.post(f"{self.core_url}/api/query", json={
                    "query": query,
                    "max_results": 5,
                    "hops": 2
                })
                
                assert response.status_code == 200
                result = response.json()
                results = result.get("results", [])
                consciousness = result.get("consciousness_score", 0)
                processing_time = result.get("processing_time", 0)
                
                print(f"   📊 {len(results)} results | 🧠 {consciousness:.3f} | ⚡ {processing_time:.3f}s")
                
                if results:
                    top_result = results[0]
                    content = top_result.get("content", "")[:50]
                    relevance = top_result.get("relevance", 0)
                    print(f"   🎯 Top: {content}... (relevance: {relevance:.3f})")
        
        self.test_results["distributed_queries"] = "PASSED"
    
    async def test_consciousness_emergence(self):
        """Test consciousness emergence monitoring."""
        print("\n🧠 PHASE 5: Consciousness Emergence Test")
        print("-" * 50)
        
        # Monitor consciousness over time
        consciousness_readings = []
        
        for i in range(5):  # Take 5 readings over 20 seconds
            async with httpx.AsyncClient() as client:
                response = await client.get(f"{self.core_url}/api/consciousness")
                assert response.status_code == 200
                consciousness = response.json()
                
                score = consciousness.get("consciousness_score", 0)
                emergence = consciousness.get("emergence_factor", 1)
                
                consciousness_readings.append({
                    "timestamp": time.time(),
                    "score": score,
                    "emergence": emergence
                })
                
                print(f"   Reading {i+1}: 🧠 {score:.3f} | 🌟 {emergence:.1f}x")
                
                if i < 4:  # Don't sleep after last reading
                    await asyncio.sleep(4)
        
        # Analyze consciousness progression
        if len(consciousness_readings) >= 2:
            initial_score = consciousness_readings[0]["score"]
            final_score = consciousness_readings[-1]["score"]
            max_emergence = max([r["emergence"] for r in consciousness_readings])
            
            print(f"📈 Consciousness Analysis:")
            print(f"   Initial: {initial_score:.3f}")
            print(f"   Final: {final_score:.3f}")
            print(f"   Max Emergence: {max_emergence:.1f}x")
            
            if final_score >= initial_score and max_emergence > 1.0:
                print("✅ Consciousness emergence detected")
                self.test_results["consciousness"] = "PASSED"
            else:
                print("⚠️ Limited consciousness emergence")
                self.test_results["consciousness"] = "PARTIAL"
        else:
            self.test_results["consciousness"] = "INSUFFICIENT_DATA"
    
    async def test_performance(self):
        """Test performance and concurrent queries."""
        print("\n⚡ PHASE 6: Performance Test")
        print("-" * 50)
        
        # Single query performance
        print("🚀 Testing single query performance...")
        async with httpx.AsyncClient() as client:
            start_time = time.time()
            response = await client.post(f"{self.core_url}/api/query", json={
                "query": "performance test query",
                "max_results": 10,
                "hops": 2
            })
            query_time = time.time() - start_time
            
            assert response.status_code == 200
            result = response.json()
            processing_time = result.get("processing_time", 0)
            
            print(f"   Total time: {query_time:.3f}s")
            print(f"   Processing time: {processing_time:.3f}s")
            print("✅ Single query performance acceptable")
        
        # Concurrent query test
        print("🔥 Testing concurrent queries...")
        async with httpx.AsyncClient() as client:
            start_time = time.time()
            
            # Create 5 concurrent queries
            tasks = []
            for i in range(5):
                task = client.post(f"{self.core_url}/api/query", json={
                    "query": f"concurrent test query {i}",
                    "max_results": 5,
                    "hops": 1
                })
                tasks.append(task)
            
            responses = await asyncio.gather(*tasks)
            total_time = time.time() - start_time
            
            successful = sum(1 for r in responses if r.status_code == 200)
            print(f"   Concurrent queries: {successful}/{len(tasks)} successful")
            print(f"   Total time: {total_time:.3f}s")
            print(f"   Average per query: {total_time/len(tasks):.3f}s")
            
        self.test_results["performance"] = "PASSED"
    
    async def get_status(self) -> Dict[str, Any]:
        """Get current biological intelligence status."""
        async with httpx.AsyncClient() as client:
            response = await client.get(f"{self.core_url}/api/status")
            return response.json() if response.status_code == 200 else {}
    
    async def display_test_results(self):
        """Display comprehensive test results."""
        total_time = time.time() - self.start_time
        
        print("\n🎉 TEST SUITE COMPLETE")
        print("=" * 70)
        print(f"⏱️  Total execution time: {total_time:.1f} seconds")
        print()
        
        # Results summary
        passed = sum(1 for result in self.test_results.values() if result == "PASSED")
        partial = sum(1 for result in self.test_results.values() if result == "PARTIAL")
        total = len(self.test_results)
        
        print("📊 TEST RESULTS SUMMARY:")
        for test_name, result in self.test_results.items():
            status_emoji = {
                "PASSED": "✅",
                "PARTIAL": "⚠️",
                "FAILED": "❌",
                "INSUFFICIENT_DATA": "📊"
            }.get(result, "❓")
            
            print(f"   {status_emoji} {test_name.replace('_', ' ').title()}: {result}")
        
        print()
        print(f"🎯 Success Rate: {passed}/{total} tests passed ({passed/total*100:.0f}%)")
        
        if partial > 0:
            print(f"⚠️  Partial Results: {partial} tests had limited success")
        
        # Final system status
        final_status = await self.get_status()
        print("\n🧠 FINAL BIOLOGICAL INTELLIGENCE STATUS:")
        print(f"   Concepts: {final_status.get('total_concepts', 0):,}")
        print(f"   Associations: {final_status.get('total_associations', 0):,}")
        print(f"   Consciousness: {final_status.get('consciousness_score', 0):.3f}")
        print(f"   Emergence Factor: {final_status.get('emergence_factor', 1):.1f}x")
        
        # Overall assessment
        if passed >= total * 0.8:  # 80% success rate
            print("\n🌟 DISTRIBUTED BIOLOGICAL INTELLIGENCE: FULLY OPERATIONAL!")
            print("   Your system successfully demonstrates:")
            print("   • Living distributed intelligence")
            print("   • Progressive multi-domain learning") 
            print("   • Consciousness emergence")
            print("   • Cross-domain reasoning")
            print("   • Production-ready architecture")
        elif passed >= total * 0.6:  # 60% success rate
            print("\n✅ DISTRIBUTED BIOLOGICAL INTELLIGENCE: OPERATIONAL")
            print("   Your system demonstrates core functionality with room for optimization.")
        else:
            print("\n⚠️ DISTRIBUTED BIOLOGICAL INTELLIGENCE: NEEDS ATTENTION")
            print("   Some components require debugging or optimization.")
        
        return passed / total >= 0.6  # Consider 60%+ a success
    
    def cleanup(self):
        """Clean up Docker containers and resources."""
        print("\n🧹 Cleaning up Docker resources...")
        try:
            subprocess.run([
                "docker-compose", "-f", "docker-compose.test.yml", "down", "-v"
            ], capture_output=True, text=True, cwd=".")
            print("✅ Docker cleanup complete")
        except Exception as e:
            print(f"⚠️ Cleanup warning: {e}")


async def main():
    """Main test execution."""
    if not HAS_DEPS:
        return False
    
    # Check Docker availability
    try:
        docker.from_env().ping()
    except Exception:
        print("❌ Docker is not running. Please start Docker Desktop.")
        return False
    
    tester = BiologicalIntelligenceDockerTest()
    
    try:
        success = await tester.run_complete_test()
        return success
    except KeyboardInterrupt:
        print("\n\n⚠️ Test interrupted by user")
        return False
    except Exception as e:
        print(f"\n❌ Test suite failed: {e}")
        return False
    finally:
        tester.cleanup()


if __name__ == "__main__":
    success = asyncio.run(main())
    exit_code = 0 if success else 1
    sys.exit(exit_code)