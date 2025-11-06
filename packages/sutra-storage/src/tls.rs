//! TLS configuration for secure TCP connections
//! 
//! Provides certificate loading, validation, and TLS acceptor creation.

#![allow(unexpected_cfgs)]  // dev-tools feature is optional

use anyhow::{anyhow, Result};
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use tokio_rustls::TlsAcceptor;

/// TLS configuration builder
pub struct TlsConfigBuilder {
    cert_path: Option<String>,
    key_path: Option<String>,
    client_auth_required: bool,
}

impl TlsConfigBuilder {
    pub fn new() -> Self {
        Self {
            cert_path: None,
            key_path: None,
            client_auth_required: false,
        }
    }
    
    /// Set certificate path
    pub fn cert_path(mut self, path: String) -> Self {
        self.cert_path = Some(path);
        self
    }
    
    /// Set private key path
    pub fn key_path(mut self, path: String) -> Self {
        self.key_path = Some(path);
        self
    }
    
    /// Require client certificate authentication (mTLS)
    pub fn require_client_auth(mut self, required: bool) -> Self {
        self.client_auth_required = required;
        self
    }
    
    /// Load TLS configuration from environment variables
    pub fn from_env() -> Result<Self> {
        let cert_path = std::env::var("SUTRA_TLS_CERT")
            .map_err(|_| anyhow!("SUTRA_TLS_CERT environment variable required"))?;
        
        let key_path = std::env::var("SUTRA_TLS_KEY")
            .map_err(|_| anyhow!("SUTRA_TLS_KEY environment variable required"))?;
        
        let client_auth = std::env::var("SUTRA_TLS_CLIENT_AUTH")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);
        
        Ok(Self {
            cert_path: Some(cert_path),
            key_path: Some(key_path),
            client_auth_required: client_auth,
        })
    }
    
    /// Build TLS acceptor
    pub fn build(self) -> Result<TlsAcceptor> {
        let cert_path = self.cert_path
            .ok_or_else(|| anyhow!("Certificate path not set"))?;
        let key_path = self.key_path
            .ok_or_else(|| anyhow!("Private key path not set"))?;
        
        // Load certificates
        let certs = load_certs(&cert_path)?;
        let key = load_private_key(&key_path)?;
        
        // Build server config
        let config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(|e| anyhow!("TLS config error: {}", e))?;
        
        Ok(TlsAcceptor::from(Arc::new(config)))
    }
}

impl Default for TlsConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Load certificates from PEM file
fn load_certs(path: &str) -> Result<Vec<Certificate>> {
    let file = File::open(path)
        .map_err(|e| anyhow!("Failed to open certificate file {}: {}", path, e))?;
    let mut reader = BufReader::new(file);
    
    let certs = certs(&mut reader)
        .map_err(|e| anyhow!("Failed to parse certificates: {}", e))?
        .into_iter()
        .map(Certificate)
        .collect();
    
    Ok(certs)
}

/// Load private key from PEM file
fn load_private_key(path: &str) -> Result<PrivateKey> {
    let file = File::open(path)
        .map_err(|e| anyhow!("Failed to open private key file {}: {}", path, e))?;
    let mut reader = BufReader::new(file);
    
    let keys = pkcs8_private_keys(&mut reader)
        .map_err(|e| anyhow!("Failed to parse private key: {}", e))?;
    
    if keys.is_empty() {
        return Err(anyhow!("No private keys found in {}", path));
    }
    
    if keys.len() > 1 {
        log::warn!("Multiple private keys found, using first one");
    }
    
    Ok(PrivateKey(keys[0].clone()))
}

/// Check if TLS is enabled via environment
pub fn is_tls_enabled() -> bool {
    std::env::var("SUTRA_TLS_ENABLED")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap_or(false)
}

/// Generate self-signed certificate for development (requires openssl command)
#[cfg(feature = "dev-tools")]
pub fn generate_self_signed_cert(output_dir: &Path) -> Result<()> {
    use std::process::Command;
    
    std::fs::create_dir_all(output_dir)?;
    
    let cert_path = output_dir.join("cert.pem");
    let key_path = output_dir.join("key.pem");
    
    let output = Command::new("openssl")
        .args(&[
            "req", "-x509", "-newkey", "rsa:4096",
            "-keyout", key_path.to_str().unwrap(),
            "-out", cert_path.to_str().unwrap(),
            "-days", "365",
            "-nodes",
            "-subj", "/CN=localhost",
        ])
        .output()?;
    
    if !output.status.success() {
        return Err(anyhow!(
            "Failed to generate certificate: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    
    log::info!("✅ Generated self-signed certificate:");
    log::info!("   Cert: {}", cert_path.display());
    log::info!("   Key: {}", key_path.display());
    log::warn!("⚠️  Self-signed certificates should ONLY be used for development");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;
    
    const TEST_CERT: &str = r#"-----BEGIN CERTIFICATE-----
MIICljCCAX4CCQCKz8Vz9+FVoTANBgkqhkiG9w0BAQsFADANMQswCQYDVQQDDAJ0
ZTAeFw0yNDAxMDEwMDAwMDBaFw0yNTAxMDEwMDAwMDBaMA0xCzAJBgNVBAMMAml0
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAvMxPmH0tZbFsLz5qDxtx
-----END CERTIFICATE-----"#;
    
    const TEST_KEY: &str = r#"-----BEGIN PRIVATE KEY-----
MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQC8zE+YfS1lsWwv
-----END PRIVATE KEY-----"#;
    
    #[test]
    fn test_tls_config_from_env() {
        let dir = tempdir().unwrap();
        let cert_path = dir.path().join("cert.pem");
        let key_path = dir.path().join("key.pem");
        
        // Write test files
        std::fs::write(&cert_path, TEST_CERT).unwrap();
        std::fs::write(&key_path, TEST_KEY).unwrap();
        
        std::env::set_var("SUTRA_TLS_CERT", cert_path.to_str().unwrap());
        std::env::set_var("SUTRA_TLS_KEY", key_path.to_str().unwrap());
        
        let builder = TlsConfigBuilder::from_env();
        assert!(builder.is_ok());
    }
}
