# Testing

We use pytest with a lightweight, architecture-aligned suite.

Install dev deps:
```
pip install -r requirements-dev.txt  # if present
```

Run unit tests:
```
pytest -q
```

Run integration tests (Rust storage):
```
pytest -q -m integration
```

CI:
- Unit tests run on push/PR via GitHub Actions (.github/workflows/ci.yml)
- Integration tests run manually via "Run workflow" with input run_integration=true

Notes:
- Unit tests avoid heavy models by using fake or small embeddings
- QueryProcessor tests pass numpy arrays directly to storage.vector_search
- Storage integration tests are marked with @pytest.mark.integration and skipped by default; they use a temporary directory and call storage.save() for deterministic teardown
