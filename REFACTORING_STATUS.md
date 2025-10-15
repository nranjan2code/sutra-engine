# Sutra AI - Aggressive Refactoring Plan

**Status**: IN PROGRESS  
**Date**: October 15, 2025  
**No Backward Compatibility Required**

## Progress Tracker

### Phase 1: Type Safety & Validation ✅ IN PROGRESS
- [x] Created validation.py with comprehensive input validation
- [x] Fixed import statements (Any vs any)
- [x] Fixed query.py type annotations (Dict[str, Any])
- [x] Fixed engine.py **kwargs type annotations
- [x] Fixed indentation in explain_reasoning
- [ ] Fix remaining Dict annotations in engine.py
- [ ] Fix paths.py type issues
- [ ] Fix mppa.py type issues
- [ ] Add validation calls to all public APIs

### Phase 2: NLP Upgrade 🔄 READY
- [ ] Install spacy and en_core_web_sm model
- [ ] Create TextProcessor class with spacy
- [ ] Replace extract_words() with lemmatization
- [ ] Add entity extraction
- [ ] Update association extraction with dependency parsing
- [ ] Add negation detection

### Phase 3: Reasoning Optimization 📋 PLANNED
- [ ] Fix co-occurrence explosion
- [ ] Implement selective cache invalidation
- [ ] Fix bidirectional search bug
- [ ] Implement harmonic mean confidence propagation
- [ ] Add path quality thresholds

### Phase 4: Scalability 📋 PLANNED  
- [ ] Implement HNSW vector index
- [ ] Replace MPPA clustering with embeddings
- [ ] Add batch operations
- [ ] Optimize graph traversal

### Phase 5: Storage 📋 PLANNED
- [ ] Create SQLite storage backend
- [ ] Implement atomic transactions
- [ ] Add schema versioning
- [ ] Remove JSON serialization

### Phase 6: Testing 📋 PLANNED
- [ ] Unit tests for all modules
- [ ] Integration tests for reasoning
- [ ] Performance benchmarks
- [ ] Concurrent access tests

### Phase 7: Query Understanding 📋 PLANNED
- [ ] Semantic query classification
- [ ] Intent recognition with embeddings
- [ ] Query rewriting
- [ ] Multi-intent support

## Current Critical Paths

### Must Complete for Basic Functionality
1. ✅ Input validation framework
2. 🔄 Type safety (80% done)
3. ⏳ NLP upgrade (blocks association quality)
4. ⏳ Index consistency fixes (blocks reliability)

### Can Defer
- Vector search (current linear scan works for < 100K concepts)
- Storage layer (JSON works for prototype)
- Query understanding (basic patterns sufficient initially)

## Key Decisions

### Breaking Changes Approved
- ✅ No JSON backward compatibility
- ✅ No API versioning required
- ✅ Can change internal data structures freely
- ✅ Can replace dependencies without migration

### Technology Choices
- **NLP**: spacy (not NLTK) - better quality, faster
- **Vector DB**: hnswlib (not FAISS) - simpler, no GPU needed
- **Storage**: SQLite initially (Rust layer later)
- **Embeddings**: sentence-transformers all-MiniLM-L6-v2

## Next Actions (Priority Order)

1. **Complete type fixes** - 30 min
2. **Integrate validation** - 1 hour
3. **Install and setup spacy** - 30 min
4. **Rewrite text processing** - 2 hours
5. **Fix association extraction** - 3 hours
6. **Fix index consistency** - 1 hour
7. **Implement selective caching** - 2 hours

Total: ~10 hours for core fixes

## Files Modified So Far

```
packages/sutra-core/sutra_core/
├── validation.py (NEW ✅)
├── reasoning/
│   ├── query.py (MODIFIED ✅)
│   └── engine.py (MODIFIED 🔄)
```

## Files Pending

```
packages/sutra-core/sutra_core/
├── utils/
│   └── text.py (NEEDS COMPLETE REWRITE)
├── learning/
│   ├── associations.py (NEEDS MAJOR CHANGES)
│   └── adaptive.py (NEEDS MINOR CHANGES)
├── reasoning/
│   ├── paths.py (NEEDS TYPE FIXES)
│   ├── mppa.py (NEEDS TYPE FIXES)
│   └── planner.py (OK, add validation)
└── graph/
    └── concepts.py (OK, minor validation adds)
```

