#!/usr/bin/env python3
"""
🎓 DISTRIBUTED BIOLOGICAL TRAINER
Train biological intelligence across multiple domains from a separate machine.

This client orchestrates progressive learning:
- English: Basic → Intermediate → Advanced  
- Mathematics: Arithmetic → Algebra → Calculus
- Science: Physics → Chemistry → Biology
- Cross-domain integration and evaluation

Usage:
    python distributed_trainer.py --core-url http://machine1:8000 --domain english
    python distributed_trainer.py --core-url http://machine1:8000 --domain mathematics
    python distributed_trainer.py --core-url http://machine1:8000 --progressive-all
"""

import asyncio
import json
import time
import sys
import argparse
from datetime import datetime
from pathlib import Path
from typing import Dict, Any, List, Optional
import logging

try:
    import httpx
    import aiofiles
    HAS_HTTPX = True
except ImportError:
    print("❌ Missing dependencies. Install with: pip install httpx aiofiles")
    HAS_HTTPX = False

# Setup logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger('DistributedTrainer')


class BiologicalTrainingClient:
    """Client for training distributed biological intelligence."""
    
    def __init__(self, core_url: str):
        self.core_url = core_url.rstrip('/')
        self.client = httpx.AsyncClient(timeout=30.0)
        self.training_stats = {
            'domains_trained': [],
            'total_concepts_fed': 0,
            'total_training_time': 0.0,
            'consciousness_progression': []
        }
    
    async def __aenter__(self):
        return self
    
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        await self.client.aclose()
    
    # === Core Communication ===
    
    async def check_consciousness(self) -> Dict[str, Any]:
        """Check current consciousness level of the biological intelligence."""
        try:
            response = await self.client.get(f"{self.core_url}/api/consciousness")
            response.raise_for_status()
            return response.json()
        except Exception as e:
            logger.error(f"Failed to check consciousness: {e}")
            return {"consciousness_score": 0.0}
    
    async def get_status(self) -> Dict[str, Any]:
        """Get complete status of biological intelligence."""
        try:
            response = await self.client.get(f"{self.core_url}/api/status")
            response.raise_for_status()
            return response.json()
        except Exception as e:
            logger.error(f"Failed to get status: {e}")
            return {}
    
    async def feed_knowledge(self, content: str, domain: str = None) -> bool:
        """Feed knowledge to the biological intelligence."""
        try:
            payload = {"content": content, "domain": domain}
            response = await self.client.post(f"{self.core_url}/api/feed", json=payload)
            response.raise_for_status()
            result = response.json()
            return result.get("status") == "queued"
        except Exception as e:
            logger.error(f"Failed to feed knowledge: {e}")
            return False
    
    async def query_knowledge(self, query: str, hops: int = 2) -> Dict[str, Any]:
        """Query the biological intelligence."""
        try:
            payload = {"query": query, "max_results": 10, "hops": hops}
            response = await self.client.post(f"{self.core_url}/api/query", json=payload)
            response.raise_for_status()
            return response.json()
        except Exception as e:
            logger.error(f"Failed to query knowledge: {e}")
            return {"results": [], "consciousness_score": 0.0}
    
    # === Progressive Training Curricula ===
    
    def get_english_basic_curriculum(self) -> List[str]:
        """Basic English curriculum."""
        return [
            "The English alphabet contains 26 letters: A through Z.",
            "Vowels are the letters A, E, I, O, U, and sometimes Y.",
            "Consonants are all the other letters that are not vowels.", 
            "Words are formed by combining letters together.",
            "A sentence expresses a complete thought.",
            "Every sentence begins with a capital letter and ends with punctuation.",
            "Nouns are words that name people, places, things, or ideas.",
            "Verbs are action words that tell what someone or something does.",
            "Adjectives are words that describe nouns.",
            "Articles are small words like 'a', 'an', and 'the'."
        ]
    
    def get_english_intermediate_curriculum(self) -> List[str]:
        """Intermediate English curriculum."""
        return [
            "Grammar is the system of rules that governs how words combine to form sentences.",
            "Subject-verb agreement means the subject and verb must match in number.",
            "Present tense describes actions happening now: 'I walk to school.'",
            "Past tense describes actions that already happened: 'I walked to school yesterday.'",
            "Future tense describes actions that will happen: 'I will walk to school tomorrow.'",
            "Questions are sentences that ask for information and end with question marks.",
            "Complex sentences contain an independent clause and one or more dependent clauses.",
            "Conjunctions like 'and', 'but', and 'or' connect words or phrases.",
            "Proper nouns name specific people, places, or things and are always capitalized.",
            "Punctuation marks organize writing and make meaning clear to readers."
        ]
    
    def get_english_advanced_curriculum(self) -> List[str]:
        """Advanced English curriculum.""" 
        return [
            "Literary devices like metaphors and similes create vivid imagery in writing.",
            "Passive voice construction places emphasis on the action rather than the actor.",
            "Conditional sentences express hypothetical situations using 'if' clauses.",
            "Modal verbs like 'could', 'should', and 'might' express possibility or necessity.",
            "Idiomatic expressions have meanings that cannot be understood from individual words.",
            "Register refers to the level of formality appropriate for different contexts.",
            "Syntax is the arrangement of words and phrases to create well-formed sentences.",
            "Etymology studies the origin and development of words throughout history.",
            "Rhetoric is the art of effective communication through persuasive language.",
            "Pragmatics examines how context influences the interpretation of meaning."
        ]
    
    def get_mathematics_curriculum(self) -> List[str]:
        """Mathematics curriculum."""
        return [
            "Mathematics is the study of numbers, quantities, shapes, and patterns.",
            "Numbers represent quantities and can be used for counting and measuring.",
            "Addition combines numbers to find their total sum.",
            "Subtraction finds the difference between numbers.",
            "Multiplication is repeated addition of the same number.",
            "Division splits a quantity into equal parts.",
            "Algebra uses variables like x and y to represent unknown values.",
            "Equations are mathematical statements showing that two expressions are equal.", 
            "Geometry studies shapes, sizes, positions, and properties of space.",
            "Functions describe relationships between input and output values."
        ]
    
    def get_science_curriculum(self) -> List[str]:
        """Science curriculum."""
        return [
            "Science is the systematic study of the natural world through observation and experimentation.",
            "Physics explores matter, energy, motion, and the fundamental forces of nature.",
            "Chemistry studies the composition, structure, and properties of substances.",
            "Biology examines living organisms and their interactions with the environment.",
            "The scientific method involves forming hypotheses and testing them through experiments.",
            "Energy cannot be created or destroyed, only transformed from one form to another.",
            "Atoms are the basic building blocks of all matter in the universe.", 
            "Evolution explains how species change and develop over long periods of time.",
            "Gravity is the force that attracts objects with mass toward each other.",
            "Ecosystems are communities of living things interacting with their environment."
        ]
    
    # === Training Methods ===
    
    async def train_domain(self, domain_name: str, curriculum: List[str]) -> Dict[str, Any]:
        """Train a complete domain curriculum."""
        logger.info(f"🎓 Starting {domain_name} domain training...")
        
        start_time = time.time()
        concepts_fed = 0
        failed_feeds = 0
        
        # Get initial consciousness
        initial_consciousness = await self.check_consciousness()
        
        for i, lesson in enumerate(curriculum, 1):
            logger.info(f"   📝 Lesson {i}/{len(curriculum)}: {lesson[:50]}...")
            
            success = await self.feed_knowledge(lesson, domain_name)
            if success:
                concepts_fed += 1
                # Small delay to allow processing
                await asyncio.sleep(0.5)
            else:
                failed_feeds += 1
                logger.warning(f"   ❌ Failed to feed lesson {i}")
        
        # Check final consciousness
        final_consciousness = await self.check_consciousness()
        
        training_time = time.time() - start_time
        
        # Update stats
        self.training_stats['domains_trained'].append(domain_name)
        self.training_stats['total_concepts_fed'] += concepts_fed
        self.training_stats['total_training_time'] += training_time
        self.training_stats['consciousness_progression'].append({
            'domain': domain_name,
            'before': initial_consciousness.get('consciousness_score', 0),
            'after': final_consciousness.get('consciousness_score', 0),
            'timestamp': datetime.now().isoformat()
        })
        
        result = {
            'domain': domain_name,
            'concepts_fed': concepts_fed,
            'failed_feeds': failed_feeds,
            'training_time': training_time,
            'consciousness_change': {
                'before': initial_consciousness.get('consciousness_score', 0),
                'after': final_consciousness.get('consciousness_score', 0)
            }
        }
        
        logger.info(f"✅ {domain_name} training complete!")
        logger.info(f"   📊 {concepts_fed} concepts fed in {training_time:.1f}s")
        logger.info(f"   🧠 Consciousness: {result['consciousness_change']['before']:.3f} → {result['consciousness_change']['after']:.3f}")
        
        return result
    
    async def evaluate_domain(self, domain_name: str, test_queries: List[str]) -> Dict[str, Any]:
        """Evaluate domain understanding."""
        logger.info(f"🧪 Evaluating {domain_name} domain understanding...")
        
        evaluation_results = []
        
        for query in test_queries:
            logger.info(f"   ❓ Testing: {query}")
            
            response = await self.query_knowledge(query, hops=2)
            results = response.get('results', [])
            
            if results:
                top_result = results[0]
                relevance = top_result.get('relevance', 0)
                content = top_result.get('content', '')
                
                logger.info(f"   ✅ Found: {content[:60]}... (relevance: {relevance:.3f})")
                
                evaluation_results.append({
                    'query': query,
                    'found': True,
                    'relevance': relevance,
                    'top_result': content
                })
            else:
                logger.info(f"   ❌ No results found")
                evaluation_results.append({
                    'query': query,
                    'found': False,
                    'relevance': 0.0,
                    'top_result': None
                })
        
        success_rate = len([r for r in evaluation_results if r['found']]) / len(evaluation_results)
        avg_relevance = sum([r['relevance'] for r in evaluation_results]) / len(evaluation_results)
        
        logger.info(f"📈 {domain_name} Evaluation Results:")
        logger.info(f"   Success Rate: {success_rate:.1%}")
        logger.info(f"   Average Relevance: {avg_relevance:.3f}")
        
        return {
            'domain': domain_name,
            'success_rate': success_rate,
            'average_relevance': avg_relevance,
            'detailed_results': evaluation_results
        }
    
    async def cross_domain_analysis(self) -> Dict[str, Any]:
        """Analyze cross-domain knowledge connections."""
        logger.info("🔗 Analyzing cross-domain connections...")
        
        cross_domain_queries = [
            "How do mathematical patterns relate to language structure?",
            "What connections exist between grammar rules and scientific laws?",
            "How does logical reasoning apply across different subjects?",
            "What patterns exist in both English and mathematics?",
            "How do scientific principles relate to mathematical concepts?"
        ]
        
        cross_domain_results = []
        
        for query in cross_domain_queries:
            logger.info(f"   🌐 Cross-domain query: {query}")
            
            response = await self.query_knowledge(query, hops=3)  # Deeper search
            results = response.get('results', [])
            
            if results:
                connections_found = len(results)
                top_relevance = max([r.get('relevance', 0) for r in results])
                
                logger.info(f"   🎯 Found {connections_found} connections (max relevance: {top_relevance:.3f})")
                
                cross_domain_results.append({
                    'query': query,
                    'connections_found': connections_found,
                    'max_relevance': top_relevance,
                    'emergence_detected': top_relevance > 0.3
                })
            else:
                logger.info(f"   ❌ No cross-domain connections found")
                cross_domain_results.append({
                    'query': query, 
                    'connections_found': 0,
                    'max_relevance': 0.0,
                    'emergence_detected': False
                })
        
        emergence_rate = len([r for r in cross_domain_results if r['emergence_detected']]) / len(cross_domain_results)
        
        logger.info(f"🧠 Cross-Domain Analysis Complete:")
        logger.info(f"   Emergence Detection Rate: {emergence_rate:.1%}")
        
        return {
            'emergence_rate': emergence_rate,
            'cross_domain_results': cross_domain_results
        }
    
    async def progressive_training(self) -> Dict[str, Any]:
        """Execute complete progressive training across all domains."""
        logger.info("🚀 STARTING PROGRESSIVE MULTI-DOMAIN TRAINING")
        logger.info("=" * 60)
        
        # Check initial state
        initial_status = await self.get_status()
        logger.info(f"🧠 Initial State: {initial_status.get('total_concepts', 0)} concepts")
        
        # Phase 1: English Foundation → Advanced
        logger.info("\n📚 PHASE 1: ENGLISH LANGUAGE PROGRESSION")
        english_basic = await self.train_domain("english_basic", self.get_english_basic_curriculum())
        await asyncio.sleep(2)  # Let it consolidate
        
        english_intermediate = await self.train_domain("english_intermediate", self.get_english_intermediate_curriculum())
        await asyncio.sleep(2)
        
        english_advanced = await self.train_domain("english_advanced", self.get_english_advanced_curriculum())
        await asyncio.sleep(3)  # More consolidation for advanced
        
        # Evaluate English
        english_eval = await self.evaluate_domain("english", [
            "vowels and consonants",
            "grammar rules",
            "sentence structure", 
            "literary devices"
        ])
        
        # Phase 2: Mathematics
        logger.info("\n🔢 PHASE 2: MATHEMATICS DOMAIN")
        math_result = await self.train_domain("mathematics", self.get_mathematics_curriculum())
        await asyncio.sleep(3)
        
        math_eval = await self.evaluate_domain("mathematics", [
            "arithmetic operations",
            "algebra variables", 
            "equations and functions",
            "geometric shapes"
        ])
        
        # Phase 3: Science
        logger.info("\n🔬 PHASE 3: SCIENCE DOMAIN")
        science_result = await self.train_domain("science", self.get_science_curriculum())
        await asyncio.sleep(3)
        
        science_eval = await self.evaluate_domain("science", [
            "scientific method",
            "physics and energy",
            "chemistry and atoms",
            "biology and evolution"
        ])
        
        # Phase 4: Cross-Domain Analysis
        logger.info("\n🌐 PHASE 4: CROSS-DOMAIN INTEGRATION")
        cross_domain = await self.cross_domain_analysis()
        
        # Final consciousness check
        final_status = await self.get_status()
        final_consciousness = await self.check_consciousness()
        
        # Comprehensive results
        results = {
            'training_complete': True,
            'total_training_time': self.training_stats['total_training_time'],
            'total_concepts_fed': self.training_stats['total_concepts_fed'],
            'domains_trained': self.training_stats['domains_trained'],
            'consciousness_progression': self.training_stats['consciousness_progression'],
            'domain_results': {
                'english_basic': english_basic,
                'english_intermediate': english_intermediate, 
                'english_advanced': english_advanced,
                'mathematics': math_result,
                'science': science_result
            },
            'evaluations': {
                'english': english_eval,
                'mathematics': math_eval,
                'science': science_eval
            },
            'cross_domain_analysis': cross_domain,
            'final_status': {
                'concepts': final_status.get('total_concepts', 0),
                'associations': final_status.get('total_associations', 0),
                'consciousness_score': final_consciousness.get('consciousness_score', 0),
                'emergence_factor': final_status.get('emergence_factor', 1.0)
            }
        }
        
        logger.info("\n🎉 PROGRESSIVE TRAINING COMPLETE!")
        logger.info("=" * 60)
        logger.info(f"📊 Total Training Time: {results['total_training_time']:.1f} seconds")
        logger.info(f"📝 Concepts Fed: {results['total_concepts_fed']}")
        logger.info(f"🧠 Final Consciousness: {results['final_status']['consciousness_score']:.3f}")
        logger.info(f"🌟 Emergence Factor: {results['final_status']['emergence_factor']:.1f}x")
        logger.info(f"🔗 Cross-Domain Emergence: {cross_domain['emergence_rate']:.1%}")
        
        return results


async def main():
    """Main training orchestrator."""
    parser = argparse.ArgumentParser(description="Distributed Biological Intelligence Trainer")
    parser.add_argument("--core-url", required=True, help="URL of biological intelligence core service")
    parser.add_argument("--domain", choices=["english", "mathematics", "science"], help="Train specific domain")
    parser.add_argument("--progressive-all", action="store_true", help="Run complete progressive training")
    parser.add_argument("--evaluate", action="store_true", help="Run evaluation tests")
    
    args = parser.parse_args()
    
    if not HAS_HTTPX:
        sys.exit(1)
    
    async with BiologicalTrainingClient(args.core_url) as trainer:
        # Check core service
        try:
            status = await trainer.get_status()
            logger.info(f"🧠 Connected to biological intelligence core")
            logger.info(f"   Concepts: {status.get('total_concepts', 0)}")
            logger.info(f"   Consciousness: {status.get('consciousness_score', 0):.3f}")
        except Exception as e:
            logger.error(f"❌ Cannot connect to core service: {e}")
            sys.exit(1)
        
        if args.progressive_all:
            # Run complete progressive training
            results = await trainer.progressive_training()
            
            # Save results
            results_file = Path(f"training_results_{datetime.now().strftime('%Y%m%d_%H%M%S')}.json")
            with open(results_file, 'w') as f:
                json.dump(results, f, indent=2, default=str)
            logger.info(f"💾 Results saved to {results_file}")
            
        elif args.domain:
            # Train specific domain
            if args.domain == "english":
                curriculum = trainer.get_english_basic_curriculum()
            elif args.domain == "mathematics":
                curriculum = trainer.get_mathematics_curriculum()  
            elif args.domain == "science":
                curriculum = trainer.get_science_curriculum()
            
            result = await trainer.train_domain(args.domain, curriculum)
            
            if args.evaluate:
                if args.domain == "english":
                    eval_queries = ["vowels", "grammar", "sentences"]
                elif args.domain == "mathematics":
                    eval_queries = ["numbers", "algebra", "equations"]
                elif args.domain == "science":
                    eval_queries = ["physics", "chemistry", "biology"]
                
                evaluation = await trainer.evaluate_domain(args.domain, eval_queries)
                logger.info(f"📈 Evaluation: {evaluation['success_rate']:.1%} success rate")
        
        else:
            parser.print_help()


if __name__ == "__main__":
    asyncio.run(main())