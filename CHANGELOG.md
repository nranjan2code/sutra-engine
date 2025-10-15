# Changelog

All notable changes to this project will be documented in this file.

## 2025-10 (October 2025)

### Core (sutra-core)
- Association model
  - Added `last_used` timestamp; persisted in save/load
  - `strengthen()` now increases both `weight` and `confidence` (capped at 1.0) and updates `last_used`
- Traversal and indexing
  - PathFinder refreshes `association.last_used` on edge expansion (best-first, breadth-first, bidirectional)
  - Post-load neighbor indexing rebuilt symmetrically to match runtime indexing (fixes bidirectional search)
- Query processing and thresholds
  - Context expansion now uses `confidence >= 0.6` for links (aligns with central link default)
  - Final consensus confidence is clamped to `[0, 1]` after complexity adjustment
  - Centralized path selection via `PathFinder.select_diverse_paths(...)`
- Learning and extraction
  - Co-occurrence extraction hard-capped (`max_cooccurrence_links=200`) to limit graph growth
  - Concept/phrase IDs extended from 12 to 16 hex chars (MD5)
- Logging
  - Reduced per-query logs to DEBUG in ReasoningEngine and QueryProcessor
- Maintenance APIs (new)
  - `ReasoningEngine.get_health_snapshot()` returns compact runtime stats
  - `ReasoningEngine.decay_and_prune(...)` decays inactive concepts and prunes stale/low-confidence associations

### Linting and tooling
- Added repo `.flake8` aligned with Black (max-line-length=88, ignore E203/W503)
- `make lint` now targets `packages/sutra-core/sutra_core` (core only)
- Added `make lint-all` to lint the entire repo

### Documentation updates
- Updated WARP.md to reflect:
  - Maintenance APIs, symmetric neighbor indexing, confidence clamp, co-occurrence cap, stronger IDs
  - Lint policy and commands (`make lint`, `make lint-all`)
- API_REFERENCE.md:
  - Documented `get_health_snapshot()` and `decay_and_prune(...)`
  - Added `Association.last_used`; noted confidence clamp and context threshold
  - Noted traversal updates `last_used`
- docs/packages/sutra-core.md:
  - Added features for reasoning engine and maintenance APIs; updated notes
- docs/development/setup.md:
  - Documented lint policy and `make lint-all`
- docs/quickstart.md:
  - Added “Core Maintenance (ReasoningEngine)” examples
- Root README:
  - Added maintenance snippet and `make lint-all`

### Tests
- Core test suite remains 60 tests; passes after changes

---
