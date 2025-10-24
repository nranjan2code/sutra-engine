# Development Guide

**Complete guide for developing Sutra AI**

---

## Prerequisites

- Python 3.11+
- Rust 1.82+
- Node.js 18+
- Docker & Docker Compose

---

## Quick Setup

```bash
# Clone repository
git clone https://github.com/your-org/sutra-models.git
cd sutra-models

# Create virtual environment
python3 -m venv venv
source venv/bin/activate

# Install dependencies
pip install -r requirements-dev.txt

# Build Rust packages
cargo build --release

# Run tests
pytest tests/ -v
cargo test --release
```

---

## Project Structure

```
sutra-models/
├── packages/          # All packages (16 total)
├── docs/             # Documentation
├── tests/            # Integration tests
├── scripts/          # Build/deploy scripts
└── docker/           # Docker configs
```

---

## Development Workflow

1. **Make changes** to code
2. **Write tests** for new features
3. **Run tests** locally
4. **Build** packages
5. **Deploy** locally
6. **Validate** with smoke tests
7. **Commit** changes

---

## Testing

```bash
# Python tests
pytest tests/ -v

# Rust tests
cargo test --release

# Integration tests
./sutra-deploy.sh up
./scripts/smoke-test-embeddings.sh
```

---

## Building

```bash
# Rust packages
cargo build --release

# Python packages
pip install -e packages/sutra-core/

# Docker images
./build-all.sh
```

---

## License

MIT License

**Built with ❤️ by the Sutra AI Team**
