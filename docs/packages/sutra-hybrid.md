# Hybrid (sutra-hybrid)

Optional embedding orchestration and strategy comparison.

Highlights:
- Batch embedding via SentenceTransformers (EmbeddingGemma, MiniLM, etc.)
- Prompted retrieval embeddings (e.g., Retrieval-query)
- Agreement scoring between graph-only and semantic-enhanced modes

Usage:
- Use when you want accelerated learning/queries via batched GPU/MPS
- QueryProcessor will prefer embedding_processor if provided
