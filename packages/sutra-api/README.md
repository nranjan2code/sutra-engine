# Sutra API - REST Service

FastAPI service for the Sutra AI system. Provides endpoints for learning, reasoning, semantic search, and system stats.

## Endpoints

- GET /health
  - Service health, version, uptime, loaded concepts
- POST /learn
  - Learn a single knowledge item
- POST /learn/batch
  - Learn multiple items efficiently
- POST /reason
  - Perform reasoning using the core ReasoningEngine (multi-path + consensus)
  - Request body: { query: str, max_steps?: int, num_paths?: int, threshold?: float }
- POST /semantic-search
  - Find conceptually similar items using embeddings (semantic or TF-IDF)
- GET /concepts/{concept_id}
  - Details for a specific concept; created_at is ISO-8601
- GET /stats
  - Hybrid-level stats (concepts, associations, embeddings)
- GET /reasoner/stats
  - ReasoningEngine-level stats (cache stats, learning stats, association stats)
- POST /save
  - Persist all data to disk
- POST /load
  - Reload data from disk
- DELETE /reset
  - Reset the in-memory system (clears knowledge)

## Configuration (env vars)

- SUTRA_CACHE_TTL_SECONDS: float|null
  - TTL for ReasoningEngine query cache; set to e.g., 300 for 5 minutes; omit for no TTL
- SUTRA_COMPOSITIONAL_LINKS: true|false (default true)
  - Enable links from central learned concept to extracted phrases
- SUTRA_COMPOSITIONAL_CONFIDENCE: float (default 0.6)
  - Confidence applied to the central links
- SUTRA_COMPOSITIONAL_TYPE: compositional|hierarchical|semantic|causal|temporal (default compositional)
  - Association type used for central links
- Other settings (from config.py):
  - SUTRA_USE_SEMANTIC_EMBEDDINGS=true|false
  - SUTRA_STORAGE_PATH=./api_knowledge
  - SUTRA_LOG_LEVEL=INFO, etc.

## Quick Start

```bash
# Install API package (dev mode)
pip install -e packages/sutra-api/

# Run
python -m uvicorn sutra_api.main:app --host 0.0.0.0 --port 8000 --reload

# Or via __main__
python packages/sutra-api/sutra_api/main.py
```

## Notes

- /reason now uses the full reasoning pipeline (QueryProcessor + PathFinder + MPPA).
- Reasoning paths returned by /reason are mapped from the engine’s supporting paths with ordered concept IDs and a human-readable explanation.
- Hybrid storage and core engine persistence accept both legacy "|" and current ":" association key separators.
- TF-IDF vectorizer state is persisted as tfidf_vectorizer.pkl and restored on load when available.
