# 🌍 Wikipedia Real-World Performance Suite

A comprehensive performance testing suite using **real Wikipedia articles** instead of synthetic data.

## Overview

This test suite measures real-world performance of the Sutra AI system by:
1. **Learning from Wikipedia** - Ingests actual encyclopedia articles
2. **Question Answering** - Tests reasoning with real questions
3. **Knowledge Graph Quality** - Measures concept and association formation

## Quick Start

### 1. Setup

```bash
# Activate your virtual environment
source venv/bin/activate

# Run setup script
./scripts/setup_wikipedia_test.sh
```

### 2. Get a Hugging Face Token (Optional but Recommended)

To avoid rate limits when downloading datasets:

1. Go to https://huggingface.co/settings/tokens
2. Create a new token (read access is enough)
3. Set it in your environment:

```bash
export HF_TOKEN='hf_your_token_here'
```

### 3. Run Tests

```bash
# Quick test with 100 articles (~2 minutes)
python scripts/wikipedia_performance_suite.py 100

# Standard test with 1,000 articles (~20 minutes)
python scripts/wikipedia_performance_suite.py 1000

# Large test with 5,000 articles (~2 hours)
python scripts/wikipedia_performance_suite.py 5000
```

## What Gets Tested

### Test 1: Learning from Wikipedia
- Downloads real Wikipedia articles (Simple English or full English)
- Learns from each article (creates concepts, associations, embeddings)
- Measures:
  - Learning throughput (articles/sec)
  - Memory usage
  - Disk usage
  - Concept/association formation rate

### Test 2: Question Answering
- Tests reasoning with real questions like:
  - "What is photosynthesis?"
  - "Who was Albert Einstein?"
  - "What caused World War 2?"
- Measures:
  - Query response time
  - Answer confidence
  - Source concept usage

## Output

Results are saved to `performance_results/wikipedia_*.json` with:

```json
{
  "test_type": "wikipedia_real_world",
  "num_articles": 1000,
  "results": [
    {
      "operation": "wikipedia_learn",
      "throughput": 45.2,
      "total_concepts": 12500,
      "total_associations": 45000,
      "memory_mb": 850.5,
      "disk_mb": 125.3
    },
    {
      "operation": "wikipedia_qa",
      "sample_queries": [
        {
          "question": "What is photosynthesis?",
          "answer": "Photosynthesis is the process...",
          "confidence": 0.87,
          "time_ms": 45.2
        }
      ]
    }
  ]
}
```

## Comparison with Synthetic Tests

| Aspect | Synthetic Test | Wikipedia Test |
|--------|---------------|----------------|
| **Data** | "Performance test concept N" | Real encyclopedia articles |
| **Complexity** | Fixed, predictable | Variable, natural language |
| **Associations** | Limited patterns | Rich semantic relationships |
| **Real-world** | ❌ No | ✅ Yes |
| **Speed** | ⚡ Faster | 🐢 Slower (more processing) |
| **Use case** | System benchmarking | Real capability testing |

## Example Output

```
================================================================================
                🌍 WIKIPEDIA REAL-WORLD PERFORMANCE TEST 🌍                
================================================================================

System Configuration:
  ▸ Platform            : Darwin
  ▸ CPU Cores           : 8
  ▸ Total Memory        : 16.0 GB
  ▸ Test Scale          : 1,000 Wikipedia articles

────────────────────────────────────────────────────────────────────────────────
📚 TEST 1: LEARNING FROM 1,000 WIKIPEDIA ARTICLES
────────────────────────────────────────────────────────────────────────────────

📥 Downloading 1,000 Wikipedia articles (simple)...
✅ Loaded 987 articles
  ▸ Average article length  : 2,847 chars
🔧 Initializing ReasoningEngine...
✅ Engine ready!

📖 Learning from Wikipedia articles...
Progress |██████████████████████████████████████████████████| 100.0% (987/987)

💾 Saving knowledge base...
✅ Saved in 1.23s (245.8 MB)

┌──────────────────────────────────────────────────────────────────────────────┐
│                          WIKIPEDIA_LEARN                                     │
├──────────────────────────────────────────────────────────────────────────────┤
│ Articles processed        : 987                                              │
│ Total time               : 21.8s                                             │
│ Throughput               : 45.2 articles/sec                                 │
│ Total concepts           : 12,543                                            │
│ Total associations       : 45,231                                            │
│ Memory used              : 850.5 MB                                          │
│ Disk used                : 245.8 MB                                          │
│ Success rate             : 100.0% (0 errors)                                 │
└──────────────────────────────────────────────────────────────────────────────┘
```

## Advanced Usage

### Custom Questions

You can provide your own test questions:

```python
from scripts.wikipedia_performance_suite import WikipediaPerformanceTester

tester = WikipediaPerformanceTester()
tester.benchmark_wikipedia_learning(num_articles=500)

custom_questions = [
    "What is machine learning?",
    "How does the internet work?",
    "What are black holes?"
]

tester.benchmark_wikipedia_qa(
    storage_path="./wiki_knowledge",
    test_questions=custom_questions
)
```

### Different Wikipedia Versions

```python
# Use Simple English Wikipedia (default - cleaner, shorter)
articles = tester.load_wikipedia_articles(num_articles=1000, language="simple")

# Use full English Wikipedia (longer, more detailed)
articles = tester.load_wikipedia_articles(num_articles=1000, language="en")
```

## Troubleshooting

### Rate Limit Errors
```
Error: Rate limit exceeded
```
**Solution**: Set your HF_TOKEN environment variable (see setup above)

### Memory Errors
```
MemoryError: Unable to allocate...
```
**Solution**: Reduce number of articles or increase system memory

### Import Errors
```
ImportError: No module named 'datasets'
```
**Solution**: Run `pip install datasets`

## Performance Expectations

Based on typical hardware (8-core CPU, 16GB RAM):

| Articles | Learning Time | Memory | Concepts | Associations |
|----------|--------------|--------|----------|--------------|
| 100      | ~2 min       | ~100 MB | ~1,200   | ~4,500      |
| 1,000    | ~20 min      | ~850 MB | ~12,000  | ~45,000     |
| 5,000    | ~2 hours     | ~4 GB   | ~60,000  | ~225,000    |
| 10,000   | ~4 hours     | ~8 GB   | ~120,000 | ~450,000    |

## Next Steps

After running the Wikipedia test:

1. Compare results with synthetic tests
2. Test with your own domain-specific data
3. Tune performance based on results
4. Deploy with real knowledge base

## Contributing

To add new test scenarios:

1. Create a new method in `WikipediaPerformanceTester`
2. Add to `main()` function
3. Document expected behavior
4. Test with small dataset first

## License

MIT License - See main project LICENSE file
