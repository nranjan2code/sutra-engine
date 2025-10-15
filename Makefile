# Sutra AI Monorepo Makefile
.PHONY: help install install-dev test test-core test-hybrid test-api lint format clean build docs

# Default target
help:
	@echo "🚀 Sutra AI Monorepo - Available Commands"
	@echo "====================================================" 
	@echo "📦 SETUP & INSTALLATION:"
	@echo "  setup        - Set up development environment (creates venv)"
	@echo "  clean        - Clean build artifacts and caches"
	@echo ""
	@echo "🧪 TESTING & DEMOS:"
	@echo "  test-core    - Run sutra-core tests (60/60 passing ✓)"
	@echo "  demo-core    - Run basic core functionality demo"
	@echo "  demo-ai      - Run advanced AI reasoning demo (NEW)"
	@echo "  make demo-ai - Showcase sophisticated AI capabilities"
	@echo ""
	@echo "🔧 CODE QUALITY:"
	@echo "  format       - Format code (black, isort)"
	@echo "  lint         - Run linting (flake8, mypy)"
	@echo "  check        - Run full quality checks"
	@echo ""
	@echo "📚 PACKAGES:"
	@echo "  ✅ sutra-core    - Graph-based reasoning (IMPLEMENTED)"
	@echo "  🚧 sutra-hybrid  - Semantic embeddings (TODO)"
	@echo "  🚧 sutra-api     - REST API service (TODO)"
	@echo "  🚧 sutra-cli     - Command-line interface (TODO)"
	@echo ""
	@echo "💡 QUICK START:"
	@echo "  make setup && make demo-core"
	@echo "===================================================="

# Development setup
setup:
	@./scripts/setup-dev.sh
	@echo "✅ Development environment ready!"

install:
	@echo "⚠️  Please run 'make setup' first to create virtual environment"
	@echo "Or manually: source venv/bin/activate && pip install -e packages/sutra-core/"

install-dev: setup
	@echo "✅ Development environment ready via setup script"
	@echo "✅ Development dependencies installed"

# Testing (requires virtual environment: source venv/bin/activate)
test:
	@echo "⚠️  Make sure to activate virtual environment: source venv/bin/activate"
	python -m pytest packages/*/tests/ -v --cov-report=html --cov-report=term

test-core:
	@echo "⚠️  Make sure to activate virtual environment: source venv/bin/activate"
	PYTHONPATH=packages/sutra-core python -m pytest packages/sutra-core/tests/ -v

test-hybrid:
	python3 -m pytest packages/sutra-hybrid/tests/ -v  

test-api:
	python3 -m pytest packages/sutra-api/tests/ -v

# Code quality
lint:
	flake8 packages/sutra-core/sutra_core
	mypy packages/sutra-core/sutra_core

lint-all:
	flake8 packages/
	mypy packages/

format:
	black packages/
	isort packages/

check: lint
	black --check packages/
	isort --check packages/
	@echo "✅ Code formatting and linting checks passed"

# Build and packaging
build:
	cd packages/sutra-core && python -m build
	cd packages/sutra-hybrid && python -m build  
	cd packages/sutra-api && python -m build
	cd packages/sutra-cli && python -m build
	@echo "✅ All packages built"

clean:
	find . -type d -name "__pycache__" -exec rm -rf {} +
	find . -type d -name "*.egg-info" -exec rm -rf {} +
	find . -type d -name "build" -exec rm -rf {} +
	find . -type d -name "dist" -exec rm -rf {} +
	find . -name "*.pyc" -delete
	rm -rf htmlcov/
	rm -rf .coverage
	rm -rf .pytest_cache/
	rm -rf .mypy_cache/
	@echo "✅ Cleaned build artifacts"

# Documentation  
docs:
	@echo "📚 Generating documentation..."
	@echo "TODO: Add documentation generation"

# Docker (will be available when sutra-api package is implemented)
docker-build:
	@echo "⚠️  Docker support will be available when sutra-api package is implemented"
	@echo "For now, use: make demo-core"

docker-run:
	@echo "⚠️  Docker support will be available when sutra-api package is implemented"
	@echo "For now, use: make demo-core"

# Development helpers
demo-core:
	@echo "🚀 Running Sutra Core Demo (new structure)..."
	source venv/bin/activate && python packages/sutra-core/examples/basic_demo.py

demo-ai:
	@echo "🧠 Running Advanced AI Reasoning Demo..."
	source venv/bin/activate && python packages/sutra-core/examples/ai_reasoning_demo.py


demo-hybrid:
	cd packages/sutra-hybrid && python examples/hybrid_demo.py

demo-api:
	cd packages/sutra-api && python examples/client_example.py

# Workspace management
list-packages:
	@echo "📦 Workspace packages:"
	@echo "  - sutra-core    (Graph-based reasoning)"
	@echo "  - sutra-hybrid  (Hybrid AI with embeddings)"
	@echo "  - sutra-api     (REST API service)" 
	@echo "  - sutra-cli     (Command-line interface)"

deps-graph:
	@echo "🔗 Package dependencies:"
	@echo "  sutra-core     <- (base package)"
	@echo "  sutra-hybrid   <- sutra-core"
	@echo "  sutra-api      <- sutra-core, sutra-hybrid"
	@echo "  sutra-cli      <- sutra-core, sutra-hybrid"

# CI/CD helpers
ci-install: install-dev

ci-test: test lint

ci-build: clean build