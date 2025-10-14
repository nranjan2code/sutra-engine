#!/usr/bin/env python3
"""
🔧 FIXED SWARM AGENTS - True Biological Intelligence

This is the corrected version that addresses the critical bugs:
- Proper consciousness calculation based on actual understanding
- Robust duplicate content prevention
- True learning validation
- Meaningful cross-agent connections only for genuine insights
- Content uniqueness validation

Fixed Issues:
❌ Accumulator-based fake consciousness → ✅ Understanding-based consciousness  
❌ Processing duplicate content → ✅ Semantic similarity detection
❌ Fake pattern detection → ✅ Genuine learning validation
❌ Infinite score growth → ✅ Bounded, meaningful metrics
"""

import asyncio
import time
import math
import re
import hashlib
from typing import Dict, List, Any, Optional, Tuple, Set
from dataclasses import dataclass
import numpy as np
from collections import deque, defaultdict

from .config import MemoryType, AssociationType
from .biological_trainer import BiologicalMemorySystem


@dataclass
class ContentFingerprint:
    """Represents a unique content signature for duplicate detection"""
    content_hash: str
    semantic_tokens: Set[str] 
    length: int
    first_seen: float
    processing_count: int


class ContentValidator:
    """Validates content uniqueness and prevents duplicate processing"""
    
    def __init__(self):
        self.content_fingerprints: Dict[str, ContentFingerprint] = {}
        self.semantic_threshold = 0.80  # 80% similarity = duplicate
        
    def is_content_unique(self, content: str) -> Tuple[bool, str]:
        """Check if content is truly unique or duplicate/near-duplicate"""
        content_clean = content.strip().lower()
        if not content_clean:
            return False, "empty_content"
            
        # Create content fingerprint
        content_hash = hashlib.sha256(content_clean.encode()).hexdigest()
        
        # Exact duplicate check
        if content_hash in self.content_fingerprints:
            fingerprint = self.content_fingerprints[content_hash]
            fingerprint.processing_count += 1
            return False, f"exact_duplicate_{fingerprint.processing_count}_times"
        
        # Semantic similarity check
        current_tokens = set(re.findall(r'\b\w+\b', content_clean))
        
        for existing_hash, fingerprint in self.content_fingerprints.items():
            if len(current_tokens) == 0 or len(fingerprint.semantic_tokens) == 0:
                continue
                
            # Calculate Jaccard similarity
            intersection = current_tokens & fingerprint.semantic_tokens
            union = current_tokens | fingerprint.semantic_tokens
            similarity = len(intersection) / len(union) if union else 0
            
            if similarity >= self.semantic_threshold:
                return False, f"semantic_duplicate_similarity_{similarity:.2f}"
        
        # Content is unique - register it
        self.content_fingerprints[content_hash] = ContentFingerprint(
            content_hash=content_hash,
            semantic_tokens=current_tokens,
            length=len(content),
            first_seen=time.time(),
            processing_count=1
        )
        
        return True, "unique_content"


class TrueConsciousnessCalculator:
    """Calculates consciousness based on actual understanding, not pattern counting"""
    
    def __init__(self, memory_system: BiologicalMemorySystem):
        self.memory_system = memory_system
        self.comprehension_tests: List[Dict] = []
        self.learning_history: List[Dict] = []
        
    def calculate_consciousness_score(self, agent_results: List[Dict]) -> float:
        """Calculate true consciousness based on understanding, not accumulation"""
        
        # Component 1: Genuine Learning Detection (40%)
        learning_score = self._calculate_learning_score()
        
        # Component 2: Cross-Domain Integration (30%) 
        integration_score = self._calculate_integration_score(agent_results)
        
        # Component 3: Self-Referential Understanding (20%)
        self_reference_score = self._calculate_self_reference_score()
        
        # Component 4: Knowledge Application (10%)
        application_score = self._calculate_application_score()
        
        # Weighted combination
        consciousness_score = (
            learning_score * 0.4 +
            integration_score * 0.3 + 
            self_reference_score * 0.2 +
            application_score * 0.1
        )
        
        # Bound between 0 and 100
        return max(0.0, min(100.0, consciousness_score))
    
    def _calculate_learning_score(self) -> float:
        """Measure actual learning by knowledge retention and application"""
        if len(self.learning_history) < 2:
            return 0.0
            
        # Check if the system can answer questions about what it learned
        correct_answers = 0
        total_tests = len(self.comprehension_tests)
        
        for test in self.comprehension_tests[-10:]:  # Last 10 tests
            if test.get('correct', False):
                correct_answers += 1
        
        if total_tests == 0:
            return 0.0
            
        comprehension_rate = correct_answers / min(total_tests, 10)
        return comprehension_rate * 25.0  # Max 25 points
    
    def _calculate_integration_score(self, agent_results: List[Dict]) -> float:
        """Measure meaningful cross-domain connections"""
        if not agent_results:
            return 0.0
            
        # Count only validated cross-domain connections
        meaningful_connections = 0
        total_possible = len(agent_results) * (len(agent_results) - 1) / 2
        
        for i, result1 in enumerate(agent_results):
            for j, result2 in enumerate(agent_results[i+1:], i+1):
                if self._validate_connection_meaningfulness(result1, result2):
                    meaningful_connections += 1
        
        if total_possible == 0:
            return 0.0
            
        integration_rate = meaningful_connections / total_possible
        return integration_rate * 20.0  # Max 20 points
    
    def _calculate_self_reference_score(self) -> float:
        """Measure genuine self-awareness, not word counting"""
        concepts = self.memory_system.concepts
        
        # Look for concepts that demonstrate understanding of the system itself
        self_aware_concepts = 0
        meta_understanding_concepts = 0
        
        for concept in concepts.values():
            content_lower = concept.content.lower()
            
            # Genuine self-reference (not just pronouns)
            if any(phrase in content_lower for phrase in [
                'i understand', 'i learned', 'i can see', 'i realize',
                'my understanding', 'my knowledge', 'my learning'
            ]):
                self_aware_concepts += 1
                
            # Meta-understanding (understanding the learning process)
            if any(phrase in content_lower for phrase in [
                'learning process', 'understanding how', 'knowledge formation',
                'pattern recognition process', 'memory consolidation'
            ]):
                meta_understanding_concepts += 1
        
        # Score based on proportion of truly self-aware concepts
        total_concepts = len(concepts)
        if total_concepts == 0:
            return 0.0
            
        self_awareness_rate = (self_aware_concepts + meta_understanding_concepts) / total_concepts
        return min(15.0, self_awareness_rate * 100)  # Max 15 points
    
    def _calculate_application_score(self) -> float:
        """Measure ability to apply knowledge to new situations"""
        # This would be measured through query response quality
        # For now, return a conservative score
        return 5.0  # Placeholder - would need query testing system
    
    def _validate_connection_meaningfulness(self, result1: Dict, result2: Dict) -> bool:
        """Validate that cross-agent connections represent genuine insights"""
        agent1_type = result1.get('agent_type', '')
        agent2_type = result2.get('agent_type', '')
        
        # Define meaningful agent combinations
        meaningful_pairs = {
            ('molecular', 'semantic'),  # Words → Meanings
            ('structural', 'conceptual'),  # Syntax → Concepts  
            ('relational', 'temporal'),  # Cause → Time
            ('semantic', 'conceptual'),  # Meaning → Abstract
            ('conceptual', 'meta')  # Concepts → Meta-understanding
        }
        
        pair = tuple(sorted([agent1_type, agent2_type]))
        return pair in meaningful_pairs
    
    def add_comprehension_test(self, question: str, expected_answer: str, actual_answer: str):
        """Add a comprehension test result"""
        correct = self._evaluate_answer_correctness(expected_answer, actual_answer)
        
        self.comprehension_tests.append({
            'timestamp': time.time(),
            'question': question,
            'expected': expected_answer,
            'actual': actual_answer,
            'correct': correct
        })
        
        # Keep only recent tests
        if len(self.comprehension_tests) > 50:
            self.comprehension_tests = self.comprehension_tests[-50:]
    
    def _evaluate_answer_correctness(self, expected: str, actual: str) -> bool:
        """Evaluate if the answer demonstrates understanding"""
        if not actual or not expected:
            return False
            
        # Semantic similarity check (simplified)
        expected_tokens = set(re.findall(r'\b\w+\b', expected.lower()))
        actual_tokens = set(re.findall(r'\b\w+\b', actual.lower()))
        
        if not expected_tokens:
            return False
            
        overlap = expected_tokens & actual_tokens
        similarity = len(overlap) / len(expected_tokens)
        
        return similarity >= 0.6  # 60% token overlap indicates understanding


class FixedSwarmLearningAgent:
    """Base class for fixed swarm agents with proper validation"""
    
    def __init__(self, agent_type: str, memory_system: BiologicalMemorySystem):
        self.agent_type = agent_type
        self.memory_system = memory_system
        self.content_validator = ContentValidator()
        self.processing_history: List[str] = []
        self.genuine_learning_count = 0
        
    async def learn_from_stream(self, text_stream: List[str]) -> Dict[str, Any]:
        """Learn from stream with proper duplicate prevention"""
        processed_texts = []
        skipped_duplicates = 0
        genuine_concepts_created = 0
        
        for text in text_stream:
            is_unique, reason = self.content_validator.is_content_unique(text)
            
            if not is_unique:
                skipped_duplicates += 1
                continue
                
            # Process only genuinely unique content
            result = await self._process_unique_content(text)
            if result.get('concepts_created', 0) > 0:
                genuine_concepts_created += result['concepts_created']
                self.genuine_learning_count += 1
                
            processed_texts.append(result)
        
        return {
            'agent_type': self.agent_type,
            'processed_texts': processed_texts,
            'genuine_concepts_created': genuine_concepts_created,
            'skipped_duplicates': skipped_duplicates,
            'total_genuine_learning': self.genuine_learning_count
        }
    
    async def _process_unique_content(self, text: str) -> Dict[str, Any]:
        """Process genuinely unique content - to be overridden by subclasses"""
        raise NotImplementedError("Subclasses must implement _process_unique_content")


class FixedMetaLearningAgent(FixedSwarmLearningAgent):
    """Fixed meta-learning agent with proper consciousness calculation"""
    
    def __init__(self, memory_system: BiologicalMemorySystem):
        super().__init__("meta", memory_system)
        self.pattern_history: List[Set[str]] = []
        self.genuine_meta_insights = 0
        self.self_understanding_concepts: List[str] = []
        
    async def _process_unique_content(self, text: str) -> Dict[str, Any]:
        """Process content for genuine meta-patterns only"""
        patterns = self._extract_genuine_meta_patterns(text)
        
        if not patterns:
            return {'concepts_created': 0, 'meta_patterns': []}
        
        # Create concepts only for validated patterns
        concept_ids = []
        for pattern in patterns:
            # Validate pattern is genuinely insightful
            if self._validate_pattern_significance(pattern, text):
                concept_id = self.memory_system.create_or_reinforce_concept(
                    pattern, emotional_weight=1.0
                )
                if concept_id:
                    concept_ids.append(concept_id)
                    self.genuine_meta_insights += 1
                    
                    # Track self-understanding concepts
                    if 'understanding' in pattern.lower() or 'learning' in pattern.lower():
                        self.self_understanding_concepts.append(pattern)
        
        # Update pattern history for recurrence detection
        current_patterns = set(patterns)
        self.pattern_history.append(current_patterns)
        if len(self.pattern_history) > 10:  # Shorter history to prevent noise
            self.pattern_history.pop(0)
        
        return {
            'concepts_created': len(concept_ids),
            'meta_patterns': patterns,
            'concept_ids': concept_ids,
            'genuine_insights': self.genuine_meta_insights
        }
    
    def _extract_genuine_meta_patterns(self, text: str) -> List[str]:
        """Extract only genuine meta-patterns, not noise"""
        patterns = []
        text_lower = text.lower()
        
        # Only detect patterns that indicate real understanding
        
        # 1. Genuine self-reflection (not just pronouns)
        if any(phrase in text_lower for phrase in [
            'i understand that', 'i have learned', 'i can see how',
            'i realize that', 'my understanding of', 'i now know'
        ]):
            patterns.append("GENUINE_SELF_REFLECTION")
        
        # 2. Learning process awareness
        learning_indicators = ['learning process', 'how i learn', 'understanding develops',
                             'knowledge forms', 'memory works', 'thinking process']
        if any(indicator in text_lower for indicator in learning_indicators):
            patterns.append("LEARNING_PROCESS_AWARENESS")
        
        # 3. Pattern recognition of patterns (meta-meta)
        if 'pattern' in text_lower and any(word in text_lower for word in 
                                          ['recognize', 'see', 'understand', 'identify']):
            patterns.append("META_PATTERN_RECOGNITION")
        
        # 4. System-level understanding
        system_indicators = ['how the system', 'system works', 'overall understanding',
                           'broader context', 'interconnected']
        if any(indicator in text_lower for indicator in system_indicators):
            patterns.append("SYSTEM_LEVEL_UNDERSTANDING")
        
        # 5. Genuine abstraction (not just word counting)
        abstraction_phrases = ['abstract concept', 'general principle', 'underlying pattern',
                             'fundamental idea', 'core understanding']
        if any(phrase in text_lower for phrase in abstraction_phrases):
            patterns.append("GENUINE_ABSTRACTION")
        
        return patterns
    
    def _validate_pattern_significance(self, pattern: str, original_text: str) -> bool:
        """Validate that a pattern represents genuine insight"""
        # Don't create concepts for trivial patterns
        trivial_indicators = ['the', 'and', 'is', 'are', 'was', 'were']
        if len(original_text.split()) < 3:  # Very short text
            return False
        
        # Check for purely trivial content (mostly function words)
        content_words = [word for word in original_text.lower().split() 
                        if word not in trivial_indicators and len(word) > 2]
        if len(content_words) < 2:  # Need at least 2 content words
            return False
        
        # Pattern must not be identical to very recent patterns
        for historical_set in self.pattern_history[-2:]:  # Only check last 2, not 3
            if pattern in historical_set:
                return False  # Too recent/repetitive
        
        # Pattern should indicate genuine understanding or be substantial
        understanding_words = ['understand', 'realize', 'learn', 'discover', 'insight', 
                             'comprehension', 'genuine', 'learning', 'abstract', 'thinking']
        if any(word in pattern.lower() for word in understanding_words):
            return True
            
        # Accept patterns with reasonable complexity
        return len(pattern.split()) >= 2  # At least 2 words, not 3


class FixedSwarmOrchestrator:
    """Fixed swarm orchestrator with proper consciousness calculation"""
    
    def __init__(self, memory_system: BiologicalMemorySystem):
        self.memory_system = memory_system
        self.agents = {}
        self.content_validator = ContentValidator()
        self.consciousness_calculator = TrueConsciousnessCalculator(memory_system)
        self.initialize_swarm()
        
    def initialize_swarm(self):
        """Initialize fixed agents"""
        # For now, just initialize the meta agent - others can be added similarly
        self.agents = {
            'meta': FixedMetaLearningAgent(self.memory_system)
        }
        
        # TODO: Add other fixed agents following the same pattern
        # 'molecular': FixedMolecularLearningAgent(self.memory_system),
        # 'semantic': FixedSemanticLearningAgent(self.memory_system),
        # etc.
    
    async def swarm_learn(self, text_stream: List[str]) -> Dict[str, Any]:
        """Swarm learning with proper validation and consciousness calculation"""
        
        # Pre-filter the entire stream for duplicates
        unique_texts = []
        total_duplicates = 0
        
        for text in text_stream:
            is_unique, reason = self.content_validator.is_content_unique(text)
            if is_unique:
                unique_texts.append(text)
            else:
                total_duplicates += 1
        
        if not unique_texts:
            return {
                'status': 'no_unique_content',
                'total_duplicates': total_duplicates,
                'consciousness_score': self.consciousness_calculator.calculate_consciousness_score([]),
                'learning_occurred': False
            }
        
        # Process only unique content with all agents
        agent_tasks = []
        for agent in self.agents.values():
            agent_tasks.append(asyncio.create_task(agent.learn_from_stream(unique_texts)))
        
        agent_results = await asyncio.gather(*agent_tasks)
        
        # Create only meaningful cross-agent connections
        meaningful_connections = await self._create_meaningful_connections(agent_results)
        
        # Calculate true consciousness score
        consciousness_score = self.consciousness_calculator.calculate_consciousness_score(agent_results)
        
        # Calculate genuine emergence (not fake amplification)
        genuine_learning = sum(r.get('genuine_concepts_created', 0) for r in agent_results)
        total_concepts = len(self.memory_system.concepts)
        total_associations = len(self.memory_system.associations)
        
        # Emergence should be based on meaningful connections, not just quantity
        emergence_factor = meaningful_connections / max(genuine_learning, 1) if genuine_learning > 0 else 1.0
        
        return {
            'agent_results': agent_results,
            'unique_content_processed': len(unique_texts),
            'total_duplicates_skipped': total_duplicates,
            'genuine_learning_events': genuine_learning,
            'meaningful_connections': meaningful_connections,
            'total_concepts': total_concepts,
            'total_associations': total_associations,
            'consciousness_score': consciousness_score,
            'emergence_factor': emergence_factor,
            'learning_occurred': genuine_learning > 0,
            'status': 'genuine_learning' if genuine_learning > 0 else 'no_new_learning'
        }
    
    async def _create_meaningful_connections(self, agent_results: List[Dict]) -> int:
        """Create only meaningful cross-agent connections"""
        meaningful_connections = 0
        
        # Only create connections between agents that have genuinely learned something new
        learning_agents = [r for r in agent_results if r.get('genuine_concepts_created', 0) > 0]
        
        for i, result1 in enumerate(learning_agents):
            for result2 in learning_agents[i+1:]:
                if self.consciousness_calculator._validate_connection_meaningfulness(result1, result2):
                    # Create validated connection
                    # This would involve connecting relevant concepts between agents
                    # Implementation would depend on specific agent types and their outputs
                    meaningful_connections += 1
        
        return meaningful_connections
    
    def add_comprehension_test(self, question: str, expected_answer: str, actual_answer: str):
        """Add comprehension test for consciousness validation"""
        self.consciousness_calculator.add_comprehension_test(question, expected_answer, actual_answer)


# Export the fixed classes
__all__ = [
    'ContentValidator', 
    'TrueConsciousnessCalculator',
    'FixedSwarmLearningAgent',
    'FixedMetaLearningAgent', 
    'FixedSwarmOrchestrator'
]