# Setup

Prerequisites:
- Python 3.10+
- Rust toolchain (for storage)

Quickstart:
```
make setup
```

Manual:
```
python3 -m venv venv
source venv/bin/activate
pip install -e packages/sutra-core/
# Build Rust extension
cd packages/sutra-storage
maturin develop --release
```

Environment:
```
export SUTRA_STORAGE_PATH=./knowledge
export SUTRA_USE_SEMANTIC_EMBEDDINGS=true
```
