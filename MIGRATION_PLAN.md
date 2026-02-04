# Sutra Engine Separation Plan

## Objective
Decouple the core infrastructure (**Sutra Engine**) from the product application (**Sutra Memory**) to create a focused, multi-repo architecture.

## Strategy: Fork & Prune
We will use a preservation strategy that creates two distinct histories from the current codebase.

### Repository 1: Sutra Engine (New)
*Contains the "Hard Tech": Model, Storage, Protocol, Infrastructure.*

**Locations to Extract:**
- `packages/sutraworks-model` → `crates/model`
- `packages/sutra-storage` → `crates/storage`
- `packages/sutra-protocol` → `crates/protocol`
- `packages/sutra-grid-master` → `crates/grid-master`
- `packages/sutra-grid-agent` → `crates/grid-agent`
- `packages/sutra-grid-events` → `crates/grid-events`
- `packages/sutra-bulk-ingester` → `crates/bulk-ingester`
- `packages/sutra-embedder` → `crates/embedder`
- `packages/sutra-core` (Python) → `python/core`
- `packages/sutra-hybrid` (Python) → `python/hybrid`

**Process:**
1. Create branch `release/engine-v1`.
2. Remove all product-specific code (Client, UI, API).
3. Move engine packages to a clean root structure (`crates/`).
4. This branch is then pushed to the new `sutra-engine` repository.

### Repository 2: Sutra Memory (Current)
*Contains the Product: Studio, API, Client, Agents.*

**Content to Keep:**
- `packages/sutra-client`
- `packages/sutra-ui-framework`
- `packages/sutra-api`
- `packages/sutra-control`
- `packages/sutra-explorer`
- `desktop`
- `docs` (Product docs)

**Process:**
1. In `main` branch, remove the extracted engine packages.
2. Update dependencies to point to the new Engine repo (or mocked for now).
3. Update build scripts to pull the engine binaries or crates.

## Execution Steps

I have prepared a script `scripts/prepare_engine_release.sh` to automate the creation of the Engine branch.

### Step 1: Prepare Engine Branch
1. Run the script to create `release/engine-v1`.
2. This script refactors the directory structure *only on that branch*.
3. You will verify the structure.

### Step 2: Push to New Repo
You will need to create a new empty repository on GitHub/GitLab (e.g., `sutra-engine`) and run:
```bash
git remote add engine <NEW_REPO_URL>
git push engine release/engine-v1:main
```

### Step 3: Cleanup Product Repo
Once the engine is safe, we will clean the current repository's `main` branch.
