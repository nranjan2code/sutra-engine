# Deployment Validation Checklist

**Version**: 1.0  
**Status**: ✅ **MANDATORY** for all deployments  
**Purpose**: Prevent reasoning path failures and ensure system functionality  

## 🎯 Overview

This checklist **MUST** be completed for every deployment to prevent critical system failures. Based on the 2025-10-20 technical debt resolution, these validations are now **MANDATORY**.

**Background**: A critical system failure caused reasoning paths to break, leading to wrong answers and non-functional semantic search. This checklist prevents regression.

## 📋 Pre-Deployment Checklist

### 1. ✅ Configuration Validation

#### Environment Variables
```bash
# Verify critical environment variables
echo "SUTRA_EMBEDDING_MODEL: $SUTRA_EMBEDDING_MODEL"
echo "SUTRA_VECTOR_DIMENSION: $SUTRA_VECTOR_DIMENSION"  
echo "SUTRA_USE_SEMANTIC_EMBEDDINGS: $SUTRA_USE_SEMANTIC_EMBEDDINGS"

# Required values:
# SUTRA_EMBEDDING_MODEL=nomic-embed-text
# SUTRA_VECTOR_DIMENSION=768
# SUTRA_USE_SEMANTIC_EMBEDDINGS=true
```

#### Docker Compose Configuration
```bash
# Verify docker-compose-grid.yml contains:
grep -A5 "SUTRA_EMBEDDING_MODEL" docker-compose-grid.yml
grep -A5 "VECTOR_DIMENSION" docker-compose-grid.yml

# Both storage-server AND sutra-hybrid must have:
# - SUTRA_EMBEDDING_MODEL=nomic-embed-text
# - VECTOR_DIMENSION=768 (storage-server)
# - SUTRA_VECTOR_DIMENSION=768 (sutra-hybrid)
```

### 2. ✅ Code Validation

#### Initialization Order (CRITICAL)
```bash
# Verify hybrid service uses correct initialization order
grep -A20 "def __init__" packages/sutra-hybrid/sutra_hybrid/engine.py

# Must contain:
# 1. OllamaNLPProcessor created BEFORE ReasoningEngine
# 2. QueryProcessor recreated AFTER ReasoningEngine
# 3. "Recreated QueryProcessor with OllamaNLPProcessor" log message
```

#### No Forbidden Patterns
```bash
# Check for forbidden fallback patterns
grep -r "FALLBACK to spaCy" packages/sutra-core/ && echo "❌ FORBIDDEN FALLBACK FOUND"
grep -r "sentence-transformers" packages/sutra-hybrid/ && echo "❌ FORBIDDEN MODEL FOUND"

# Should return no matches or exit with error
```

## 🚀 Deployment Process

### 3. ✅ Service Startup

#### Clean Deployment
```bash
# 1. Stop all services
./sutra-deploy.sh down

# 2. Clean old data (prevents embedding corruption)
docker volume rm sutra-models_storage-data

# 3. Build fresh images
docker-compose -f docker-compose-grid.yml build --no-cache

# 4. Start services
./sutra-deploy.sh up

# 5. Wait for health checks
sleep 30
```

#### Startup Log Validation
```bash
# Check for successful initialization messages
docker logs sutra-hybrid | grep "✅ PRODUCTION: Created Ollama NLP processor"
docker logs sutra-hybrid | grep "Recreated QueryProcessor with OllamaNLPProcessor"
docker logs sutra-storage | grep "Vector dimension: 768"

# Should see all three messages
```

### 4. ✅ System Health Validation

#### Service Status
```bash
# All services must be healthy
./sutra-deploy.sh status | grep -c "healthy"
# Should return 9 (healthy services count)
```

#### Embedding Model Verification
```bash
# Verify nomic-embed-text is loaded
curl -s http://localhost:11434/api/tags | jq '.models[].name' | grep "nomic-embed-text"
# Should return: "nomic-embed-text:latest"

# Check embedding dimension
docker logs sutra-hybrid | grep "768-d"
# Should see: "nomic-embed-text (768-d)"
```

## 🧪 Functional Testing

### 5. ✅ Learning Pipeline Test

#### Learn Test Concepts
```bash
# Learn via hybrid service (with embeddings)
curl -X POST http://localhost:8001/sutra/learn \
  -H "Content-Type: application/json" \
  -d '{"text":"Mount Everest is the tallest mountain in the world at 29,029 feet above sea level."}'

# Should return: {"success": true, "concepts_learned": 1}

curl -X POST http://localhost:8001/sutra/learn \
  -H "Content-Type: application/json" \
  -d '{"text":"The Eiffel Tower is located in Paris, France and stands 324 meters tall."}'

# Should return: {"success": true, "concepts_learned": 1}
```

#### Verify Embeddings Generated
```bash
# Check storage statistics
curl -s http://localhost:8001/sutra/stats | jq '.total_concepts'
# Should return: 2

# CRITICAL: Check that embeddings were generated
# This was the root cause of the original failure
curl -s http://localhost:8000/stats | jq '.total_embeddings'
# Should return: 2 (same as total_concepts)

# If total_embeddings = 0, deployment has FAILED
```

### 6. ✅ Reasoning Pipeline Test (CRITICAL)

#### Test Semantic Search
```bash
# Test query processing with reasoning validation
RESPONSE=$(curl -s -X POST http://localhost:8001/sutra/query \
  -H "Content-Type: application/json" \
  -d '{"query": "What is the tallest mountain?", "include_reasoning": true}')

echo $RESPONSE | jq '.'
```

#### Validate Response Structure
```bash
# CRITICAL VALIDATIONS:

# 1. Correct answer (not wrong/random answer)
echo $RESPONSE | jq -r '.answer' | grep -i "everest"
# Should contain "Everest" or "Mount Everest"

# 2. Semantic support exists (embeddings working)
SEMANTIC_COUNT=$(echo $RESPONSE | jq '.semantic_support | length')
echo "Semantic support count: $SEMANTIC_COUNT"
# Should be > 0

# 3. Similarity scores present (real embeddings)
echo $RESPONSE | jq '.semantic_support[0].similarity'
# Should return a number between 0.0 and 1.0

# 4. No error messages
echo $RESPONSE | jq '.detail' | grep -i "error" && echo "❌ ERROR FOUND"
# Should not return error messages
```

#### Test Different Query Types
```bash
# Test multiple query types to ensure consistency
QUERIES=(
  "What is the tallest mountain?"
  "Where is the Eiffel Tower?"
  "How tall is Mount Everest?"
)

for query in "${QUERIES[@]}"; do
  echo "Testing: $query"
  RESULT=$(curl -s -X POST http://localhost:8001/sutra/query \
    -H "Content-Type: application/json" \
    -d "{\"query\": \"$query\"}")
  
  # Check for semantic support
  SUPPORT_COUNT=$(echo $RESULT | jq '.semantic_support | length')
  echo "  Semantic support: $SUPPORT_COUNT"
  
  # Should be > 0 for all queries
  if [ "$SUPPORT_COUNT" -eq 0 ]; then
    echo "  ❌ FAILED: No semantic support"
    exit 1
  else
    echo "  ✅ PASSED"
  fi
done
```

### 7. ✅ Error Condition Testing

#### No Embedding Processor Error
```bash
# This error should NEVER occur in a properly deployed system
RESPONSE=$(curl -s -X POST http://localhost:8001/sutra/query \
  -H "Content-Type: application/json" \
  -d '{"query": "test"}')

echo $RESPONSE | grep -i "no embedding processor" && {
  echo "❌ CRITICAL FAILURE: Embedding processor not available"
  echo "This indicates initialization order problem"
  exit 1
}
echo "✅ No embedding processor errors found"
```

#### Fallback Detection
```bash
# Check logs for forbidden fallback messages
docker logs sutra-hybrid 2>&1 | grep -i "fallback" && {
  echo "❌ CRITICAL: Forbidden fallback detected"
  exit 1
}
echo "✅ No forbidden fallbacks detected"
```

## 📊 Validation Results Template

```
# DEPLOYMENT VALIDATION RESULTS
Date: $(date)
Environment: [production/staging/development]

## Configuration ✅/❌
- [ ] Environment variables correct
- [ ] Docker compose configuration valid  
- [ ] Initialization order correct
- [ ] No forbidden patterns found

## Services ✅/❌
- [ ] All services healthy
- [ ] Embedding model loaded
- [ ] Correct log messages present
- [ ] No error messages in logs

## Functional Tests ✅/❌
- [ ] Learning pipeline working
- [ ] Embeddings generated (count matches concepts)
- [ ] Semantic search working
- [ ] Correct answers returned
- [ ] No embedding processor errors
- [ ] No forbidden fallbacks

## Summary
Status: [PASS/FAIL]
Issues: [List any issues found]
Action Required: [Next steps if failures]
```

## 🚨 Failure Response

### If Any Check Fails:
1. **DO NOT PROCEED** with deployment
2. Review `docs/TECHNICAL_DEBT_REASONING_PATHS_RESOLVED.md`
3. Check `docs/COMPONENT_INITIALIZATION_GUIDELINES.md`
4. Fix issues and re-run validation
5. Contact system architect if unclear

### Common Failure Scenarios:

#### "No embedding processor available"
- **Cause**: Initialization order problem
- **Fix**: Recreate QueryProcessor with correct NLP processor
- **Reference**: Component Initialization Guidelines

#### "total_embeddings: 0"
- **Cause**: Learning without embedding generation
- **Fix**: Ensure unified learning pipeline used
- **Reference**: Production Requirements

#### Wrong/Random Answers
- **Cause**: Semantic space corruption from mixed models
- **Fix**: Enforce single nomic-embed-text model
- **Reference**: Embedding Troubleshooting

## 📚 References

- [Technical Debt Resolution](./TECHNICAL_DEBT_REASONING_PATHS_RESOLVED.md)
- [Component Initialization Guidelines](./COMPONENT_INITIALIZATION_GUIDELINES.md)
- [Production Requirements](../PRODUCTION_REQUIREMENTS.md)
- [Embedding Troubleshooting](./EMBEDDING_TROUBLESHOOTING.md)

---

**Compliance**: **MANDATORY** for all deployments  
**Frequency**: Every deployment, upgrade, or configuration change  
**Owner**: Deployment engineer + System architect review