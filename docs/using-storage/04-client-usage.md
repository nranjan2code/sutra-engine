# Client Usage

This document provides comprehensive examples and best practices for using the Sutra Storage client in your applications. Whether you're building a medical AI system, legal research tool, or financial compliance platform, these patterns will help you integrate Sutra effectively.

## Installation and Setup

### Python Client

```bash
# Install the storage client
pip install sutra-storage-client-tcp

# Or from source
cd packages/sutra-storage-client-tcp
pip install -e .
```

### Basic Connection

```python
from sutra_storage_client import StorageClient

# Development setup (localhost)
client = StorageClient("localhost:50051")

# Production setup with multiple servers
client = StorageClient(
    server_address="storage-prod-01.company.com:50051",
    timeout_seconds=30,
    max_retries=3
)

# Test the connection
health = client.health_check()
print(f"Storage server healthy: {health['healthy']}")
print(f"Uptime: {health['uptime_seconds']} seconds")
```

## Core Operations

### 1. Learning Concepts

#### Simple Learning (Recommended)

The unified learning pipeline automatically handles embeddings and associations:

```python
# Learn a single concept
concept_id = client.learn_concept_v2(
    content="Hypertension is defined as systolic BP ≥140 mmHg or diastolic BP ≥90 mmHg."
)
print(f"Learned concept: {concept_id}")

# Learn with custom options
concept_id = client.learn_concept_v2(
    content="Regular exercise reduces cardiovascular disease risk by 30-35%.",
    options={
        "embedding_model": "granite-embedding:30m",  # Custom model
        "extract_associations": True,               # Find related concepts
        "min_association_confidence": 0.75,        # Higher quality threshold
        "strength": 0.95,                          # High importance
        "confidence": 0.90                         # High reliability
    }
)
```

#### Batch Learning (High Performance)

For loading large datasets efficiently:

```python
# Batch learning for medical protocols
medical_protocols = [
    "Diabetes management requires HbA1c monitoring every 3-6 months.",
    "Statin therapy is recommended for LDL cholesterol >190 mg/dL.",
    "Blood pressure should be checked annually for adults over 40.",
    "Mammography screening recommended biennially for women 50-74 years.",
    "Colonoscopy screening recommended every 10 years starting at age 50."
]

# Learn all concepts in one efficient batch
concept_ids = client.learn_batch_v2(
    contents=medical_protocols,
    options={
        "embedding_model": "granite-embedding:30m",
        "extract_associations": True,
        "min_association_confidence": 0.8
    }
)

print(f"Learned {len(concept_ids)} medical protocols")
for i, concept_id in enumerate(concept_ids):
    print(f"Protocol {i+1}: {concept_id}")
```

#### Legacy Explicit Learning

For fine-grained control (advanced use cases):

```python
# Manual concept creation with pre-computed embedding
import hashlib

def generate_concept_id(content: str) -> str:
    return hashlib.md5(content.encode()).hexdigest()

concept_id = generate_concept_id("Manual concept content")
sequence = client.learn_concept(
    concept_id=concept_id,
    content="Manual concept content for legacy compatibility.",
    embedding=[0.1, 0.2, 0.3] * 256,  # Pre-computed 768-d vector
    strength=0.8,
    confidence=0.9
)
print(f"Learned concept with sequence: {sequence}")
```

### 2. Querying Concepts

```python
# Get concept details
concept = client.query_concept(concept_id)
if concept['found']:
    print(f"Content: {concept['content']}")
    print(f"Strength: {concept['strength']}")
    print(f"Confidence: {concept['confidence']}")
    if concept['metadata']:
        print(f"Type: {concept['metadata']['concept_type']}")
        print(f"Tags: {concept['metadata']['tags']}")
else:
    print("Concept not found")

# Bulk concept retrieval
concept_ids = ["abc123", "def456", "ghi789"]
concepts = []
for cid in concept_ids:
    concept = client.query_concept(cid)
    if concept['found']:
        concepts.append(concept)

print(f"Retrieved {len(concepts)} concepts")
```

### 3. Working with Associations

#### Creating Associations

```python
from sutra_storage_client import AssociationType

# Create semantic relationship
sequence = client.learn_association(
    source_id=hypertension_concept_id,
    target_id=cardiovascular_disease_id,
    assoc_type=AssociationType.CAUSAL,  # Cause-and-effect
    confidence=0.85
)

# Create hierarchical relationship (category)
sequence = client.learn_association(
    source_id=statin_therapy_id,
    target_id=cholesterol_management_id,
    assoc_type=AssociationType.HIERARCHICAL,  # Part-of
    confidence=0.90
)

# Create temporal relationship (sequence)
sequence = client.learn_association(
    source_id=diagnosis_concept_id,
    target_id=treatment_concept_id,
    assoc_type=AssociationType.TEMPORAL,  # Happens-after
    confidence=0.95
)
```

#### Finding Related Concepts

```python
# Get immediate neighbors
neighbor_ids = client.get_neighbors(concept_id)
print(f"Found {len(neighbor_ids)} related concepts")

for neighbor_id in neighbor_ids[:5]:  # Show first 5
    neighbor = client.query_concept(neighbor_id)
    if neighbor['found']:
        print(f"Related: {neighbor['content'][:100]}...")

# Get specific association details (if server supports it)
try:
    association = client.get_association(source_id, target_id)
    if association:
        print(f"Association type: {association['type']}")
        print(f"Confidence: {association['confidence']}")
except AttributeError:
    print("Association details not available in this client version")
```

### 4. Vector Search

#### Semantic Search

```python
# Search by text query (automatic embedding generation)
results = client.vector_search(
    query_text="heart disease prevention strategies",
    k=10,              # Top 10 results
    ef_search=50       # Search quality parameter
)

print(f"Found {len(results)} similar concepts:")
for result in results:
    concept = client.query_concept(result['concept_id'])
    if concept['found']:
        print(f"Similarity: {result['similarity']:.3f}")
        print(f"Content: {concept['content'][:150]}...")
        print("---")

# Search with pre-computed vector
import numpy as np
query_vector = np.random.random(768).tolist()  # Your embedding
results = client.vector_search(
    query_vector=query_vector,
    k=5,
    ef_search=100  # Higher quality search
)
```

#### Multi-Tenant Search

```python
# Search within specific organization
hospital_results = client.vector_search(
    query_text="emergency cardiac protocols",
    k=20,
    organization_id="hospital-st-marys"  # Filter by tenant
)

print(f"Found {len(hospital_results)} protocols for St. Mary's Hospital")
```

## Advanced Usage Patterns

### 1. Domain-Specific Knowledge Base

#### Medical AI System

```python
class MedicalKnowledgeBase:
    def __init__(self, storage_client: StorageClient):
        self.client = storage_client
        self.medical_concepts = {}
    
    def add_clinical_guideline(self, guideline: str, specialty: str, 
                            evidence_level: str = "A") -> str:
        """Add a clinical guideline with medical metadata."""
        concept_id = self.client.learn_concept_v2(
            content=guideline,
            options={
                "strength": self._evidence_to_strength(evidence_level),
                "confidence": 0.95,
                "extract_associations": True
            }
        )
        
        # Store local reference with medical metadata
        self.medical_concepts[concept_id] = {
            "specialty": specialty,
            "evidence_level": evidence_level,
            "type": "clinical_guideline"
        }
        
        return concept_id
    
    def find_related_guidelines(self, condition: str, specialty: str = None) -> list:
        """Find clinical guidelines related to a medical condition."""
        results = self.client.vector_search(
            query_text=condition,
            k=15,
            ef_search=75
        )
        
        # Filter by specialty if specified
        guidelines = []
        for result in results:
            if concept_id in self.medical_concepts:
                meta = self.medical_concepts[concept_id] 
                if not specialty or meta['specialty'] == specialty:
                    concept = self.client.query_concept(result['concept_id'])
                    if concept['found']:
                        guidelines.append({
                            'content': concept['content'],
                            'similarity': result['similarity'],
                            'specialty': meta['specialty'],
                            'evidence_level': meta['evidence_level']
                        })
        
        return sorted(guidelines, key=lambda x: x['similarity'], reverse=True)
    
    def _evidence_to_strength(self, level: str) -> float:
        """Convert evidence level to concept strength."""
        mapping = {"A": 0.95, "B": 0.8, "C": 0.6, "D": 0.4}
        return mapping.get(level, 0.5)

# Usage
kb = MedicalKnowledgeBase(client)

# Add guidelines
diabetes_id = kb.add_clinical_guideline(
    guideline="Type 2 diabetes management should include lifestyle modification, "
             "metformin as first-line therapy, and HbA1c monitoring every 3 months.",
    specialty="endocrinology",
    evidence_level="A"
)

hypertension_id = kb.add_clinical_guideline(
    guideline="Stage 1 hypertension (130-139/80-89 mmHg) should be managed with "
             "lifestyle changes and may require medication in high-risk patients.",
    specialty="cardiology", 
    evidence_level="A"
)

# Find related guidelines
related = kb.find_related_guidelines("diabetes management", "endocrinology")
for guideline in related[:3]:
    print(f"Evidence {guideline['evidence_level']} "
          f"(similarity: {guideline['similarity']:.2f})")
    print(f"Content: {guideline['content'][:200]}...")
    print()
```

#### Legal Research System

```python
class LegalResearchSystem:
    def __init__(self, storage_client: StorageClient):
        self.client = storage_client
        
    def add_case_law(self, case_name: str, citation: str, 
                    holding: str, jurisdiction: str) -> str:
        """Add legal case with structured content."""
        content = f"Case: {case_name}\nCitation: {citation}\nHolding: {holding}"
        
        concept_id = self.client.learn_concept_v2(
            content=content,
            options={
                "extract_associations": True,
                "min_association_confidence": 0.7,
                "strength": 0.9
            }
        )
        
        # Create associations for legal hierarchy
        if "Supreme Court" in citation:
            # Supreme Court cases have higher precedential value
            self.client.learn_association(
                source_id=concept_id,
                target_id=self._get_or_create_jurisdiction_concept(jurisdiction),
                assoc_type=AssociationType.HIERARCHICAL,
                confidence=1.0
            )
        
        return concept_id
    
    def research_legal_issue(self, legal_query: str, jurisdiction: str = None) -> list:
        """Research cases related to a legal issue."""
        results = self.client.vector_search(
            query_text=legal_query,
            k=20,
            ef_search=80
        )
        
        relevant_cases = []
        for result in results:
            concept = self.client.query_concept(result['concept_id'])
            if concept['found'] and self._is_relevant_case(concept['content'], jurisdiction):
                relevant_cases.append({
                    'content': concept['content'],
                    'similarity': result['similarity'],
                    'precedential_value': self._calculate_precedential_value(concept['content'])
                })
        
        # Sort by precedential value and similarity
        return sorted(relevant_cases, 
                     key=lambda x: (x['precedential_value'], x['similarity']), 
                     reverse=True)
    
    def _get_or_create_jurisdiction_concept(self, jurisdiction: str) -> str:
        # Implementation for jurisdiction concept management
        pass
    
    def _is_relevant_case(self, content: str, jurisdiction: str) -> bool:
        # Implementation for jurisdiction filtering
        return jurisdiction is None or jurisdiction in content
    
    def _calculate_precedential_value(self, content: str) -> float:
        # Implementation for precedential value calculation
        if "Supreme Court" in content:
            return 1.0
        elif "Circuit" in content or "Court of Appeals" in content:
            return 0.8
        elif "District" in content:
            return 0.6
        else:
            return 0.4
```

### 2. Real-Time Learning System

```python
import threading
import queue
import time
from typing import Dict, List

class RealTimeLearningSystem:
    def __init__(self, storage_client: StorageClient, batch_size: int = 50):
        self.client = storage_client
        self.batch_size = batch_size
        self.learning_queue = queue.Queue()
        self.stats = {"learned": 0, "errors": 0}
        self.running = False
        
    def start(self):
        """Start background learning thread."""
        self.running = True
        self.learning_thread = threading.Thread(target=self._learning_worker)
        self.learning_thread.daemon = True
        self.learning_thread.start()
        print("Real-time learning system started")
    
    def stop(self):
        """Stop background learning."""
        self.running = False
        if hasattr(self, 'learning_thread'):
            self.learning_thread.join()
        print(f"Learning system stopped. Stats: {self.stats}")
    
    def learn_async(self, content: str, priority: int = 1) -> None:
        """Queue content for asynchronous learning."""
        self.learning_queue.put({
            "content": content,
            "priority": priority,
            "timestamp": time.time()
        })
    
    def _learning_worker(self):
        """Background worker for batch learning."""
        batch = []
        
        while self.running:
            try:
                # Collect batch
                while len(batch) < self.batch_size and self.running:
                    try:
                        item = self.learning_queue.get(timeout=1.0)
                        batch.append(item)
                    except queue.Empty:
                        break
                
                if batch:
                    # Sort by priority (higher first)
                    batch.sort(key=lambda x: x['priority'], reverse=True)
                    
                    # Extract content for batch learning
                    contents = [item['content'] for item in batch]
                    
                    try:
                        concept_ids = self.client.learn_batch_v2(contents)
                        self.stats["learned"] += len(concept_ids)
                        print(f"Learned batch of {len(concept_ids)} concepts")
                    except Exception as e:
                        print(f"Batch learning error: {e}")
                        self.stats["errors"] += len(batch)
                    
                    batch.clear()
                    
            except Exception as e:
                print(f"Learning worker error: {e}")
                self.stats["errors"] += 1

# Usage
learning_system = RealTimeLearningSystem(client, batch_size=25)
learning_system.start()

# Add content continuously
medical_updates = [
    "New study shows 40% reduction in heart disease with Mediterranean diet.",
    "FDA approves new diabetes medication with improved side effect profile.",
    "Clinical trial demonstrates efficacy of AI-assisted diagnosis in radiology."
]

for update in medical_updates:
    learning_system.learn_async(update, priority=2)  # High priority

time.sleep(5)  # Let system process
learning_system.stop()
```

### 3. Concept Validation and Quality Control

```python
class ConceptQualityManager:
    def __init__(self, storage_client: StorageClient):
        self.client = storage_client
        self.validation_rules = []
    
    def add_validation_rule(self, rule_func, description: str):
        """Add a validation rule function."""
        self.validation_rules.append({
            "function": rule_func,
            "description": description
        })
    
    def validate_and_learn(self, content: str, min_quality: float = 0.7) -> Dict:
        """Validate content quality before learning."""
        validation_results = []
        total_score = 0.0
        
        for rule in self.validation_rules:
            try:
                score = rule["function"](content)
                validation_results.append({
                    "rule": rule["description"],
                    "score": score,
                    "passed": score >= min_quality
                })
                total_score += score
            except Exception as e:
                validation_results.append({
                    "rule": rule["description"],
                    "score": 0.0,
                    "passed": False,
                    "error": str(e)
                })
        
        avg_score = total_score / len(self.validation_rules) if self.validation_rules else 0.0
        
        result = {
            "content": content,
            "quality_score": avg_score,
            "validation_results": validation_results,
            "passed_validation": avg_score >= min_quality,
            "concept_id": None
        }
        
        if result["passed_validation"]:
            # Learn the concept with quality-adjusted confidence
            concept_id = self.client.learn_concept_v2(
                content=content,
                options={
                    "confidence": min(avg_score, 1.0),
                    "strength": avg_score
                }
            )
            result["concept_id"] = concept_id
        
        return result

# Define validation rules
def medical_terminology_check(content: str) -> float:
    """Check for proper medical terminology."""
    medical_terms = ["diagnosis", "treatment", "patient", "therapy", "clinical"]
    term_count = sum(1 for term in medical_terms if term.lower() in content.lower())
    return min(term_count / 3.0, 1.0)  # Normalize to 0-1

def content_length_check(content: str) -> float:
    """Check for appropriate content length."""
    length = len(content.split())
    if 10 <= length <= 200:
        return 1.0
    elif length < 10:
        return length / 10.0
    else:
        return max(0.5, 200.0 / length)

def citation_check(content: str) -> float:
    """Check for citations or references."""
    citation_indicators = ["study", "research", "trial", "journal", "published"]
    has_citation = any(indicator in content.lower() for indicator in citation_indicators)
    return 0.8 if has_citation else 0.6

# Usage
quality_manager = ConceptQualityManager(client)
quality_manager.add_validation_rule(medical_terminology_check, "Medical terminology")
quality_manager.add_validation_rule(content_length_check, "Content length")
quality_manager.add_validation_rule(citation_check, "Citation presence")

# Validate and learn content
medical_content = """
A recent clinical trial published in NEJM demonstrated that patients with 
type 2 diabetes who received intensive lifestyle counseling showed a 25% 
reduction in HbA1c levels compared to standard care controls.
"""

result = quality_manager.validate_and_learn(medical_content, min_quality=0.75)
print(f"Quality score: {result['quality_score']:.2f}")
print(f"Passed validation: {result['passed_validation']}")
if result['concept_id']:
    print(f"Learned as concept: {result['concept_id']}")

for validation in result['validation_results']:
    print(f"- {validation['rule']}: {validation['score']:.2f} "
          f"({'PASS' if validation['passed'] else 'FAIL'})")
```

## Error Handling and Resilience

### Connection Management

```python
import time
import logging
from typing import Optional

class ResilientStorageClient:
    def __init__(self, server_addresses: List[str], max_retries: int = 3):
        self.server_addresses = server_addresses
        self.max_retries = max_retries
        self.current_server_index = 0
        self.client: Optional[StorageClient] = None
        self.logger = logging.getLogger(__name__)
        self._connect()
    
    def _connect(self):
        """Connect to next available server."""
        for i in range(len(self.server_addresses)):
            try:
                address = self.server_addresses[self.current_server_index]
                self.client = StorageClient(address)
                # Test connection
                self.client.health_check()
                self.logger.info(f"Connected to storage server: {address}")
                return
            except Exception as e:
                self.logger.warning(f"Failed to connect to {address}: {e}")
                self.current_server_index = (self.current_server_index + 1) % len(self.server_addresses)
        
        raise ConnectionError("All storage servers unavailable")
    
    def _retry_operation(self, operation_func, *args, **kwargs):
        """Retry operation with exponential backoff."""
        last_exception = None
        
        for attempt in range(self.max_retries):
            try:
                return operation_func(*args, **kwargs)
            except ConnectionError as e:
                last_exception = e
                self.logger.warning(f"Operation failed (attempt {attempt + 1}): {e}")
                
                if attempt < self.max_retries - 1:
                    # Try reconnecting to next server
                    try:
                        self._connect()
                    except ConnectionError:
                        # All servers down, wait before retry
                        wait_time = 2 ** attempt
                        self.logger.info(f"Waiting {wait_time}s before retry...")
                        time.sleep(wait_time)
            except Exception as e:
                # Non-connection error, don't retry
                raise e
        
        raise last_exception
    
    def learn_concept_v2(self, content: str, options: dict = None) -> str:
        """Resilient concept learning."""
        return self._retry_operation(
            self.client.learn_concept_v2, content, options
        )
    
    def query_concept(self, concept_id: str) -> dict:
        """Resilient concept query."""
        return self._retry_operation(
            self.client.query_concept, concept_id
        )
    
    def vector_search(self, query_text: str = None, query_vector: list = None, 
                     k: int = 10, ef_search: int = 50) -> list:
        """Resilient vector search."""
        if query_text:
            return self._retry_operation(
                self.client.vector_search, 
                query_text=query_text, k=k, ef_search=ef_search
            )
        else:
            return self._retry_operation(
                self.client.vector_search,
                query_vector=query_vector, k=k, ef_search=ef_search
            )

# Usage
resilient_client = ResilientStorageClient([
    "storage-01.company.com:50051",
    "storage-02.company.com:50051", 
    "storage-03.company.com:50051"
])

# Operations automatically handle failures and retry with different servers
concept_id = resilient_client.learn_concept_v2("Medical protocol content...")
```

### Data Consistency Checks

```python
def verify_concept_integrity(client: StorageClient, concept_id: str) -> Dict:
    """Verify concept data integrity."""
    result = {
        "concept_id": concept_id,
        "exists": False,
        "content_valid": False,
        "associations_count": 0,
        "embedding_available": False,
        "issues": []
    }
    
    try:
        # Check if concept exists
        concept = client.query_concept(concept_id)
        if not concept['found']:
            result["issues"].append("Concept not found")
            return result
        
        result["exists"] = True
        
        # Validate content
        if concept['content'] and len(concept['content'].strip()) > 0:
            result["content_valid"] = True
        else:
            result["issues"].append("Empty or invalid content")
        
        # Check confidence and strength ranges
        if not (0.0 <= concept['confidence'] <= 1.0):
            result["issues"].append(f"Invalid confidence: {concept['confidence']}")
        
        if not (0.0 <= concept['strength'] <= 1.0):
            result["issues"].append(f"Invalid strength: {concept['strength']}")
        
        # Check associations
        try:
            neighbors = client.get_neighbors(concept_id)
            result["associations_count"] = len(neighbors)
        except Exception as e:
            result["issues"].append(f"Association check failed: {e}")
        
        # Check vector search capability (implies embedding exists)
        try:
            search_results = client.vector_search(
                query_text=concept['content'][:100], k=1
            )
            result["embedding_available"] = len(search_results) > 0
        except Exception as e:
            result["issues"].append(f"Vector search failed: {e}")
        
    except Exception as e:
        result["issues"].append(f"Concept query failed: {e}")
    
    result["healthy"] = len(result["issues"]) == 0
    return result

# Usage
concept_health = verify_concept_integrity(client, concept_id)
print(f"Concept {concept_id} health: {concept_health}")
```

## Performance Optimization

### Batch Operations Strategy

```python
class OptimizedLearningManager:
    def __init__(self, storage_client: StorageClient):
        self.client = storage_client
        self.pending_concepts = []
        self.pending_associations = []
        
    def add_concept(self, content: str) -> str:
        """Add concept to batch queue."""
        self.pending_concepts.append(content)
        # Return estimated concept ID
        return hashlib.md5(content.encode()).hexdigest()
    
    def add_association(self, source_id: str, target_id: str, 
                       assoc_type: int, confidence: float):
        """Add association to batch queue."""
        self.pending_associations.append({
            "source_id": source_id,
            "target_id": target_id,
            "assoc_type": assoc_type,
            "confidence": confidence
        })
    
    def flush(self) -> Dict[str, List]:
        """Execute all pending operations in batches."""
        results = {"concepts": [], "associations": []}
        
        # Process concept batches
        batch_size = 100
        for i in range(0, len(self.pending_concepts), batch_size):
            batch = self.pending_concepts[i:i + batch_size]
            try:
                concept_ids = self.client.learn_batch_v2(batch)
                results["concepts"].extend(concept_ids)
            except Exception as e:
                print(f"Concept batch failed: {e}")
        
        # Process association batches
        for i in range(0, len(self.pending_associations), batch_size):
            batch = self.pending_associations[i:i + batch_size]
            for assoc in batch:
                try:
                    seq = self.client.learn_association(**assoc)
                    results["associations"].append(seq)
                except Exception as e:
                    print(f"Association failed: {e}")
        
        # Clear queues
        self.pending_concepts.clear()
        self.pending_associations.clear()
        
        return results

# Usage for large dataset loading
manager = OptimizedLearningManager(client)

# Queue many operations
for protocol in medical_protocols:
    manager.add_concept(protocol)

for source, target in related_pairs:
    manager.add_association(source, target, AssociationType.SEMANTIC, 0.8)

# Execute efficiently
results = manager.flush()
print(f"Processed {len(results['concepts'])} concepts and "
      f"{len(results['associations'])} associations")
```

## Next Steps

- [**Learning Pipeline**](./05-learning-pipeline.md) - Advanced learning features
- [**Multi-Tenancy**](./06-multi-tenancy.md) - Organization isolation
- [**Performance Guide**](./08-performance.md) - Optimization strategies

---

*These client usage patterns provide the foundation for building sophisticated AI reasoning applications with Sutra Storage.*