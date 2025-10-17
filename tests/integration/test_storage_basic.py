import os
import numpy as np
import time
from pathlib import Path
import pytest

integration = pytest.mark.integration

try:
    from sutra_core.storage.rust_adapter import RustStorageAdapter
    RUST_AVAILABLE = True
except Exception:
    RUST_AVAILABLE = False


@integration
@pytest.mark.skipif(not RUST_AVAILABLE, reason="Rust storage not available")
def test_storage_learn_and_search_smoke(tmp_path: Path):
    # Use default dimension (768) to match storage
    dim = 768
    storage = RustStorageAdapter(tmp_path, vector_dimension=dim)

    v_a = np.zeros(dim, dtype=np.float32); v_a[0] = 1.0
    v_b = np.zeros(dim, dtype=np.float32); v_b[1] = 1.0

    from sutra_core.graph.concepts import Concept
    storage.add_concept(Concept(id="a", content="A concept", strength=1.0, confidence=1.0), embedding=v_a)
    storage.add_concept(Concept(id="b", content="B concept", strength=1.0, confidence=1.0), embedding=v_b)

    # Give reconciler a brief moment (defensive)
    time.sleep(0.05)

    # Query near A
    q = np.zeros(dim, dtype=np.float32); q[0] = 0.9; q[1] = 0.1
    results = storage.vector_search(q, k=2)

    assert isinstance(results, list)
    assert len(results) >= 1

    # Flush explicitly to ensure deterministic teardown
    storage.save()
