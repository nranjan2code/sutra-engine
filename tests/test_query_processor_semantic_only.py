import numpy as np
import tempfile
from pathlib import Path

from sutra_core.reasoning.query import QueryProcessor
from sutra_core.learning.associations import AssociationExtractor
from sutra_core.reasoning.paths import PathFinder
from sutra_core.reasoning.mppa import MultiPathAggregator
from sutra_core.graph.concepts import Concept


class FakeEmbeddingProcessor:
    def __init__(self, mapping):
        self.mapping = mapping  # dict[str, np.ndarray]

    def encode_single(self, text: str, prompt_name: str = "Retrieval-query"):
        # Return exact vector for known queries; else a default
        return self.mapping.get(text, np.zeros(4, dtype=np.float32))


def test_query_processor_direct_semantic_answer():
    # Prepare in-memory fake storage with two concepts
    concepts = {
        "py": Concept(id="py", content="Python is a programming language.", strength=1.0, confidence=1.0),
        "rs": Concept(id="rs", content="Rust is a systems programming language.", strength=1.0, confidence=1.0),
    }

    class FakeStorage:
        def get_concept(self, cid):
            return concepts.get(cid)
        def vector_search(self, query_embedding, k=10):
            # Always rank Python higher for this test
            return [("py", 0.99), ("rs", 0.50)][:k]
        def get_neighbors(self, cid):
            return []
        def get_association(self, a, b):
            return None
        def find_paths(self, start_ids, target_ids, max_depth=5, num_paths=5, query=""):
            return []

    storage = FakeStorage()

    # Fake embedder maps query text to a fixed small vector
    v_python = np.array([1, 0, 0, 0], dtype=np.float32)
    fake_embed = FakeEmbeddingProcessor({
        "what is python?": v_python,
    })

    qp = QueryProcessor(
        storage=storage,
        association_extractor=AssociationExtractor(storage),
        path_finder=PathFinder(storage),
        mppa=MultiPathAggregator(),
        embedding_processor=fake_embed,
        nlp_processor=None,
    )

    result = qp.process_query("what is python?", num_reasoning_paths=3, max_concepts=5)

    assert result is not None
    assert result.primary_answer and isinstance(result.primary_answer, str)
    assert result.confidence >= 0.99
    assert "Direct semantic match" in result.reasoning_explanation
