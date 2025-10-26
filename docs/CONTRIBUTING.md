# Contributing to Sutra AI

Welcome to the Sutra AI project! This guide will help you get started with contributing to our explainable graph-based AI system.

## Table of Contents

- [Getting Started](#getting-started)
- [Development Environment](#development-environment)
- [Development Workflow](#development-workflow)
- [Code Style and Standards](#code-style-and-standards)
- [Testing Guidelines](#testing-guidelines)
- [Documentation Standards](#documentation-standards)
- [Commit Guidelines](#commit-guidelines)
- [Pull Request Process](#pull-request-process)
- [Release Process](#release-process)
- [Getting Help](#getting-help)

## Getting Started

### Prerequisites

- **Python 3.9+** (3.10 recommended)
- **Git** for version control
- **Make** for build automation
- **2GB RAM minimum** for development
- **macOS, Linux, or WSL2** (Windows with WSL2)

### Recommended Tools

- **VS Code** with Python extension
- **PyCharm** Community or Professional
- **pytest** for testing (included in dev dependencies)
- **black** and **isort** for code formatting (included)

## Development Environment

### Initial Setup

```bash
# Clone the repository
git clone <repository-url>
cd sutra-models

# One-command setup (creates venv, installs all packages)
make setup

# Manual setup (alternative)
python3 -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate
pip install -e packages/sutra-core/
pip install -r requirements-dev.txt
```

### Virtual Environment Activation

Always activate the virtual environment before development:

```bash
source venv/bin/activate  # macOS/Linux
# or
venv\Scripts\activate  # Windows
```

### Verifying Setup

```bash
# Run core tests
make test-core

# Run demo
make demo-core

# Check code quality
make check
```

## Development Workflow

### Standard Development Cycle

1. **Create a feature branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes** in the appropriate package
   - `packages/sutra-core/` - Core reasoning engine
   - `packages/sutra-hybrid/` - Semantic embeddings (in progress)
   - `packages/sutra-api/` - REST API (planned)
   - `packages/sutra-cli/` - CLI interface (planned)

3. **Run tests frequently**
   ```bash
   make test-core  # After each logical change
   ```

4. **Format and lint**
   ```bash
   make format  # Auto-format with black and isort
   make lint    # Check style with flake8 and mypy
   ```

5. **Run demos to verify**
   ```bash
   make demo-core
   # or
   python packages/sutra-core/examples/ai_reasoning_demo.py
   ```

6. **Commit your changes** (see [Commit Guidelines](#commit-guidelines))

7. **Push and create PR** (see [Pull Request Process](#pull-request-process))

### Common Make Commands

```bash
make help           # Show all available commands
make setup          # Initial environment setup
make test           # Run all tests
make test-core      # Run sutra-core tests only
make demo-core      # Run core functionality demo
make format         # Auto-format code (black + isort)
make lint           # Lint core package (flake8 + mypy)
make lint-all       # Lint all packages
make check          # Run format + lint + test
make build          # Build distribution packages
make clean          # Remove build artifacts
```

### Running Specific Tests

```bash
# Single test file
PYTHONPATH=packages/sutra-core python -m pytest packages/sutra-core/tests/test_basic.py -v

# Single test method
PYTHONPATH=packages/sutra-core python -m pytest packages/sutra-core/tests/test_basic.py::test_concept_creation -v

# Tests matching pattern
PYTHONPATH=packages/sutra-core python -m pytest packages/sutra-core/tests/ -k "test_text" -v

# With coverage report
PYTHONPATH=packages/sutra-core python -m pytest packages/sutra-core/tests/ -v --cov=sutra_core --cov-report=html
```

## Code Style and Standards

### Python Style Guidelines

We follow **PEP 8** with some project-specific conventions:

- **Line length**: 88 characters (black default)
- **Quotes**: Double quotes for strings (black default)
- **Imports**: Organized by isort (stdlib, third-party, local)
- **Type hints**: Required for all public functions
- **Docstrings**: Required for all public classes and functions (Google style)

### Code Quality Requirements

- **Zero linter errors** (flake8 and mypy must pass)
- **Test coverage**: Maintain 90%+ coverage for new code
- **Type hints**: All public APIs must have complete type annotations
- **Docstrings**: All public functions/classes must have docstrings

### Automated Formatting

```bash
# Format entire codebase
make format

# Format specific package
black packages/sutra-core/
isort packages/sutra-core/
```

### Linting

```bash
# Lint core package
make lint

# Manual linting
flake8 packages/sutra-core/sutra_core/
mypy packages/sutra-core/sutra_core/
```

### Import Organization

```python
# Standard library imports
import json
import logging
from typing import Dict, List, Optional

# Third-party imports
import numpy as np
from sklearn.feature_extraction.text import TfidfVectorizer

# Local imports
from sutra_core import Concept, Association
from sutra_core.learning import AdaptiveLearner
from sutra_core.utils import extract_words
```

## Testing Guidelines

### Test Organization

Tests are organized by functionality:

```
packages/sutra-core/tests/
├── test_basic.py           # Core concept/association tests
├── test_text_utils.py      # Text processing utilities
├── test_associations.py    # Association extraction
└── test_reasoning.py       # Advanced reasoning engine
```

### Writing Tests

```python
import pytest
from sutra_core import Concept, Association, ConceptError

class TestConcepts:
    """Test suite for Concept class."""
    
    def test_concept_creation(self):
        """Test basic concept creation."""
        concept = Concept(id="test", content="test content")
        assert concept.id == "test"
        assert concept.strength == 1.0
    
    def test_concept_strengthening(self):
        """Test concept strength increases on access."""
        concept = Concept(id="test", content="test")
        initial = concept.strength
        concept.access()
        assert concept.strength > initial
    
    def test_invalid_concept_raises_error(self):
        """Test that invalid concepts raise ConceptError."""
        with pytest.raises(ConceptError):
            Concept(id="", content="test")
```

### Test Requirements

- **Descriptive names**: `test_<what>_<condition>_<expected>`
- **Docstrings**: Brief description of what is tested
- **Isolation**: Each test should be independent
- **Coverage**: Test happy paths, edge cases, and error conditions
- **Assertions**: Use specific assertions (`assert x == 5`, not `assert x`)

### Running Tests

```bash
# All tests
make test

# Core package only
make test-core

# With verbose output
PYTHONPATH=packages/sutra-core python -m pytest packages/sutra-core/tests/ -v

# With coverage
PYTHONPATH=packages/sutra-core python -m pytest packages/sutra-core/tests/ --cov=sutra_core --cov-report=term-missing
```

## Documentation Standards

### Code Documentation

#### Docstring Style (Google Format)

```python
def calculate_confidence(self, path: List[Association]) -> float:
    """Calculate overall confidence for a reasoning path.
    
    Multiplies confidence scores of all associations in the path,
    applying a decay factor to account for path length.
    
    Args:
        path: List of associations forming the reasoning path.
        
    Returns:
        Float between 0.0 and 1.0 representing path confidence.
        
    Raises:
        ValueError: If path is empty or contains invalid associations.
        
    Example:
        >>> engine = ReasoningEngine()
        >>> path = [assoc1, assoc2, assoc3]
        >>> confidence = engine.calculate_confidence(path)
        >>> print(f"Path confidence: {confidence:.2f}")
    """
    if not path:
        raise ValueError("Cannot calculate confidence for empty path")
    
    confidence = 1.0
    for assoc in path:
        confidence *= assoc.confidence * 0.9  # Decay factor
    return min(1.0, confidence)
```

#### Type Hints

```python
from typing import Dict, List, Optional, Tuple, Union

def learn_adaptive(
    self,
    content: str,
    base_boost: float = 1.05,
    deep_extraction: bool = True
) -> Dict[str, List[Association]]:
    """Learn with adaptive reinforcement."""
    ...
```

### Project Documentation

When updating documentation files:

1. **ARCHITECTURE.md**: System design, components, data flow
2. **DESIGN.md**: Design decisions, patterns, trade-offs
3. **ALGORITHMS.md**: Core algorithms with pseudocode
4. **README.md**: Quick start, overview, basic usage
5. **WARP.md**: AI assistant guidance (internal)
6. **CHANGELOG.md**: Version history and changes

### Documentation Cross-References

Use relative links for cross-referencing:

```markdown
See [Architecture Overview](ARCHITECTURE.md#system-overview) for details.

Refer to the [Multi-Path Aggregation](ALGORITHMS.md#multi-path-plan-aggregation) algorithm.

Check [Design Principles](DESIGN.md#design-philosophy) for context.
```

## Commit Guidelines

### Commit Message Format

We follow the **Conventional Commits** specification:

```
<type>(<scope>): <subject>

<body>

<footer>
```

#### Types

- **feat**: New feature
- **fix**: Bug fix
- **docs**: Documentation changes
- **style**: Code style changes (formatting, missing semicolons, etc.)
- **refactor**: Code refactoring without feature changes
- **perf**: Performance improvements
- **test**: Adding or updating tests
- **chore**: Maintenance tasks (dependencies, build, etc.)

#### Scopes

- **core**: sutra-core package
- **hybrid**: sutra-hybrid package
- **api**: sutra-api package
- **cli**: sutra-cli package
- **build**: Build system or dependencies
- **docs**: Documentation files

#### Examples

```
feat(core): add multi-path consensus voting

Implement MPPA algorithm with clustering and robustness analysis.
Includes confidence boosting for consensus paths and fallback to
best single path when consensus is not reached.

Closes #42

---

fix(core): prevent infinite loops in bidirectional search

Add proper visited tracking for both forward and backward
search directions.

Fixes #58

---

docs(architecture): add data flow diagrams

Add Mermaid diagrams showing query processing and learning flows.

---

test(core): increase coverage for text utils

Add edge case tests for extract_words and clean_text.
Coverage increased from 85% to 96%.

---

chore(build): update pytest to 7.4.0

Update test framework and fix deprecated assertions.
```

### Commit Best Practices

- **Atomic commits**: One logical change per commit
- **Present tense**: "Add feature" not "Added feature"
- **Imperative mood**: "Fix bug" not "Fixes bug"
- **Limit subject to 72 characters**
- **Wrap body at 72 characters**
- **Reference issues**: Use `Closes #123` or `Fixes #456`

## Pull Request Process

### Before Creating a PR

1. **Run full test suite**
   ```bash
   make test-core  # or make test for all packages
   ```

2. **Check code quality**
   ```bash
   make check  # Runs format, lint, and test
   ```

3. **Update documentation** if adding features

4. **Add tests** for new functionality (maintain 90%+ coverage)

5. **Update CHANGELOG.md** with your changes

### Creating a Pull Request

1. **Push your branch**
   ```bash
   git push origin feature/your-feature-name
   ```

2. **Create PR with descriptive title**
   ```
   feat(core): Add bidirectional search for graph traversal
   ```

3. **Fill out PR template** (if provided)

4. **Link related issues**
   ```markdown
   Closes #42
   Related to #38
   ```

### PR Description Template

```markdown
## Description
Brief description of what this PR does and why.

## Changes
- List of key changes
- Another change
- Third change

## Testing
How has this been tested? What test cases were added?

## Documentation
What documentation was updated or added?

## Checklist
- [ ] Tests pass (`make test-core`)
- [ ] Code is formatted (`make format`)
- [ ] Linting passes (`make lint`)
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] Type hints added
- [ ] Docstrings added
```

### PR Review Process

1. **Automated checks** must pass (CI/CD if configured)
2. **Code review** by maintainer
3. **Address feedback** with additional commits
4. **Squash or rebase** if requested
5. **Merge** once approved

### After PR is Merged

1. **Delete feature branch**
   ```bash
   git branch -d feature/your-feature-name
   git push origin --delete feature/your-feature-name
   ```

2. **Update local main**
   ```bash
   git checkout main
   git pull origin main
   ```

## Release Process

### Versioning

We follow **Semantic Versioning** (SemVer):

```
MAJOR.MINOR.PATCH

1.2.3
│ │ └─ Patch: Bug fixes
│ └─── Minor: New features (backward compatible)
└───── Major: Breaking changes
```

### Release Checklist

1. **Update version numbers**
   - `packages/sutra-core/setup.py`
   - `packages/sutra-core/sutra_core/__init__.py`

2. **Update CHANGELOG.md**
   ```markdown
   ## [1.2.0] - 2025-10-15
   
   ### Added
   - Multi-path plan aggregation with consensus voting
   - Bidirectional search for graph traversal
   
   ### Fixed
   - Infinite loop bug in pathfinding
   
   ### Changed
   - Improved concept strengthening formula
   ```

3. **Run full test suite**
   ```bash
   make test
   ```

4. **Build distribution packages**
   ```bash
   make build
   ```

5. **Create git tag**
   ```bash
   git tag -a v1.2.0 -m "Release version 1.2.0"
   git push origin v1.2.0
   ```

6. **Create GitHub release** with changelog

7. **Publish to PyPI** (if configured)
   ```bash
   python -m twine upload dist/*
   ```

## Getting Help

### Resources

- **Documentation**: Start with [README.md](README.md)
- **Architecture**: See [ARCHITECTURE.md](ARCHITECTURE.md)
- **Design Decisions**: See [DESIGN.md](DESIGN.md)
- **Algorithms**: See [ALGORITHMS.md](ALGORITHMS.md)
- **Examples**: Check `packages/sutra-core/examples/`

### Common Issues

#### Import Errors
```bash
# Solution: Set PYTHONPATH
PYTHONPATH=packages/sutra-core python -m pytest packages/sutra-core/tests/
```

#### Virtual Environment Not Found
```bash
# Solution: Create manually
python3 -m venv venv
source venv/bin/activate
pip install -e packages/sutra-core/
pip install -r requirements-dev.txt
```

#### Tests Failing
```bash
# Solution: Check you're in the right directory and venv is activated
pwd  # Should be repository root
which python  # Should point to venv/bin/python
```

#### Linting Errors
```bash
# Solution: Run formatters first
make format
make lint
```

### Contact

- **Issues**: Open an issue on GitHub
- **Discussions**: Use GitHub Discussions for questions
- **Security**: Email security concerns privately (see SECURITY.md if available)

---

**Thank you for contributing to Sutra AI!** Your efforts help build a truly explainable, production-ready AI system that rivals traditional LLMs while running efficiently on CPU. 🚀
