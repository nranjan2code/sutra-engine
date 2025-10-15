# Testing

This document explains how to run tests across packages and how to write new tests.

## Test Matrix

| Package      | Command                                                                                 | Notes                      |
|--------------|------------------------------------------------------------------------------------------|----------------------------|
| sutra-core   | `PYTHONPATH=packages/sutra-core python -m pytest packages/sutra-core/tests -v`           | 60 tests                    |
| sutra-hybrid | `PYTHONPATH=packages/sutra-hybrid:packages/sutra-core python -m pytest packages/sutra-hybrid/tests -v` | 9 tests, persistence focus |
| all          | `make test`                                                                              | If Makefile available      |

## Running Specific Tests

Single file:

```bash
PYTHONPATH=packages/sutra-core python -m pytest packages/sutra-core/tests/test_basic.py -v
```

Single test:

```bash
PYTHONPATH=packages/sutra-core python -m pytest \
  packages/sutra-core/tests/test_basic.py::test_concept_creation -v
```

With coverage report:

```bash
PYTHONPATH=packages/sutra-core python -m pytest \
  packages/sutra-core/tests -v --cov=sutra_core --cov-report=html
```

## Hybrid Persistence Tests

A dedicated suite validates TF-IDF vectorizer persistence via pickle.

Run:

```bash
PYTHONPATH=packages/sutra-hybrid:packages/sutra-core \
  python -m pytest packages/sutra-hybrid/tests/test_tfidf_persistence.py -v
```

What is validated:
- Save/load cycles preserve sklearn TfidfVectorizer
- Embeddings are identical pre/post load
- Semantic search works after reload
- Multiple save/load cycles maintain integrity

## API Testing (to add)

Recommended stack:
- `pytest`
- `pytest-asyncio`
- `httpx` (AsyncClient)

Example structure (planned):

```
packages/sutra-api/tests/
├── test_health.py
├── test_learning.py
├── test_reasoning.py
└── test_management.py
```

Sample async test:

```python
import pytest
from httpx import AsyncClient
from sutra_api.main import app

@pytest.mark.asyncio
async def test_health():
    async with AsyncClient(app=app, base_url="http://test") as ac:
        resp = await ac.get("/health")
        assert resp.status_code == 200
        data = resp.json()
        assert data["status"] == "healthy"
```

## Linting and Formatting

Run formatters:

```bash
black packages/
isort packages/
```

Run linter (core package):

```bash
make lint
# or
flake8 packages/sutra-core/sutra_core
```

Type checking (where configured):

```bash
mypy packages/sutra-core/sutra_core/
```

## Test Writing Guidelines

- Use realistic inputs and edge cases
- Keep tests independent and idempotent
- For persistence tests, use a temporary directory (e.g., `tempfile.mkdtemp()`)
- Assert both behavior and invariants (counts, lengths, types)
- Prefer `np.testing.assert_array_almost_equal` for float arrays

## Troubleshooting

- If tests fail due to import errors, ensure PYTHONPATH includes required packages
- Clear caches when flaky behavior occurs:

```bash
rm -rf .pytest_cache packages/*/.pytest_cache
```

- Confirm virtualenv is active:

```bash
which python
python -c "import sys; print(sys.executable)"
```