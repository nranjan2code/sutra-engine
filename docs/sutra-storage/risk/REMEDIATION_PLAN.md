# Remediation Plan - Sutra Storage Security

**Plan Version**: 1.0  
**Creation Date**: October 28, 2025  
**Target Completion**: November 15, 2025  
**Owner**: Security Engineering Team

---

## Executive Summary

This remediation plan addresses critical security vulnerabilities identified in the Sutra AI storage system. The plan is structured in three phases prioritized by risk level and implementation complexity.

**Total Estimated Effort**: 80-120 hours  
**Critical Path**: 2 weeks for P0 items  
**Full Completion**: 4-6 weeks

---

## Phase 1: Critical Security Integration (P0)

**Timeline**: 2-3 days  
**Effort**: 8-12 hours  
**Risk**: CRITICAL

### REM-001: Enable Authentication in Storage Server

**Priority**: P0 - Critical  
**Effort**: 2 hours  
**Owner**: Storage Team

#### Current State
The `SecureStorageServer` wrapper exists but is not used by the main storage binary despite `SUTRA_SECURE_MODE=true` being set.

#### Implementation

**File**: `packages/sutra-storage/src/bin/storage_server.rs`

```rust
// Lines 89-100: Update secure mode detection
let secure_mode = std::env::var("SUTRA_SECURE_MODE")
    .map(|v| v.to_lowercase() == "true")
    .unwrap_or_else(|_| {
        // Default to secure in production builds
        #[cfg(debug_assertions)]
        { false }
        #[cfg(not(debug_assertions))]  
        { true }
    });

// Add production safety check
if !secure_mode && !cfg!(debug_assertions) {
    error!("❌ Production builds require SUTRA_SECURE_MODE=true");
    error!("   Set environment variable or use debug build for development");
    return Err("Security enforcement failed".into());
}
```

**Lines 191-210: Ensure secure server is used**

```rust
// Create server (secure or insecure based on mode)
let server = if secure_mode {
    info!("🔒 Creating secure storage server with authentication");
    
    // Validate auth manager was created successfully
    let auth_manager = auth_manager.ok_or("Authentication manager required in secure mode")?;
    
    // Create secure wrapper
    let insecure_server = StorageServer::new(storage).await;
    let secure_server = SecureStorageServer::new(insecure_server, Some(auth_manager))
        .await
        .map_err(|e| format!("Failed to create secure server: {}", e))?;
    
    Arc::new(secure_server) as Arc<dyn StorageServerTrait>
} else {
    warn!("⚠️  Starting INSECURE storage server (development only)");
    warn!("   Set SUTRA_SECURE_MODE=true for production deployment");
    
    let insecure_server = StorageServer::new(storage).await;
    Arc::new(insecure_server) as Arc<dyn StorageServerTrait>
};

info!("🚀 Starting storage server on {}", addr);
if let Err(e) = server.serve(addr).await {
    error!("Server error: {}", e);
    return Err(e.into());
}
```

#### Validation Steps

1. **Development Mode Test**:
```bash
# Should work without authentication
export SUTRA_SECURE_MODE=false
cargo run --bin storage-server

# Verify unauthenticated access works
nc localhost 50051
```

2. **Production Mode Test**:
```bash  
# Should require authentication
export SUTRA_SECURE_MODE=true
export SUTRA_AUTH_SECRET="development-secret-key-32-chars-here"
cargo run --release --bin storage-server

# Verify authentication is required
nc localhost 50051  # Should fail or require auth handshake
```

3. **Default Security Test**:
```bash
# Production build should default to secure
unset SUTRA_SECURE_MODE
cargo build --release
./target/release/storage-server  # Should require auth setup
```

### REM-002: Fix Rate Limiting Bypass

**Priority**: P0 - Critical  
**Effort**: 1 hour  
**Owner**: API Team

#### Current State
Rate limiting trusts `X-Forwarded-For` and `X-Real-IP` headers without validation, allowing complete bypass.

#### Implementation

**File**: `packages/sutra-api/sutra_api/middleware/rate_limit.py`

```python
# Lines 17-45: Update middleware configuration
class RateLimitMiddleware(BaseHTTPMiddleware):
    def __init__(
        self,
        app,
        default_limit: int = 60,
        window_seconds: int = 60,
        endpoint_limits: Dict[str, int] = None,
        trusted_proxies: list = None,
        behind_proxy: bool = False,
        require_proxy_validation: bool = True,  # NEW: Enforce proxy validation
    ):
        super().__init__(app)
        self.default_limit = default_limit
        self.window_seconds = window_seconds
        self.endpoint_limits = endpoint_limits or {}
        self.behind_proxy = behind_proxy
        self.require_proxy_validation = require_proxy_validation
        
        # Parse trusted proxy networks
        self.trusted_networks = []
        if trusted_proxies:
            for proxy in trusted_proxies:
                try:
                    import ipaddress
                    self.trusted_networks.append(ipaddress.ip_network(proxy))
                except ValueError:
                    logger.warning(f"Invalid proxy network format: {proxy}")
```

**Lines 90-110: Secure IP extraction**

```python
def _get_client_ip(self, request: Request) -> str:
    """Extract client IP with proper proxy validation"""
    
    # For direct connections, always use actual client IP
    if not self.behind_proxy:
        return request.client.host
    
    # For proxy deployments, validate proxy source
    proxy_ip = request.client.host
    
    # Check if request comes from trusted proxy
    if not self._is_trusted_proxy(proxy_ip):
        if self.require_proxy_validation:
            logger.warning(f"Untrusted proxy attempt from {proxy_ip}")
            return proxy_ip  # Use proxy IP for rate limiting
        else:
            # Fallback mode - log but allow
            logger.info(f"Using unvalidated proxy IP: {proxy_ip}")
    
    # Extract forwarded IP from trusted proxy
    forwarded = request.headers.get("X-Forwarded-For")
    if forwarded:
        # Take first IP in chain (original client)
        client_ip = forwarded.split(",")[0].strip()
        
        # Validate IP format
        if self._is_valid_ip(client_ip):
            return client_ip
        else:
            logger.warning(f"Invalid forwarded IP format: {client_ip}")
            return proxy_ip
    
    # No forwarded header - use proxy IP
    return proxy_ip

def _is_trusted_proxy(self, ip: str) -> bool:
    """Check if IP is in trusted proxy networks"""
    if not self.trusted_networks:
        return False
        
    try:
        import ipaddress
        client_ip = ipaddress.ip_address(ip)
        
        for network in self.trusted_networks:
            if client_ip in network:
                return True
    except ValueError:
        return False
    
    return False

def _is_valid_ip(self, ip: str) -> bool:
    """Validate IP address format"""
    try:
        import ipaddress
        ipaddress.ip_address(ip)
        return True
    except ValueError:
        return False
```

**File**: `packages/sutra-api/sutra_api/main.py` - Update middleware configuration

```python
# Lines 70-90: Configure rate limiting with trusted proxies
app.add_middleware(
    RateLimitMiddleware,
    default_limit=60,
    window_seconds=60,
    endpoint_limits={
        "/learn": settings.rate_limit_learn,
        "/learn/batch": settings.rate_limit_learn // 2,
        "/query": settings.rate_limit_query,
        "/search": settings.rate_limit_search,
    },
    # Configure trusted proxy networks
    trusted_proxies=[
        "10.0.0.0/8",      # Private networks
        "172.16.0.0/12", 
        "192.168.0.0/16",
        "127.0.0.0/8",     # Localhost
    ],
    behind_proxy=settings.behind_proxy,
    require_proxy_validation=True,  # Enforce proxy validation
)
```

#### Validation Steps

1. **Direct Connection Test**:
```bash
# Should use actual IP for rate limiting
for i in {1..20}; do
  curl -H "X-Forwarded-For: fake.$i.fake.fake" \
       http://localhost:8000/learn \
       -d '{"content":"test"}' 
done
# Should hit rate limit based on actual client IP
```

2. **Trusted Proxy Test**:
```bash
# Configure trusted proxy
export BEHIND_PROXY=true
export TRUSTED_PROXIES="192.168.1.0/24"

# Requests from trusted proxy should use forwarded IP
curl --interface 192.168.1.100 \
     -H "X-Forwarded-For: 203.0.113.1" \
     http://localhost:8000/learn
```

3. **Untrusted Proxy Test**:
```bash
# Requests from untrusted source should use proxy IP
curl --interface 203.0.113.200 \
     -H "X-Forwarded-For: fake.fake.fake.fake" \
     http://localhost:8000/learn
# Should rate limit based on 203.0.113.200, not fake IP
```

### REM-003: Environment Variable Security

**Priority**: P0 - Critical  
**Effort**: 1 hour  
**Owner**: DevOps Team

#### Implementation

**File**: `docker-compose.yml` or deployment configuration

```yaml
services:
  sutra-storage:
    environment:
      # Force secure mode in production
      - SUTRA_SECURE_MODE=true
      
      # Authentication configuration
      - SUTRA_AUTH_METHOD=hmac
      - SUTRA_AUTH_SECRET=${SUTRA_AUTH_SECRET}  # From external secret
      - SUTRA_TOKEN_TTL_SECONDS=3600
      
      # TLS configuration  
      - SUTRA_TLS_ENABLED=true
      - SUTRA_TLS_CERT=/certs/server.crt
      - SUTRA_TLS_KEY=/certs/server.key
      
      # Rate limiting configuration
      - BEHIND_PROXY=true
      - TRUSTED_PROXIES=10.0.0.0/8,172.16.0.0/12,192.168.0.0/16
```

**File**: `scripts/generate-auth-secret.sh`

```bash
#!/bin/bash
# Generate secure authentication secret

set -euo pipefail

SECRET_LENGTH=64
SECRET_FILE="${1:-/etc/sutra/auth-secret}"

# Generate cryptographically secure random secret
openssl rand -hex $SECRET_LENGTH > "$SECRET_FILE"

# Set restrictive permissions
chmod 600 "$SECRET_FILE" 
chown sutra:sutra "$SECRET_FILE"

echo "Generated authentication secret: $SECRET_FILE"
echo "Set environment variable: export SUTRA_AUTH_SECRET=\$(cat $SECRET_FILE)"
```

---

## Phase 2: Network Security Hardening (P1)

**Timeline**: 1 week  
**Effort**: 24-32 hours  
**Risk**: HIGH

### REM-004: Grid System Authentication

**Priority**: P1 - High  
**Effort**: 16 hours  
**Owner**: Grid Team

#### Current State
Grid master accepts agent registrations without authentication, allowing rogue nodes to join the cluster.

#### Implementation

**File**: `packages/sutra-grid-master/Cargo.toml` - Add JWT dependency

```toml
[dependencies]
jsonwebtoken = "8.3"
serde = { version = "1.0", features = ["derive"] }
chrono = { version = "0.4", features = ["serde"] }
```

**File**: `packages/sutra-grid-master/src/auth.rs` (NEW)

```rust
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::{anyhow, Result};

#[derive(Debug, Serialize, Deserialize)]
pub struct GridClaims {
    pub agent_id: String,
    pub cluster_id: String, 
    pub roles: Vec<String>,
    pub exp: u64,  // Expiration time
    pub iat: u64,  // Issued at
}

pub struct GridAuthManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    cluster_id: String,
    token_ttl: u64,
}

impl GridAuthManager {
    pub fn new(secret: &str, cluster_id: String) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            cluster_id,
            token_ttl: 3600 * 24, // 24 hours
        }
    }
    
    pub fn generate_agent_token(&self, agent_id: &str, roles: Vec<String>) -> Result<String> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        
        let claims = GridClaims {
            agent_id: agent_id.to_string(),
            cluster_id: self.cluster_id.clone(),
            roles,
            exp: now + self.token_ttl,
            iat: now,
        };
        
        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| anyhow!("Token generation failed: {}", e))
    }
    
    pub fn validate_agent_token(&self, token: &str) -> Result<GridClaims> {
        let validation = Validation::new(Algorithm::HS256);
        
        let token_data = decode::<GridClaims>(
            token,
            &self.decoding_key,
            &validation,
        ).map_err(|e| anyhow!("Token validation failed: {}", e))?;
        
        // Verify cluster ID matches
        if token_data.claims.cluster_id != self.cluster_id {
            return Err(anyhow!("Invalid cluster ID in token"));
        }
        
        Ok(token_data.claims)
    }
    
    pub fn is_agent_authorized(&self, claims: &GridClaims, required_role: &str) -> bool {
        claims.roles.contains(&required_role.to_string()) || 
        claims.roles.contains(&"admin".to_string())
    }
}
```

**File**: `packages/sutra-grid-master/src/main.rs` - Update registration handler

```rust
use crate::auth::{GridAuthManager, GridClaims};

struct GridMasterService {
    agents: Arc<RwLock<HashMap<String, MasterAgentRecord>>>,
    events: Option<EventEmitter>,
    auth_manager: Option<GridAuthManager>,  // NEW: Authentication
}

impl GridMasterService {
    fn new() -> Self {
        // Initialize auth manager from environment
        let auth_manager = Self::init_auth_manager();
        
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            events: None,
            auth_manager,
        }
    }
    
    fn init_auth_manager() -> Option<GridAuthManager> {
        match (
            std::env::var("GRID_AUTH_SECRET"),
            std::env::var("GRID_CLUSTER_ID")
        ) {
            (Ok(secret), Ok(cluster_id)) => {
                if secret.len() < 32 {
                    log::error!("GRID_AUTH_SECRET must be at least 32 characters");
                    return None;
                }
                
                Some(GridAuthManager::new(&secret, cluster_id))
            }
            _ => {
                log::warn!("Grid authentication disabled - set GRID_AUTH_SECRET and GRID_CLUSTER_ID");
                None
            }
        }
    }
}

// Update protocol to include authentication
async fn handle_register_agent(
    &self, 
    agent_record: AgentRecord,
    auth_token: Option<String>
) -> GridResponse {
    
    // Require authentication if configured
    if let Some(ref auth_manager) = self.auth_manager {
        let token = auth_token.ok_or("Authentication token required")?;
        
        let claims = match auth_manager.validate_agent_token(&token) {
            Ok(claims) => claims,
            Err(e) => {
                log::warn!("Agent authentication failed: {}", e);
                return GridResponse::Error {
                    message: "Authentication failed".to_string()
                };
            }
        };
        
        // Verify agent ID matches token
        if claims.agent_id != agent_record.agent_id {
            log::warn!("Agent ID mismatch: token={}, record={}", 
                      claims.agent_id, agent_record.agent_id);
            return GridResponse::Error {
                message: "Agent ID mismatch".to_string()
            };
        }
        
        // Verify agent has required role
        if !auth_manager.is_agent_authorized(&claims, "agent") {
            log::warn!("Agent {} lacks required permissions", claims.agent_id);
            return GridResponse::Error {
                message: "Insufficient permissions".to_string()
            };
        }
        
        log::info!("✅ Agent {} authenticated successfully", claims.agent_id);
    } else {
        log::warn!("⚠️ Agent {} registered without authentication (auth disabled)", 
                  agent_record.agent_id);
    }
    
    // Proceed with registration
    let mut agents = self.agents.write().await;
    agents.insert(agent_record.agent_id.clone(), MasterAgentRecord {
        record: agent_record.clone(),
        last_heartbeat: current_timestamp(),
        status: AgentStatus::Healthy,
        current_storage_nodes: 0,
        storage_nodes: Vec::new(),
    });
    
    GridResponse::RegisterSuccess {
        agent_id: agent_record.agent_id,
        assigned_role: "storage".to_string(),
    }
}
```

### REM-005: TLS Enforcement

**Priority**: P1 - High  
**Effort**: 8 hours  
**Owner**: Infrastructure Team

#### Implementation

**File**: `packages/sutra-storage/src/bin/storage_server.rs` - Enforce TLS in production

```rust
// Add TLS validation after auth manager initialization
if secure_mode {
    // Check TLS configuration
    let tls_enabled = std::env::var("SUTRA_TLS_ENABLED")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false);
    
    // Require TLS in production builds
    #[cfg(not(debug_assertions))]
    {
        if !tls_enabled {
            error!("❌ Production builds require TLS encryption");
            error!("   Set SUTRA_TLS_ENABLED=true and provide certificates");
            return Err("TLS enforcement failed".into());
        }
    }
    
    #[cfg(debug_assertions)]
    {
        if !tls_enabled {
            warn!("⚠️ TLS disabled in development mode");
            warn!("   Enable TLS for production deployment");
        }
    }
}
```

**File**: `scripts/generate-tls-certs.sh` (NEW)

```bash
#!/bin/bash
# Generate TLS certificates for development and testing

set -euo pipefail

CERT_DIR="${1:-/etc/sutra/certs}"
DAYS_VALID=365

# Create certificate directory
mkdir -p "$CERT_DIR"
cd "$CERT_DIR"

# Generate CA private key
openssl genrsa -out ca-key.pem 4096

# Generate CA certificate  
openssl req -new -x509 -days $DAYS_VALID -key ca-key.pem -out ca-cert.pem \
    -subj "/C=US/ST=CA/L=San Francisco/O=Sutra AI/CN=Sutra Root CA"

# Generate server private key
openssl genrsa -out server-key.pem 4096

# Generate server certificate signing request
openssl req -new -key server-key.pem -out server.csr \
    -subj "/C=US/ST=CA/L=San Francisco/O=Sutra AI/CN=sutra-storage"

# Generate server certificate
openssl x509 -req -days $DAYS_VALID -in server.csr \
    -CA ca-cert.pem -CAkey ca-key.pem -CAcreateserial \
    -out server-cert.pem

# Set secure permissions
chmod 600 *-key.pem
chmod 644 *-cert.pem

echo "TLS certificates generated in $CERT_DIR"
echo "Configure environment variables:"
echo "  SUTRA_TLS_CERT=$CERT_DIR/server-cert.pem"
echo "  SUTRA_TLS_KEY=$CERT_DIR/server-key.pem"
```

### REM-006: Protocol Message Validation

**Priority**: P1 - Medium  
**Effort**: 8 hours  
**Owner**: Protocol Team

#### Implementation

**File**: `packages/sutra-storage/src/tcp_server.rs` - Add message validation

```rust
use std::time::{Duration, Instant};
use tokio::time::timeout;

// Enhanced security constants
const MAX_MESSAGE_SIZE: u32 = 16 * 1024 * 1024;     // 16MB
const MAX_CONTENT_SIZE: usize = 10 * 1024 * 1024;   // 10MB  
const MAX_EMBEDDING_DIM: usize = 2048;              // Max embedding dimension
const MAX_NESTING_DEPTH: usize = 32;                // Prevent deep recursion
const DESERIALIZE_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_BATCH_SIZE: usize = 1000;                 // Max batch operations

async fn read_and_validate_message(stream: &mut TcpStream) -> Result<StorageRequest, std::io::Error> {
    // Read message length with timeout
    let len = timeout(
        Duration::from_secs(10),
        stream.read_u32()
    ).await
    .map_err(|_| std::io::Error::new(std::io::ErrorKind::TimedOut, "Read timeout"))?? as usize;
    
    // Validate message size
    if len == 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Empty message not allowed"
        ));
    }
    
    if len > MAX_MESSAGE_SIZE as usize {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Message too large: {} bytes (max: {})", len, MAX_MESSAGE_SIZE)
        ));
    }
    
    // Read message data
    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf).await?;
    
    // Deserialize with timeout and validation
    let request = timeout(
        DESERIALIZE_TIMEOUT,
        validate_and_deserialize(&buf)
    ).await
    .map_err(|_| std::io::Error::new(
        std::io::ErrorKind::TimedOut, 
        "Deserialization timeout"
    ))??;
    
    Ok(request)
}

async fn validate_and_deserialize(data: &[u8]) -> Result<StorageRequest, std::io::Error> {
    // Basic MessagePack structure validation
    if data.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Empty payload"
        ));
    }
    
    // Deserialize with error handling
    let request: StorageRequest = rmp_serde::from_slice(data)
        .map_err(|e| std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Deserialization failed: {}", e)
        ))?;
    
    // Validate request content
    validate_request_content(&request)?;
    
    Ok(request)
}

fn validate_request_content(request: &StorageRequest) -> Result<(), std::io::Error> {
    match request {
        StorageRequest::LearnConceptV2 { content, embedding, .. } => {
            // Validate content size
            if content.len() > MAX_CONTENT_SIZE {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Content too large: {} bytes (max: {})", 
                           content.len(), MAX_CONTENT_SIZE)
                ));
            }
            
            // Validate embedding dimension
            if let Some(emb) = embedding {
                if emb.len() > MAX_EMBEDDING_DIM {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Embedding dimension too large: {} (max: {})",
                               emb.len(), MAX_EMBEDDING_DIM)
                    ));
                }
            }
        }
        
        StorageRequest::BatchLearnConcepts { concepts } => {
            if concepts.len() > MAX_BATCH_SIZE {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Batch too large: {} concepts (max: {})",
                           concepts.len(), MAX_BATCH_SIZE)
                ));
            }
            
            // Validate each concept in batch
            for concept in concepts {
                if concept.content.len() > MAX_CONTENT_SIZE {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Batch contains oversized content"
                    ));
                }
            }
        }
        
        _ => {} // Other request types - add validation as needed
    }
    
    Ok(())
}
```

---

## Phase 3: Advanced Security Features (P2)

**Timeline**: 3-4 weeks  
**Effort**: 48-64 hours  
**Risk**: MEDIUM

### REM-007: Plugin Security Framework

**Priority**: P2 - Medium  
**Effort**: 24 hours  
**Owner**: Bulk Ingestion Team

#### Implementation Overview

1. **Plugin Signature Verification** - Cryptographic signatures for all plugins
2. **Sandboxing** - Container-based isolation for plugin execution  
3. **Permission System** - Granular permissions for plugin capabilities
4. **Resource Limits** - CPU, memory, and I/O limits for plugins

#### Plugin Manifest System

**File**: `packages/sutra-bulk-ingester/src/plugin_manifest.rs` (NEW)

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use sha2::{Sha256, Digest};

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    
    // Security metadata
    pub checksum: String,      // SHA256 of plugin code
    pub signature: String,     // Cryptographic signature
    pub permissions: Vec<Permission>,
    
    // Resource limits
    pub limits: ResourceLimits,
    
    // Dependencies
    pub dependencies: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Permission {
    FileRead { paths: Vec<String> },
    FileWrite { paths: Vec<String> },
    NetworkAccess { domains: Vec<String> },
    SystemCall { calls: Vec<String> },
    EnvironmentAccess { variables: Vec<String> },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_mb: u64,
    pub max_cpu_percent: u8,
    pub max_execution_time_sec: u64,
    pub max_file_size_mb: u64,
}

impl PluginManifest {
    pub fn verify_integrity(&self, plugin_data: &[u8]) -> Result<(), String> {
        // Calculate actual checksum
        let mut hasher = Sha256::new();
        hasher.update(plugin_data);
        let actual_checksum = format!("{:x}", hasher.finalize());
        
        // Verify checksum matches manifest
        if actual_checksum != self.checksum {
            return Err(format!("Checksum mismatch: expected {}, got {}", 
                              self.checksum, actual_checksum));
        }
        
        Ok(())
    }
    
    pub fn has_permission(&self, required: &Permission) -> bool {
        // Check if manifest grants required permission
        // Implementation depends on permission type matching logic
        self.permissions.iter().any(|p| permissions_match(p, required))
    }
}

fn permissions_match(granted: &Permission, required: &Permission) -> bool {
    match (granted, required) {
        (Permission::FileRead { paths: granted_paths }, 
         Permission::FileRead { paths: required_paths }) => {
            required_paths.iter().all(|req_path| 
                granted_paths.iter().any(|granted_path| 
                    path_matches(granted_path, req_path)))
        }
        // Similar logic for other permission types
        _ => false
    }
}

fn path_matches(pattern: &str, path: &str) -> bool {
    // Simple glob-style matching
    // In production, use proper glob library
    pattern.contains('*') || pattern == path
}
```

### REM-008: Security Monitoring System

**Priority**: P2 - Medium  
**Effort**: 16 hours  
**Owner**: DevOps Team

#### Implementation

**File**: `packages/sutra-storage/src/security_monitor.rs` (NEW)

```rust
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub timestamp: SystemTime,
    pub event_type: SecurityEventType,
    pub source_ip: String,
    pub user_id: Option<String>,
    pub details: HashMap<String, String>,
    pub severity: Severity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEventType {
    AuthenticationFailure,
    AuthorizationFailure, 
    RateLimitExceeded,
    InvalidRequest,
    SuspiciousActivity,
    PluginSecurityViolation,
    TLSHandshakeFailure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,  
    High,
    Critical,
}

pub struct SecurityMonitor {
    event_sender: mpsc::UnboundedSender<SecurityEvent>,
    alert_thresholds: HashMap<SecurityEventType, AlertThreshold>,
}

struct AlertThreshold {
    count: u32,
    window: Duration,
    action: AlertAction,
}

enum AlertAction {
    Log,
    Alert,
    Block,
    Shutdown,
}

impl SecurityMonitor {
    pub fn new() -> (Self, mpsc::UnboundedReceiver<SecurityEvent>) {
        let (sender, receiver) = mpsc::unbounded_channel();
        
        let mut thresholds = HashMap::new();
        thresholds.insert(
            SecurityEventType::AuthenticationFailure,
            AlertThreshold {
                count: 5,
                window: Duration::from_secs(300), // 5 failures in 5 minutes
                action: AlertAction::Block,
            }
        );
        
        thresholds.insert(
            SecurityEventType::RateLimitExceeded,
            AlertThreshold {
                count: 10,
                window: Duration::from_secs(60), // 10 violations in 1 minute
                action: AlertAction::Alert,
            }
        );
        
        let monitor = Self {
            event_sender: sender,
            alert_thresholds: thresholds,
        };
        
        (monitor, receiver)
    }
    
    pub fn report_event(&self, event: SecurityEvent) {
        let _ = self.event_sender.send(event);
    }
    
    pub fn report_auth_failure(&self, source_ip: String, details: HashMap<String, String>) {
        let event = SecurityEvent {
            timestamp: SystemTime::now(),
            event_type: SecurityEventType::AuthenticationFailure,
            source_ip,
            user_id: None,
            details,
            severity: Severity::Medium,
        };
        
        self.report_event(event);
    }
    
    pub fn report_rate_limit_exceeded(&self, source_ip: String, endpoint: String) {
        let mut details = HashMap::new();
        details.insert("endpoint".to_string(), endpoint);
        
        let event = SecurityEvent {
            timestamp: SystemTime::now(),
            event_type: SecurityEventType::RateLimitExceeded,
            source_ip,
            user_id: None,
            details,
            severity: Severity::Low,
        };
        
        self.report_event(event);
    }
}

// Security event processor
pub async fn process_security_events(
    mut receiver: mpsc::UnboundedReceiver<SecurityEvent>
) {
    let mut event_history: Vec<SecurityEvent> = Vec::new();
    
    while let Some(event) = receiver.recv().await {
        // Store event
        event_history.push(event.clone());
        
        // Clean old events (keep last 1000)
        if event_history.len() > 1000 {
            event_history.drain(0..100);
        }
        
        // Process event
        match event.severity {
            Severity::Critical => {
                eprintln!("🚨 CRITICAL SECURITY EVENT: {:?}", event);
                // Send to SIEM, trigger alerts, etc.
            }
            Severity::High => {
                eprintln!("⚠️ HIGH SECURITY EVENT: {:?}", event);
                // Log and alert
            }
            Severity::Medium | Severity::Low => {
                println!("📝 Security event: {:?}", event);
                // Log only
            }
        }
        
        // Check for attack patterns
        detect_attack_patterns(&event_history);
    }
}

fn detect_attack_patterns(events: &[SecurityEvent]) {
    // Look for suspicious patterns in recent events
    let recent_events: Vec<_> = events.iter()
        .filter(|e| e.timestamp.elapsed().unwrap_or(Duration::ZERO) < Duration::from_secs(300))
        .collect();
    
    // Example: Multiple auth failures from same IP
    let mut ip_failures: HashMap<String, u32> = HashMap::new();
    for event in &recent_events {
        if matches!(event.event_type, SecurityEventType::AuthenticationFailure) {
            *ip_failures.entry(event.source_ip.clone()).or_insert(0) += 1;
        }
    }
    
    for (ip, count) in ip_failures {
        if count >= 5 {
            eprintln!("🚨 ATTACK DETECTED: {} auth failures from {}", count, ip);
            // Trigger automated response (IP blocking, etc.)
        }
    }
}
```

### REM-009: Zero-Trust Architecture

**Priority**: P2 - Low  
**Effort**: 8 hours  
**Owner**: Architecture Team

#### Implementation Outline

1. **Service-to-Service Authentication** - All internal services require authentication
2. **Mutual TLS** - Cryptographic identity verification between services
3. **Network Segmentation** - Isolated networks with controlled access
4. **Least Privilege** - Minimal required permissions for each service

---

## Testing and Validation

### Security Test Suite

**File**: `tests/security/test_authentication.py` (NEW)

```python
import pytest
import socket
import jwt
import time
from datetime import datetime, timedelta

class TestAuthenticationSecurity:
    """Test suite for authentication security fixes"""
    
    def test_unauthenticated_access_blocked(self):
        """Verify storage server blocks unauthenticated access"""
        with pytest.raises(ConnectionRefusedError):
            sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            sock.connect(('localhost', 50051))
            sock.send(b"test message")
    
    def test_invalid_token_rejected(self):
        """Verify invalid authentication tokens are rejected"""
        # Test with malformed token
        # Test with expired token  
        # Test with wrong signature
        pass
    
    def test_rate_limiting_enforced(self):
        """Verify rate limiting cannot be bypassed"""
        responses = []
        
        # Try to bypass with spoofed headers
        for i in range(100):
            # Each request with different spoofed IP
            headers = {'X-Forwarded-For': f'fake.{i}.{i}.{i}'}
            
            resp = requests.post(
                'http://localhost:8000/learn',
                headers=headers,
                json={'content': f'test content {i}'}
            )
            responses.append(resp.status_code)
            
            # Break early if we hit rate limit (expected behavior)
            if resp.status_code == 429:
                break
        
        # Should have 429 responses after limit exceeded
        assert 429 in responses, "Rate limiting bypass detected"
    
    def test_tls_enforced(self):
        """Verify TLS is required in production mode"""
        # Test that plain TCP connections are rejected when TLS is enabled
        pass

class TestGridSecurity:
    """Test suite for grid system security"""
    
    def test_agent_authentication_required(self):
        """Verify grid agents must authenticate"""
        # Try to register agent without token
        # Verify registration is rejected
        pass
    
    def test_invalid_agent_token_rejected(self):
        """Verify invalid agent tokens are rejected"""  
        # Test with expired token
        # Test with wrong cluster ID
        # Test with insufficient permissions
        pass

class TestProtocolSecurity:
    """Test suite for protocol security"""
    
    def test_message_size_limits(self):
        """Verify message size limits are enforced"""
        # Try to send oversized message
        # Verify connection is terminated
        pass
    
    def test_deserialization_timeout(self):
        """Verify deserialization has timeout protection"""
        # Send complex nested structure  
        # Verify request times out appropriately
        pass
```

### Continuous Security Testing

**File**: `.github/workflows/security-tests.yml` (NEW)

```yaml
name: Security Tests

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
  schedule:
    - cron: '0 2 * * *'  # Daily security tests

jobs:
  security-tests:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Setup Python
      uses: actions/setup-python@v4
      with:
        python-version: '3.11'
    
    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
    
    - name: Install security testing tools
      run: |
        pip install pytest requests PyJWT
        cargo install cargo-audit
    
    - name: Run security unit tests
      run: pytest tests/security/ -v
    
    - name: Security audit for Rust dependencies
      run: cargo audit
    
    - name: Test authentication enforcement
      run: |
        export SUTRA_SECURE_MODE=true
        export SUTRA_AUTH_SECRET="test-secret-key-32-characters-long"
        ./scripts/test-security-enforcement.sh
    
    - name: Test rate limiting  
      run: ./scripts/test-rate-limiting.sh
    
    - name: Generate security report
      run: |
        python scripts/generate-security-report.py \
          --output security-report.json
    
    - name: Upload security report
      uses: actions/upload-artifact@v3
      with:
        name: security-report
        path: security-report.json
```

---

## Rollback Plan

### Phase 1 Rollback (Authentication Issues)

If authentication causes service disruption:

1. **Emergency Disable**:
```bash
# Temporary disable security for critical operations
export SUTRA_SECURE_MODE=false
systemctl restart sutra-storage
```

2. **Partial Rollback**:
```bash
# Revert to previous binary
cp /backup/storage-server /usr/bin/storage-server
systemctl restart sutra-storage
```

### Phase 2 Rollback (Rate Limiting Issues)

If rate limiting blocks legitimate traffic:

1. **Increase Limits**:
```python
# Temporarily increase rate limits
app.add_middleware(
    RateLimitMiddleware,
    default_limit=1000,  # Increased from 60
    # ...
)
```

2. **Disable Rate Limiting**:
```python
# Comment out rate limiting middleware
# app.add_middleware(RateLimitMiddleware, ...)
```

---

## Success Metrics

### Security Metrics

| Metric | Current | Target | Measurement |
|--------|---------|---------|-------------|
| Authentication Coverage | 0% | 100% | Services requiring auth |
| Rate Limit Bypass Rate | 100% | 0% | Successful bypass attempts |
| Unauthorized Grid Access | Unlimited | 0 | Rogue agent registrations |
| Security Event Detection | None | Real-time | MTTR for security incidents |

### Performance Metrics

| Metric | Baseline | Acceptable Impact | Measurement |
|--------|----------|-------------------|-------------|
| Authentication Latency | N/A | <10ms | Per-request overhead |
| TLS Handshake Time | N/A | <100ms | Connection establishment |
| Rate Limiting Overhead | N/A | <1ms | Per-request processing |
| Memory Usage | Current | <5% increase | RSS memory consumption |

### Compliance Metrics

| Standard | Current Status | Target | Evidence |
|----------|----------------|---------|----------|
| SOC 2 Type II | Non-compliant | Compliant | Control implementation |
| GDPR Article 32 | Non-compliant | Compliant | Technical measures |
| NIST CSF | Partial | Complete | Framework alignment |
| ISO 27001 | Non-compliant | Compliant | Control objectives |

---

## Communication Plan

### Stakeholder Updates

#### Week 1 (Phase 1 Implementation)
- **Daily**: Engineering team standups
- **Wednesday**: Security team review
- **Friday**: Executive summary to leadership

#### Week 2-3 (Phase 2 Implementation)  
- **Weekly**: Progress report to stakeholders
- **Bi-weekly**: Customer advisory on security improvements

#### Week 4-6 (Phase 3 Implementation)
- **Monthly**: Compliance team review
- **Quarterly**: Board-level security briefing

### Documentation Updates

1. **Security Configuration Guide** - Updated with new settings
2. **Deployment Documentation** - Include security requirements
3. **API Documentation** - Document authentication requirements  
4. **Troubleshooting Guide** - Common security configuration issues

---

## Conclusion

This remediation plan addresses critical security vulnerabilities through a phased approach prioritized by risk level. The plan focuses on:

1. **Quick wins** - Enabling existing security code (Phase 1)
2. **Risk reduction** - Fixing authentication and network security (Phase 2)  
3. **Defense in depth** - Advanced security features (Phase 3)

**Critical Success Factor**: Phase 1 implementation is essential and can be completed in 2-3 days with minimal disruption. The existing security infrastructure just needs to be properly integrated and enabled.

**Next Actions**:
1. Review and approve this remediation plan
2. Assign team members to each remediation item
3. Begin Phase 1 implementation immediately
4. Schedule security review meetings
5. Prepare rollback procedures

---
**Document Version**: 1.0  
**Next Review**: November 1, 2025  
**Approval Required**: Security Team Lead, Engineering Manager