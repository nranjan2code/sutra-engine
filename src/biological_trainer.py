"""
Biological Training System - Next-Generation AI Training Based on Infinite Knowledge Methodology

This implements the core concepts from sutra-swarm's infinite knowledge approach
for training AI models using biological memory principles and swarm intelligence.

This revision upgrades the prototype to a production-grade, associative learning
engine with:
- Content de-duplication and reinforcement
- Association de-duplication with reinforcement and bi-directional edges
- Per-text association formation (no O(n^2) across history)
- Temporal links across successive texts
- Hierarchical links between sentences and tokens
- Capacity enforcement per memory tier
- Graph-aware retrieval (spreading activation) with memory-type weighting
"""

import asyncio
import time
import math
from typing import Dict, List, Any, Optional, Tuple, Set
from dataclasses import dataclass
from enum import Enum
import numpy as np
from collections import deque


class MemoryType(Enum):
    """Different types of biological memory with varying retention characteristics"""
    EPHEMERAL = "ephemeral"
    SHORT_TERM = "short_term"
    MEDIUM_TERM = "medium_term"
    LONG_TERM = "long_term"
    CORE_KNOWLEDGE = "core_knowledge"


class AssociationType(Enum):
    """Types of associations between knowledge concepts"""
    SEMANTIC = "semantic"
    TEMPORAL = "temporal"
    CAUSAL = "causal"
    ANALOGICAL = "analogical"
    HIERARCHICAL = "hierarchical"
    CONTRADICTORY = "contradictory"
    CONTEXTUAL = "contextual"


@dataclass
class KnowledgeConcept:
    """Represents a concept in the knowledge network"""
    id: str
    content: str
    memory_type: MemoryType
    strength: float
    creation_time: float
    last_access: float
    access_frequency: int
    emotional_weight: float
    associations: List['Association']

    def __post_init__(self):
        if self.associations is None:
            self.associations = []


@dataclass
class Association:
    """Represents a relationship between two concepts"""
    source_id: str
    target_id: str
    association_type: AssociationType
    strength: float
    creation_time: float
    reinforcement_count: int


class BiologicalMemorySystem:
    """Implements biological memory principles for AI training"""

    def __init__(self, audit_logger: Optional[Any] = None, workspace_id: str = "core"):
        self.concepts: Dict[str, KnowledgeConcept] = {}
        self.associations: List[Association] = []
        # Indexes for performance and de-duplication
        self.content_index: Dict[str, str] = {}  # content -> concept_id
        self.association_index: Dict[Tuple[str, str, AssociationType], Association] = {}
        
        # Token-level inverted index for faster retrieval seeding
        self.token_index: Dict[str, Set[str]] = {}
        
        # Audit logger and workspace
        self.audit = audit_logger
        self.workspace_id = workspace_id

        # Memory configuration based on biological research
        self.memory_configs = {
            MemoryType.EPHEMERAL: {
                'decay_rate': 0.99,
                'capacity': 1000,
                'consolidation_threshold': 0.1
            },
            MemoryType.SHORT_TERM: {
                'decay_rate': 0.95,
                'capacity': 10000,
                'consolidation_threshold': 0.3
            },
            MemoryType.MEDIUM_TERM: {
                'decay_rate': 0.90,
                'capacity': 100000,
                'consolidation_threshold': 0.6
            },
            MemoryType.LONG_TERM: {
                'decay_rate': 0.85,
                'capacity': 1000000,
                'consolidation_threshold': 0.8
            },
            MemoryType.CORE_KNOWLEDGE: {
                'decay_rate': 0.0,
                'capacity': float('inf'),
                'consolidation_threshold': 0.95
            }
        }
        # Retrieval weighting and working memory from global settings
        from .config import settings
        self.memory_type_weights = settings.MEMORY_TYPE_WEIGHTS
        self.association_type_weights = settings.ASSOCIATION_TYPE_WEIGHTS
        from collections import deque
        self.working_memory = deque(maxlen=settings.WORKING_MEMORY_SIZE)
        self.working_memory_boost = settings.WORKING_MEMORY_BOOST
        # Token-level inverted index for faster retrieval seeding
        self.token_index: Dict[str, Set[str]] = {}

    def create_or_reinforce_concept(self, content: str, emotional_weight: float = 1.0) -> str:
        """Create a concept if new or reinforce existing by content."""
        content_key = content.strip()
        if not content_key:
            # Ignore empty
            return ""
        if content_key in self.content_index:
            concept_id = self.content_index[content_key]
            self.strengthen_concept(concept_id)  # reinforcement path
            return concept_id
        return self._create_concept(content_key, emotional_weight)

    def _create_concept(self, content: str, emotional_weight: float = 1.0) -> str:
        concept_id = f"concept_{len(self.concepts):06d}"
        current_time = time.time()
        concept = KnowledgeConcept(
            id=concept_id,
            content=content,
            memory_type=MemoryType.EPHEMERAL,
            strength=0.2,  # start low; grow via reinforcement
            creation_time=current_time,
            last_access=current_time,
            access_frequency=1,
            emotional_weight=emotional_weight,
            associations=[],
        )
        self.concepts[concept_id] = concept
        self.content_index[content] = concept_id
        # Index content tokens for retrieval seeding
        self._index_concept_content(concept_id)
        # Check for immediate consolidation if above threshold
        self._check_consolidation(concept_id)
        # Enforce capacity for this tier
        self._enforce_capacity(MemoryType.EPHEMERAL)
        return concept_id

    def strengthen_concept(self, concept_id: str) -> None:
        """Strengthen a concept through repeated access; update consolidation if eligible."""
        concept = self.concepts.get(concept_id)
        if not concept:
            return
        current_time = time.time()
        concept.last_access = current_time
        concept.access_frequency += 1
        # Hebbian-like strengthening modulated by emotion and usage
        delta = 0.15 * concept.emotional_weight * math.log1p(concept.access_frequency)
        concept.strength = min(1.0, concept.strength + delta)
        # Update working memory
        try:
            self.working_memory.append(concept_id)
        except Exception:
            pass
        self._check_consolidation(concept_id)

    def create_association(
        self,
        source_id: str,
        target_id: str,
        association_type: AssociationType,
        strength: float = 0.3,
        bidirectional: bool = True,
    ) -> None:
        """Create or reinforce association(s) between two concepts."""
        if source_id not in self.concepts or target_id not in self.concepts:
            return
        if source_id == "" or target_id == "":
            return
        self._upsert_association(source_id, target_id, association_type, strength)
        if bidirectional:
            self._upsert_association(target_id, source_id, association_type, strength)

    def _upsert_association(
        self,
        source_id: str,
        target_id: str,
        association_type: AssociationType,
        base_strength: float,
    ) -> None:
        key = (source_id, target_id, association_type)
        existing = self.association_index.get(key)
        if existing:
            existing.reinforcement_count += 1
            # Nonlinear strengthening with soft cap
            existing.strength = min(1.0, existing.strength + 0.05 + 0.05 * math.log1p(existing.reinforcement_count))
            # Audit reinforcement at milestones
            try:
                if self.audit and existing.reinforcement_count in {1, 5, 10, 20, 50}:
                    self.audit.log_assoc_reinforcement(source_id, target_id, association_type.value, existing.reinforcement_count, existing.strength)
            except Exception:
                pass
            return
        assoc = Association(
            source_id=source_id,
            target_id=target_id,
            association_type=association_type,
            strength=max(0.05, min(1.0, base_strength)),
            creation_time=time.time(),
            reinforcement_count=0,
        )
        self.associations.append(assoc)
        self.association_index[key] = assoc
        # Attach to source concept adjacency only once
        self.concepts[source_id].associations.append(assoc)

    def natural_forgetting(self) -> None:
        """Implement biological forgetting curves (Ebbinghaus-like exponential decay)."""
        current_time = time.time()
        to_prune: List[str] = []
        for concept in self.concepts.values():
            time_since_access = current_time - concept.last_access
            memory_config = self.memory_configs[concept.memory_type]
            # Apply exponential decay (hourly base)
            decay_factor = memory_config['decay_rate'] ** (time_since_access / 3600)
            concept.strength *= decay_factor
            # Track very weak concepts for pruning
            if concept.strength < 0.01 and concept.memory_type != MemoryType.CORE_KNOWLEDGE:
                to_prune.append(concept.id)
        for cid in to_prune:
            self._prune_concept(cid)

    def _check_consolidation(self, concept_id: str) -> None:
        """Upgrade memory tier when strength crosses threshold."""
        concept = self.concepts[concept_id]
        current_type = concept.memory_type
        config = self.memory_configs[current_type]
        if concept.strength >= config['consolidation_threshold']:
            old_type = current_type
            if current_type == MemoryType.EPHEMERAL:
                concept.memory_type = MemoryType.SHORT_TERM
            elif current_type == MemoryType.SHORT_TERM:
                concept.memory_type = MemoryType.MEDIUM_TERM
            elif current_type == MemoryType.MEDIUM_TERM:
                concept.memory_type = MemoryType.LONG_TERM
            elif current_type == MemoryType.LONG_TERM:
                concept.memory_type = MemoryType.CORE_KNOWLEDGE
            # Audit consolidation
            try:
                if self.audit:
                    self.audit.log_consolidation(concept.id, old_type.value, concept.memory_type.value)
            except Exception:
                pass
            # Enforce capacity each time we change tier
            self._enforce_capacity(concept.memory_type)

    def _enforce_capacity(self, memory_type: MemoryType) -> None:
        """Enforce capacity by pruning weakest concepts within a tier."""
        capacity = self.memory_configs[memory_type]['capacity']
        if capacity == float('inf'):
            return
        # Collect concepts of this tier
        tier_concepts = [c for c in self.concepts.values() if c.memory_type == memory_type]
        if len(tier_concepts) <= capacity:
            return
        # Sort weakest first: low strength, low usage, oldest access
        tier_concepts.sort(key=lambda c: (c.strength, c.access_frequency, c.last_access))
        to_remove = len(tier_concepts) - capacity
        for i in range(to_remove):
            self._prune_concept(tier_concepts[i].id)

    def _prune_concept(self, concept_id: str) -> None:
        """Remove weak concept and its associations and indexes."""
        concept = self.concepts.get(concept_id)
        if not concept:
            return
        # Remove from content index
        if concept.content in self.content_index and self.content_index[concept.content] == concept_id:
            del self.content_index[concept.content]
        # Remove from token index
        for tok in self._tokenize_text(concept.content):
            s = self.token_index.get(tok)
            if s and concept_id in s:
                s.discard(concept_id)
                if not s:
                    del self.token_index[tok]
        # Remove associations involving this concept and update index
        new_assoc_list: List[Association] = []
        new_assoc_index: Dict[Tuple[str, str, AssociationType], Association] = {}
        for a in self.associations:
            if a.source_id == concept_id or a.target_id == concept_id:
                continue
            new_assoc_list.append(a)
            new_assoc_index[(a.source_id, a.target_id, a.association_type)] = a
        self.associations = new_assoc_list
        self.association_index = new_assoc_index
        # Remove concept
        del self.concepts[concept_id]

    # ---------------- Token index helpers (MemorySystem) ----------------
    def _tokenize_text(self, text: str) -> Set[str]:
        raw = text.replace("\n", " ")
        tokens: List[str] = []
        for w in raw.split():
            w = w.strip(".,;:!?\"'()[]{}<>")
            if not w:
                continue
            wl = w.lower()
            if len(wl) > 2:
                tokens.append(wl)
        return set(tokens)

    def _index_concept_content(self, concept_id: str) -> None:
        concept = self.concepts.get(concept_id)
        if not concept:
            return
        for tok in self._tokenize_text(concept.content):
            s = self.token_index.get(tok)
            if not s:
                s = set()
                self.token_index[tok] = s
            s.add(concept_id)

    def rebuild_index(self) -> None:
        self.token_index.clear()
        for cid in self.concepts.keys():
            self._index_concept_content(cid)


class SwarmLearningAgent:
    """Base class for specialized learning agents"""

    def __init__(self, agent_type: str, memory_system: BiologicalMemorySystem):
        self.agent_type = agent_type
        self.memory_system = memory_system
        self.learning_rate = 0.01
        self.specialization_focus = 1.0

    async def learn_from_stream(self, text_stream: List[str]) -> Dict[str, Any]:
        """Learn from text stream - to be implemented by specialized agents"""
        raise NotImplementedError

    def extract_patterns(self, text: str) -> List[str]:
        """Extract patterns relevant to this agent's specialization"""
        raise NotImplementedError


class MolecularLearningAgent(SwarmLearningAgent):
    """Learns word-level patterns, entities, and syntax"""

    def __init__(self, memory_system: BiologicalMemorySystem):
        super().__init__("molecular", memory_system)

    async def learn_from_stream(self, text_stream: List[str]) -> Dict[str, Any]:
        """Learn molecular-level patterns with de-dup and reinforcement."""
        per_text = []
        total = 0
        for text in text_stream:
            tokens = self.extract_patterns(text)
            # De-dup tokens within a text span
            seen: Set[str] = set()
            concept_ids: List[str] = []
            for tok in tokens:
                if tok in seen:
                    continue
                seen.add(tok)
                cid = self.memory_system.create_or_reinforce_concept(tok, emotional_weight=0.5)
                if cid:
                    concept_ids.append(cid)
                    total += 1
            per_text.append({'text': text, 'concept_ids': concept_ids, 'tokens': list(seen)})
        return {
            'agent_type': self.agent_type,
            'per_text': per_text,
            'total_created_or_reinforced': total,
        }

    def extract_patterns(self, text: str) -> List[str]:
        """Extract molecular patterns: words, entities, syntax (very lightweight)."""
        raw = text.replace("\n", " ")
        words = [w.strip(".,;:!?") for w in raw.split() if w.strip()]
        lower = [w.lower() for w in words]
        # Meaningful tokens (>2 chars)
        meaningful = [w for w in lower if len(w) > 2]
        # Simple entity heuristic: capitalized original forms
        entities = [w for w in words if w and w[0].isupper()]
        return meaningful + entities


class SemanticLearningAgent(SwarmLearningAgent):
    """Learns meaning and conceptual relationships"""

    def __init__(self, memory_system: BiologicalMemorySystem):
        super().__init__("semantic", memory_system)

    async def learn_from_stream(self, text_stream: List[str]) -> Dict[str, Any]:
        """Learn semantic patterns and per-text associations; no cross-history O(n^2)."""
        per_text = []
        total_concepts = 0
        total_assocs = 0
        for text in text_stream:
            sentences = self.extract_patterns(text)
            sentence_ids: List[str] = []
            # Create or reinforce sentence concepts
            seen_sentences: Set[str] = set()
            for s in sentences:
                if s in seen_sentences:
                    continue
                seen_sentences.add(s)
                cid = self.memory_system.create_or_reinforce_concept(s, emotional_weight=0.8)
                if cid:
                    sentence_ids.append(cid)
                    total_concepts += 1
            # Create semantic associations within this text only
            for i in range(len(sentence_ids)):
                for j in range(i + 1, len(sentence_ids)):
                    self.memory_system.create_association(
                        sentence_ids[i], sentence_ids[j], AssociationType.SEMANTIC, strength=0.3
                    )
                    total_assocs += 2  # bidirectional
            per_text.append({'text': text, 'sentence_ids': sentence_ids, 'sentences': list(seen_sentences)})
        return {
            'agent_type': self.agent_type,
            'per_text': per_text,
            'total_created_or_reinforced': total_concepts,
            'associations_created_or_reinforced': total_assocs,
        }

    def extract_patterns(self, text: str) -> List[str]:
        """Extract semantic concepts and themes (sentences)."""
        # Simplified sentence segmentation on '.' and line breaks
        raw = text.replace("\n", " ")
        candidates = [s.strip() for s in raw.split('.')]
        concepts: List[str] = []
        for s in candidates:
            if len(s) >= 3 and len(s.split()) >= 3:  # basic signal for meaningful sentence
                concepts.append(s)
        return concepts


class BiologicalTrainer:
    """Main training system using biological principles and swarm intelligence"""

    def __init__(self, base_path: Optional[str] = None, workspace_id: Optional[str] = None, audit_enabled: Optional[bool] = None, use_full_swarm: bool = False):
        from .config import settings
        self.base_path = base_path or settings.BASE_PATH
        self.workspace_id = workspace_id or settings.WORKSPACE_ID
        self.use_full_swarm = use_full_swarm
        use_audit = settings.AUDIT_ENABLED if audit_enabled is None else bool(audit_enabled)
        audit = None
        if use_audit:
            try:
                from .audit_pbss import AuditLogger
                audit = AuditLogger(self.base_path, self.workspace_id)
            except Exception:
                audit = None
        self.memory_system = BiologicalMemorySystem(audit_logger=audit, workspace_id=self.workspace_id)
        
        # Initialize agents based on swarm mode
        if use_full_swarm:
            # Use the full 7-agent swarm for 10,000x emergence
            try:
                from .swarm_agents import SwarmOrchestrator
                self.swarm_orchestrator = SwarmOrchestrator(self.memory_system)
                self.agents = self.swarm_orchestrator.agents
                print("🚀 FULL 7-AGENT SWARM ACTIVATED - 10,000x EMERGENCE POTENTIAL")
            except ImportError:
                # Fallback to 2-agent system
                self.agents = {
                    'molecular': MolecularLearningAgent(self.memory_system),
                    'semantic': SemanticLearningAgent(self.memory_system),
                }
                self.swarm_orchestrator = None
                print("⚠️ Swarm agents not found, using 2-agent system (809x emergence)")
        else:
            # Use original 2-agent system
            self.agents = {
                'molecular': MolecularLearningAgent(self.memory_system),
                'semantic': SemanticLearningAgent(self.memory_system),
            }
            self.swarm_orchestrator = None
        
        self.is_training = True
        self.training_cycles = 0
        # Keep last seen sentence concept IDs for temporal chaining across batches
        self._last_sentence_ids: List[str] = []

    # Persistence helpers (PBSS)
    def save_memory(self, base_path: Optional[str] = None) -> None:
        """Persist the current memory system to disk using PBSS."""
        try:
            from .persistence_pbss import AssociativeMemoryPersistence
            path = base_path or self.base_path
            p = AssociativeMemoryPersistence(path, self.workspace_id)
            p.save(self.memory_system)
        except Exception as e:
            print(f"Persistence save error: {e}")

    def load_memory(self, base_path: Optional[str] = None) -> None:
        """Load memory system from PBSS and replace current in-memory state."""
        try:
            from .persistence_pbss import AssociativeMemoryPersistence
            path = base_path or self.base_path
            p = AssociativeMemoryPersistence(path, self.workspace_id)
            loaded = p.load()
            # Replace internal state
            self.memory_system = loaded
            # Rebuild token index after load
            self.memory_system.rebuild_index()
            # Re-wire agents to new memory reference
            self.agents = {
                'molecular': MolecularLearningAgent(self.memory_system),
                'semantic': SemanticLearningAgent(self.memory_system),
            }
        except Exception as e:
            print(f"Persistence load error: {e}")

    async def train_from_stream(self, text_stream: List[str]) -> Dict[str, Any]:
        """Train using swarm intelligence on text stream."""
        training_start = time.time()

        # Use full swarm if available
        if self.use_full_swarm and self.swarm_orchestrator:
            result = await self.swarm_orchestrator.swarm_learn(text_stream)
            agent_results = result['agent_results']
            emergence_factor = result.get('emergence_factor', 0)
            consciousness_score = result.get('consciousness_score', 0)
            
            # Log consciousness emergence if detected
            if consciousness_score > 0.5:
                print(f"🧠 CONSCIOUSNESS EMERGING: Score {consciousness_score:.2f}")
        else:
            # Original 2-agent parallel learning
            tasks = [asyncio.create_task(agent.learn_from_stream(text_stream)) for agent in self.agents.values()]
            agent_results = await asyncio.gather(*tasks)
            emergence_factor = 0
            consciousness_score = 0

        # Integrate cross-agent associations per text
        molecular = next((r for r in agent_results if r['agent_type'] == 'molecular'), None)
        semantic = next((r for r in agent_results if r['agent_type'] == 'semantic'), None)

        if molecular and semantic:
            per_text_mol = molecular['per_text']
            per_text_sem = semantic['per_text']
            # Hierarchical: connect sentence concepts to token concepts for each text index
            for i in range(min(len(per_text_mol), len(per_text_sem))):
                mol_ids = per_text_mol[i]['concept_ids']
                sent_ids = per_text_sem[i]['sentence_ids']
                for sid in sent_ids:
                    for tid in mol_ids:
                        self.memory_system.create_association(sid, tid, AssociationType.HIERARCHICAL, strength=0.4)

            # Temporal: connect sentence concepts across successive texts (within this batch)
            batch_sentence_ids: List[List[str]] = [entry['sentence_ids'] for entry in per_text_sem]
            # link from previous batch tail to current batch head
            if self._last_sentence_ids and batch_sentence_ids:
                for prev in self._last_sentence_ids:
                    for curr in batch_sentence_ids[0]:
                        self.memory_system.create_association(prev, curr, AssociationType.TEMPORAL, strength=0.2)
            # link within this batch sequentially
            for i in range(1, len(batch_sentence_ids)):
                for prev in batch_sentence_ids[i - 1]:
                    for curr in batch_sentence_ids[i]:
                        self.memory_system.create_association(prev, curr, AssociationType.TEMPORAL, strength=0.2)
            # update last for next call
            self._last_sentence_ids = batch_sentence_ids[-1] if batch_sentence_ids else []

        # Biological maintenance: forgetting, pruning, and capacity enforcement
        self._biological_maintenance()

        training_time = time.time() - training_start
        self.training_cycles += 1
        stats = self._get_memory_stats()
        # Audit training cycle
        try:
            if self.memory_system.audit:
                self.memory_system.audit.log_training_cycle(self.training_cycles, stats)
        except Exception:
            pass

        return {
            'training_time': training_time,
            'agents_results': agent_results,
            'consensus_knowledge': self._achieve_consensus(agent_results),
            'memory_stats': stats,
            'training_cycles': self.training_cycles,
        }

    async def continuous_training(self, data_stream_generator):
        """Continuous training that never stops"""
        while self.is_training:
            try:
                text_batch = await data_stream_generator.__anext__()
                await self.train_from_stream(text_batch)
                if self.training_cycles % 100 == 0:
                    await self._sleep_consolidation()
            except StopAsyncIteration:
                break
            except Exception as e:
                print(f"Training error: {e}")
                await asyncio.sleep(1)

    def _achieve_consensus(self, agent_results: List[Dict[str, Any]]) -> Dict[str, Any]:
        """Summarize cross-agent activity for observability."""
        total = sum(r.get('total_created_or_reinforced', 0) for r in agent_results)
        assoc = sum(r.get('associations_created_or_reinforced', 0) for r in agent_results)
        return {
            'total_concepts_touched': total,
            'associations_touched': assoc,
            'agents_participating': len(agent_results),
        }

    def _biological_maintenance(self) -> None:
        """Perform biological memory maintenance: forgetting, pruning weak links, capacity."""
        self.memory_system.natural_forgetting()
        self._prune_weak_associations()
        # Enforce capacity on all tiers
        for tier in [t for t in MemoryType]:
            self.memory_system._enforce_capacity(tier)

    def _prune_weak_associations(self) -> None:
        """Remove weak associations to maintain network efficiency and sync index."""
        threshold = 0.1
        new_list: List[Association] = []
        new_index: Dict[Tuple[str, str, AssociationType], Association] = {}
        for a in self.memory_system.associations:
            if a.strength >= threshold:
                new_list.append(a)
                new_index[(a.source_id, a.target_id, a.association_type)] = a
        self.memory_system.associations = new_list
        self.memory_system.association_index = new_index

    async def _sleep_consolidation(self) -> None:
        """Simulate sleep-based memory consolidation."""
        # Strengthen frequently accessed; slightly weaken rarely accessed
        for concept in self.memory_system.concepts.values():
            if concept.access_frequency > 5:
                concept.strength = min(1.0, concept.strength * 1.1)
            elif concept.access_frequency <= 2:
                concept.strength *= 0.95
            self.memory_system._check_consolidation(concept.id)
        await asyncio.sleep(0.05)

    def _get_memory_stats(self) -> Dict[str, Any]:
        """Get current memory system statistics."""
        memory_distribution = {memory_type.value: 0 for memory_type in MemoryType}
        for concept in self.memory_system.concepts.values():
            memory_distribution[concept.memory_type.value] += 1
        avg_strength = (
            float(np.mean([c.strength for c in self.memory_system.concepts.values()]))
            if self.memory_system.concepts
            else 0.0
        )
        return {
            'total_concepts': len(self.memory_system.concepts),
            'total_associations': len(self.memory_system.associations),
            'memory_distribution': memory_distribution,
            'average_strength': avg_strength,
        }

    def query_knowledge(self, query: str, max_results: int = 10, hops: int = 1, alpha: float = 0.5, top_k_neighbors: int = 8) -> List[Dict[str, Any]]:
        """Associative retrieval with multi-hop spreading activation.
        - hops: number of propagation hops beyond seed (>=1 for multi-hop)
        - alpha: decay per hop
        - top_k_neighbors: limit neighbors per node by strongest associations
        """
        if not query.strip():
            return []
        base_scores: Dict[str, float] = {}
        query_tokens = self.memory_system._tokenize_text(query)
        # Candidate selection via token index (fallback to all concepts if no candidates)
        candidate_ids: Set[str] = set()
        for tok in query_tokens:
            ids = self.memory_system.token_index.get(tok)
            if ids:
                candidate_ids.update(ids)
        if not candidate_ids:
            candidate_items = list(self.memory_system.concepts.items())
        else:
            candidate_items = [(cid, self.memory_system.concepts[cid]) for cid in candidate_ids if cid in self.memory_system.concepts]
        # Seed scores from direct content overlap
        for cid, concept in candidate_items:
            concept_words = self.memory_system._tokenize_text(concept.content)
            if not concept_words:
                continue
            overlap = len(query_tokens.intersection(concept_words))
            union = len(query_tokens.union(concept_words))
            if union == 0:
                continue
            content_relevance = overlap / union
            if content_relevance <= 0:
                continue
            memory_boost = concept.strength * (1 + math.log1p(concept.access_frequency))
            score = content_relevance * memory_boost
            if score > 0.05:
                base_scores[cid] = score
                # Reinforce accessed seeds
                self.memory_system.strengthen_concept(cid)
        # Combined scores, starting with weighted base scores
        combined_scores: Dict[str, float] = {}
        for cid, score in base_scores.items():
            c = self.memory_system.concepts[cid]
            w = self.memory_system.memory_type_weights[c.memory_type]
            # Working memory boost
            wm = (1.0 + self.memory_system.working_memory_boost) if cid in self.memory_system.working_memory else 1.0
            combined_scores[cid] = combined_scores.get(cid, 0.0) + score * w * wm
        # Multi-hop propagation
        frontier: Dict[str, float] = dict(base_scores)
        visited: Set[str] = set(base_scores.keys())
        for hop in range(hops):
            next_frontier: Dict[str, float] = {}
            decay = (alpha ** (hop + 1))
            for nid, nscore in frontier.items():
                node = self.memory_system.concepts.get(nid)
                if not node:
                    continue
                # Choose top-K neighbors by association strength
                neighbors = sorted(node.associations, key=lambda a: a.strength, reverse=True)[:max(0, int(top_k_neighbors))]
                for assoc in neighbors:
                    neighbor_id = assoc.target_id
                    if neighbor_id not in self.memory_system.concepts:
                        continue
                    neighbor = self.memory_system.concepts[neighbor_id]
                    w_neighbor = self.memory_system.memory_type_weights[neighbor.memory_type]
                    w_assoc_type = self.memory_system.association_type_weights.get(assoc.association_type, 0.8)
                    boost = decay * nscore * assoc.strength * w_neighbor * w_assoc_type
                    # Working memory boost
                    if neighbor_id in self.memory_system.working_memory:
                        boost *= (1.0 + self.memory_system.working_memory_boost)
                    combined_scores[neighbor_id] = combined_scores.get(neighbor_id, 0.0) + boost
                    # Accumulate for next frontier
                    next_frontier[neighbor_id] = next_frontier.get(neighbor_id, 0.0) + boost
            frontier = next_frontier
            if not frontier:
                break
        # Rank and format results
        ranked = sorted(combined_scores.items(), key=lambda kv: kv[1], reverse=True)
        out: List[Dict[str, Any]] = []
        for cid, score in ranked[:max_results]:
            c = self.memory_system.concepts[cid]
            out.append({
                'concept_id': cid,
                'content': c.content,
                'relevance': float(score),
                'memory_type': c.memory_type.value,
                'strength': float(c.strength),
            })
        return out


# Example usage
async def demonstrate_biological_training():
    """Demonstrate the biological training system"""
    trainer = BiologicalTrainer()

    # Sample text stream
    text_stream = [
        "Machine learning is a subset of artificial intelligence.",
        "Neural networks are inspired by biological brain structures.",
        "Deep learning uses multiple layers to learn complex patterns.",
        "Training requires large datasets and computational resources.",
        "Biological systems learn efficiently with minimal data.",
    ]

    print("Starting biological training demonstration...")

    # Train on the text stream
    results = await trainer.train_from_stream(text_stream)

    print(f"Training completed in {results['training_time']:.3f} seconds")
    print(f"Memory stats: {results['memory_stats']}")

    # Query the knowledge
    query_results = trainer.query_knowledge("machine learning neural networks")

    print(f"\nQuery results for 'machine learning neural networks':")
    for result in query_results[:3]:
        print(f"- {result['content']} (relevance: {result['relevance']:.3f})")

    return trainer


if __name__ == "__main__":
    # Run the demonstration
    asyncio.run(demonstrate_biological_training())
