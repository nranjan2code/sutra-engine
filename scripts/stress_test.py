#!/usr/bin/env python3
"""
Sutra AI Platform Stress Test

Proper stress testing to find actual system limits and performance characteristics.
Tests concurrent throughput, batch processing, and identifies bottlenecks.
"""

import asyncio
import aiohttp
import time
import json
import argparse
from dataclasses import dataclass
from typing import List, Dict, Any, Optional
from datetime import datetime
import statistics
from concurrent.futures import ThreadPoolExecutor, as_completed
import threading
from rich.console import Console
from rich.progress import Progress, BarColumn, TextColumn, TimeRemainingColumn
from rich.table import Table
from rich.panel import Panel
from rich.live import Live

console = Console()

@dataclass
class StressTestResult:
    """Results from a stress test run."""
    test_name: str
    total_requests: int
    successful_requests: int
    failed_requests: int
    total_time_seconds: float
    requests_per_second: float
    average_latency_ms: float
    p50_latency_ms: float
    p95_latency_ms: float
    p99_latency_ms: float
    max_latency_ms: float
    min_latency_ms: float
    errors: List[str]

class SutraStressTester:
    """Professional stress testing for Sutra AI platform."""
    
    def __init__(self, base_url: str = "http://localhost:8080"):
        self.base_url = base_url
        self.console = Console()
        self.results: List[StressTestResult] = []
    
    def generate_test_concept(self, test_id: int, variation: str = "basic") -> Dict[str, Any]:
        """Generate test concepts with different complexity levels."""
        base_time = datetime.now().isoformat()
        
        if variation == "simple":
            return {
                "text": f"Simple test concept {test_id} at {base_time}",
                "metadata": {"test_id": test_id, "type": "simple_test"}
            }
        elif variation == "complex":
            return {
                "text": f"""Complex Financial Analysis Test {test_id}
                
Market Analysis for Test Company {test_id}:
• Stock Performance: Strong upward trend with 15.2% gain YTD
• Revenue Growth: $2.4B quarterly revenue, beating estimates by 8%
• Market Sentiment: Bullish outlook driven by AI adoption and cloud expansion
• Technical Indicators: RSI at 65.2, MACD showing positive momentum
• Analyst Ratings: 12 Buy, 3 Hold, 1 Sell with average target of $185
• Key Risks: Regulatory concerns, competitive pressure, market volatility
• Opportunities: AI integration, emerging markets, strategic partnerships
• Quarterly Outlook: Expected EPS of $3.45, revenue guidance $2.6B-2.8B

This represents a comprehensive analysis incorporating multiple data sources
and requiring semantic understanding of financial terminology, causal relationships,
and temporal reasoning about market trends and predictions.
                """,
                "metadata": {
                    "test_id": test_id,
                    "type": "complex_financial",
                    "sector": "technology",
                    "timestamp": base_time,
                    "complexity": "high",
                    "requires_embedding": True,
                    "semantic_analysis": True
                }
            }
        else:  # medium
            return {
                "text": f"""Medium Test Concept {test_id}
                
Analysis: Test company {test_id} shows positive growth indicators.
Key metrics include revenue increase and market expansion.
Technical analysis suggests continued upward momentum.
Risk factors include market volatility and competitive pressure.
                """,
                "metadata": {
                    "test_id": test_id,
                    "type": "medium_test", 
                    "timestamp": base_time
                }
            }
    
    async def send_async_request(self, session: aiohttp.ClientSession, concept: Dict[str, Any]) -> Dict[str, Any]:
        """Send async request and measure timing."""
        start_time = time.time()
        try:
            # Reuse connections with connection pooling
            async with session.post(
                f"{self.base_url}/sutra/learn",
                json=concept,
                timeout=aiohttp.ClientTimeout(total=60),  # Increased to 60s for batch processing
                headers={"Connection": "keep-alive"}  # Explicit keep-alive
            ) as response:
                end_time = time.time()
                result = await response.json()
                return {
                    "success": response.status == 200 and result.get("success", False),
                    "latency_ms": (end_time - start_time) * 1000,
                    "status_code": response.status,
                    "error": None if response.status == 200 else f"HTTP {response.status}"
                }
        except asyncio.TimeoutError:
            end_time = time.time()
            return {
                "success": False,
                "latency_ms": (end_time - start_time) * 1000,
                "status_code": 0,
                "error": "Timeout after 60s"
            }
        except Exception as e:
            end_time = time.time()
            return {
                "success": False,
                "latency_ms": (end_time - start_time) * 1000,
                "status_code": 0,
                "error": str(e)
            }
    
    def send_sync_request(self, concept: Dict[str, Any]) -> Dict[str, Any]:
        """Send synchronous request for threaded tests."""
        import requests
        start_time = time.time()
        try:
            response = requests.post(
                f"{self.base_url}/sutra/learn",
                json=concept,
                timeout=60,  # Increased to 60s
                headers={"Connection": "keep-alive"}  # Keep-alive for connection reuse
            )
            end_time = time.time()
            result = response.json()
            return {
                "success": response.status_code == 200 and result.get("success", False),
                "latency_ms": (end_time - start_time) * 1000,
                "status_code": response.status_code,
                "error": None if response.status_code == 200 else f"HTTP {response.status_code}"
            }
        except requests.exceptions.Timeout:
            end_time = time.time()
            return {
                "success": False,
                "latency_ms": (end_time - start_time) * 1000,
                "status_code": 0,
                "error": "Timeout after 60s"
            }
        except Exception as e:
            end_time = time.time()
            return {
                "success": False,
                "latency_ms": (end_time - start_time) * 1000,
                "status_code": 0,
                "error": str(e)
            }
    
    async def test_concurrent_async(self, num_requests: int, concurrency: int, complexity: str = "medium") -> StressTestResult:
        """Test concurrent async requests."""
        test_name = f"Async Concurrent ({concurrency} parallel, {complexity})"
        
        concepts = [self.generate_test_concept(i, complexity) for i in range(num_requests)]
        results = []
        errors = []
        
        start_time = time.time()
        
        # Optimize connection pooling for concurrent requests
        connector = aiohttp.TCPConnector(
            limit=concurrency * 2,  # Double the connection pool
            limit_per_host=concurrency,
            ttl_dns_cache=300,
            force_close=False,  # Keep connections alive
            enable_cleanup_closed=True
        )
        timeout = aiohttp.ClientTimeout(total=60, connect=10)
        
        async with aiohttp.ClientSession(connector=connector, timeout=timeout) as session:
            
            # Process in batches to control concurrency
            for i in range(0, num_requests, concurrency):
                batch = concepts[i:i + concurrency]
                tasks = [self.send_async_request(session, concept) for concept in batch]
                batch_results = await asyncio.gather(*tasks)
                results.extend(batch_results)
                
                # Show progress
                self.console.print(f"Completed batch {i//concurrency + 1}/{(num_requests + concurrency - 1)//concurrency}")
        
        end_time = time.time()
        total_time = end_time - start_time
        
        # Process results
        successful = sum(1 for r in results if r["success"])
        failed = len(results) - successful
        latencies = [r["latency_ms"] for r in results]
        errors = [r["error"] for r in results if r["error"]]
        
        return StressTestResult(
            test_name=test_name,
            total_requests=num_requests,
            successful_requests=successful,
            failed_requests=failed,
            total_time_seconds=total_time,
            requests_per_second=successful / total_time if total_time > 0 else 0,
            average_latency_ms=statistics.mean(latencies) if latencies else 0,
            p50_latency_ms=statistics.median(latencies) if latencies else 0,
            p95_latency_ms=statistics.quantiles(latencies, n=20)[18] if len(latencies) >= 20 else (max(latencies) if latencies else 0),
            p99_latency_ms=statistics.quantiles(latencies, n=100)[98] if len(latencies) >= 100 else (max(latencies) if latencies else 0),
            max_latency_ms=max(latencies) if latencies else 0,
            min_latency_ms=min(latencies) if latencies else 0,
            errors=errors[:10]  # Show first 10 errors
        )
    
    def test_concurrent_threads(self, num_requests: int, num_threads: int, complexity: str = "medium") -> StressTestResult:
        """Test concurrent requests using thread pool."""
        test_name = f"Thread Concurrent ({num_threads} threads, {complexity})"
        
        concepts = [self.generate_test_concept(i, complexity) for i in range(num_requests)]
        results = []
        errors = []
        
        start_time = time.time()
        
        with ThreadPoolExecutor(max_workers=num_threads) as executor:
            future_to_concept = {executor.submit(self.send_sync_request, concept): concept for concept in concepts}
            
            completed = 0
            for future in as_completed(future_to_concept):
                result = future.result()
                results.append(result)
                completed += 1
                
                if completed % 10 == 0 or completed == num_requests:
                    self.console.print(f"Completed {completed}/{num_requests} requests")
        
        end_time = time.time()
        total_time = end_time - start_time
        
        # Process results
        successful = sum(1 for r in results if r["success"])
        failed = len(results) - successful
        latencies = [r["latency_ms"] for r in results]
        errors = [r["error"] for r in results if r["error"]]
        
        return StressTestResult(
            test_name=test_name,
            total_requests=num_requests,
            successful_requests=successful,
            failed_requests=failed,
            total_time_seconds=total_time,
            requests_per_second=successful / total_time if total_time > 0 else 0,
            average_latency_ms=statistics.mean(latencies) if latencies else 0,
            p50_latency_ms=statistics.median(latencies) if latencies else 0,
            p95_latency_ms=statistics.quantiles(latencies, n=20)[18] if len(latencies) >= 20 else (max(latencies) if latencies else 0),
            p99_latency_ms=statistics.quantiles(latencies, n=100)[98] if len(latencies) >= 100 else (max(latencies) if latencies else 0),
            max_latency_ms=max(latencies) if latencies else 0,
            min_latency_ms=min(latencies) if latencies else 0,
            errors=errors[:10]
        )
    
    def test_sequential_baseline(self, num_requests: int, complexity: str = "medium") -> StressTestResult:
        """Test sequential requests to establish baseline."""
        test_name = f"Sequential Baseline ({complexity})"
        
        concepts = [self.generate_test_concept(i, complexity) for i in range(num_requests)]
        results = []
        
        start_time = time.time()
        
        for i, concept in enumerate(concepts):
            result = self.send_sync_request(concept)
            results.append(result)
            
            if (i + 1) % 5 == 0 or i == len(concepts) - 1:
                self.console.print(f"Completed {i + 1}/{num_requests} requests")
        
        end_time = time.time()
        total_time = end_time - start_time
        
        # Process results
        successful = sum(1 for r in results if r["success"])
        failed = len(results) - successful
        latencies = [r["latency_ms"] for r in results]
        errors = [r["error"] for r in results if r["error"]]
        
        return StressTestResult(
            test_name=test_name,
            total_requests=num_requests,
            successful_requests=successful,
            failed_requests=failed,
            total_time_seconds=total_time,
            requests_per_second=successful / total_time if total_time > 0 else 0,
            average_latency_ms=statistics.mean(latencies) if latencies else 0,
            p50_latency_ms=statistics.median(latencies) if latencies else 0,
            p95_latency_ms=statistics.quantiles(latencies, n=20)[18] if len(latencies) >= 20 else (max(latencies) if latencies else 0),
            p99_latency_ms=statistics.quantiles(latencies, n=100)[98] if len(latencies) >= 100 else (max(latencies) if latencies else 0),
            max_latency_ms=max(latencies) if latencies else 0,
            min_latency_ms=min(latencies) if latencies else 0,
            errors=errors[:10]
        )
    
    def print_result(self, result: StressTestResult):
        """Print formatted test result."""
        success_rate = (result.successful_requests / result.total_requests) * 100
        
        # Color coding based on performance
        if success_rate >= 95 and result.requests_per_second > 1.0:
            color = "green"
            icon = "✅"
        elif success_rate >= 90 and result.requests_per_second > 0.5:
            color = "yellow" 
            icon = "⚠️"
        else:
            color = "red"
            icon = "❌"
        
        table = Table(title=f"{icon} {result.test_name}")
        table.add_column("Metric", style="cyan")
        table.add_column("Value", style=color)
        
        table.add_row("Requests", f"{result.successful_requests}/{result.total_requests}")
        table.add_row("Success Rate", f"{success_rate:.1f}%")
        table.add_row("Throughput", f"{result.requests_per_second:.2f} req/sec")
        table.add_row("Total Time", f"{result.total_time_seconds:.1f}s")
        table.add_row("Avg Latency", f"{result.average_latency_ms:.0f}ms")
        table.add_row("P50 Latency", f"{result.p50_latency_ms:.0f}ms")
        table.add_row("P95 Latency", f"{result.p95_latency_ms:.0f}ms")
        table.add_row("Max Latency", f"{result.max_latency_ms:.0f}ms")
        
        if result.errors:
            table.add_row("Sample Errors", "; ".join(result.errors[:3]))
        
        self.console.print(table)
        self.console.print()
    
    def print_summary(self):
        """Print comprehensive test summary."""
        if not self.results:
            return
        
        # Summary table
        summary_table = Table(title="🏁 Stress Test Summary")
        summary_table.add_column("Test", style="cyan")
        summary_table.add_column("Success Rate", style="green")
        summary_table.add_column("Throughput", style="yellow")
        summary_table.add_column("Avg Latency", style="blue")
        
        for result in self.results:
            success_rate = (result.successful_requests / result.total_requests) * 100
            summary_table.add_row(
                result.test_name,
                f"{success_rate:.1f}%",
                f"{result.requests_per_second:.2f} req/sec",
                f"{result.average_latency_ms:.0f}ms"
            )
        
        self.console.print(summary_table)
        
        # Performance insights
        best_throughput = max(self.results, key=lambda r: r.requests_per_second)
        best_latency = min(self.results, key=lambda r: r.average_latency_ms)
        
        insights = Panel(f"""
🚀 Best Throughput: {best_throughput.test_name} ({best_throughput.requests_per_second:.2f} req/sec)
⚡ Best Latency: {best_latency.test_name} ({best_latency.average_latency_ms:.0f}ms)

🎯 Platform Analysis:
• Scaling Configuration: {'✅ Enabled' if any(r.requests_per_second > 1.0 for r in self.results) else '❌ Not Optimal'}
• Concurrent Processing: {'✅ Working' if any('Concurrent' in r.test_name and r.requests_per_second > 0.5 for r in self.results) else '❌ Issues Detected'}
• System Stability: {'✅ Stable' if all(r.successful_requests / r.total_requests > 0.9 for r in self.results) else '⚠️ Some Failures'}
        """, title="Performance Insights", style="bold")
        
        self.console.print(insights)
    
    async def run_comprehensive_test(self, quick: bool = False):
        """Run comprehensive stress test suite."""
        self.console.print(Panel("🔬 Sutra AI Platform Stress Test", style="bold blue"))
        
        if quick:
            # Quick test suite for faster results
            tests = [
                ("sequential", 10, "simple"),
                ("threads_2", 20, 2, "medium"),
                ("async_5", 25, 5, "medium"),
            ]
        else:
            # Comprehensive test suite
            tests = [
                ("sequential", 15, "simple"),
                ("sequential", 10, "medium"),
                ("sequential", 5, "complex"),
                ("threads_2", 20, 2, "medium"),
                ("threads_5", 25, 5, "medium"), 
                ("threads_10", 30, 10, "medium"),
                ("async_5", 25, 5, "medium"),
                ("async_10", 30, 10, "medium"),
                ("async_20", 40, 20, "medium"),
            ]
        
        for test_config in tests:
            test_type = test_config[0]
            num_requests = test_config[1]
            
            self.console.print(f"\n🧪 Running: {test_type}")
            
            try:
                if test_type == "sequential":
                    complexity = test_config[2] if len(test_config) > 2 else "medium"
                    result = self.test_sequential_baseline(num_requests, complexity)
                elif test_type.startswith("threads"):
                    num_threads = test_config[2]
                    complexity = test_config[3] if len(test_config) > 3 else "medium"
                    result = self.test_concurrent_threads(num_requests, num_threads, complexity)
                elif test_type.startswith("async"):
                    concurrency = test_config[2]
                    complexity = test_config[3] if len(test_config) > 3 else "medium"
                    result = await self.test_concurrent_async(num_requests, concurrency, complexity)
                
                self.results.append(result)
                self.print_result(result)
                
            except Exception as e:
                self.console.print(f"❌ Test {test_type} failed: {str(e)}")
        
        self.print_summary()
        
        # Save results
        timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')
        filename = f"stress_test_results_{timestamp}.json"
        
        with open(filename, 'w') as f:
            json.dump([{
                "test_name": r.test_name,
                "total_requests": r.total_requests,
                "successful_requests": r.successful_requests,
                "failed_requests": r.failed_requests,
                "total_time_seconds": r.total_time_seconds,
                "requests_per_second": r.requests_per_second,
                "average_latency_ms": r.average_latency_ms,
                "p95_latency_ms": r.p95_latency_ms,
                "errors": r.errors
            } for r in self.results], f, indent=2)
        
        self.console.print(f"\n📄 Results saved to: {filename}")

async def main():
    parser = argparse.ArgumentParser(description="Sutra AI Platform Stress Test")
    parser.add_argument("--url", default="http://localhost:8080", help="API base URL")
    parser.add_argument("--quick", action="store_true", help="Run quick test suite")
    
    args = parser.parse_args()
    
    tester = SutraStressTester(args.url)
    await tester.run_comprehensive_test(quick=args.quick)

if __name__ == "__main__":
    asyncio.run(main())