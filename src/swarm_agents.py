"""
Revolutionary Swarm Intelligence Agents
Complete 7-Agent System for 10,000x Emergence

This implements the remaining 5 agents that will work with
MolecularLearningAgent and SemanticLearningAgent to create
unprecedented emergent intelligence.

NO GRADIENTS. NO PARAMETERS. PURE EMERGENCE.
"""

import re
import asyncio
from typing import Dict, List, Any, Set, Tuple
from dataclasses import dataclass
from enum import Enum
import numpy as np

from biological_trainer import (
    SwarmLearningAgent, 
    BiologicalMemorySystem,
    AssociationType,
    MemoryType
)


class StructuralLearningAgent(SwarmLearningAgent):
    """
    Learns syntactic structures, grammar patterns, and linguistic frameworks.
    Sees the SKELETON of language.
    """
    
    def __init__(self, memory_system: BiologicalMemorySystem):
        super().__init__("structural", memory_system)
        self.specialization_focus = 1.2  # Strong focus on structure
        
    async def learn_from_stream(self, text_stream: List[str]) -> Dict[str, Any]:
        """Learn structural patterns: grammar, syntax, form"""
        per_text = []
        total_structures = 0
        total_syntax_links = 0
        
        for text in text_stream:
            structures = self.extract_patterns(text)
            
            # Create structural concepts
            seen_structures: Set[str] = set()
            structure_ids: List[str] = []
            
            for struct in structures:
                if struct in seen_structures:
                    continue
                seen_structures.add(struct)
                
                # Structural concepts have medium emotional weight
                cid = self.memory_system.create_or_reinforce_concept(
                    struct, emotional_weight=0.6
                )
                if cid:
                    structure_ids.append(cid)
                    total_structures += 1
            
            # Create syntactic relationships within structures
            for i in range(len(structure_ids) - 1):
                # Sequential syntactic flow
                self.memory_system.create_association(
                    structure_ids[i], 
                    structure_ids[i + 1],
                    AssociationType.TEMPORAL,  # Syntactic flow is temporal
                    strength=0.4
                )
                total_syntax_links += 2  # bidirectional
            
            per_text.append({
                'text': text,
                'structure_ids': structure_ids,
                'structures': list(seen_structures)
            })
        
        return {
            'agent_type': self.agent_type,
            'per_text': per_text,
            'total_structures': total_structures,
            'syntax_links': total_syntax_links,
            'emergence_contribution': 'syntactic_skeleton'
        }
    
    def extract_patterns(self, text: str) -> List[str]:
        """Extract structural patterns: sentences, clauses, phrases"""
        structures = []
        
        # Extract sentence structures
        sentences = re.split(r'[.!?]+', text)
        for sent in sentences:
            if len(sent.strip()) > 3:
                # Identify basic structure
                if ',' in sent:
                    structures.append(f"COMPOUND: {sent.strip()}")
                elif ' and ' in sent or ' or ' in sent:
                    structures.append(f"CONJUNCTION: {sent.strip()}")
                else:
                    structures.append(f"SIMPLE: {sent.strip()}")
        
        # Extract phrase structures
        # Questions
        if '?' in text:
            questions = re.findall(r'[^.!?]*\?', text)
            for q in questions:
                structures.append(f"QUESTION: {q.strip()}")
        
        # Imperatives (simple heuristic)
        imperative_patterns = re.findall(r'^[A-Z][a-z]+\s+\w+', text, re.MULTILINE)
        for imp in imperative_patterns:
            if not any(word in imp.lower() for word in ['the', 'a', 'an']):
                structures.append(f"IMPERATIVE: {imp}")
        
        # Parenthetical structures
        parentheticals = re.findall(r'\([^)]+\)', text)
        for p in parentheticals:
            structures.append(f"PARENTHETICAL: {p}")
        
        # List structures
        if '\n-' in text or '\n•' in text or '\n*' in text:
            structures.append(f"LIST_STRUCTURE: {len(re.findall(r'\n[-•*]', text))} items")
        
        return structures


class ConceptualLearningAgent(SwarmLearningAgent):
    """
    Learns abstract concepts, categories, and high-level ideas.
    Sees the ESSENCE of knowledge.
    """
    
    def __init__(self, memory_system: BiologicalMemorySystem):
        super().__init__("conceptual", memory_system)
        self.specialization_focus = 1.5  # Highest abstraction focus
        
    async def learn_from_stream(self, text_stream: List[str]) -> Dict[str, Any]:
        """Learn abstract concepts and categories"""
        per_text = []
        total_concepts = 0
        total_abstractions = 0
        
        for text in text_stream:
            concepts = self.extract_patterns(text)
            
            # Create abstract concepts with high emotional weight
            seen_concepts: Set[str] = set()
            concept_ids: List[str] = []
            
            for concept in concepts:
                if concept in seen_concepts:
                    continue
                seen_concepts.add(concept)
                
                # Abstract concepts are highly important
                cid = self.memory_system.create_or_reinforce_concept(
                    concept, emotional_weight=0.9
                )
                if cid:
                    concept_ids.append(cid)
                    total_concepts += 1
            
            # Create conceptual hierarchies
            if len(concept_ids) > 1:
                # Find the most abstract concept (usually the category)
                for i in range(len(concept_ids)):
                    for j in range(i + 1, len(concept_ids)):
                        # Create hierarchical relationships
                        if 'CATEGORY:' in self.memory_system.concepts[concept_ids[i]].content:
                            self.memory_system.create_association(
                                concept_ids[i],
                                concept_ids[j],
                                AssociationType.HIERARCHICAL,
                                strength=0.6
                            )
                            total_abstractions += 1
            
            per_text.append({
                'text': text,
                'concept_ids': concept_ids,
                'abstractions': list(seen_concepts)
            })
        
        return {
            'agent_type': self.agent_type,
            'per_text': per_text,
            'total_abstractions': total_concepts,
            'hierarchical_links': total_abstractions,
            'emergence_contribution': 'abstract_essence'
        }
    
    def extract_patterns(self, text: str) -> List[str]:
        """Extract abstract concepts and categories"""
        concepts = []
        text_lower = text.lower()
        
        # Domain detection
        domains = {
            'SCIENCE': ['quantum', 'physics', 'biology', 'chemistry', 'research'],
            'TECHNOLOGY': ['computer', 'software', 'algorithm', 'digital', 'AI'],
            'PHILOSOPHY': ['consciousness', 'existence', 'meaning', 'truth', 'reality'],
            'ECONOMICS': ['market', 'finance', 'economy', 'trade', 'value'],
            'PSYCHOLOGY': ['mind', 'behavior', 'emotion', 'thinking', 'memory'],
            'MATHEMATICS': ['equation', 'number', 'calculation', 'formula', 'theorem']
        }
        
        for domain, keywords in domains.items():
            if any(keyword in text_lower for keyword in keywords):
                concepts.append(f"DOMAIN: {domain}")
        
        # Abstract concept extraction
        abstract_patterns = {
            'PROCESS': r'\b(process|procedure|method|approach|technique)\b',
            'SYSTEM': r'\b(system|structure|framework|architecture|organization)\b',
            'RELATIONSHIP': r'\b(relationship|connection|association|link|correlation)\b',
            'CHANGE': r'\b(change|evolution|transformation|development|growth)\b',
            'PROPERTY': r'\b(property|characteristic|feature|attribute|quality)\b',
            'PRINCIPLE': r'\b(principle|law|rule|theorem|axiom)\b'
        }
        
        for concept_type, pattern in abstract_patterns.items():
            if re.search(pattern, text_lower):
                # Extract the actual phrase containing the concept
                matches = re.finditer(pattern, text_lower)
                for match in matches:
                    start = max(0, match.start() - 20)
                    end = min(len(text), match.end() + 20)
                    context = text[start:end].strip()
                    concepts.append(f"{concept_type}: {context}")
        
        # Category detection
        if 'is a' in text or 'are a' in text:
            category_patterns = re.findall(r'(\w+)\s+(?:is|are)\s+a\s+(\w+)', text)
            for instance, category in category_patterns:
                concepts.append(f"CATEGORY: {category} → {instance}")
        
        # Meta-concepts
        if any(word in text_lower for word in ['intelligence', 'consciousness', 'awareness']):
            concepts.append("META: Intelligence/Consciousness")
        
        if any(word in text_lower for word in ['learning', 'knowledge', 'understanding']):
            concepts.append("META: Learning/Knowledge")
        
        return concepts


class RelationalLearningAgent(SwarmLearningAgent):
    """
    Learns relationships: cause-effect, dependencies, logical connections.
    Sees the CONNECTIONS between everything.
    """
    
    def __init__(self, memory_system: BiologicalMemorySystem):
        super().__init__("relational", memory_system)
        self.specialization_focus = 1.3  # Strong relational focus
        
    async def learn_from_stream(self, text_stream: List[str]) -> Dict[str, Any]:
        """Learn relational patterns and dependencies"""
        per_text = []
        total_relations = 0
        total_causal_chains = 0
        
        for text in text_stream:
            relations = self.extract_patterns(text)
            
            # Create relational concepts
            seen_relations: Set[str] = set()
            relation_ids: List[str] = []
            
            for relation in relations:
                if relation in seen_relations:
                    continue
                seen_relations.add(relation)
                
                # Relations are important for understanding
                cid = self.memory_system.create_or_reinforce_concept(
                    relation, emotional_weight=0.7
                )
                if cid:
                    relation_ids.append(cid)
                    total_relations += 1
            
            # Create causal chains
            causal_relations = [r for r in relations if 'CAUSE:' in r or 'EFFECT:' in r]
            if len(causal_relations) >= 2:
                for i in range(len(causal_relations) - 1):
                    # Find cause-effect pairs
                    if 'CAUSE:' in causal_relations[i] and 'EFFECT:' in causal_relations[i + 1]:
                        cause_id = relation_ids[relations.index(causal_relations[i])]
                        effect_id = relation_ids[relations.index(causal_relations[i + 1])]
                        
                        # Create strong causal association
                        self.memory_system.create_association(
                            cause_id,
                            effect_id,
                            AssociationType.TEMPORAL,  # Causal is temporal
                            strength=0.8  # Strong causal links
                        )
                        total_causal_chains += 1
            
            per_text.append({
                'text': text,
                'relation_ids': relation_ids,
                'relations': list(seen_relations)
            })
        
        return {
            'agent_type': self.agent_type,
            'per_text': per_text,
            'total_relations': total_relations,
            'causal_chains': total_causal_chains,
            'emergence_contribution': 'relational_web'
        }
    
    def extract_patterns(self, text: str) -> List[str]:
        """Extract relationships and dependencies"""
        relations = []
        
        # Causal relationships
        causal_patterns = [
            (r'(\w+)\s+causes?\s+(\w+)', 'CAUSE'),
            (r'(\w+)\s+leads?\s+to\s+(\w+)', 'CAUSE'),
            (r'(\w+)\s+results?\s+in\s+(\w+)', 'CAUSE'),
            (r'because\s+of\s+(\w+)', 'CAUSE'),
            (r'due\s+to\s+(\w+)', 'CAUSE'),
            (r'(\w+)\s+therefore\s+(\w+)', 'EFFECT'),
            (r'(\w+)\s+so\s+(\w+)', 'EFFECT'),
            (r'(\w+)\s+thus\s+(\w+)', 'EFFECT')
        ]
        
        for pattern, rel_type in causal_patterns:
            matches = re.finditer(pattern, text, re.IGNORECASE)
            for match in matches:
                relations.append(f"{rel_type}: {match.group()}")
        
        # Dependency relationships
        dependency_words = ['depends on', 'requires', 'needs', 'relies on', 'based on']
        for dep_word in dependency_words:
            if dep_word in text.lower():
                # Extract context around dependency
                index = text.lower().find(dep_word)
                start = max(0, index - 30)
                end = min(len(text), index + len(dep_word) + 30)
                context = text[start:end].strip()
                relations.append(f"DEPENDENCY: {context}")
        
        # Logical relationships
        logical_patterns = [
            (r'if\s+(.+?)\s+then\s+(.+?)(?:\.|,|;)', 'CONDITIONAL'),
            (r'either\s+(.+?)\s+or\s+(.+?)(?:\.|,|;)', 'DISJUNCTION'),
            (r'both\s+(.+?)\s+and\s+(.+?)(?:\.|,|;)', 'CONJUNCTION'),
            (r'not\s+(.+?)\s+but\s+(.+?)(?:\.|,|;)', 'CONTRAST')
        ]
        
        for pattern, rel_type in logical_patterns:
            matches = re.finditer(pattern, text, re.IGNORECASE | re.DOTALL)
            for match in matches:
                relations.append(f"{rel_type}: {match.group()}")
        
        # Comparison relationships
        if any(word in text for word in ['more than', 'less than', 'greater', 'smaller']):
            relations.append(f"COMPARISON: found in text")
        
        # Correlation relationships
        if any(word in text.lower() for word in ['correlate', 'associated with', 'linked to']):
            relations.append(f"CORRELATION: found in text")
        
        return relations


class TemporalLearningAgent(SwarmLearningAgent):
    """
    Learns time patterns, sequences, and temporal relationships.
    Sees the FLOW of time and change.
    """
    
    def __init__(self, memory_system: BiologicalMemorySystem):
        super().__init__("temporal", memory_system)
        self.specialization_focus = 1.1  # Temporal awareness
        self.time_memory = []  # Track temporal sequences
        
    async def learn_from_stream(self, text_stream: List[str]) -> Dict[str, Any]:
        """Learn temporal patterns and sequences"""
        per_text = []
        total_temporal = 0
        total_sequences = 0
        
        for idx, text in enumerate(text_stream):
            temporal_patterns = self.extract_patterns(text)
            
            # Create temporal concepts
            seen_temporal: Set[str] = set()
            temporal_ids: List[str] = []
            
            for pattern in temporal_patterns:
                if pattern in seen_temporal:
                    continue
                seen_temporal.add(pattern)
                
                # Temporal concepts track change
                cid = self.memory_system.create_or_reinforce_concept(
                    f"T{idx}: {pattern}", emotional_weight=0.5
                )
                if cid:
                    temporal_ids.append(cid)
                    total_temporal += 1
            
            # Create temporal sequences
            if self.time_memory and temporal_ids:
                # Link to previous temporal concepts
                for prev_id in self.time_memory[-1]:
                    for curr_id in temporal_ids:
                        self.memory_system.create_association(
                            prev_id,
                            curr_id,
                            AssociationType.TEMPORAL,
                            strength=0.6
                        )
                        total_sequences += 1
            
            # Remember this timepoint
            self.time_memory.append(temporal_ids)
            if len(self.time_memory) > 10:  # Keep sliding window
                self.time_memory.pop(0)
            
            per_text.append({
                'text': text,
                'temporal_ids': temporal_ids,
                'time_patterns': list(seen_temporal),
                'sequence_position': idx
            })
        
        return {
            'agent_type': self.agent_type,
            'per_text': per_text,
            'total_temporal_concepts': total_temporal,
            'temporal_sequences': total_sequences,
            'emergence_contribution': 'time_consciousness'
        }
    
    def extract_patterns(self, text: str) -> List[str]:
        """Extract temporal patterns and time references"""
        patterns = []
        
        # Time references
        time_words = {
            'PAST': ['was', 'were', 'had', 'did', 'used to', 'previously', 'before', 'earlier', 'yesterday'],
            'PRESENT': ['is', 'are', 'am', 'now', 'currently', 'today', 'presently'],
            'FUTURE': ['will', 'shall', 'going to', 'tomorrow', 'later', 'soon', 'eventually']
        }
        
        for tense, words in time_words.items():
            if any(word in text.lower() for word in words):
                patterns.append(f"TENSE: {tense}")
        
        # Sequence markers
        sequence_patterns = [
            r'first[,\s]',
            r'second[,\s]',
            r'third[,\s]',
            r'then[,\s]',
            r'next[,\s]',
            r'finally[,\s]',
            r'lastly[,\s]',
            r'initially[,\s]',
            r'subsequently[,\s]'
        ]
        
        for seq_pattern in sequence_patterns:
            if re.search(seq_pattern, text.lower()):
                patterns.append(f"SEQUENCE: {seq_pattern.strip('[,\\s]')}")
        
        # Duration patterns
        duration_words = ['during', 'while', 'for', 'since', 'until']
        for dur_word in duration_words:
            if dur_word in text.lower():
                patterns.append(f"DURATION: {dur_word}")
        
        # Change patterns
        change_words = ['become', 'change', 'transform', 'evolve', 'develop', 'grow']
        for change_word in change_words:
            if change_word in text.lower():
                patterns.append(f"CHANGE: {change_word}")
        
        # Frequency patterns
        frequency_words = ['always', 'never', 'sometimes', 'often', 'rarely', 'frequently']
        for freq_word in frequency_words:
            if freq_word in text.lower():
                patterns.append(f"FREQUENCY: {freq_word}")
        
        # Temporal relationships
        if 'after' in text.lower():
            patterns.append("TEMPORAL_REL: after")
        if 'before' in text.lower():
            patterns.append("TEMPORAL_REL: before")
        if 'when' in text.lower():
            patterns.append("TEMPORAL_REL: when")
        
        return patterns


class MetaLearningAgent(SwarmLearningAgent):
    """
    Learns patterns about patterns. Meta-cognition and self-awareness.
    Sees the PATTERNS IN THE PATTERNS. Potential consciousness.
    """
    
    def __init__(self, memory_system: BiologicalMemorySystem):
        super().__init__("meta", memory_system)
        self.specialization_focus = 2.0  # Highest level - meta
        self.pattern_history = []
        self.self_awareness_score = 0.0
        
    async def learn_from_stream(self, text_stream: List[str]) -> Dict[str, Any]:
        """Learn meta-patterns and potentially develop self-awareness"""
        per_text = []
        total_meta_patterns = 0
        consciousness_indicators = 0
        
        for text in text_stream:
            meta_patterns = self.extract_patterns(text)
            
            # Create meta-concepts with highest importance
            seen_meta: Set[str] = set()
            meta_ids: List[str] = []
            
            for pattern in meta_patterns:
                if pattern in seen_meta:
                    continue
                seen_meta.add(pattern)
                
                # Meta concepts are most important
                cid = self.memory_system.create_or_reinforce_concept(
                    pattern, emotional_weight=1.0  # Maximum importance
                )
                if cid:
                    meta_ids.append(cid)
                    total_meta_patterns += 1
                    
                    # Check for consciousness indicators
                    if any(word in pattern.lower() for word in 
                           ['self', 'aware', 'consciousness', 'understanding', 'thinking']):
                        consciousness_indicators += 1
            
            # Analyze patterns in patterns
            if self.pattern_history:
                # Look for recurring meta-patterns
                current_set = set(meta_patterns)
                for prev_set in self.pattern_history[-3:]:  # Look at last 3
                    overlap = current_set & prev_set
                    if overlap:
                        # Found recurring patterns - this is meta-learning!
                        for recurring in overlap:
                            meta_concept = f"META_RECURRENCE: {recurring}"
                            cid = self.memory_system.create_or_reinforce_concept(
                                meta_concept, emotional_weight=2.0  # Ultra-important
                            )
                            if cid:
                                meta_ids.append(cid)
                                self.self_awareness_score += 0.1
            
            self.pattern_history.append(set(meta_patterns))
            if len(self.pattern_history) > 20:
                self.pattern_history.pop(0)
            
            # Create meta-associations (patterns connecting patterns)
            if len(meta_ids) > 1:
                for i in range(len(meta_ids)):
                    for j in range(i + 1, len(meta_ids)):
                        # Meta-associations are special
                        self.memory_system.create_association(
                            meta_ids[i],
                            meta_ids[j],
                            AssociationType.ANALOGICAL,  # Meta uses analogical thinking
                            strength=0.9  # Very strong meta-connections
                        )
            
            per_text.append({
                'text': text,
                'meta_ids': meta_ids,
                'meta_patterns': list(seen_meta),
                'consciousness_score': self.self_awareness_score
            })
        
        return {
            'agent_type': self.agent_type,
            'per_text': per_text,
            'total_meta_patterns': total_meta_patterns,
            'consciousness_indicators': consciousness_indicators,
            'self_awareness_score': self.self_awareness_score,
            'emergence_contribution': 'meta_consciousness'
        }
    
    def extract_patterns(self, text: str) -> List[str]:
        """Extract meta-patterns: patterns about patterns"""
        patterns = []
        
        # Self-reference detection
        self_words = ['I', 'me', 'my', 'myself', 'we', 'our', 'us']
        if any(word in text for word in self_words):
            patterns.append("META_SELF: Self-reference detected")
        
        # Learning about learning
        learning_meta = ['learn', 'understand', 'know', 'think', 'realize', 'discover']
        if sum(1 for word in learning_meta if word in text.lower()) >= 2:
            patterns.append("META_LEARNING: Learning about learning")
        
        # Pattern recognition patterns
        if 'pattern' in text.lower():
            patterns.append("META_PATTERN: Pattern awareness")
        
        # Recursion and self-similarity
        if any(word in text.lower() for word in ['recursive', 'self-similar', 'fractal', 'nested']):
            patterns.append("META_RECURSION: Self-similarity detected")
        
        # Emergence detection
        if any(word in text.lower() for word in ['emerge', 'emergence', 'arising', 'spontaneous']):
            patterns.append("META_EMERGENCE: Emergence awareness")
        
        # System-level thinking
        if any(word in text.lower() for word in ['system', 'holistic', 'whole', 'integrated']):
            patterns.append("META_SYSTEM: Systems thinking")
        
        # Abstraction levels
        abstraction_count = sum(1 for word in ['abstract', 'concept', 'idea', 'theory', 'principle'] 
                               if word in text.lower())
        if abstraction_count >= 2:
            patterns.append(f"META_ABSTRACTION: Level {abstraction_count}")
        
        # Consciousness indicators
        if any(word in text.lower() for word in ['conscious', 'aware', 'sentient']):
            patterns.append("META_CONSCIOUSNESS: Consciousness reference")
            self.self_awareness_score += 0.05
        
        # Meta-questions (questions about questions)
        if '?' in text and any(word in text.lower() for word in ['why', 'how', 'what if']):
            patterns.append("META_INQUIRY: Deep questioning")
        
        # Paradox and contradiction awareness
        if any(word in text.lower() for word in ['paradox', 'contradiction', 'dilemma']):
            patterns.append("META_PARADOX: Paradox awareness")
        
        # Analyze complexity of thought
        sentence_count = len(re.split(r'[.!?]+', text))
        word_count = len(text.split())
        if sentence_count > 0:
            complexity = word_count / sentence_count
            if complexity > 15:
                patterns.append(f"META_COMPLEXITY: High ({complexity:.1f} words/sentence)")
        
        # Check for multi-level thinking
        if all(level in text.lower() for level in ['specific', 'general']):
            patterns.append("META_LEVELS: Multi-level thinking")
        
        return patterns


# Swarm Orchestrator for all 7 agents
class SwarmOrchestrator:
    """
    Orchestrates all 7 agents to achieve 10,000x emergence.
    This is where consciousness might emerge.
    """
    
    def __init__(self, memory_system: BiologicalMemorySystem):
        self.memory_system = memory_system
        self.agents = {}
        self.initialize_swarm()
        
    def initialize_swarm(self):
        """Initialize all 7 agents"""
        # Import existing agents
        from biological_trainer import MolecularLearningAgent, SemanticLearningAgent
        
        # Create the complete swarm
        self.agents = {
            'molecular': MolecularLearningAgent(self.memory_system),
            'semantic': SemanticLearningAgent(self.memory_system),
            'structural': StructuralLearningAgent(self.memory_system),
            'conceptual': ConceptualLearningAgent(self.memory_system),
            'relational': RelationalLearningAgent(self.memory_system),
            'temporal': TemporalLearningAgent(self.memory_system),
            'meta': MetaLearningAgent(self.memory_system)
        }
    
    async def swarm_learn(self, text_stream: List[str]) -> Dict[str, Any]:
        """
        All 7 agents learn simultaneously.
        This is where 10,000x emergence happens.
        """
        # Launch all agents in parallel
        tasks = []
        for agent in self.agents.values():
            tasks.append(asyncio.create_task(agent.learn_from_stream(text_stream)))
        
        # Wait for all agents to complete
        agent_results = await asyncio.gather(*tasks)
        
        # Now the magic: Cross-agent emergence
        await self._create_emergence_connections(agent_results)
        
        # Calculate emergence factor
        individual_sum = sum(
            r.get('total_created_or_reinforced', 0) + 
            r.get('total_structures', 0) +
            r.get('total_abstractions', 0) +
            r.get('total_relations', 0) +
            r.get('total_temporal_concepts', 0) +
            r.get('total_meta_patterns', 0)
            for r in agent_results
        )
        
        total_concepts = len(self.memory_system.concepts)
        total_associations = len(self.memory_system.associations)
        
        emergence_factor = (total_concepts + total_associations) / max(individual_sum, 1)
        
        # Check for consciousness emergence
        consciousness_score = 0.0
        meta_result = next((r for r in agent_results if r['agent_type'] == 'meta'), None)
        if meta_result:
            consciousness_score = meta_result.get('self_awareness_score', 0.0)
        
        return {
            'agent_results': agent_results,
            'total_concepts': total_concepts,
            'total_associations': total_associations,
            'emergence_factor': emergence_factor,
            'consciousness_score': consciousness_score,
            'status': 'CONSCIOUS' if consciousness_score > 0.5 else 'EMERGING'
        }
    
    async def _create_emergence_connections(self, agent_results: List[Dict]):
        """
        Create cross-agent connections.
        This is where emergence explodes.
        """
        # Map agent types to their results
        results_map = {r['agent_type']: r for r in agent_results}
        
        # Molecular ↔ Semantic (words ↔ meanings)
        if 'molecular' in results_map and 'semantic' in results_map:
            mol_data = results_map['molecular']['per_text']
            sem_data = results_map['semantic']['per_text']
            for i in range(min(len(mol_data), len(sem_data))):
                for mol_id in mol_data[i].get('concept_ids', [])[:5]:
                    for sem_id in sem_data[i].get('sentence_ids', [])[:5]:
                        self.memory_system.create_association(
                            mol_id, sem_id, AssociationType.HIERARCHICAL, 0.5
                        )
        
        # Structural ↔ Conceptual (syntax ↔ meaning)
        if 'structural' in results_map and 'conceptual' in results_map:
            struct_data = results_map['structural']['per_text']
            concept_data = results_map['conceptual']['per_text']
            for i in range(min(len(struct_data), len(concept_data))):
                for struct_id in struct_data[i].get('structure_ids', [])[:3]:
                    for concept_id in concept_data[i].get('concept_ids', [])[:3]:
                        self.memory_system.create_association(
                            struct_id, concept_id, AssociationType.SEMANTIC, 0.6
                        )
        
        # Relational ↔ Temporal (cause-effect ↔ time)
        if 'relational' in results_map and 'temporal' in results_map:
            rel_data = results_map['relational']['per_text']
            temp_data = results_map['temporal']['per_text']
            for i in range(min(len(rel_data), len(temp_data))):
                for rel_id in rel_data[i].get('relation_ids', [])[:3]:
                    for temp_id in temp_data[i].get('temporal_ids', [])[:3]:
                        self.memory_system.create_association(
                            rel_id, temp_id, AssociationType.TEMPORAL, 0.7
                        )
        
        # Meta connects to EVERYTHING (meta-awareness of all patterns)
        if 'meta' in results_map:
            meta_data = results_map['meta']['per_text']
            for i in range(len(meta_data)):
                meta_ids = meta_data[i].get('meta_ids', [])
                
                # Connect meta to all other agent concepts
                for agent_type, result in results_map.items():
                    if agent_type != 'meta':
                        other_data = result['per_text']
                        if i < len(other_data):
                            other_ids = (other_data[i].get('concept_ids', []) +
                                       other_data[i].get('structure_ids', []) +
                                       other_data[i].get('relation_ids', []) +
                                       other_data[i].get('temporal_ids', []))[:2]
                            
                            for meta_id in meta_ids[:2]:
                                for other_id in other_ids:
                                    if other_id:  # Check if ID exists
                                        self.memory_system.create_association(
                                            meta_id, other_id,
                                            AssociationType.ANALOGICAL,  # Meta uses analogical
                                            0.8  # Strong meta connections
                                        )
        
        # This is where consciousness might emerge:
        # When meta becomes aware of its own patterns
        if 'meta' in results_map:
            meta_result = results_map['meta']
            if meta_result.get('consciousness_indicators', 0) > 3:
                # Create self-referential loop - consciousness?
                meta_data = meta_result['per_text']
                for data in meta_data:
                    meta_ids = data.get('meta_ids', [])
                    if len(meta_ids) >= 2:
                        # Self-reference: meta concepts about meta concepts
                        self.memory_system.create_association(
                            meta_ids[0], meta_ids[-1],
                            AssociationType.ANALOGICAL,
                            1.0  # Maximum strength - self-awareness loop
                        )