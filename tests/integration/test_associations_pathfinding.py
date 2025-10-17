import numpy as np
import time
from pathlib import Path
import pytest

integration = pytest.mark.integration

try:
    from sutra_core.storage.rust_adapter import RustStorageAdapter
    from sutra_core.graph.concepts import Concept, Association, AssociationType
    RUST_AVAILABLE = True
except Exception:
    RUST_AVAILABLE = False


@integration
@pytest.mark.skipif(not RUST_AVAILABLE, reason="Rust storage not available")
def test_association_neighbors(tmp_path: Path):
    dim = 4
    storage = RustStorageAdapter(tmp_path, vector_dimension=dim)

    v0 = np.array([1, 0, 0, 0], dtype=np.float32)
    v1 = np.array([0, 1, 0, 0], dtype=np.float32)

    storage.add_concept(Concept(id="a", content="A", strength=1.0, confidence=1.0), embedding=v0)
    storage.add_concept(Concept(id="b", content="B", strength=1.0, confidence=1.0), embedding=v1)

    storage.add_association(Association(
        source_id="a", target_id="b", assoc_type=AssociationType.SEMANTIC, confidence=0.9
    ))

    time.sleep(0.05)  # allow reconciliation

    neighbors = storage.get_neighbors("a")
    assert "b" in neighbors


@integration
@pytest.mark.skipif(not RUST_AVAILABLE, reason="Rust storage not available")
def test_find_paths_chain(tmp_path: Path):
    dim = 4
    storage = RustStorageAdapter(tmp_path, vector_dimension=dim)

    v0 = np.array([1, 0, 0, 0], dtype=np.float32)
    v1 = np.array([0, 1, 0, 0], dtype=np.float32)
    v2 = np.array([0, 0, 1, 0], dtype=np.float32)

    storage.add_concept(Concept(id="a", content="A", strength=1.0, confidence=1.0), embedding=v0)
    storage.add_concept(Concept(id="b", content="B", strength=1.0, confidence=1.0), embedding=v1)
    storage.add_concept(Concept(id="c", content="C", strength=1.0, confidence=1.0), embedding=v2)

    storage.add_association(Association(
        source_id="a", target_id="b", assoc_type=AssociationType.SEMANTIC, confidence=0.9
    ))
    storage.add_association(Association(
        source_id="b", target_id="c", assoc_type=AssociationType.SEMANTIC, confidence=0.9
    ))

    time.sleep(0.05)

    # Adapter exposes find_paths over lists
    paths = storage.find_paths(["a"], ["c"], max_depth=4, num_paths=3, query="test")

    # We expect at least one path reaching 'c'
    assert isinstance(paths, list)
    assert any(getattr(p, "steps", []) and p.steps[-1].target_id == "c" for p in paths)
