# Mass Learning Adapters

This module provides adapters for ingesting large amounts of knowledge from various data sources into Sutra AI's graph-based reasoning system.

## Overview

Unlike traditional RAG systems that chunk text for embedding similarity, Sutra's mass learning adapters:
- **Extract meaningful concepts and associations** from structured text
- **Leverage parallel processing** for efficient association extraction
- **Preserve context and structure** for better reasoning
- **Build a searchable knowledge graph** rather than just storing embeddings

## Quick Start

```python
from sutra_core import ReasoningEngine
from sutra_core.adapters import DatasetAdapter

# Initialize reasoning engine (includes storage and learner)
engine = ReasoningEngine()

# Create dataset adapter for Wikipedia-style files
adapter = DatasetAdapter(
    batch_size=20,  # Process 20 articles in parallel
    min_article_length=100,  # Skip very short articles
    max_article_length=5000  # Split very long articles
)

# Learn from first 50KB of Wikipedia dataset
with open("dataset/wikipedia.txt", 'r', encoding='utf-8') as f:
    content = f.read(50000)

articles = content.split('\n\n\n')
learned_titles = []

for article_text in articles[:20]:  # Learn 20 articles
    if len(article_text.strip()) > 100:
        engine.learn(
            content=article_text,
            source="Wikipedia Demo",
            category="encyclopedia"
        )
        title = article_text.split('\n')[0]
        learned_titles.append(title)

print(f"Learned {len(learned_titles)} articles: {learned_titles[:5]}")

# Query learned knowledge
result = engine.ask("What is April?")
print(f"Answer: {result.primary_answer[:100]}...")
print(f"Confidence: {result.confidence}")
```

## FileAdapter Features

### Intelligent Text Processing
- **Wikipedia format**: Handles articles with sections and subsections
- **Plain text**: Processes by paragraphs and sentences  
- **Auto-detection**: Automatically detects file format
- **Context preservation**: Maintains surrounding context for better understanding

### Parallel Processing
- Automatically uses `ParallelAssociationExtractor` for large files (20+ segments)
- Leverages all CPU cores for association extraction
- 3-4x speedup on multi-core systems
- Graceful fallback to sequential processing

### Memory Efficiency
- Streams large files instead of loading entirely into memory
- Processes content in configurable batches
- Progress tracking with real-time callbacks

## Configuration Options

### FileAdapter Parameters
- `batch_size`: Number of segments to process in parallel (default: 50)
- `chunk_size`: Target characters per segment (default: 1500)  
- `min_segment_length`: Minimum segment size (default: 100)
- `max_segment_length`: Maximum before splitting (default: 3000)
- `progress_callback`: Function to track progress

### File Format Support
- `'auto'`: Auto-detect based on filename and content
- `'wikipedia'`: Wikipedia-style with === headers and --- sections
- `'plain'`: Regular text split by paragraphs
- `'markdown'`: Markdown format (planned)

### Content Categories
Auto-inferred or manually specified:
- `'science'`, `'history'`, `'technology'`, `'culture'`, `'general'`

## Advanced Usage

### Custom Progress Tracking
```python
def detailed_progress(progress):
    print(f"Processed: {progress.chunks_processed}/{progress.total_chunks}")
    print(f"Concepts: {progress.concepts_created}")
    print(f"Associations: {progress.associations_created}")
    print(f"Rate: {progress.bytes_per_second/1000:.1f} KB/s")
    print(f"Errors: {len(progress.errors)}")

adapter = FileAdapter(progress_callback=detailed_progress)
```

### Large File Processing
```python
# For very large files (100MB+), increase batch size
adapter = FileAdapter(
    batch_size=100,  # Larger batches for better parallelization
    chunk_size=2000,  # Larger segments
    max_segment_length=5000  # Allow longer segments
)
```

### Multiple File Processing
```python
files = ["wiki_science.txt", "wiki_history.txt", "wiki_tech.txt"]
categories = ["science", "history", "technology"]

for file_path, category in zip(files, categories):
    progress = adapter.learn_from_source(
        learner=learner,
        source=file_path,
        category=category,
        file_format='wikipedia'
    )
    print(f"{file_path}: {progress.concepts_created} concepts")
```

## Performance Characteristics

Based on testing with the existing parallel extraction system:

- **Small files** (<1MB): ~1-5 seconds
- **Medium files** (1-10MB): ~10-60 seconds with parallel processing
- **Large files** (10MB+): Scales linearly with parallel processing
- **Memory usage**: ~0.1KB per concept created
- **Throughput**: Typically 50-200 KB/s depending on text complexity

## Integration with Sutra Architecture

The adapters work seamlessly with existing Sutra components:

1. **AdaptiveLearner**: Uses adaptive reinforcement based on concept difficulty
2. **AssociationExtractor**: Leverages pattern-based and co-occurrence extraction
3. **ParallelAssociationExtractor**: Automatically used for better performance
4. **ConcurrentStorage**: All concepts and associations stored in lock-free Rust backend (57K writes/sec)
5. **ReasoningEngine**: Can immediately query learned knowledge via vector index

## Extending with New Adapters

To create adapters for other data sources:

```python
from sutra_core.adapters.base import MassLearningAdapter

class DatabaseAdapter(MassLearningAdapter):
    def get_chunks(self, source: str, **kwargs):
        # Connect to database and yield content chunks
        pass
        
    def estimate_total_chunks(self, source: str, **kwargs) -> int:
        # Return estimated number of records
        pass
        
    def get_source_info(self, source: str, **kwargs) -> dict:
        # Return database connection info
        pass
```

## Planned Adapters

- **DatabaseAdapter**: SQL/NoSQL database ingestion
- **StreamAdapter**: Real-time data streams (Kafka, RabbitMQ)
- **DocumentAdapter**: PDF, DOCX, and other document formats
- **APIAdapter**: Web APIs and RSS feeds
- **ArchiveAdapter**: ZIP, TAR archives with multiple files