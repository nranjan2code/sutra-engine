"""
Associative Memory Persistence using Pure Binary Storage (PBSS)

This module persists and restores the BiologicalMemorySystem without any
traditional database, using a binary node store adapted from sutra-swarm.
"""

import os
import time
from typing import Dict, Any, List, Optional

from .pure_binary_storage import PureBinaryStorage
from .biological_trainer import (
    BiologicalMemorySystem,
    KnowledgeConcept,
    Association,
    MemoryType,
    AssociationType,
)


class AssociativeMemoryPersistence:
    def __init__(self, base_path: str = "knowledge_store", workspace_id: str = "core"):
        self.base_path = base_path
        self.workspace_id = workspace_id
        self.nodes_path = os.path.join(base_path, "nodes")
        os.makedirs(self.nodes_path, exist_ok=True)
        self.storage = PureBinaryStorage(self.nodes_path)

    # --------- Save ---------
    def save(self, mem: BiologicalMemorySystem) -> None:
        # Save concepts
        for c in mem.concepts.values():
            node = {
                "id": c.id,
                "level": "concept",
                "content": c.content,
                "timestamp": c.creation_time,
                "metadata": {
                    "format": "v1",
                    "workspace_id": self.workspace_id,
                    "memory_type": c.memory_type.value,
                    "strength": str(c.strength),
                    "last_access": str(c.last_access),
                    "access_frequency": str(c.access_frequency),
                    "emotional_weight": str(c.emotional_weight),
                },
            }
            self.storage.store_node(node)
        # Save associations
        for a in mem.associations:
            node = {
                "id": f"assoc:{a.source_id}->{a.target_id}:{a.association_type.value}",
                "level": "association",
                "content": "",
                "timestamp": a.creation_time,
                "metadata": {
                    "format": "v1",
                    "workspace_id": self.workspace_id,
                    "source_id": a.source_id,
                    "target_id": a.target_id,
                    "type": a.association_type.value,
                    "strength": str(a.strength),
                    "reinforcement_count": str(a.reinforcement_count),
                },
            }
            self.storage.store_node(node)

    # --------- Load ---------
    def load(self) -> BiologicalMemorySystem:
        nodes = self.storage.list_all_nodes()
        # Simple workspace filtering - only exact matches or no workspace_id
        filtered = []
        for node in nodes:
            md = node.get("metadata", {})
            ws = md.get("workspace_id")
            # Only include exact matches or nodes without workspace_id
            if ws is None or ws == self.workspace_id:
                filtered.append(node)
        nodes = filtered
        mem = BiologicalMemorySystem()
        # Clear any initial state
        mem.concepts.clear()
        mem.associations.clear()
        mem.content_index.clear()
        mem.association_index.clear()

        # Reconstruct concepts
        for node in nodes:
            if node.get("level") != "concept":
                continue
            md = node.get("metadata", {})
            cid = node.get("id")
            content = node.get("content", "")
            try:
                concept = KnowledgeConcept(
                    id=cid,
                    content=content,
                    memory_type=MemoryType(md.get("memory_type", MemoryType.EPHEMERAL.value)),
                    strength=float(md.get("strength", "0.2")),
                    creation_time=float(node.get("timestamp", time.time())),
                    last_access=float(md.get("last_access", node.get("timestamp", time.time()))),
                    access_frequency=int(float(md.get("access_frequency", "1"))),
                    emotional_weight=float(md.get("emotional_weight", "1.0")),
                    associations=[],
                )
            except Exception:
                # Fallbacks in case of malformed node
                concept = KnowledgeConcept(
                    id=cid or f"concept_{len(mem.concepts):06d}",
                    content=content,
                    memory_type=MemoryType.EPHEMERAL,
                    strength=0.2,
                    creation_time=time.time(),
                    last_access=time.time(),
                    access_frequency=1,
                    emotional_weight=1.0,
                    associations=[],
                )
            mem.concepts[concept.id] = concept
            if content:
                mem.content_index[content] = concept.id

        # Reconstruct associations
        for node in nodes:
            if node.get("level") != "association":
                continue
            md = node.get("metadata", {})
            src = md.get("source_id", "")
            tgt = md.get("target_id", "")
            atype = md.get("type", AssociationType.SEMANTIC.value)
            try:
                assoc = Association(
                    source_id=src,
                    target_id=tgt,
                    association_type=AssociationType(atype),
                    strength=float(md.get("strength", "0.3")),
                    creation_time=float(node.get("timestamp", time.time())),
                    reinforcement_count=int(float(md.get("reinforcement_count", "0"))),
                )
            except Exception:
                continue
            # Only attach if concepts exist
            if src in mem.concepts and tgt in mem.concepts:
                mem.associations.append(assoc)
                mem.concepts[src].associations.append(assoc)
                mem.association_index[(src, tgt, assoc.association_type)] = assoc

        return mem