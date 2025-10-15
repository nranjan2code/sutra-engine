# Development Setup

This document describes how to set up a development environment for the Sutra Models monorepo.

## Prerequisites

- Python 3.8+
- git
- macOS/Linux/Windows (WSL2 recommended on Windows)

## Create and Activate Virtual Environment

```bash
python3 -m venv venv
# macOS/Linux
source venv/bin/activate
# Windows PowerShell
venv\Scripts\Activate.ps1
```

Verify Python version:

```bash
python --version
```

## Install Packages in Editable Mode

Minimal core-only:

```bash
pip install -e packages/sutra-core/
```

Core + Hybrid (TF-IDF only):

```bash
pip install -e packages/sutra-core/
pip install -e packages/sutra-hybrid/
```

Hybrid with semantic embeddings (recommended):

```bash
pip install -e packages/sutra-core/
pip install -e "packages/sutra-hybrid/[embeddings]"
```

API service:

```bash
pip install -e packages/sutra-api/
```

(Optional) CLI (planned):

```bash
pip install -e packages/sutra-cli/
```

## Install Development Tooling

```bash
pip install -r requirements-dev.txt
# Or install ad-hoc
pip install black isort flake8 mypy pytest pytest-asyncio httpx
```

## Environment Variables

These variables control API and runtime behavior.

```bash
# API server
export SUTRA_HOST="0.0.0.0"
export SUTRA_PORT=8000
export SUTRA_STORAGE_PATH="./api_knowledge"
export SUTRA_USE_SEMANTIC_EMBEDDINGS=true
export SUTRA_LOG_LEVEL="INFO"

# PYTHONPATH for running tests manually
export PYTHONPATH=packages/sutra-core:packages/sutra-hybrid
```

## Makefile Shortcuts (if available)

```bash
# One-time setup
make setup

# Run core tests
make test-core

# Format and lint
make format
make lint           # Lint core package (flake8, mypy)
make lint-all       # Lint entire repo (may report issues in non-core packages)

# Build all packages
make build
```

## Repository Layout

```
sutra-models/
├── packages/
│   ├── sutra-core/
│   ├── sutra-hybrid/
│   ├── sutra-api/
│   └── sutra-cli/
├── docs/
└── venv/
```

## IDE Configuration

- Enable Black and isort on save
- Configure flake8 via the repo’s .flake8 (max-line-length 88; ignore E203, W503)
- Set project root as working directory so relative paths resolve

## Verification

```bash
# Core package smoke test
python -c "from sutra_core import Concept; print('sutra-core OK')"

# Hybrid smoke test
python -c "from sutra_hybrid import HybridAI; print('sutra-hybrid OK')"

# API smoke test
python -c "from sutra_api import settings; print('sutra-api OK')"
```