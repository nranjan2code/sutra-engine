# Multi-Tenancy

Sutra Storage provides robust multi-tenancy support for SaaS applications, healthcare systems, and enterprise deployments. This guide covers organization isolation, tenant-aware queries, and security considerations.

## Architecture Overview

### Dual Storage Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Multi-Tenant Storage                         │
├─────────────────────────────────────────────────────────────────┤
│  Domain Storage (domain-storage.dat)                           │
│  ├─ Shared knowledge across all tenants                        │
│  ├─ Medical protocols, legal precedents                        │
│  ├─ Regulatory guidelines, best practices                      │
│  └─ No organization_id required                                │
├─────────────────────────────────────────────────────────────────┤
│  User Storage (user-storage.dat)                               │
│  ├─ Tenant-specific data with organization_id                  │
│  ├─ Users, sessions, conversations                             │
│  ├─ Custom knowledge bases                                     │
│  └─ Audit logs and access control                             │
└─────────────────────────────────────────────────────────────────┘
```

### Benefits of Separation

1. **Performance**: Domain knowledge optimized for read-heavy reasoning
2. **Security**: User data isolated with tenant-specific access control  
3. **Compliance**: Clear audit boundaries for regulatory requirements
4. **Scaling**: Different optimization strategies per storage type

## Concept Types and Organization

### Domain Concepts (Shared)

```python
# Domain concepts - shared across all tenants
domain_concept = client.learn_concept_v2(
    content="Hypertension is defined as systolic BP ≥140 mmHg or diastolic BP ≥90 mmHg.",
    # No metadata needed - stored in domain-storage.dat
)

# All organizations can access this knowledge
concept = client.query_concept(domain_concept)
print(f"Available to all tenants: {concept['content']}")
```

### User Concepts (Tenant-Isolated)

```python
from sutra_storage_client import ConceptType

# User concepts - require organization_id
user_metadata = {
    "concept_type": ConceptType.User,
    "organization_id": "hospital-st-marys",  # Required!
    "created_by": "admin-001",
    "tags": ["staff", "active"],
    "attributes": {
        "email": "dr.smith@stmarys.com",
        "department": "cardiology",
        "role": "physician"
    }
}

user_concept = client.learn_concept_with_metadata(
    content="Dr. Sarah Smith, MD - Interventional Cardiologist, 15 years experience",
    metadata=user_metadata
)

# Conversation concepts - also tenant-isolated
conversation_metadata = {
    "concept_type": ConceptType.Conversation,
    "organization_id": "hospital-st-marys",
    "created_by": "dr.smith@stmarys.com",
    "attributes": {
        "title": "Patient consultation - Acute MI case",
        "domain_storage": "cardiology-protocols",  # Links to domain knowledge
        "patient_id": "encrypted-patient-123"
    }
}

conversation_concept = client.learn_concept_with_metadata(
    content="Consultation regarding 65-year-old male presenting with chest pain and elevated troponins.",
    metadata=conversation_metadata
)
```

## Tenant-Aware Queries

### Organization-Scoped Search

```python
# Search within specific organization only
hospital_staff = client.query_by_metadata(
    concept_type=ConceptType.User,
    organization_id="hospital-st-marys",  # Tenant filter
    tags=["staff"],
    limit=50
)

print(f"Found {len(hospital_staff)} staff members for St. Mary's Hospital")

# Department-specific search
cardiology_staff = client.query_by_metadata(
    concept_type=ConceptType.User,
    organization_id="hospital-st-marys",
    attributes={"department": "cardiology"},
    limit=20
)
```

### Cross-Tenant Domain Search

```python
# Search domain knowledge (available to all tenants)
cardiac_protocols = client.vector_search(
    query_text="acute myocardial infarction treatment protocol",
    k=10,
    # No organization_id - searches domain storage
)

# Tenant-specific vector search
hospital_conversations = client.vector_search(
    query_text="patient consultation cardiac symptoms",
    k=15,
    organization_id="hospital-st-marys"  # Only this tenant's data
)
```

## Implementation Patterns

### 1. Healthcare Multi-Tenancy

```python
class HealthcareMultiTenantSystem:
    def __init__(self, storage_client: StorageClient):
        self.client = storage_client
        self.domain_protocols = {}  # Shared medical knowledge
        
    def add_medical_protocol(self, protocol: str, category: str) -> str:
        """Add shared medical protocol available to all hospitals."""
        # Domain concepts - no organization_id needed
        concept_id = self.client.learn_concept_v2(
            content=f"[{category}] {protocol}",
            options={
                "extract_associations": True,
                "min_association_confidence": 0.85  # High medical accuracy
            }
        )
        
        self.domain_protocols[concept_id] = {
            "category": category,
            "shared": True
        }
        
        return concept_id
    
    def create_hospital_organization(self, hospital_name: str, 
                                   hospital_id: str) -> Dict[str, str]:
        """Set up a new hospital organization."""
        
        # Create organization concept
        org_metadata = {
            "concept_type": ConceptType.Organization,
            "organization_id": hospital_id,  # Self-referential
            "attributes": {
                "org_name": hospital_name,
                "org_type": "healthcare",
                "billing_tier": "standard",
                "max_users": "500"
            }
        }
        
        org_concept_id = self.client.learn_concept_with_metadata(
            content=f"Healthcare Organization: {hospital_name}",
            metadata=org_metadata
        )
        
        # Create default admin user
        admin_metadata = {
            "concept_type": ConceptType.User,
            "organization_id": hospital_id,
            "created_by": "system",
            "attributes": {
                "email": f"admin@{hospital_id}.com",
                "role": "administrator",
                "department": "administration"
            }
        }
        
        admin_concept_id = self.client.learn_concept_with_metadata(
            content=f"System Administrator for {hospital_name}",
            metadata=admin_metadata
        )
        
        # Create association: admin belongs to organization
        self.client.learn_association(
            source_id=admin_concept_id,
            target_id=org_concept_id,
            assoc_type=AssociationType.BelongsToOrganization,
            confidence=1.0
        )
        
        return {
            "organization_id": org_concept_id,
            "admin_id": admin_concept_id,
            "hospital_id": hospital_id
        }
    
    def add_hospital_staff(self, hospital_id: str, staff_info: Dict) -> str:
        """Add staff member to specific hospital."""
        
        staff_metadata = {
            "concept_type": ConceptType.User,
            "organization_id": hospital_id,
            "created_by": staff_info.get("created_by", "admin"),
            "attributes": {
                "email": staff_info["email"],
                "role": staff_info["role"],
                "department": staff_info["department"],
                "license_number": staff_info.get("license", ""),
                "specialization": staff_info.get("specialization", "")
            }
        }
        
        content = f"Medical Staff: {staff_info['name']} - {staff_info['role']}, {staff_info['department']}"
        
        staff_concept_id = self.client.learn_concept_with_metadata(
            content=content,
            metadata=staff_metadata
        )
        
        return staff_concept_id
    
    def create_patient_consultation(self, hospital_id: str, 
                                  consultation_data: Dict) -> str:
        """Create patient consultation with privacy protection."""
        
        # Patient data is encrypted/anonymized at application level
        consultation_metadata = {
            "concept_type": ConceptType.Conversation,
            "organization_id": hospital_id,
            "created_by": consultation_data["physician_id"],
            "attributes": {
                "consultation_type": consultation_data["type"],
                "department": consultation_data["department"],
                "patient_id_hash": consultation_data["patient_id_hash"],  # Encrypted
                "timestamp": str(time.time())
            }
        }
        
        # Content excludes PII
        content = f"Medical consultation - {consultation_data['type']} in {consultation_data['department']}"
        
        consultation_id = self.client.learn_concept_with_metadata(
            content=content,
            metadata=consultation_metadata
        )
        
        return consultation_id
    
    def search_hospital_resources(self, hospital_id: str, 
                                query: str) -> Dict:
        """Search both shared protocols and hospital-specific data."""
        
        # Search shared medical protocols (domain storage)
        domain_results = self.client.vector_search(
            query_text=query,
            k=10
            # No organization_id - searches domain storage
        )
        
        # Search hospital-specific data (user storage)
        hospital_results = self.client.vector_search(
            query_text=query,
            k=10,
            organization_id=hospital_id
        )
        
        return {
            "shared_protocols": domain_results,
            "hospital_specific": hospital_results,
            "total_results": len(domain_results) + len(hospital_results)
        }

# Usage
healthcare_system = HealthcareMultiTenantSystem(client)

# Add shared medical protocols
cardiac_protocol_id = healthcare_system.add_medical_protocol(
    protocol="STEMI patients should receive primary PCI within 90 minutes of first medical contact",
    category="cardiology"
)

stroke_protocol_id = healthcare_system.add_medical_protocol(
    protocol="Acute stroke patients eligible for tPA should receive treatment within 4.5 hours",
    category="neurology"
)

# Set up hospitals
st_marys = healthcare_system.create_hospital_organization(
    hospital_name="St. Mary's Medical Center",
    hospital_id="hospital-st-marys"
)

general_hospital = healthcare_system.create_hospital_organization(
    hospital_name="General Hospital",
    hospital_id="hospital-general"
)

# Add staff to specific hospitals
cardiologist_id = healthcare_system.add_hospital_staff(
    hospital_id="hospital-st-marys",
    staff_info={
        "name": "Dr. Sarah Johnson",
        "email": "s.johnson@stmarys.com",
        "role": "physician",
        "department": "cardiology",
        "specialization": "interventional_cardiology",
        "license": "MD-12345"
    }
)

# Create patient consultation
consultation_id = healthcare_system.create_patient_consultation(
    hospital_id="hospital-st-marys",
    consultation_data={
        "physician_id": cardiologist_id,
        "type": "acute_coronary_syndrome",
        "department": "emergency",
        "patient_id_hash": "encrypted-patient-abc123"
    }
)

# Search resources
search_results = healthcare_system.search_hospital_resources(
    hospital_id="hospital-st-marys",
    query="cardiac emergency protocols"
)

print(f"Found {search_results['total_results']} relevant resources:")
print(f"- Shared protocols: {len(search_results['shared_protocols'])}")
print(f"- Hospital-specific: {len(search_results['hospital_specific'])}")
```

### 2. Legal Research Multi-Tenancy

```python
class LegalMultiTenantSystem:
    def __init__(self, storage_client: StorageClient):
        self.client = storage_client
        
    def add_case_law(self, case_data: Dict) -> str:
        """Add case law to shared legal knowledge."""
        # Domain concept - available to all law firms
        content = f"Case: {case_data['name']}\n" \
                 f"Citation: {case_data['citation']}\n" \
                 f"Holding: {case_data['holding']}"
        
        concept_id = self.client.learn_concept_v2(
            content=content,
            options={
                "extract_associations": True,
                "min_association_confidence": 0.8
            }
        )
        
        return concept_id
    
    def create_law_firm(self, firm_name: str, firm_id: str, 
                       specializations: List[str]) -> str:
        """Create law firm organization."""
        
        firm_metadata = {
            "concept_type": ConceptType.Organization,
            "organization_id": firm_id,
            "attributes": {
                "org_name": firm_name,
                "org_type": "legal",
                "specializations": ",".join(specializations),
                "billing_tier": "professional"
            }
        }
        
        firm_concept_id = self.client.learn_concept_with_metadata(
            content=f"Law Firm: {firm_name} - Specializations: {', '.join(specializations)}",
            metadata=firm_metadata
        )
        
        return firm_concept_id
    
    def add_client_case(self, firm_id: str, case_info: Dict) -> str:
        """Add confidential client case."""
        
        case_metadata = {
            "concept_type": ConceptType.Conversation,  # Using conversation for case files
            "organization_id": firm_id,
            "created_by": case_info["attorney_id"],
            "attributes": {
                "case_type": case_info["type"],
                "practice_area": case_info["practice_area"],
                "client_id_hash": case_info["client_hash"],  # Encrypted client ID
                "case_status": case_info.get("status", "active"),
                "confidentiality": "attorney_client_privilege"
            }
        }
        
        # Content excludes sensitive information
        content = f"Legal case - {case_info['type']} in {case_info['practice_area']}"
        
        case_concept_id = self.client.learn_concept_with_metadata(
            content=content,
            metadata=case_metadata
        )
        
        return case_concept_id
    
    def research_legal_precedents(self, firm_id: str, legal_query: str) -> Dict:
        """Research both public case law and firm's cases."""
        
        # Search public case law (domain storage)
        public_cases = self.client.vector_search(
            query_text=legal_query,
            k=15
            # No organization filter - searches all case law
        )
        
        # Search firm's cases (user storage with privacy)
        firm_cases = self.client.vector_search(
            query_text=legal_query,
            k=10,
            organization_id=firm_id
        )
        
        return {
            "public_precedents": public_cases,
            "firm_cases": firm_cases,
            "research_scope": "confidential_included"
        }

# Usage
legal_system = LegalMultiTenantSystem(client)

# Add public case law
brown_v_board = legal_system.add_case_law({
    "name": "Brown v. Board of Education",
    "citation": "347 U.S. 483 (1954)",
    "holding": "Separate educational facilities are inherently unequal and violate Equal Protection Clause"
})

# Create law firms
corporate_firm_id = legal_system.create_law_firm(
    firm_name="Smith & Associates",
    firm_id="firm-smith-associates",
    specializations=["corporate", "securities", "mergers"]
)

civil_rights_firm_id = legal_system.create_law_firm(
    firm_name="Justice Legal Group", 
    firm_id="firm-justice-legal",
    specializations=["civil_rights", "employment", "discrimination"]
)

# Add confidential client cases
corporate_case = legal_system.add_client_case(
    firm_id="firm-smith-associates",
    case_info={
        "attorney_id": "attorney-001",
        "type": "merger_acquisition",
        "practice_area": "corporate",
        "client_hash": "encrypted-client-corp-123",
        "status": "active"
    }
)

civil_rights_case = legal_system.add_client_case(
    firm_id="firm-justice-legal",
    case_info={
        "attorney_id": "attorney-002", 
        "type": "employment_discrimination",
        "practice_area": "civil_rights",
        "client_hash": "encrypted-client-employee-456"
    }
)

# Legal research respects tenant boundaries
corporate_research = legal_system.research_legal_precedents(
    firm_id="firm-smith-associates",
    legal_query="corporate merger regulatory approval"
)

civil_rights_research = legal_system.research_legal_precedents(
    firm_id="firm-justice-legal", 
    legal_query="employment discrimination equal protection"
)

print("Corporate firm research:")
print(f"- Public precedents: {len(corporate_research['public_precedents'])}")
print(f"- Firm cases: {len(corporate_research['firm_cases'])}")

print("Civil rights firm research:")
print(f"- Public precedents: {len(civil_rights_research['public_precedents'])}")
print(f"- Firm cases: {len(civil_rights_research['firm_cases'])}")
```

## Security and Access Control

### Data Isolation Validation

```python
def validate_tenant_isolation(client: StorageClient, 
                            org1_id: str, org2_id: str) -> Dict:
    """Validate that tenant data is properly isolated."""
    
    isolation_test = {
        'org1_concepts': [],
        'org2_concepts': [],
        'cross_tenant_leaks': [],
        'isolation_verified': False
    }
    
    # Create test concepts for each organization
    for i in range(5):
        # Organization 1 concepts
        concept1_metadata = {
            "concept_type": ConceptType.Message,
            "organization_id": org1_id,
            "attributes": {"test_marker": f"org1_test_{i}"}
        }
        
        concept1_id = client.learn_concept_with_metadata(
            content=f"Organization 1 test concept {i}",
            metadata=concept1_metadata
        )
        isolation_test['org1_concepts'].append(concept1_id)
        
        # Organization 2 concepts
        concept2_metadata = {
            "concept_type": ConceptType.Message,
            "organization_id": org2_id,
            "attributes": {"test_marker": f"org2_test_{i}"}
        }
        
        concept2_id = client.learn_concept_with_metadata(
            content=f"Organization 2 test concept {i}",
            metadata=concept2_metadata
        )
        isolation_test['org2_concepts'].append(concept2_id)
    
    # Test isolation: search from org1 should not return org2 data
    org1_search = client.vector_search(
        query_text="organization test concept",
        k=20,
        organization_id=org1_id
    )
    
    org2_search = client.vector_search(
        query_text="organization test concept", 
        k=20,
        organization_id=org2_id
    )
    
    # Check for cross-tenant leaks
    org1_result_ids = {result['concept_id'] for result in org1_search}
    org2_result_ids = {result['concept_id'] for result in org2_search}
    
    # Org1 search should not return org2 concepts
    org1_leaks = org1_result_ids.intersection(set(isolation_test['org2_concepts']))
    org2_leaks = org2_result_ids.intersection(set(isolation_test['org1_concepts']))
    
    if org1_leaks:
        isolation_test['cross_tenant_leaks'].extend([
            f"Org1 search returned Org2 concept: {concept_id}" 
            for concept_id in org1_leaks
        ])
    
    if org2_leaks:
        isolation_test['cross_tenant_leaks'].extend([
            f"Org2 search returned Org1 concept: {concept_id}"
            for concept_id in org2_leaks
        ])
    
    isolation_test['isolation_verified'] = len(isolation_test['cross_tenant_leaks']) == 0
    
    return isolation_test

# Test tenant isolation
isolation_results = validate_tenant_isolation(
    client, 
    "hospital-st-marys", 
    "hospital-general"
)

print(f"Tenant Isolation Test:")
print(f"- Isolation verified: {'✅' if isolation_results['isolation_verified'] else '❌'}")
print(f"- Org1 concepts created: {len(isolation_results['org1_concepts'])}")
print(f"- Org2 concepts created: {len(isolation_results['org2_concepts'])}")

if isolation_results['cross_tenant_leaks']:
    print("⚠️  Cross-tenant leaks detected:")
    for leak in isolation_results['cross_tenant_leaks']:
        print(f"  - {leak}")
else:
    print("✅ No cross-tenant data leaks detected")
```

### Access Control Patterns

```python
class MultiTenantAccessControl:
    def __init__(self, storage_client: StorageClient):
        self.client = storage_client
        self.user_sessions = {}  # In production, use Redis or similar
        
    def authenticate_user(self, user_id: str, organization_id: str) -> Optional[Dict]:
        """Authenticate user and return session info."""
        
        # Query user concept with organization filter
        users = self.client.query_by_metadata(
            concept_type=ConceptType.User,
            organization_id=organization_id,
            attributes={"user_id": user_id},
            limit=1
        )
        
        if not users:
            return None
        
        user = users[0]
        
        # Create session
        session_id = f"session_{int(time.time())}_{user_id}"
        session_info = {
            "session_id": session_id,
            "user_id": user_id,
            "organization_id": organization_id,
            "permissions": self._get_user_permissions(user),
            "created_at": time.time(),
            "expires_at": time.time() + 3600  # 1 hour
        }
        
        self.user_sessions[session_id] = session_info
        return session_info
    
    def _get_user_permissions(self, user_concept: Dict) -> List[str]:
        """Extract user permissions from concept metadata."""
        attributes = user_concept.get('metadata', {}).get('attributes', {})
        role = attributes.get('role', 'user')
        
        permission_map = {
            'admin': ['read', 'write', 'delete', 'manage_users'],
            'physician': ['read', 'write', 'create_consultations'],
            'nurse': ['read', 'create_notes'],
            'user': ['read']
        }
        
        return permission_map.get(role, ['read'])
    
    def authorize_operation(self, session_id: str, operation: str, 
                          resource_org_id: str = None) -> bool:
        """Check if user is authorized for operation."""
        
        if session_id not in self.user_sessions:
            return False
        
        session = self.user_sessions[session_id]
        
        # Check session expiry
        if time.time() > session['expires_at']:
            del self.user_sessions[session_id]
            return False
        
        # Check organization boundary
        if resource_org_id and resource_org_id != session['organization_id']:
            return False
        
        # Check operation permission
        return operation in session['permissions']
    
    def secure_learn_concept(self, session_id: str, content: str, 
                           concept_type: ConceptType) -> Optional[str]:
        """Learn concept with access control."""
        
        if not self.authorize_operation(session_id, 'write'):
            raise PermissionError("User not authorized to create concepts")
        
        session = self.user_sessions[session_id]
        
        # Automatically set organization context
        if concept_type.is_user_storage_type():
            metadata = {
                "concept_type": concept_type,
                "organization_id": session['organization_id'],
                "created_by": session['user_id'],
            }
            
            return self.client.learn_concept_with_metadata(
                content=content,
                metadata=metadata
            )
        else:
            # Domain concept - check admin permission
            if not self.authorize_operation(session_id, 'manage_users'):
                raise PermissionError("Only admins can create domain concepts")
            
            return self.client.learn_concept_v2(content)
    
    def secure_vector_search(self, session_id: str, query: str, 
                           k: int = 10) -> List[Dict]:
        """Vector search with automatic tenant filtering."""
        
        if not self.authorize_operation(session_id, 'read'):
            raise PermissionError("User not authorized to search")
        
        session = self.user_sessions[session_id]
        
        # Search both domain and tenant-specific data
        results = []
        
        # Domain search (available to all)
        domain_results = self.client.vector_search(
            query_text=query,
            k=k // 2  # Split results between domain and tenant
        )
        results.extend(domain_results)
        
        # Tenant-specific search
        tenant_results = self.client.vector_search(
            query_text=query,
            k=k // 2,
            organization_id=session['organization_id']
        )
        results.extend(tenant_results)
        
        return results[:k]

# Usage with access control
access_control = MultiTenantAccessControl(client)

# Authenticate users
admin_session = access_control.authenticate_user(
    user_id="admin-001", 
    organization_id="hospital-st-marys"
)

physician_session = access_control.authenticate_user(
    user_id="dr-smith",
    organization_id="hospital-st-marys"  
)

if admin_session:
    print(f"Admin authenticated: {admin_session['permissions']}")
    
    # Admin can create domain concepts
    try:
        domain_concept = access_control.secure_learn_concept(
            session_id=admin_session['session_id'],
            content="New medical protocol for emergency care",
            concept_type=ConceptType.DomainConcept
        )
        print(f"Domain concept created: {domain_concept}")
    except PermissionError as e:
        print(f"Permission denied: {e}")

if physician_session:
    print(f"Physician authenticated: {physician_session['permissions']}")
    
    # Physician can create consultations
    consultation = access_control.secure_learn_concept(
        session_id=physician_session['session_id'],
        content="Patient consultation for cardiac symptoms",
        concept_type=ConceptType.Conversation
    )
    print(f"Consultation created: {consultation}")
    
    # Physician can search within their organization
    search_results = access_control.secure_vector_search(
        session_id=physician_session['session_id'],
        query="cardiac emergency protocols"
    )
    print(f"Found {len(search_results)} relevant protocols")
```

## Best Practices

### 1. Organization Design

- **Unique Organization IDs**: Use consistent, collision-resistant naming (e.g., `hospital-{hash}`, `firm-{uuid}`)
- **Hierarchical Organizations**: Consider parent-child relationships for large enterprises
- **Data Classification**: Clearly separate shared vs. tenant-specific knowledge

### 2. Security Considerations

- **Encryption**: Encrypt sensitive data before storing in concepts
- **Access Logs**: Log all cross-tenant operations for audit
- **Regular Validation**: Periodically test tenant isolation
- **Backup Isolation**: Ensure backups maintain tenant boundaries

### 3. Performance Optimization

- **Organization-Aware Caching**: Cache frequently accessed tenant data
- **Index Partitioning**: Consider separate vector indexes per tenant for large deployments
- **Query Optimization**: Use organization filters early in query pipeline

## Next Steps

- [**Vector Search**](./07-vector-search.md) - Tenant-aware search optimization
- [**Performance Guide**](./08-performance.md) - Multi-tenant scaling strategies
- [**Troubleshooting**](./09-troubleshooting.md) - Debugging tenant isolation issues

---

*Multi-tenancy in Sutra Storage provides enterprise-grade data isolation while maintaining the benefits of shared domain knowledge for AI reasoning applications.*