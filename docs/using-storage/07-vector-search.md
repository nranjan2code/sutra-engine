# Vector Search

Sutra Storage provides high-performance vector search capabilities using HNSW (Hierarchical Navigable Small World) indexing with persistent memory mapping. This guide covers semantic search, similarity queries, and optimization strategies.

## Architecture Overview

### Vector Search Pipeline

```
┌─────────────────────────────────────────────────────────────────┐
│                    Vector Search Architecture                    │
├─────────────────────────────────────────────────────────────────┤
│  1. Query Processing                                            │
│     ├─ Text → Embedding (via embedding service)                │
│     ├─ Query vector normalization                              │
│     └─ Search parameter validation                             │
├─────────────────────────────────────────────────────────────────┤
│  2. HNSW Index Search                                           │
│     ├─ Persistent memory-mapped index                          │
│     ├─ Multi-level graph traversal                             │
│     ├─ Approximate nearest neighbor                            │
│     └─ Confidence scoring                                      │
├─────────────────────────────────────────────────────────────────┤
│  3. Results Processing                                          │
│     ├─ Tenant filtering (if organization_id provided)          │
│     ├─ Metadata enrichment                                     │
│     ├─ Similarity scoring                                      │
│     └─ Result ranking and limiting                             │
└─────────────────────────────────────────────────────────────────┘
```

### Performance Characteristics

- **Index Type**: HNSW with persistent mmap storage
- **Startup Time**: 3.5ms for 1M vectors (94x faster than rebuilding)
- **Search Latency**: <0.01ms for similarity queries
- **Memory Usage**: Efficient mmap with OS page management
- **Accuracy**: Configurable recall/speed tradeoffs

## Basic Vector Search

### Simple Semantic Search

```python
from sutra_storage_client import StorageClient

client = StorageClient()

# Basic vector search - searches all concepts
results = client.vector_search(
    query_text="artificial intelligence machine learning",
    k=10  # Return top 10 most similar concepts
)

print(f"Found {len(results)} similar concepts:")
for i, result in enumerate(results, 1):
    print(f"{i}. [{result['similarity']:.3f}] {result['content'][:100]}...")
```

### Tenant-Aware Vector Search

```python
# Search within specific organization only
hospital_results = client.vector_search(
    query_text="patient cardiac symptoms chest pain",
    k=15,
    organization_id="hospital-st-marys"  # Tenant filter
)

# Search shared domain knowledge (no organization filter)
domain_results = client.vector_search(
    query_text="cardiac protocols emergency procedures", 
    k=10
    # No organization_id = searches domain storage
)

print("Hospital-specific results:")
for result in hospital_results[:3]:
    org_id = result.get('organization_id', 'domain')
    print(f"- [{org_id}] {result['content'][:80]}...")

print("\nDomain knowledge results:")
for result in domain_results[:3]:
    print(f"- [shared] {result['content'][:80]}...")
```

## Advanced Search Patterns

### Multi-Modal Search Strategy

```python
class SemanticSearchEngine:
    def __init__(self, storage_client: StorageClient):
        self.client = storage_client
        
    def hybrid_search(self, query: str, organization_id: str = None, 
                     search_depth: int = 3) -> Dict:
        """
        Multi-level search combining different strategies.
        
        Args:
            query: Natural language search query
            organization_id: Tenant filter (optional)
            search_depth: Search intensity (1=fast, 3=thorough)
        """
        
        results = {
            'primary_results': [],
            'related_concepts': [],
            'domain_knowledge': [],
            'total_concepts_searched': 0,
            'search_strategy': f'depth_{search_depth}'
        }
        
        # Step 1: Primary vector search
        primary_k = 10 * search_depth
        primary_results = self.client.vector_search(
            query_text=query,
            k=primary_k,
            organization_id=organization_id
        )
        
        results['primary_results'] = primary_results
        results['total_concepts_searched'] += len(primary_results)
        
        # Step 2: If tenant search, also get domain knowledge
        if organization_id:
            domain_results = self.client.vector_search(
                query_text=query,
                k=primary_k // 2
                # No organization_id for domain search
            )
            
            results['domain_knowledge'] = domain_results
            results['total_concepts_searched'] += len(domain_results)
        
        # Step 3: Association-based expansion (if depth >= 2)
        if search_depth >= 2 and primary_results:
            related_concepts = self._find_related_concepts(
                primary_results[:5],  # Use top 5 for association expansion
                organization_id
            )
            
            results['related_concepts'] = related_concepts
            results['total_concepts_searched'] += len(related_concepts)
        
        return results
    
    def _find_related_concepts(self, primary_results: List[Dict], 
                              organization_id: str = None) -> List[Dict]:
        """Find concepts related through associations."""
        
        related_concepts = []
        processed_ids = set()
        
        for result in primary_results:
            concept_id = result['concept_id']
            
            if concept_id in processed_ids:
                continue
                
            processed_ids.add(concept_id)
            
            # Get associations for this concept
            try:
                associations = self.client.get_associations(concept_id)
                
                for assoc in associations[:3]:  # Limit association depth
                    target_id = assoc.get('target_id')
                    
                    if target_id and target_id not in processed_ids:
                        try:
                            target_concept = self.client.query_concept(target_id)
                            
                            # Check tenant boundary
                            if organization_id:
                                concept_org = target_concept.get('organization_id')
                                if concept_org and concept_org != organization_id:
                                    continue  # Skip cross-tenant associations
                            
                            target_concept['association_type'] = assoc.get('association_type', 'unknown')
                            target_concept['association_confidence'] = assoc.get('confidence', 0.0)
                            related_concepts.append(target_concept)
                            processed_ids.add(target_id)
                            
                        except Exception as e:
                            print(f"Warning: Could not retrieve concept {target_id}: {e}")
                            
            except Exception as e:
                print(f"Warning: Could not get associations for {concept_id}: {e}")
        
        return related_concepts
    
    def contextual_search(self, query: str, context_concepts: List[str],
                         organization_id: str = None) -> List[Dict]:
        """Search with additional context from existing concepts."""
        
        # Combine query with context
        if context_concepts:
            context_text = " ".join([
                self.client.query_concept(cid)['content'] 
                for cid in context_concepts[:3]  # Limit context size
            ])
            
            enhanced_query = f"{query} Context: {context_text[:200]}"  # Limit length
        else:
            enhanced_query = query
        
        return self.client.vector_search(
            query_text=enhanced_query,
            k=15,
            organization_id=organization_id
        )
    
    def similarity_threshold_search(self, query: str, min_similarity: float = 0.7,
                                  organization_id: str = None) -> List[Dict]:
        """Return only results above similarity threshold."""
        
        # Get more results initially
        candidates = self.client.vector_search(
            query_text=query,
            k=50,  # Large initial set
            organization_id=organization_id
        )
        
        # Filter by similarity threshold
        filtered_results = [
            result for result in candidates 
            if result.get('similarity', 0.0) >= min_similarity
        ]
        
        return filtered_results
    
    def multi_query_search(self, queries: List[str], 
                          organization_id: str = None) -> Dict:
        """Search using multiple related queries."""
        
        all_results = {}
        concept_scores = {}  # Aggregate scoring
        
        for i, query in enumerate(queries):
            query_results = self.client.vector_search(
                query_text=query,
                k=10,
                organization_id=organization_id
            )
            
            all_results[f'query_{i+1}'] = query_results
            
            # Aggregate concept scores
            for result in query_results:
                concept_id = result['concept_id']
                similarity = result.get('similarity', 0.0)
                
                if concept_id in concept_scores:
                    concept_scores[concept_id]['total_score'] += similarity
                    concept_scores[concept_id]['query_count'] += 1
                else:
                    concept_scores[concept_id] = {
                        'concept': result,
                        'total_score': similarity,
                        'query_count': 1
                    }
        
        # Calculate average scores and rank
        ranked_concepts = []
        for concept_id, score_data in concept_scores.items():
            avg_score = score_data['total_score'] / score_data['query_count']
            concept_data = score_data['concept'].copy()
            concept_data['average_similarity'] = avg_score
            concept_data['matched_queries'] = score_data['query_count']
            ranked_concepts.append(concept_data)
        
        # Sort by average similarity
        ranked_concepts.sort(key=lambda x: x['average_similarity'], reverse=True)
        
        return {
            'individual_queries': all_results,
            'aggregated_ranking': ranked_concepts[:15],  # Top 15 overall
            'total_unique_concepts': len(concept_scores)
        }

# Usage Examples

search_engine = SemanticSearchEngine(client)

# 1. Hybrid search with multiple strategies
hybrid_results = search_engine.hybrid_search(
    query="cardiac emergency treatment protocols",
    organization_id="hospital-st-marys",
    search_depth=3
)

print("Hybrid Search Results:")
print(f"- Primary results: {len(hybrid_results['primary_results'])}")
print(f"- Related concepts: {len(hybrid_results['related_concepts'])}")
print(f"- Domain knowledge: {len(hybrid_results['domain_knowledge'])}")
print(f"- Total searched: {hybrid_results['total_concepts_searched']}")

# 2. High-confidence results only
high_confidence = search_engine.similarity_threshold_search(
    query="acute myocardial infarction treatment",
    min_similarity=0.85,
    organization_id="hospital-st-marys"
)

print(f"\nHigh-confidence results ({len(high_confidence)} concepts):")
for result in high_confidence[:5]:
    print(f"- [{result['similarity']:.3f}] {result['content'][:60]}...")

# 3. Multi-query aggregation
cardiology_queries = [
    "heart attack symptoms diagnosis",
    "cardiac catheterization procedure", 
    "coronary artery disease treatment",
    "ECG abnormalities cardiac"
]

multi_query_results = search_engine.multi_query_search(
    queries=cardiology_queries,
    organization_id="hospital-st-marys"
)

print(f"\nMulti-query search:")
print(f"- Unique concepts found: {multi_query_results['total_unique_concepts']}")
print("Top aggregated results:")
for i, result in enumerate(multi_query_results['aggregated_ranking'][:3], 1):
    print(f"{i}. [{result['average_similarity']:.3f}] "
          f"({result['matched_queries']} queries) {result['content'][:50]}...")

# 4. Contextual search with existing concepts
context_concept_ids = [hybrid_results['primary_results'][0]['concept_id']]
contextual_results = search_engine.contextual_search(
    query="emergency treatment",
    context_concepts=context_concept_ids,
    organization_id="hospital-st-marys"
)

print(f"\nContextual search: {len(contextual_results)} results")
```

## Search Optimization Strategies

### Performance-Optimized Search

```python
class OptimizedVectorSearch:
    def __init__(self, storage_client: StorageClient):
        self.client = storage_client
        self.query_cache = {}  # Simple in-memory cache
        self.cache_ttl = 300  # 5 minutes
        
    def cached_search(self, query: str, k: int = 10,
                     organization_id: str = None) -> List[Dict]:
        """Search with result caching for repeated queries."""
        
        # Create cache key
        cache_key = f"{query}|{k}|{organization_id or 'all'}"
        
        # Check cache
        if cache_key in self.query_cache:
            cached_result, timestamp = self.query_cache[cache_key]
            if time.time() - timestamp < self.cache_ttl:
                print(f"Cache hit for query: {query[:30]}...")
                return cached_result
        
        # Perform search
        results = self.client.vector_search(
            query_text=query,
            k=k,
            organization_id=organization_id
        )
        
        # Cache results
        self.query_cache[cache_key] = (results, time.time())
        
        # Clean old cache entries (simple cleanup)
        if len(self.query_cache) > 1000:
            self._cleanup_cache()
        
        return results
    
    def _cleanup_cache(self):
        """Remove expired cache entries."""
        current_time = time.time()
        expired_keys = [
            key for key, (_, timestamp) in self.query_cache.items()
            if current_time - timestamp > self.cache_ttl
        ]
        
        for key in expired_keys:
            del self.query_cache[key]
    
    def batch_search(self, queries: List[str], k: int = 10,
                    organization_id: str = None) -> Dict[str, List[Dict]]:
        """Efficiently perform multiple searches."""
        
        results = {}
        
        for query in queries:
            try:
                query_results = self.cached_search(
                    query=query,
                    k=k,
                    organization_id=organization_id
                )
                results[query] = query_results
                
            except Exception as e:
                print(f"Search failed for query '{query}': {e}")
                results[query] = []
        
        return results
    
    def progressive_search(self, query: str, target_results: int = 50,
                          organization_id: str = None) -> List[Dict]:
        """Progressively search until target number found."""
        
        all_results = []
        k_increment = 10
        max_k = 200
        current_k = k_increment
        
        while len(all_results) < target_results and current_k <= max_k:
            batch_results = self.client.vector_search(
                query_text=query,
                k=current_k,
                organization_id=organization_id
            )
            
            # Add new results (avoid duplicates)
            existing_ids = {r['concept_id'] for r in all_results}
            new_results = [
                r for r in batch_results 
                if r['concept_id'] not in existing_ids
            ]
            
            all_results.extend(new_results)
            current_k += k_increment
            
            print(f"Progressive search: {len(all_results)} results found (k={current_k})")
            
            if len(new_results) == 0:
                break  # No more new results
        
        return all_results[:target_results]

# Performance optimization usage
optimized_search = OptimizedVectorSearch(client)

# Cached searches for repeated queries
frequent_queries = [
    "cardiac emergency protocols",
    "patient admission procedures", 
    "medication administration guidelines"
]

print("Performing cached searches:")
for query in frequent_queries:
    results = optimized_search.cached_search(query, k=5, organization_id="hospital-st-marys")
    print(f"- {query}: {len(results)} results")

# Second run should hit cache
print("\nRepeating searches (should use cache):")
for query in frequent_queries:
    results = optimized_search.cached_search(query, k=5, organization_id="hospital-st-marys")
    print(f"- {query}: {len(results)} results")

# Batch processing
batch_queries = [
    "infection control procedures",
    "surgical preparation protocols",
    "discharge planning guidelines",
    "emergency medication dosages"
]

batch_results = optimized_search.batch_search(
    queries=batch_queries,
    k=8,
    organization_id="hospital-st-marys"
)

print(f"\nBatch search completed: {len(batch_results)} queries processed")
for query, results in batch_results.items():
    print(f"- {query}: {len(results)} results")

# Progressive search for comprehensive results
comprehensive_results = optimized_search.progressive_search(
    query="cardiac patient care protocols",
    target_results=30,
    organization_id="hospital-st-marys"
)

print(f"\nProgressive search: {len(comprehensive_results)} comprehensive results")
```

## Domain-Specific Search Applications

### Medical Knowledge Search

```python
class MedicalKnowledgeSearch:
    def __init__(self, storage_client: StorageClient):
        self.client = storage_client
        
    def diagnostic_search(self, symptoms: List[str], patient_context: Dict,
                         organization_id: str) -> Dict:
        """Search for diagnostic information based on symptoms."""
        
        # Construct diagnostic query
        symptom_text = ", ".join(symptoms)
        context_text = f"Age: {patient_context.get('age', 'unknown')}, " \
                      f"Gender: {patient_context.get('gender', 'unknown')}, " \
                      f"Medical history: {patient_context.get('history', 'none')}"
        
        diagnostic_query = f"Symptoms: {symptom_text}. Patient: {context_text}"
        
        # Search domain knowledge (medical protocols)
        protocol_results = self.client.vector_search(
            query_text=diagnostic_query,
            k=15
            # No organization_id for shared medical knowledge
        )
        
        # Search similar cases in hospital
        case_results = self.client.vector_search(
            query_text=f"patient case {symptom_text}",
            k=10,
            organization_id=organization_id
        )
        
        # Search for specific symptoms
        symptom_results = []
        for symptom in symptoms[:3]:  # Limit to top 3 symptoms
            symptom_specific = self.client.vector_search(
                query_text=f"medical condition symptom {symptom}",
                k=5
            )
            symptom_results.extend(symptom_specific)
        
        return {
            'diagnostic_protocols': protocol_results,
            'similar_cases': case_results,
            'symptom_specific': symptom_results,
            'patient_context': patient_context,
            'search_query': diagnostic_query
        }
    
    def treatment_search(self, diagnosis: str, patient_factors: Dict,
                        organization_id: str) -> Dict:
        """Search for treatment protocols and guidelines."""
        
        # Construct treatment query
        factors_text = ", ".join([
            f"{key}: {value}" for key, value in patient_factors.items()
        ])
        
        treatment_query = f"Treatment for {diagnosis}. Patient factors: {factors_text}"
        
        # Search treatment protocols
        treatment_protocols = self.client.vector_search(
            query_text=f"treatment protocol {diagnosis}",
            k=12
        )
        
        # Search medication guidelines
        medication_guidelines = self.client.vector_search(
            query_text=f"medication dosage {diagnosis} {factors_text}",
            k=8
        )
        
        # Search contraindications and precautions
        safety_info = self.client.vector_search(
            query_text=f"contraindications precautions {diagnosis}",
            k=6
        )
        
        return {
            'treatment_protocols': treatment_protocols,
            'medication_guidelines': medication_guidelines,
            'safety_information': safety_info,
            'patient_factors': patient_factors
        }
    
    def research_search(self, research_question: str, 
                       evidence_level: str = "high") -> Dict:
        """Search for research evidence and clinical studies."""
        
        evidence_queries = {
            'high': 'randomized controlled trial systematic review meta-analysis',
            'moderate': 'clinical study cohort study case-control',
            'low': 'case series expert opinion clinical experience'
        }
        
        evidence_terms = evidence_queries.get(evidence_level, evidence_queries['moderate'])
        research_query = f"{research_question} {evidence_terms}"
        
        # Search research literature
        research_results = self.client.vector_search(
            query_text=research_query,
            k=20
        )
        
        # Search clinical guidelines
        guideline_results = self.client.vector_search(
            query_text=f"clinical guideline recommendation {research_question}",
            k=10
        )
        
        return {
            'research_evidence': research_results,
            'clinical_guidelines': guideline_results,
            'evidence_level': evidence_level,
            'search_query': research_query
        }

# Medical search usage
medical_search = MedicalKnowledgeSearch(client)

# Diagnostic search
diagnostic_results = medical_search.diagnostic_search(
    symptoms=["chest pain", "shortness of breath", "elevated troponins"],
    patient_context={
        "age": 65,
        "gender": "male",
        "history": "hypertension, diabetes"
    },
    organization_id="hospital-st-marys"
)

print("Diagnostic Search Results:")
print(f"- Protocol results: {len(diagnostic_results['diagnostic_protocols'])}")
print(f"- Similar cases: {len(diagnostic_results['similar_cases'])}")
print(f"- Symptom-specific: {len(diagnostic_results['symptom_specific'])}")

# Treatment search  
treatment_results = medical_search.treatment_search(
    diagnosis="acute myocardial infarction",
    patient_factors={
        "age": 65,
        "weight": "80kg", 
        "allergies": "none",
        "renal_function": "normal"
    },
    organization_id="hospital-st-marys"
)

print("\nTreatment Search Results:")
print(f"- Treatment protocols: {len(treatment_results['treatment_protocols'])}")
print(f"- Medication guidelines: {len(treatment_results['medication_guidelines'])}")
print(f"- Safety information: {len(treatment_results['safety_information'])}")

# Research evidence search
research_results = medical_search.research_search(
    research_question="effectiveness of early PCI in STEMI patients",
    evidence_level="high"
)

print("\nResearch Search Results:")
print(f"- Research evidence: {len(research_results['research_evidence'])}")
print(f"- Clinical guidelines: {len(research_results['clinical_guidelines'])}")
print(f"- Evidence level: {research_results['evidence_level']}")
```

## Performance Monitoring

### Search Analytics

```python
class VectorSearchAnalytics:
    def __init__(self, storage_client: StorageClient):
        self.client = storage_client
        self.search_metrics = {
            'query_count': 0,
            'total_latency': 0.0,
            'avg_results_per_query': 0.0,
            'cache_hit_rate': 0.0,
            'tenant_distribution': {},
            'query_patterns': {}
        }
        
    def monitored_search(self, query: str, k: int = 10,
                        organization_id: str = None) -> Tuple[List[Dict], Dict]:
        """Perform search with performance monitoring."""
        
        start_time = time.time()
        
        try:
            results = self.client.vector_search(
                query_text=query,
                k=k,
                organization_id=organization_id
            )
            
            search_time = time.time() - start_time
            
            # Update metrics
            self.search_metrics['query_count'] += 1
            self.search_metrics['total_latency'] += search_time
            
            # Calculate rolling averages
            query_count = self.search_metrics['query_count']
            self.search_metrics['avg_latency'] = self.search_metrics['total_latency'] / query_count
            
            # Update results per query average
            prev_avg = self.search_metrics['avg_results_per_query']
            new_avg = ((prev_avg * (query_count - 1)) + len(results)) / query_count
            self.search_metrics['avg_results_per_query'] = new_avg
            
            # Track tenant distribution
            tenant_key = organization_id or 'domain'
            self.search_metrics['tenant_distribution'][tenant_key] = \
                self.search_metrics['tenant_distribution'].get(tenant_key, 0) + 1
            
            # Track query patterns
            query_length = len(query.split())
            length_category = f"{query_length // 5 * 5}-{query_length // 5 * 5 + 4}_words"
            self.search_metrics['query_patterns'][length_category] = \
                self.search_metrics['query_patterns'].get(length_category, 0) + 1
            
            search_metadata = {
                'latency_ms': search_time * 1000,
                'result_count': len(results),
                'query_length': query_length,
                'tenant': tenant_key,
                'timestamp': time.time()
            }
            
            return results, search_metadata
            
        except Exception as e:
            error_metadata = {
                'error': str(e),
                'latency_ms': (time.time() - start_time) * 1000,
                'query_length': len(query.split()),
                'tenant': organization_id or 'domain',
                'timestamp': time.time()
            }
            
            return [], error_metadata
    
    def get_performance_report(self) -> Dict:
        """Generate comprehensive performance report."""
        
        metrics = self.search_metrics.copy()
        
        # Calculate additional statistics
        if metrics['query_count'] > 0:
            metrics['avg_latency_ms'] = (metrics['total_latency'] / metrics['query_count']) * 1000
        else:
            metrics['avg_latency_ms'] = 0.0
        
        # Tenant usage percentage
        total_queries = metrics['query_count']
        if total_queries > 0:
            tenant_percentages = {
                tenant: (count / total_queries) * 100
                for tenant, count in metrics['tenant_distribution'].items()
            }
            metrics['tenant_usage_percent'] = tenant_percentages
        
        return {
            'performance_metrics': metrics,
            'recommendations': self._generate_recommendations(metrics),
            'report_timestamp': time.time()
        }
    
    def _generate_recommendations(self, metrics: Dict) -> List[str]:
        """Generate performance optimization recommendations."""
        
        recommendations = []
        
        if metrics.get('avg_latency_ms', 0) > 100:
            recommendations.append(
                "High average latency detected. Consider implementing query caching."
            )
        
        if metrics.get('avg_results_per_query', 0) > 50:
            recommendations.append(
                "Large result sets detected. Consider lowering k parameter for better performance."
            )
        
        # Analyze tenant distribution
        tenant_dist = metrics.get('tenant_distribution', {})
        if len(tenant_dist) > 1:
            max_tenant_queries = max(tenant_dist.values()) if tenant_dist else 0
            total_queries = metrics.get('query_count', 0)
            
            if max_tenant_queries > total_queries * 0.8:
                recommendations.append(
                    "Single tenant dominates queries. Consider tenant-specific optimization."
                )
        
        return recommendations

# Analytics usage
analytics = VectorSearchAnalytics(client)

# Monitored searches
test_queries = [
    "cardiac emergency procedures",
    "patient medication guidelines", 
    "diagnostic imaging protocols",
    "surgical preparation checklist",
    "infection control procedures"
]

print("Performing monitored searches...")
for query in test_queries:
    results, metadata = analytics.monitored_search(
        query=query,
        k=10,
        organization_id="hospital-st-marys"
    )
    
    print(f"Query: {query[:30]}...")
    print(f"  - Results: {metadata['result_count']}")
    print(f"  - Latency: {metadata['latency_ms']:.2f}ms")
    print(f"  - Query length: {metadata['query_length']} words")

# Generate performance report
performance_report = analytics.get_performance_report()

print("\n=== Vector Search Performance Report ===")
print(f"Total queries: {performance_report['performance_metrics']['query_count']}")
print(f"Average latency: {performance_report['performance_metrics']['avg_latency_ms']:.2f}ms")
print(f"Average results per query: {performance_report['performance_metrics']['avg_results_per_query']:.1f}")

print("\nTenant distribution:")
for tenant, percentage in performance_report['performance_metrics'].get('tenant_usage_percent', {}).items():
    print(f"  - {tenant}: {percentage:.1f}%")

print("\nQuery patterns:")
for pattern, count in performance_report['performance_metrics'].get('query_patterns', {}).items():
    print(f"  - {pattern}: {count} queries")

if performance_report['recommendations']:
    print("\nRecommendations:")
    for rec in performance_report['recommendations']:
        print(f"  - {rec}")
```

## Best Practices

### 1. Query Optimization

- **Specific Queries**: Use precise, domain-specific terminology
- **Query Length**: 3-15 words optimal for semantic search
- **Context Addition**: Include relevant context for better matching

### 2. Result Processing

- **Similarity Thresholds**: Filter results below meaningful similarity scores
- **Result Limiting**: Use appropriate k values (5-20 for most use cases)  
- **Multi-stage Search**: Combine vector search with metadata filtering

### 3. Performance Considerations

- **Caching**: Implement query result caching for repeated searches
- **Batch Processing**: Group related searches for efficiency
- **Tenant Awareness**: Use organization filters to limit search scope

## Next Steps

- [**Performance Guide**](./08-performance.md) - Advanced optimization strategies
- [**Multi-Tenancy**](./06-multi-tenancy.md) - Tenant-aware search patterns
- [**Troubleshooting**](./09-troubleshooting.md) - Debugging search issues

---

*Vector search in Sutra Storage provides sub-millisecond semantic similarity queries with tenant isolation and persistent HNSW indexing for production AI applications.*