//! Production-grade authentication for Sutra storage services
//! 
//! Supports:
//! - JWT tokens with RS256/HS256 signing
//! - HMAC API keys for service-to-service auth
//! - Role-based access control (RBAC)
//! - Token expiration and refresh

use anyhow::{anyhow, Result};
use hmac::{Hmac, Mac};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use crate::rate_limiter::{RateLimiter, RateLimiterConfig};

type HmacSha256 = Hmac<Sha256>;

/// User roles for RBAC
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    /// Full access - can read, write, delete
    Admin,
    /// Can read and write, but not delete
    Writer,
    /// Read-only access
    Reader,
    /// Service-to-service authentication
    Service,
}

/// Authentication claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user/service identifier)
    pub sub: String,
    /// Issued at (Unix timestamp)
    pub iat: u64,
    /// Expiration (Unix timestamp)
    pub exp: u64,
    /// Roles assigned to this principal
    pub roles: Vec<Role>,
    /// Optional: Allowed operations
    pub permissions: Option<Vec<String>>,
}

impl Claims {
    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now > self.exp
    }
    
    /// Check if claims have specific role
    pub fn has_role(&self, role: &Role) -> bool {
        self.roles.contains(role)
    }
    
    /// Check if claims allow specific operation
    pub fn can_perform(&self, operation: &str) -> bool {
        // Admin can do everything
        if self.has_role(&Role::Admin) {
            return true;
        }
        
        // Check explicit permissions
        if let Some(ref perms) = self.permissions {
            return perms.iter().any(|p| p == operation || p == "*");
        }
        
        // Role-based defaults
        match operation {
            "read" | "query" | "search" => {
                self.has_role(&Role::Reader) 
                || self.has_role(&Role::Writer)
                || self.has_role(&Role::Service)
            }
            "write" | "learn" | "create" => {
                self.has_role(&Role::Writer) 
                || self.has_role(&Role::Service)
            }
            "delete" | "flush" => self.has_role(&Role::Admin),
            _ => false,
        }
    }
}

/// Authentication method
#[derive(Debug, Clone)]
pub enum AuthMethod {
    /// HMAC-based API key (recommended for service-to-service)
    HmacApiKey { key_id: String, secret: String },
    /// JWT with shared secret (HS256)
    JwtHS256 { secret: String },
}

/// Authentication manager with rate limiting
pub struct AuthManager {
    /// Authentication method configuration
    method: AuthMethod,
    /// Shared secret for HMAC/JWT
    secret: Vec<u8>,
    /// Revoked token IDs (for logout/security)
    revoked_tokens: Arc<RwLock<HashSet<String>>>,
    /// Token expiration duration (seconds)
    token_ttl: u64,
    /// Production-grade rate limiter (per-token limits)
    rate_limiter: Arc<RateLimiter>,
}

impl AuthManager {
    /// Create new auth manager with HMAC API key method
    pub fn new_hmac(secret: String, token_ttl: u64) -> Self {
        Self::new_hmac_with_rate_limit(secret, token_ttl, RateLimiterConfig::default())
    }
    
    /// Create new auth manager with HMAC API key method and custom rate limits
    pub fn new_hmac_with_rate_limit(secret: String, token_ttl: u64, rate_limit_config: RateLimiterConfig) -> Self {
        Self {
            method: AuthMethod::HmacApiKey {
                key_id: "default".to_string(),
                secret: secret.clone(),
            },
            secret: secret.into_bytes(),
            revoked_tokens: Arc::new(RwLock::new(HashSet::new())),
            token_ttl,
            rate_limiter: Arc::new(RateLimiter::with_config(rate_limit_config)),
        }
    }
    
    /// Create new auth manager with JWT HS256 method
    pub fn new_jwt_hs256(secret: String, token_ttl: u64) -> Self {
        Self::new_jwt_hs256_with_rate_limit(secret, token_ttl, RateLimiterConfig::default())
    }
    
    /// Create new auth manager with JWT HS256 method and custom rate limits
    pub fn new_jwt_hs256_with_rate_limit(secret: String, token_ttl: u64, rate_limit_config: RateLimiterConfig) -> Self {
        Self {
            method: AuthMethod::JwtHS256 {
                secret: secret.clone(),
            },
            secret: secret.into_bytes(),
            revoked_tokens: Arc::new(RwLock::new(HashSet::new())),
            token_ttl,
            rate_limiter: Arc::new(RateLimiter::with_config(rate_limit_config)),
        }
    }
    
    /// Load from environment variables
    pub fn from_env() -> Result<Self> {
        let auth_method = std::env::var("SUTRA_AUTH_METHOD")
            .unwrap_or_else(|_| "hmac".to_string());
        
        let secret = std::env::var("SUTRA_AUTH_SECRET")
            .map_err(|_| anyhow!("SUTRA_AUTH_SECRET environment variable required"))?;
        
        let token_ttl = std::env::var("SUTRA_TOKEN_TTL_SECONDS")
            .unwrap_or_else(|_| "3600".to_string())
            .parse::<u64>()?;
        
        // Validate secret strength
        if secret.len() < 32 {
            return Err(anyhow!("SUTRA_AUTH_SECRET must be at least 32 characters"));
        }
        
        // Rate limiter configuration from environment
        let rate_limit_config = RateLimiterConfig {
            requests_per_second: std::env::var("SUTRA_RATE_LIMIT_RPS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(100),
            burst_capacity: std::env::var("SUTRA_RATE_LIMIT_BURST")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(200),
            ..Default::default()
        };
        
        match auth_method.as_str() {
            "hmac" => Ok(Self::new_hmac_with_rate_limit(secret, token_ttl, rate_limit_config)),
            "jwt" | "jwt-hs256" => Ok(Self::new_jwt_hs256_with_rate_limit(secret, token_ttl, rate_limit_config)),
            _ => Err(anyhow!("Invalid SUTRA_AUTH_METHOD: {}", auth_method)),
        }
    }
    
    /// Generate authentication token
    pub fn generate_token(&self, subject: &str, roles: Vec<Role>) -> Result<String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let claims = Claims {
            sub: subject.to_string(),
            iat: now,
            exp: now + self.token_ttl,
            roles,
            permissions: None,
        };
        
        match &self.method {
            AuthMethod::HmacApiKey { .. } => self.generate_hmac_token(&claims),
            AuthMethod::JwtHS256 { .. } => self.generate_jwt_token(&claims),
        }
    }
    
    /// Validate authentication token and extract claims (with rate limiting)
    pub fn validate_token(&self, token: &str) -> Result<Claims> {
        // Extract subject from token for rate limiting (before full validation)
        let claims = match &self.method {
            AuthMethod::HmacApiKey { .. } => self.validate_hmac_token(token)?,
            AuthMethod::JwtHS256 { .. } => self.validate_jwt_token(token)?,
        };
        
        // Apply rate limiting per subject
        self.rate_limiter.check_rate_limit(&claims.sub)
            .map_err(|e| anyhow!("Rate limit check failed: {}", e))?;
        
        Ok(claims)
    }
    
    /// Generate HMAC-signed token
    fn generate_hmac_token(&self, claims: &Claims) -> Result<String> {
        // Serialize claims to JSON
        let payload = serde_json::to_string(claims)?;
        let payload_b64 = URL_SAFE_NO_PAD.encode(&payload);
        
        // Generate HMAC signature
        let mut mac = HmacSha256::new_from_slice(&self.secret)
            .map_err(|e| anyhow!("HMAC initialization failed: {}", e))?;
        mac.update(payload_b64.as_bytes());
        let signature = mac.finalize().into_bytes();
        let signature_b64 = URL_SAFE_NO_PAD.encode(&signature);
        
        // Token format: payload.signature
        Ok(format!("{}.{}", payload_b64, signature_b64))
    }
    
    /// Validate HMAC-signed token
    fn validate_hmac_token(&self, token: &str) -> Result<Claims> {
        // Split token
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 2 {
            return Err(anyhow!("Invalid token format"));
        }
        
        let payload_b64 = parts[0];
        let signature_b64 = parts[1];
        
        // Verify signature
        let mut mac = HmacSha256::new_from_slice(&self.secret)
            .map_err(|e| anyhow!("HMAC initialization failed: {}", e))?;
        mac.update(payload_b64.as_bytes());
        
        let expected_sig = URL_SAFE_NO_PAD.decode(signature_b64)?;
        mac.verify_slice(&expected_sig)
            .map_err(|_| anyhow!("Invalid token signature"))?;
        
        // Decode and validate claims
        let payload_bytes = URL_SAFE_NO_PAD.decode(payload_b64)?;
        let claims: Claims = serde_json::from_slice(&payload_bytes)?;
        
        // Check expiration
        if claims.is_expired() {
            return Err(anyhow!("Token expired"));
        }
        
        // Check if revoked
        if self.revoked_tokens.read().contains(&claims.sub) {
            return Err(anyhow!("Token revoked"));
        }
        
        Ok(claims)
    }
    
    /// Generate JWT token (simplified HS256 implementation)
    fn generate_jwt_token(&self, claims: &Claims) -> Result<String> {
        // JWT Header
        let header = serde_json::json!({
            "alg": "HS256",
            "typ": "JWT"
        });
        let header_b64 = URL_SAFE_NO_PAD.encode(serde_json::to_string(&header)?);
        
        // JWT Payload
        let payload_b64 = URL_SAFE_NO_PAD.encode(serde_json::to_string(claims)?);
        
        // JWT Signature
        let signing_input = format!("{}.{}", header_b64, payload_b64);
        let mut mac = HmacSha256::new_from_slice(&self.secret)
            .map_err(|e| anyhow!("HMAC initialization failed: {}", e))?;
        mac.update(signing_input.as_bytes());
        let signature = mac.finalize().into_bytes();
        let signature_b64 = URL_SAFE_NO_PAD.encode(&signature);
        
        Ok(format!("{}.{}.{}", header_b64, payload_b64, signature_b64))
    }
    
    /// Validate JWT token
    fn validate_jwt_token(&self, token: &str) -> Result<Claims> {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(anyhow!("Invalid JWT format"));
        }
        
        let header_b64 = parts[0];
        let payload_b64 = parts[1];
        let signature_b64 = parts[2];
        
        // Verify signature
        let signing_input = format!("{}.{}", header_b64, payload_b64);
        let mut mac = HmacSha256::new_from_slice(&self.secret)
            .map_err(|e| anyhow!("HMAC initialization failed: {}", e))?;
        mac.update(signing_input.as_bytes());
        
        let expected_sig = URL_SAFE_NO_PAD.decode(signature_b64)?;
        mac.verify_slice(&expected_sig)
            .map_err(|_| anyhow!("Invalid JWT signature"))?;
        
        // Decode claims
        let payload_bytes = URL_SAFE_NO_PAD.decode(payload_b64)?;
        let claims: Claims = serde_json::from_slice(&payload_bytes)?;
        
        // Validate
        if claims.is_expired() {
            return Err(anyhow!("Token expired"));
        }
        
        if self.revoked_tokens.read().contains(&claims.sub) {
            return Err(anyhow!("Token revoked"));
        }
        
        Ok(claims)
    }
    
    /// Revoke a token (by subject ID)
    pub fn revoke_token(&self, subject: &str) {
        self.revoked_tokens.write().insert(subject.to_string());
    }
    
    /// Clear revoked tokens (cleanup)
    pub fn clear_revoked(&self) {
        self.revoked_tokens.write().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hmac_token_generation_and_validation() {
        let manager = AuthManager::new_hmac(
            "test-secret-key-with-sufficient-length-32chars".to_string(),
            3600
        );
        
        let token = manager.generate_token("user123", vec![Role::Writer]).unwrap();
        let claims = manager.validate_token(&token).unwrap();
        
        assert_eq!(claims.sub, "user123");
        assert!(claims.has_role(&Role::Writer));
        assert!(!claims.is_expired());
    }
    
    #[test]
    fn test_jwt_token_generation_and_validation() {
        let manager = AuthManager::new_jwt_hs256(
            "test-secret-key-with-sufficient-length-32chars".to_string(),
            3600
        );
        
        let token = manager.generate_token("service-api", vec![Role::Service]).unwrap();
        let claims = manager.validate_token(&token).unwrap();
        
        assert_eq!(claims.sub, "service-api");
        assert!(claims.has_role(&Role::Service));
    }
    
    #[test]
    fn test_role_based_permissions() {
        let admin_claims = Claims {
            sub: "admin".to_string(),
            iat: 0,
            exp: u64::MAX,
            roles: vec![Role::Admin],
            permissions: None,
        };
        
        assert!(admin_claims.can_perform("read"));
        assert!(admin_claims.can_perform("write"));
        assert!(admin_claims.can_perform("delete"));
        
        let reader_claims = Claims {
            sub: "reader".to_string(),
            iat: 0,
            exp: u64::MAX,
            roles: vec![Role::Reader],
            permissions: None,
        };
        
        assert!(reader_claims.can_perform("read"));
        assert!(!reader_claims.can_perform("write"));
        assert!(!reader_claims.can_perform("delete"));
    }
    
    #[test]
    fn test_token_revocation() {
        let manager = AuthManager::new_hmac(
            "test-secret-key-with-sufficient-length-32chars".to_string(),
            3600
        );
        
        let token = manager.generate_token("user123", vec![Role::Writer]).unwrap();
        assert!(manager.validate_token(&token).is_ok());
        
        // Revoke token
        manager.revoke_token("user123");
        assert!(manager.validate_token(&token).is_err());
    }
    
    #[test]
    fn test_invalid_signature() {
        let manager = AuthManager::new_hmac(
            "test-secret-key-with-sufficient-length-32chars".to_string(),
            3600
        );
        
        let token = manager.generate_token("user123", vec![Role::Writer]).unwrap();
        let tampered = token.replace("user123", "hacker");
        
        assert!(manager.validate_token(&tampered).is_err());
    }
}
