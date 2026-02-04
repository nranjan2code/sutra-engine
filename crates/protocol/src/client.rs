//! Production-grade Grid client with connection pooling and automatic reconnection

use crate::{request_with_timeout, GridMessage, GridResponse};
use std::io;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::RwLock;

/// Grid client with automatic reconnection and health monitoring
pub struct GridClient {
    endpoint: String,
    connection: Arc<RwLock<Option<TcpStream>>>,
    timeout: Duration,
    max_retries: u32,
}

impl GridClient {
    /// Create new Grid client
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            connection: Arc::new(RwLock::new(None)),
            timeout: Duration::from_secs(10),
            max_retries: 3,
        }
    }

    /// Create client with custom timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Create client with custom retry count
    pub fn with_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// Connect to Grid Master
    async fn connect(&self) -> io::Result<TcpStream> {
        // Configure TCP settings for low latency
        let stream = TcpStream::connect(&self.endpoint).await?;
        stream.set_nodelay(true)?; // Disable Nagle's algorithm

        // Set keepalive to detect dead connections
        let socket = socket2::Socket::from(stream.into_std()?);
        let keepalive = socket2::TcpKeepalive::new()
            .with_time(Duration::from_secs(30))
            .with_interval(Duration::from_secs(10));
        socket.set_tcp_keepalive(&keepalive)?;

        TcpStream::from_std(socket.into())
    }

    /// Get or create connection
    async fn get_connection(&self) -> io::Result<TcpStream> {
        // Try to reuse existing connection
        let mut conn_guard = self.connection.write().await;

        if let Some(stream) = conn_guard.take() {
            // Test if connection is still alive
            if stream.peer_addr().is_ok() {
                return Ok(stream);
            }
        }

        // Create new connection
        let stream = self.connect().await?;
        Ok(stream)
    }

    /// Send request with automatic retry on failure
    pub async fn request(&self, message: GridMessage) -> io::Result<GridResponse> {
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            match self.try_request(&message).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = Some(e);

                    // Don't retry on final attempt
                    if attempt < self.max_retries {
                        // Exponential backoff
                        let backoff = Duration::from_millis(100 * 2_u64.pow(attempt));
                        tokio::time::sleep(backoff).await;

                        // Clear connection to force reconnect
                        *self.connection.write().await = None;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| io::Error::other("Max retries exceeded")))
    }

    /// Single request attempt
    async fn try_request(&self, message: &GridMessage) -> io::Result<GridResponse> {
        let mut stream = self.get_connection().await?;

        let response = request_with_timeout(&mut stream, message, self.timeout).await?;

        // Return connection to pool
        *self.connection.write().await = Some(stream);

        Ok(response)
    }

    /// Health check - test connection
    pub async fn health_check(&self) -> io::Result<bool> {
        match self.request(GridMessage::GetClusterStatus).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Close connection
    pub async fn close(&self) {
        *self.connection.write().await = None;
    }
}

// Ensure connection is properly closed on drop
impl Drop for GridClient {
    fn drop(&mut self) {
        // Best effort close - don't block
        let conn = self.connection.clone();
        tokio::spawn(async move {
            *conn.write().await = None;
        });
    }
}

/// Connection pool for multiple Grid clients
pub struct GridClientPool {
    clients: Vec<Arc<GridClient>>,
    next_idx: Arc<std::sync::atomic::AtomicUsize>,
}

impl GridClientPool {
    /// Create pool with multiple connections to same endpoint
    pub fn new(endpoint: impl Into<String>, pool_size: usize) -> Self {
        let endpoint = endpoint.into();
        let clients = (0..pool_size)
            .map(|_| Arc::new(GridClient::new(endpoint.clone())))
            .collect();

        Self {
            clients,
            next_idx: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }

    /// Get next client using round-robin
    pub fn get(&self) -> Arc<GridClient> {
        let idx = self
            .next_idx
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.clients[idx % self.clients.len()].clone()
    }

    /// Send request using next available client
    pub async fn request(&self, message: GridMessage) -> io::Result<GridResponse> {
        self.get().request(message).await
    }

    /// Health check all connections
    pub async fn health_check_all(&self) -> Vec<bool> {
        let mut results = Vec::new();
        for client in &self.clients {
            results.push(client.health_check().await.unwrap_or(false));
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let client = GridClient::new("localhost:7002")
            .with_timeout(Duration::from_secs(5))
            .with_retries(2);

        assert_eq!(client.timeout, Duration::from_secs(5));
        assert_eq!(client.max_retries, 2);
    }

    #[tokio::test]
    async fn test_pool_round_robin() {
        let pool = GridClientPool::new("localhost:7002", 3);

        // Verify round-robin distribution
        let addrs: Vec<_> = (0..6).map(|_| Arc::as_ptr(&pool.get())).collect();

        // First 3 should be different, then repeat
        assert_ne!(addrs[0], addrs[1]);
        assert_ne!(addrs[1], addrs[2]);
        assert_eq!(addrs[0], addrs[3]);
        assert_eq!(addrs[1], addrs[4]);
    }
}
