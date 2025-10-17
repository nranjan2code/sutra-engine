# Core (sutra-core)

Reasoning, learning, and query processing.

Highlights:
- QueryProcessor uses storage.vector_search for semantic retrieval
- PathFinder + MPPA for multi-path reasoning and consensus
- AssociationExtractor for structured links between concepts

Typical flow:
1) Learn via storage.learn_concept (embedding required)
2) Query via QueryProcessor.process_query (embedding processor required)

Key modules:
- reasoning/engine.py — high-level ReasoningEngine
- reasoning/query.py — QueryProcessor (semantic-first)
- reasoning/paths.py — path finding
- reasoning/mppa.py — consensus aggregation
- learning/associations*.py — association extraction
- storage/rust_adapter.py — Python adapter to Rust storage
