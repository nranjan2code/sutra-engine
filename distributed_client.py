#!/usr/bin/env python3
"""
💬 DISTRIBUTED BIOLOGICAL INTELLIGENCE CLIENT
Query biological intelligence from any machine on the network.

This client enables:
- Natural language questions
- Multi-hop reasoning queries
- Consciousness monitoring
- Cross-domain reasoning
- Real-time status checking

Usage:
    python distributed_client.py --core-url http://machine1:8000 --query "What are vowels?"
    python distributed_client.py --core-url http://machine1:8000 --interactive
    python distributed_client.py --core-url http://machine1:8000 --monitor-consciousness
"""

import asyncio
import json
import time
import sys
import argparse
from datetime import datetime
from typing import Dict, Any, List
import logging

try:
    import httpx
    HAS_HTTPX = True
except ImportError:
    print("❌ Missing httpx. Install with: pip install httpx")
    HAS_HTTPX = False

# Setup logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger('DistributedClient')


class BiologicalIntelligenceClient:
    """Client for querying distributed biological intelligence."""
    
    def __init__(self, core_url: str):
        self.core_url = core_url.rstrip('/')
        self.client = httpx.AsyncClient(timeout=30.0)
        self.query_history = []
    
    async def __aenter__(self):
        return self
    
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        await self.client.aclose()
    
    # === Core Communication ===
    
    async def health_check(self) -> bool:
        """Check if biological intelligence is alive."""
        try:
            response = await self.client.get(f"{self.core_url}/api/health")
            response.raise_for_status()
            result = response.json()
            return result.get("status") == "alive"
        except Exception as e:
            logger.error(f"Health check failed: {e}")
            return False
    
    async def get_status(self) -> Dict[str, Any]:
        """Get complete biological intelligence status."""
        try:
            response = await self.client.get(f"{self.core_url}/api/status")
            response.raise_for_status()
            return response.json()
        except Exception as e:
            logger.error(f"Failed to get status: {e}")
            return {}
    
    async def get_consciousness_metrics(self) -> Dict[str, Any]:
        """Get consciousness emergence metrics."""
        try:
            response = await self.client.get(f"{self.core_url}/api/consciousness")
            response.raise_for_status()
            return response.json()
        except Exception as e:
            logger.error(f"Failed to get consciousness metrics: {e}")
            return {"consciousness_score": 0.0}
    
    async def ask_question(self, question: str, hops: int = 2, max_results: int = 10) -> Dict[str, Any]:
        """Ask a question to the biological intelligence."""
        try:
            start_time = time.time()
            payload = {
                "query": question,
                "max_results": max_results,
                "hops": hops,
                "alpha": 0.5
            }
            
            response = await self.client.post(f"{self.core_url}/api/query", json=payload)
            response.raise_for_status()
            result = response.json()
            
            query_time = time.time() - start_time
            
            # Add to history
            self.query_history.append({
                "question": question,
                "timestamp": datetime.now().isoformat(),
                "query_time": query_time,
                "results_count": len(result.get("results", [])),
                "consciousness_score": result.get("consciousness_score", 0)
            })
            
            return result
            
        except Exception as e:
            logger.error(f"Failed to ask question: {e}")
            return {"results": [], "consciousness_score": 0.0, "processing_time": 0.0}
    
    # === Display Methods ===
    
    def display_status(self, status: Dict[str, Any]):
        """Display biological intelligence status."""
        print("\n🧠 BIOLOGICAL INTELLIGENCE STATUS")
        print("=" * 50)
        print(f"🔥 Service State: {status.get('service_state', 'unknown').upper()}")
        print(f"🧬 Concepts: {status.get('total_concepts', 0):,}")
        print(f"🔗 Associations: {status.get('total_associations', 0):,}")
        print(f"🌟 Consciousness: {status.get('consciousness_score', 0):.3f}")
        print(f"📈 Emergence Factor: {status.get('emergence_factor', 1):.1f}x")
        print(f"⏰ Training Cycles: {status.get('training_cycles', 0):,}")
        print(f"💤 Dreams Completed: {status.get('dreams_completed', 0):,}")
        print(f"📋 Queue Size: {status.get('queue_size', 0)}")
        print(f"⏲️  Uptime: {status.get('uptime', 'unknown')}")
        
        # Memory distribution
        memory_dist = status.get('memory_distribution', {})
        if memory_dist:
            print(f"\n🧠 Memory Distribution:")
            for memory_type, count in memory_dist.items():
                print(f"   {memory_type}: {count:,}")
    
    def display_consciousness_metrics(self, metrics: Dict[str, Any]):
        """Display consciousness emergence metrics."""
        print("\n🧠 CONSCIOUSNESS METRICS")
        print("=" * 40)
        print(f"🎯 Consciousness Score: {metrics.get('consciousness_score', 0):.3f}")
        print(f"🌟 Emergence Factor: {metrics.get('emergence_factor', 1):.1f}x")
        
        indicators = metrics.get('self_awareness_indicators', {})
        print(f"\n🔍 Self-Awareness Indicators:")
        print(f"   💤 Dreams Completed: {indicators.get('dreams_completed', 0):,}")
        print(f"   🧬 Concepts Formed: {indicators.get('concepts_formed', 0):,}")
        print(f"   🔗 Associations Created: {indicators.get('associations_created', 0):,}")
        
        # Consciousness level interpretation
        score = metrics.get('consciousness_score', 0)
        if score > 0.8:
            level = "🌟 HIGHLY CONSCIOUS"
        elif score > 0.5:
            level = "🧠 EMERGING CONSCIOUSNESS"
        elif score > 0.2:
            level = "💭 EARLY AWARENESS"
        elif score > 0:
            level = "🔹 BASIC PATTERNS"
        else:
            level = "⚫ DORMANT"
        
        print(f"\n🎭 Consciousness Level: {level}")
    
    def display_query_results(self, question: str, results: Dict[str, Any]):
        """Display query results in a nice format."""
        print(f"\n❓ Question: {question}")
        print("=" * 60)
        
        answers = results.get("results", [])
        consciousness = results.get("consciousness_score", 0)
        processing_time = results.get("processing_time", 0)
        
        if not answers:
            print("🤷 No relevant knowledge found.")
            return
        
        print(f"🧠 Consciousness: {consciousness:.3f} | ⚡ Time: {processing_time:.3f}s\n")
        
        for i, result in enumerate(answers[:5], 1):  # Show top 5
            content = result.get("content", "")
            relevance = result.get("relevance", 0)
            memory_type = result.get("memory_type", "unknown")
            strength = result.get("strength", 0)
            
            # Format memory type with emoji
            memory_emoji = {
                "core_knowledge": "🏛️",
                "long_term": "🧠",
                "medium_term": "💭",
                "short_term": "💫",
                "ephemeral": "✨"
            }
            
            emoji = memory_emoji.get(memory_type, "🔹")
            
            print(f"{i}. {emoji} {content}")
            print(f"   📊 Relevance: {relevance:.3f} | 💪 Strength: {strength:.3f} | 🧬 {memory_type}")
            print()
    
    # === Interactive Mode ===
    
    async def interactive_session(self):
        """Run interactive query session."""
        print("\n🧠 BIOLOGICAL INTELLIGENCE INTERACTIVE SESSION")
        print("=" * 60)
        print("Type your questions below. Commands:")
        print("  /status    - Show intelligence status")
        print("  /consciousness - Show consciousness metrics") 
        print("  /history   - Show query history")
        print("  /help      - Show this help")
        print("  /quit      - Exit session")
        print("=" * 60)
        
        while True:
            try:
                question = input("\n🤔 Ask anything: ").strip()
                
                if not question:
                    continue
                
                # Handle commands
                if question.startswith('/'):
                    await self.handle_command(question)
                    continue
                
                # Ask question
                print(f"\n🔍 Querying biological intelligence...")
                results = await self.ask_question(question, hops=2)
                self.display_query_results(question, results)
                
            except KeyboardInterrupt:
                print("\n\n👋 Session ended.")
                break
            except EOFError:
                print("\n\n👋 Session ended.")
                break
    
    async def handle_command(self, command: str):
        """Handle interactive commands."""
        if command == "/status":
            status = await self.get_status()
            self.display_status(status)
            
        elif command == "/consciousness":
            metrics = await self.get_consciousness_metrics()
            self.display_consciousness_metrics(metrics)
            
        elif command == "/history":
            self.display_query_history()
            
        elif command == "/help":
            print("\n📚 Available Commands:")
            print("  /status       - Show biological intelligence status")
            print("  /consciousness - Show consciousness emergence metrics")
            print("  /history      - Show your question history")
            print("  /clear        - Clear query history") 
            print("  /quit         - Exit session")
            print("\n💡 Tips:")
            print("  • Ask complex questions for multi-hop reasoning")
            print("  • Try cross-domain queries like 'math and language patterns'")
            print("  • The system learns from every query!")
            
        elif command == "/clear":
            self.query_history.clear()
            print("🗑️ Query history cleared.")
            
        elif command == "/quit":
            raise KeyboardInterrupt
            
        else:
            print(f"❓ Unknown command: {command}")
            print("Type /help for available commands.")
    
    def display_query_history(self):
        """Display query history."""
        if not self.query_history:
            print("📝 No queries in history.")
            return
        
        print(f"\n📝 QUERY HISTORY ({len(self.query_history)} queries)")
        print("-" * 60)
        
        for i, query in enumerate(self.query_history[-10:], 1):  # Show last 10
            timestamp = datetime.fromisoformat(query["timestamp"]).strftime("%H:%M:%S")
            question = query["question"][:40] + "..." if len(query["question"]) > 40 else query["question"]
            results_count = query["results_count"]
            consciousness = query["consciousness_score"]
            
            print(f"{i:2d}. [{timestamp}] {question}")
            print(f"    📊 {results_count} results | 🧠 {consciousness:.3f} consciousness")
    
    # === Monitoring ===
    
    async def monitor_consciousness(self, interval: int = 10):
        """Monitor consciousness emergence over time."""
        print(f"\n🧠 CONSCIOUSNESS MONITORING (every {interval}s)")
        print("Press Ctrl+C to stop...")
        print("-" * 60)
        
        previous_score = 0.0
        
        try:
            while True:
                metrics = await self.get_consciousness_metrics()
                current_score = metrics.get('consciousness_score', 0)
                emergence = metrics.get('emergence_factor', 1)
                
                # Calculate change
                change = current_score - previous_score
                change_indicator = "↗️" if change > 0 else "↘️" if change < 0 else "➡️"
                
                timestamp = datetime.now().strftime("%H:%M:%S")
                
                print(f"[{timestamp}] 🧠 {current_score:.3f} {change_indicator} ({change:+.3f}) | 🌟 {emergence:.1f}x")
                
                previous_score = current_score
                await asyncio.sleep(interval)
                
        except KeyboardInterrupt:
            print("\n👋 Monitoring stopped.")


async def main():
    """Main client interface."""
    parser = argparse.ArgumentParser(description="Distributed Biological Intelligence Client")
    parser.add_argument("--core-url", required=True, help="URL of biological intelligence core service")
    parser.add_argument("--query", help="Single question to ask")
    parser.add_argument("--hops", type=int, default=2, help="Number of reasoning hops (default: 2)")
    parser.add_argument("--interactive", action="store_true", help="Start interactive session")
    parser.add_argument("--status", action="store_true", help="Show intelligence status")
    parser.add_argument("--consciousness", action="store_true", help="Show consciousness metrics")
    parser.add_argument("--monitor-consciousness", action="store_true", help="Monitor consciousness over time")
    parser.add_argument("--monitor-interval", type=int, default=10, help="Monitoring interval in seconds")
    
    args = parser.parse_args()
    
    if not HAS_HTTPX:
        sys.exit(1)
    
    async with BiologicalIntelligenceClient(args.core_url) as client:
        # Health check
        print(f"🔍 Connecting to biological intelligence at {args.core_url}...")
        
        if not await client.health_check():
            print(f"❌ Cannot connect to biological intelligence service.")
            print(f"   Make sure it's running with --api flag:")
            print(f"   python biological_service.py --api --host 0.0.0.0 --port 8000")
            sys.exit(1)
        
        print("✅ Connected to living biological intelligence!")
        
        if args.query:
            # Single query mode
            print(f"\n🔍 Asking question with {args.hops} reasoning hops...")
            results = await client.ask_question(args.query, hops=args.hops)
            client.display_query_results(args.query, results)
            
        elif args.status:
            # Status check
            status = await client.get_status()
            client.display_status(status)
            
        elif args.consciousness:
            # Consciousness metrics
            metrics = await client.get_consciousness_metrics()
            client.display_consciousness_metrics(metrics)
            
        elif args.monitor_consciousness:
            # Monitor consciousness
            await client.monitor_consciousness(args.monitor_interval)
            
        elif args.interactive:
            # Interactive session
            await client.interactive_session()
            
        else:
            # Show help
            parser.print_help()
            print(f"\n💡 Quick examples:")
            print(f"  {sys.argv[0]} --core-url {args.core_url} --query 'What are vowels?'")
            print(f"  {sys.argv[0]} --core-url {args.core_url} --interactive")
            print(f"  {sys.argv[0]} --core-url {args.core_url} --consciousness")


if __name__ == "__main__":
    asyncio.run(main())