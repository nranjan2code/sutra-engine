//! Secure TCP server with authentication and TLS encryption
//! 
//! Wraps the storage server with production-grade security:
//! - HMAC/JWT authentication
//! - TLS 1.3 encryption
//! - Role-based access control
//! - Audit logging

use crate::auth::{AuthManager, Claims};
use crate::tls::{TlsConfigBuilder, is_tls_enabled};
use crate::tcp_server::{StorageRequest, StorageResponse, StorageServer};
use anyhow::{anyhow, Result};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::signal;
use tokio_rustls::server::TlsStream;
use tracing::{info, warn, error};

const AUTH_TOKEN_SIZE_LIMIT: usize = 4096; // Max auth token size

/// Secure storage server with authentication and TLS
pub struct SecureStorageServer {
    /// Underlying storage server
    inner: Arc<StorageServer>,
    /// Authentication manager
    auth_manager: Option<Arc<AuthManager>>,
    /// TLS acceptor (optional)
    tls_acceptor: Option<tokio_rustls::TlsAcceptor>,
}

impl SecureStorageServer {
    /// Create secure server with authentication
    pub async fn new(
        server: StorageServer,
        auth_manager: Option<AuthManager>,
    ) -> Result<Self> {
        let tls_acceptor = if is_tls_enabled() {
            info!("🔒 TLS enabled - loading certificates...");
            let acceptor = TlsConfigBuilder::from_env()?.build()?;
            Some(acceptor)
        } else {
            warn!("⚠️  TLS disabled - traffic will be unencrypted");
            None
        };
        
        Ok(Self {
            inner: Arc::new(server),
            auth_manager: auth_manager.map(Arc::new),
            tls_acceptor,
        })
    }
    
    /// Start secure TCP server
    pub async fn serve(self: Arc<Self>, addr: SocketAddr) -> std::io::Result<()> {
        let listener = TcpListener::bind(addr).await?;
        
        info!("🚀 Secure storage server listening on {}", addr);
        if self.auth_manager.is_some() {
            info!("   ✅ Authentication: ENABLED");
        } else {
            warn!("   ⚠️  Authentication: DISABLED");
        }
        if self.tls_acceptor.is_some() {
            info!("   ✅ TLS Encryption: ENABLED");
        } else {
            warn!("   ⚠️  TLS Encryption: DISABLED");
        }
        
        // Graceful shutdown handler
        let shutdown = signal::ctrl_c();
        tokio::pin!(shutdown);
        
        loop {
            tokio::select! {
                result = listener.accept() => {
                    match result {
                        Ok((stream, peer_addr)) => {
                            let server = self.clone();
                            tokio::spawn(async move {
                                if let Err(e) = server.handle_client(stream, peer_addr).await {
                                    error!("Client error ({}): {}", peer_addr, e);
                                }
                            });
                        }
                        Err(e) => {
                            error!("Accept error: {}", e);
                        }
                    }
                }
                _ = &mut shutdown => {
                    info!("Shutdown signal received");
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    /// Handle single client connection with TLS and authentication
    async fn handle_client(
        &self,
        stream: TcpStream,
        peer_addr: SocketAddr,
    ) -> Result<()> {
        info!("Client connecting: {}", peer_addr);
        
        // TLS handshake if enabled
        match &self.tls_acceptor {
            Some(acceptor) => {
                let tls_stream = acceptor.accept(stream).await
                    .map_err(|e| anyhow!("TLS handshake failed: {}", e))?;
                info!("✅ TLS handshake complete: {}", peer_addr);
                self.handle_authenticated_client(tls_stream, peer_addr).await
            }
            None => {
                self.handle_plain_client(stream, peer_addr).await
            }
        }
    }
    
    /// Handle TLS-encrypted client
    async fn handle_authenticated_client(
        &self,
        mut stream: TlsStream<TcpStream>,
        peer_addr: SocketAddr,
    ) -> Result<()> {
        stream.get_mut().0.set_nodelay(true)?;
        
        // 1. Authentication handshake
        let claims = if let Some(ref _auth) = self.auth_manager {
            let auth_claims = self.perform_auth_handshake(&mut stream).await?;
            info!("✅ Authenticated: {} ({})", auth_claims.sub, peer_addr);
            Some(auth_claims)
        } else {
            warn!("⚠️  No authentication required ({})", peer_addr);
            None
        };
        
        // 2. Process authenticated requests
        self.process_requests(
            &mut stream,
            peer_addr,
            claims.as_ref()
        ).await?;
        
        Ok(())
    }
    
    /// Handle plain TCP client (no TLS)
    async fn handle_plain_client(
        &self,
        mut stream: TcpStream,
        peer_addr: SocketAddr,
    ) -> Result<()> {
        stream.set_nodelay(true)?;
        
        // Authentication handshake
        let claims = if let Some(ref _auth) = self.auth_manager {
            let auth_claims = self.perform_auth_handshake(&mut stream).await?;
            info!("✅ Authenticated: {} ({})", auth_claims.sub, peer_addr);
            Some(auth_claims)
        } else {
            warn!("⚠️  No authentication required ({})", peer_addr);
            None
        };
        
        // Process authenticated requests
        self.process_requests(
            &mut stream,
            peer_addr,
            claims.as_ref()
        ).await?;
        
        Ok(())
    }
    
    /// Perform authentication handshake
    async fn perform_auth_handshake<S>(
        &self,
        stream: &mut S
    ) -> Result<Claims>
    where
        S: AsyncReadExt + AsyncWriteExt + Unpin,
    {
        let auth = self.auth_manager.as_ref()
            .ok_or_else(|| anyhow!("Authentication not configured"))?;
        
        // Read auth token length
        let token_len = stream.read_u32().await? as usize;
        if token_len > AUTH_TOKEN_SIZE_LIMIT {
            return Err(anyhow!("Auth token too large: {}", token_len));
        }
        
        // Read auth token
        let mut token_bytes = vec![0u8; token_len];
        stream.read_exact(&mut token_bytes).await?;
        
        let token = String::from_utf8(token_bytes)
            .map_err(|e| anyhow!("Invalid token encoding: {}", e))?;
        
        // Validate token
        let claims = auth.validate_token(&token)
            .map_err(|e| anyhow!("Authentication failed: {}", e))?;
        
        // Send auth success response
        stream.write_u8(1).await?; // 1 = success
        stream.flush().await?;
        
        Ok(claims)
    }
    
    /// Process authenticated storage requests
    async fn process_requests<S>(
        &self,
        stream: &mut S,
        peer_addr: SocketAddr,
        claims: Option<&Claims>,
    ) -> Result<()>
    where
        S: AsyncReadExt + AsyncWriteExt + Unpin,
    {
        loop {
            // Read request length
            let len = match stream.read_u32().await {
                Ok(len) => len,
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    info!("Client disconnected: {}", peer_addr);
                    break;
                }
                Err(e) => return Err(e.into()),
            };
            
            // Validate message size
            const MAX_MESSAGE_SIZE: usize = 100 * 1024 * 1024;
            if len as usize > MAX_MESSAGE_SIZE {
                self.send_error(stream, "Message too large").await?;
                continue;
            }
            
            // Read request payload
            let mut buf = vec![0u8; len as usize];
            stream.read_exact(&mut buf).await?;
            
            // Deserialize request
            let request: StorageRequest = rmp_serde::from_slice(&buf)
                .map_err(|e| anyhow!("Deserialization failed: {}", e))?;
            
            // Authorization check
            if let Some(claims) = claims {
                if let Err(e) = self.authorize_request(claims, &request) {
                    warn!("Authorization failed: {} ({})", e, peer_addr);
                    self.send_error(stream, &format!("Unauthorized: {}", e)).await?;
                    continue;
                }
            }
            
            // Forward to inner server
            let response = self.inner.handle_request(request).await;
            
            // Audit log (if needed)
            if matches!(response, StorageResponse::Error { .. }) {
                warn!("Request failed: {:?} ({})", response, peer_addr);
            }
            
            // Send response
            let response_bytes = rmp_serde::to_vec_named(&response)
                .map_err(|e| anyhow!("Serialization failed: {}", e))?;
            
            stream.write_u32(response_bytes.len() as u32).await?;
            stream.write_all(&response_bytes).await?;
            stream.flush().await?;
        }
        
        Ok(())
    }
    
    /// Check if claims authorize request
    fn authorize_request(&self, claims: &Claims, request: &StorageRequest) -> Result<()> {
        let operation = match request {
            StorageRequest::LearnConceptV2 { .. } |
            StorageRequest::LearnBatch { .. } |
            StorageRequest::LearnWithEmbedding { .. } |
            StorageRequest::LearnConcept { .. } |
            StorageRequest::LearnAssociation { .. } => "write",
            
            StorageRequest::QueryConcept { .. } |
            StorageRequest::GetNeighbors { .. } |
            StorageRequest::FindPath { .. } |
            StorageRequest::FindPathSemantic { .. } |
            StorageRequest::FindTemporalChain { .. } |
            StorageRequest::FindCausalChain { .. } |
            StorageRequest::FindContradictions { .. } |
            StorageRequest::QueryBySemantic { .. } |
            StorageRequest::VectorSearch { .. } |
            StorageRequest::TextSearch { .. } |
            StorageRequest::ListRecent { .. } |
            StorageRequest::GetStats { .. } |
            StorageRequest::HealthCheck => "read",
            
            StorageRequest::DeleteConcept { .. } |
            StorageRequest::ClearCollection { .. } |
            StorageRequest::Flush => "delete",
        };
        
        if !claims.can_perform(operation) {
            return Err(anyhow!(
                "Insufficient permissions: {} requires '{}'", 
                claims.sub, 
                operation
            ));
        }
        
        Ok(())
    }
    
    /// Send error response
    async fn send_error<S>(
        &self,
        stream: &mut S,
        message: &str
    ) -> Result<()>
    where
        S: AsyncWriteExt + Unpin,
    {
        let response = StorageResponse::Error {
            message: message.to_string(),
        };
        
        let response_bytes = rmp_serde::to_vec_named(&response)?;
        stream.write_u32(response_bytes.len() as u32).await?;
        stream.write_all(&response_bytes).await?;
        stream.flush().await?;
        
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::concurrent_memory::{ConcurrentMemory, ConcurrentConfig};
    
    #[tokio::test]
    async fn test_secure_server_creation() {
        let config = ConcurrentConfig::default();
        let storage = ConcurrentMemory::new(config);
        let server = StorageServer::new(storage).await;
        
        let auth = AuthManager::new_hmac(
            "test-secret-key-32-chars-long-here".to_string(),
            3600
        );
        
        let secure_server = SecureStorageServer::new(server, Some(auth)).await;
        assert!(secure_server.is_ok());
    }
}
