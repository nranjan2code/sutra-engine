#!/bin/bash
set -e

# Configuration
BRANCH_NAME="release/engine-v1"

echo "🚀 Starting Sutra Engine Separation Prep..."

# 1. Verify Git State
if [[ -n $(git status -s) ]]; then
    echo "❌ Error: Working directory not clean. Please commit changes first."
    exit 1
fi

# 2. Create Branch
echo "🌿 Creating branch $BRANCH_NAME..."
if git show-ref --verify --quiet refs/heads/$BRANCH_NAME; then
    echo "Branch exists, switching..."
    git checkout $BRANCH_NAME
else
    git checkout -b $BRANCH_NAME
fi

# 3. Create New Structure
echo "📁 Creating new directory structure..."
mkdir -p crates python infra

# 4. Move Engine Components (Preserving History via git mv)
echo "📦 Moving Engine components..."

# Rust Crates
test -d packages/sutraworks-model && git mv packages/sutraworks-model crates/model
test -d packages/sutra-storage && git mv packages/sutra-storage crates/storage
test -d packages/sutra-storage-client-tcp && git mv packages/sutra-storage-client-tcp crates/storage-client
test -d packages/sutra-protocol && git mv packages/sutra-protocol crates/protocol
test -d packages/sutra-grid-master && git mv packages/sutra-grid-master crates/grid-master
test -d packages/sutra-grid-agent && git mv packages/sutra-grid-agent crates/grid-agent
test -d packages/sutra-grid-events && git mv packages/sutra-grid-events crates/grid-events
test -d packages/sutra-bulk-ingester && git mv packages/sutra-bulk-ingester crates/bulk-ingester
test -d packages/sutra-embedder && git mv packages/sutra-embedder crates/embedder

# Python Packages
test -d packages/sutra-core && git mv packages/sutra-core python/core
test -d packages/sutra-hybrid && git mv packages/sutra-hybrid python/hybrid

# Infrastructure
test -d haproxy && git mv haproxy infra/haproxy
test -d k8s && git mv k8s infra/k8s

# 5. Remove Product Components
echo "🔥 Removing Product components..."
git rm -r --ignore-unmatch \
    packages/sutra-client \
    packages/sutra-ui-framework \
    packages/sutra-api \
    packages/sutra-control \
    packages/sutra-explorer \
    desktop \
    e2e \
    playwright.config.ts \
    packages/sutra-storage/engine_test_v2 # Likely test data, maybe keep? removing for clean engine.

# 6. Clean Root Configs
# We need to rewrite Cargo.toml to point to crates/ instead of packages/
echo "⚙️ Updating Cargo.toml..."
cat > Cargo.toml <<EOF
[workspace]
resolver = "2"
members = [
    "crates/storage",
    "crates/protocol",
    "crates/grid-master",
    "crates/grid-agent",
    "crates/grid-events",
    "crates/bulk-ingester",
    "crates/embedder",
    "crates/model",
    "crates/model/crates/*",
]

[workspace.package]
version = "1.0.0"
edition = "2021"
authors = ["SutraWorks"]
license = "MIT"
repository = "https://github.com/sutraworks/sutra-engine"

[workspace.dependencies]
tokio = { version = "1.40", features = ["rt-multi-thread", "macros", "signal", "net", "io-util"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
bincode = "1.3"
thiserror = "1.0"
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
ndarray = "0.16"
ndarray-linalg = "0.16"
rayon = "1.10"
uuid = { version = "1.0", features = ["v4", "serde"] }

# Internal mapped dependencies
sutra-core = { path = "crates/model/crates/sutra-core" }
sutra-protocol = { path = "crates/protocol" }
EOF

echo "✅ Engine branch prepared successfully!"
echo "Next Steps:"
echo "1. Run: git commit -m 'Refactor: Prepare generic sutra-engine repository'"
echo "2. Add your new remote: git remote add engine <URL>"
echo "3. Push: git push engine $BRANCH_NAME:main"
