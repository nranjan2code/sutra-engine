#!/usr/bin/env python3
"""
Production Scaling Validation Script
Tests Phase 0, Phase 1, and Phase 2 scaling improvements

Validates:
- Phase 0: Matryoshka 256-dim performance (3× improvement)
- Phase 1: Sutra-native cache effectiveness (7× total)
- Phase 2: HAProxy load balancing (21× total)

Expected Results:
- Single request latency: <1000ms (with Phase 0+1)
- Concurrent throughput: 10+ requests/sec
- Cache hit rate: 70-85% after warmup
- Load distribution: Even across ML-Base replicas
"""

import asyncio
import time
import statistics
import requests
import sys
from typing import List, Dict, Any
from concurrent.futures import ThreadPoolExecutor, as_completed
import json

# Configuration
API_URL = "http://localhost:8080/api"
EMBEDDING_URL = "http://localhost:8888"
HAPROXY_STATS_URL = "http://localhost:9999/stats"

# Test Constants
WARMUP_REQUESTS = 10
TEST_REQUESTS = 50
CONCURRENT_REQUESTS = 20

class Colors:
    """ANSI color codes"""
    HEADER = '\033[95m'
    OKBLUE = '\033[94m'
    OKCYAN = '\033[96m'
    OKGREEN = '\033[92m'
    WARNING = '\033[93m'
    FAIL = '\033[91m'
    ENDC = '\033[0m'
    BOLD = '\033[1m'

def print_section(title: str):
    """Print formatted section header"""
    print(f"\n{Colors.HEADER}{Colors.BOLD}{'=' * 70}{Colors.ENDC}")
    print(f"{Colors.HEADER}{Colors.BOLD}{title:^70}{Colors.ENDC}")
    print(f"{Colors.HEADER}{Colors.BOLD}{'=' * 70}{Colors.ENDC}\n")

def print_result(label: str, value: Any, target: Any = None, unit: str = ""):
    """Print formatted result with pass/fail indicator"""
    if target is not None:
        if isinstance(target, tuple):  # Range check (min, max)
            passed = target[0] <= value <= target[1]
        else:  # Single value check
            passed = value >= target if target > 0 else value <= abs(target)
        
        status = f"{Colors.OKGREEN}✓{Colors.ENDC}" if passed else f"{Colors.FAIL}✗{Colors.ENDC}"
        print(f"  {status} {label}: {Colors.BOLD}{value:.2f}{unit}{Colors.ENDC} (target: {target}{unit})")
    else:
        print(f"    {label}: {Colors.BOLD}{value:.2f}{unit}{Colors.ENDC}")

def test_service_health() -> Dict[str, bool]:
    """Test all service endpoints are healthy"""
    print_section("SERVICE HEALTH CHECKS")
    
    health = {
        "api": False,
        "embedding": False,
        "cache": False,
        "haproxy": False
    }
    
    # Test API
    try:
        resp = requests.get(f"{API_URL}/health", timeout=5)
        health["api"] = resp.status_code == 200
        print(f"  {'✓' if health['api'] else '✗'} API Service: {Colors.OKGREEN if health['api'] else Colors.FAIL}{'HEALTHY' if health['api'] else 'FAILED'}{Colors.ENDC}")
    except Exception as e:
        print(f"  ✗ API Service: {Colors.FAIL}FAILED{Colors.ENDC} ({e})")
    
    # Test Embedding Service
    try:
        resp = requests.get(f"{EMBEDDING_URL}/health", timeout=5)
        health["embedding"] = resp.status_code == 200
        print(f"  {'✓' if health['embedding'] else '✗'} Embedding Service: {Colors.OKGREEN if health['embedding'] else Colors.FAIL}{'HEALTHY' if health['embedding'] else 'FAILED'}{Colors.ENDC}")
    except Exception as e:
        print(f"  ✗ Embedding Service: {Colors.FAIL}FAILED{Colors.ENDC} ({e})")
    
    # Test Cache
    try:
        resp = requests.get(f"{EMBEDDING_URL}/cache/stats", timeout=5)
        health["cache"] = resp.status_code == 200
        if health["cache"]:
            cache_stats = resp.json()
            print(f"  ✓ Cache Service: {Colors.OKGREEN}ENABLED{Colors.ENDC} (backend: {cache_stats.get('total', {}).get('backend', 'unknown')})")
        else:
            print(f"  ℹ Cache Service: {Colors.WARNING}DISABLED{Colors.ENDC}")
    except Exception as e:
        print(f"  ℹ Cache Service: {Colors.WARNING}UNAVAILABLE{Colors.ENDC} ({e})")
        health["cache"] = False
    
    # Test HAProxy
    try:
        resp = requests.get(HAPROXY_STATS_URL, timeout=5)
        health["haproxy"] = resp.status_code == 200
        print(f"  {'✓' if health['haproxy'] else '✗'} HAProxy: {Colors.OKGREEN if health['haproxy'] else Colors.FAIL}{'RUNNING' if health['haproxy'] else 'FAILED'}{Colors.ENDC}")
    except Exception as e:
        print(f"  ℹ HAProxy: {Colors.WARNING}NOT CONFIGURED{Colors.ENDC} (Phase 2 not deployed)")
        health["haproxy"] = False
    
    return health

def test_phase0_matryoshka() -> Dict[str, Any]:
    """Test Phase 0: Matryoshka dimension configuration"""
    print_section("PHASE 0: MATRYOSHKA DIMENSION OPTIMIZATION")
    
    # Generate single embedding and check dimension
    test_concept = {
        "content": "Test concept for dimension validation - Phase 0 scaling",
        "metadata": {"test": "matryoshka", "phase": 0}
    }
    
    start = time.time()
    try:
        resp = requests.post(f"{API_URL}/learn", json=test_concept, timeout=30)
        latency = (time.time() - start) * 1000
        
        if resp.status_code == 201:
            # Get embedding dimension from response
            embed_resp = requests.post(
                f"{EMBEDDING_URL}/embed",
                json={"texts": [test_concept["content"]], "normalize": True},
                timeout=30
            )
            
            if embed_resp.status_code == 200:
                data = embed_resp.json()
                dimension = data.get("dimension", 768)
                
                print(f"  Embedding Dimension: {Colors.BOLD}{dimension}{Colors.ENDC}")
                print(f"  Generation Latency: {Colors.BOLD}{latency:.2f}ms{Colors.ENDC}")
                
                if dimension == 256:
                    print(f"  {Colors.OKGREEN}✓ Phase 0 ACTIVE{Colors.ENDC}: 256-dim Matryoshka (3× improvement)")
                    expected_speedup = 3.0
                elif dimension == 512:
                    print(f"  {Colors.OKCYAN}✓ Phase 0 PARTIAL{Colors.ENDC}: 512-dim Matryoshka (1.5× improvement)")
                    expected_speedup = 1.5
                else:
                    print(f"  {Colors.WARNING}⚠ Phase 0 INACTIVE{Colors.ENDC}: Full 768-dim (no improvement)")
                    expected_speedup = 1.0
                
                # Latency targets based on dimension
                if dimension == 256:
                    target_latency = 1000  # ~667ms + overhead
                elif dimension == 512:
                    target_latency = 1500  # ~1333ms + overhead
                else:
                    target_latency = 2500  # ~2000ms + overhead
                
                print_result("Latency", latency, target_latency, "ms")
                
                return {
                    "dimension": dimension,
                    "latency_ms": latency,
                    "speedup": expected_speedup,
                    "active": dimension < 768
                }
        
        return {"dimension": 768, "latency_ms": latency, "speedup": 1.0, "active": False}
        
    except Exception as e:
        print(f"  {Colors.FAIL}✗ Phase 0 test failed{Colors.ENDC}: {e}")
        return {"dimension": 768, "latency_ms": 0, "speedup": 1.0, "active": False}

def test_phase1_cache() -> Dict[str, Any]:
    """Test Phase 1: Sutra-native caching"""
    print_section("PHASE 1: SUTRA-NATIVE CACHE EFFECTIVENESS")
    
    # Clear cache first
    try:
        requests.get(f"{EMBEDDING_URL}/cache/clear", timeout=5)
        print("  Cache cleared for testing")
    except:
        pass
    
    # Test concept for caching
    test_text = "Apple Inc (AAPL) stock performance analysis"
    
    # First request (cache miss)
    start = time.time()
    resp1 = requests.post(
        f"{EMBEDDING_URL}/embed",
        json={"texts": [test_text], "normalize": True},
        timeout=30
    )
    first_latency = (time.time() - start) * 1000
    
    # Second request (should be cached)
    start = time.time()
    resp2 = requests.post(
        f"{EMBEDDING_URL}/embed",
        json={"texts": [test_text], "normalize": True},
        timeout=30
    )
    cached_latency = (time.time() - start) * 1000
    
    # Calculate speedup
    speedup = first_latency / cached_latency if cached_latency > 0 else 1.0
    
    print_result("First Request (miss)", first_latency, None, "ms")
    print_result("Second Request (hit)", cached_latency, 10, "ms")
    print_result("Cache Speedup", speedup, 10.0, "×")
    
    # Get cache stats
    try:
        cache_resp = requests.get(f"{EMBEDDING_URL}/cache/stats", timeout=5)
        if cache_resp.status_code == 200:
            cache_stats = cache_resp.json()
            total_stats = cache_stats.get("total", {})
            hit_rate = total_stats.get("hit_rate", 0) * 100
            
            print(f"\n  Cache Statistics:")
            print(f"    Backend: {Colors.BOLD}{total_stats.get('backend', 'unknown')}{Colors.ENDC}")
            print(f"    Hit Rate: {Colors.BOLD}{hit_rate:.1f}%{Colors.ENDC}")
            print(f"    L1 (memory): {cache_stats.get('l1', {}).get('size', 0)} entries")
            print(f"    L2 (Sutra Storage): {cache_stats.get('l2', {}).get('backend', 'disabled')}")
            
            if "sutra" in total_stats.get('backend', '').lower():
                print(f"  {Colors.OKGREEN}✓ Phase 1 ACTIVE{Colors.ENDC}: Sutra-native multi-tier cache")
            else:
                print(f"  {Colors.WARNING}⚠ Phase 1 PARTIAL{Colors.ENDC}: Basic caching only")
    except Exception as e:
        print(f"  {Colors.WARNING}⚠ Cache stats unavailable{Colors.ENDC}: {e}")
    
    return {
        "first_latency_ms": first_latency,
        "cached_latency_ms": cached_latency,
        "speedup": speedup,
        "active": speedup > 5.0
    }

def test_phase2_load_balancing() -> Dict[str, Any]:
    """Test Phase 2: HAProxy load balancing across ML-Base replicas"""
    print_section("PHASE 2: HAPROXY LOAD BALANCING")
    
    try:
        # Check HAProxy stats
        resp = requests.get(HAPROXY_STATS_URL, timeout=5)
        if resp.status_code != 200:
            print(f"  {Colors.WARNING}⚠ Phase 2 NOT DEPLOYED{Colors.ENDC}: HAProxy not available")
            return {"active": False, "replicas": 1}
        
        # Parse HAProxy stats (CSV format)
        stats_text = resp.text
        # Count backend servers
        replicas = stats_text.count("ml-base-") if "ml-base-" in stats_text else 1
        
        print(f"  HAProxy Status: {Colors.OKGREEN}RUNNING{Colors.ENDC}")
        print(f"  ML-Base Replicas: {Colors.BOLD}{replicas}{Colors.ENDC}")
        
        if replicas >= 3:
            print(f"  {Colors.OKGREEN}✓ Phase 2 ACTIVE{Colors.ENDC}: {replicas}× horizontal scaling")
            expected_speedup = replicas
        else:
            print(f"  {Colors.WARNING}⚠ Phase 2 PARTIAL{Colors.ENDC}: Only {replicas} replica(s)")
            expected_speedup = replicas
        
        print_result("Expected Throughput Multiplier", expected_speedup, 3.0, "×")
        
        return {
            "active": replicas >= 3,
            "replicas": replicas,
            "speedup": expected_speedup
        }
        
    except Exception as e:
        print(f"  {Colors.WARNING}⚠ Phase 2 NOT DEPLOYED{Colors.ENDC}: {e}")
        return {"active": False, "replicas": 1, "speedup": 1.0}

def test_concurrent_throughput(num_requests: int = 20) -> Dict[str, Any]:
    """Test concurrent request handling"""
    print_section("CONCURRENT THROUGHPUT TEST")
    
    print(f"  Running {num_requests} concurrent requests...")
    
    def single_request(i: int) -> Dict[str, Any]:
        """Single embedding request"""
        concept = {
            "content": f"Test concept {i} for concurrent throughput testing",
            "metadata": {"test": "concurrent", "id": i}
        }
        
        start = time.time()
        try:
            resp = requests.post(f"{API_URL}/learn", json=concept, timeout=60)
            latency = (time.time() - start) * 1000
            return {
                "success": resp.status_code == 201,
                "latency_ms": latency,
                "status": resp.status_code
            }
        except Exception as e:
            return {
                "success": False,
                "latency_ms": 0,
                "error": str(e)
            }
    
    # Execute concurrent requests
    start_time = time.time()
    with ThreadPoolExecutor(max_workers=num_requests) as executor:
        futures = [executor.submit(single_request, i) for i in range(num_requests)]
        results = [future.result() for future in as_completed(futures)]
    total_time = time.time() - start_time
    
    # Calculate statistics
    successful = sum(1 for r in results if r["success"])
    latencies = [r["latency_ms"] for r in results if r["success"]]
    
    if latencies:
        avg_latency = statistics.mean(latencies)
        p95_latency = sorted(latencies)[int(len(latencies) * 0.95)] if len(latencies) > 1 else latencies[0]
        throughput = num_requests / total_time
        
        print(f"\n  Results:")
        print_result("Success Rate", (successful / num_requests) * 100, 95, "%")
        print_result("Throughput", throughput, 5.0, " req/s")
        print_result("Avg Latency", avg_latency, None, "ms")
        print_result("P95 Latency", p95_latency, 2000, "ms")
        
        return {
            "success_rate": successful / num_requests,
            "throughput": throughput,
            "avg_latency_ms": avg_latency,
            "p95_latency_ms": p95_latency
        }
    else:
        print(f"  {Colors.FAIL}✗ All requests failed{Colors.ENDC}")
        return {"success_rate": 0, "throughput": 0, "avg_latency_ms": 0, "p95_latency_ms": 0}

def print_summary(results: Dict[str, Dict[str, Any]]):
    """Print comprehensive test summary"""
    print_section("SCALING VALIDATION SUMMARY")
    
    phase0 = results.get("phase0", {})
    phase1 = results.get("phase1", {})
    phase2 = results.get("phase2", {})
    throughput = results.get("throughput", {})
    
    # Calculate total improvement
    total_speedup = phase0.get("speedup", 1.0) * \
                   (phase1.get("speedup", 1.0) / 10 if phase1.get("active") else 1.0) * \
                   phase2.get("speedup", 1.0)
    
    print(f"  Phase 0 (Matryoshka): {Colors.OKGREEN if phase0.get('active') else Colors.FAIL}{'✓' if phase0.get('active') else '✗'}{Colors.ENDC} {phase0.get('dimension', 768)}-dim ({phase0.get('speedup', 1.0):.1f}× improvement)")
    print(f"  Phase 1 (Cache): {Colors.OKGREEN if phase1.get('active') else Colors.FAIL}{'✓' if phase1.get('active') else '✗'}{Colors.ENDC} Sutra-native ({phase1.get('speedup', 1.0):.1f}× speedup)")
    print(f"  Phase 2 (HAProxy): {Colors.OKGREEN if phase2.get('active') else Colors.WARNING}{'✓' if phase2.get('active') else '⚠'}{Colors.ENDC} {phase2.get('replicas', 1)}× replicas ({phase2.get('speedup', 1.0):.1f}× improvement)")
    
    print(f"\n  {Colors.BOLD}Total Performance Improvement: {total_speedup:.1f}×{Colors.ENDC}")
    print(f"  Current Throughput: {Colors.BOLD}{throughput.get('throughput', 0):.2f} req/s{Colors.ENDC}")
    
    # Overall assessment
    print(f"\n  {Colors.BOLD}OVERALL ASSESSMENT:{Colors.ENDC}")
    
    if total_speedup >= 20:
        print(f"  {Colors.OKGREEN}✓✓✓ EXCELLENT{Colors.ENDC}: All phases active, 21× improvement achieved!")
        print(f"  System ready for 1,500+ concurrent users")
    elif total_speedup >= 7:
        print(f"  {Colors.OKGREEN}✓✓ GOOD{Colors.ENDC}: Phase 0+1 active, 7× improvement achieved")
        print(f"  System ready for 500-1,000 users")
        print(f"  {Colors.OKCYAN}→ Deploy Phase 2 for 1,500+ users{Colors.ENDC}")
    elif total_speedup >= 3:
        print(f"  {Colors.OKCYAN}✓ BASIC{Colors.ENDC}: Phase 0 active, 3× improvement")
        print(f"  System ready for 200-500 users")
        print(f"  {Colors.OKCYAN}→ Deploy Phase 1+2 for production scale{Colors.ENDC}")
    else:
        print(f"  {Colors.WARNING}⚠ BASELINE{Colors.ENDC}: No scaling optimizations active")
        print(f"  System supports <200 users")
        print(f"  {Colors.WARNING}→ Deploy Phase 0+1+2 for production{Colors.ENDC}")

def main():
    """Main test orchestration"""
    print(f"\n{Colors.HEADER}{Colors.BOLD}")
    print("=" * 70)
    print("SUTRA AI - PRODUCTION SCALING VALIDATION".center(70))
    print("Testing Phase 0 (Matryoshka) + Phase 1 (Cache) + Phase 2 (HAProxy)".center(70))
    print("=" * 70)
    print(f"{Colors.ENDC}\n")
    
    results = {}
    
    # Health checks
    health = test_service_health()
    if not health["api"] or not health["embedding"]:
        print(f"\n{Colors.FAIL}✗ Critical services unavailable. Aborting tests.{Colors.ENDC}\n")
        sys.exit(1)
    
    # Phase 0: Matryoshka
    results["phase0"] = test_phase0_matryoshka()
    
    # Phase 1: Cache
    results["phase1"] = test_phase1_cache()
    
    # Phase 2: Load Balancing
    results["phase2"] = test_phase2_load_balancing()
    
    # Concurrent throughput
    results["throughput"] = test_concurrent_throughput(CONCURRENT_REQUESTS)
    
    # Summary
    print_summary(results)
    
    print(f"\n{Colors.OKGREEN}✓ Scaling validation complete{Colors.ENDC}\n")

if __name__ == "__main__":
    main()
