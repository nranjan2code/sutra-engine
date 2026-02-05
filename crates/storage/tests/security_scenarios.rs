use async_trait::async_trait;
use once_cell::sync::Lazy;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tempfile::TempDir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::sleep;

use sutra_storage::auth::{AuthManager, Role};
use sutra_storage::embedding_provider::EmbeddingProvider;
use sutra_storage::learning_pipeline::LearningPipeline;
use sutra_storage::secure_tcp_server::SecureStorageServer;
use sutra_storage::tcp_server::{StorageRequest, StorageResponse, StorageServer};
use sutra_storage::{ConcurrentConfig, ConcurrentMemory};

static ENV_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

fn lock_env() -> std::sync::MutexGuard<'static, ()> {
    ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner())
}

struct MockEmbeddingProvider {
    dim: usize,
}

impl MockEmbeddingProvider {
    fn new(dim: usize) -> Self {
        Self { dim }
    }

    fn embed(&self, text: &str) -> Vec<f32> {
        let mut hash: u64 = 0xcbf29ce484222325;
        for b in text.as_bytes() {
            hash ^= *b as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }

        (0..self.dim)
            .map(|i| ((hash.wrapping_add(i as u64) % 1000) as f32) / 1000.0)
            .collect()
    }
}

#[async_trait]
impl EmbeddingProvider for MockEmbeddingProvider {
    async fn generate(&self, text: &str, _normalize: bool) -> anyhow::Result<Vec<f32>> {
        Ok(self.embed(text))
    }

    async fn generate_batch(&self, texts: &[String], _normalize: bool) -> Vec<Option<Vec<f32>>> {
        texts.iter().map(|t| Some(self.embed(t))).collect()
    }
}

async fn start_secure_server(
    auth: Option<AuthManager>,
    tls: bool,
    cert_path: Option<&str>,
    key_path: Option<&str>,
) -> (
    SocketAddr,
    tokio::sync::oneshot::Sender<()>,
    tokio::task::JoinHandle<()>,
    TempDir,
) {
    if tls {
        std::env::set_var("SUTRA_TLS_ENABLED", "true");
        if let Some(cert) = cert_path {
            std::env::set_var("SUTRA_TLS_CERT", cert);
        }
        if let Some(key) = key_path {
            std::env::set_var("SUTRA_TLS_KEY", key);
        }
    } else {
        std::env::set_var("SUTRA_TLS_ENABLED", "false");
        std::env::remove_var("SUTRA_TLS_CERT");
        std::env::remove_var("SUTRA_TLS_KEY");
    }

    let temp_dir = TempDir::new().unwrap();
    let config = ConcurrentConfig {
        storage_path: temp_dir.path().to_path_buf(),
        vector_dimension: 8,
        memory_threshold: 1000,
        ..Default::default()
    };
    let storage = ConcurrentMemory::new(config);
    let provider = Arc::new(MockEmbeddingProvider::new(8));
    let pipeline = LearningPipeline::new_with_provider(provider).await.unwrap();
    let server = StorageServer::new_with_pipeline(storage, pipeline);
    let secure = SecureStorageServer::new(server, auth).await.unwrap();

    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    let handle = tokio::spawn(async move {
        let _ = Arc::new(secure)
            .serve_with_shutdown(addr, async {
                let _ = shutdown_rx.await;
            })
            .await;
    });

    (addr, shutdown_tx, handle, temp_dir)
}

async fn send_request<S>(
    stream: &mut S,
    request: &StorageRequest,
) -> anyhow::Result<StorageResponse>
where
    S: AsyncReadExt + AsyncWriteExt + Unpin,
{
    let bytes = rmp_serde::to_vec_named(request)?;
    stream.write_u32(bytes.len() as u32).await?;
    stream.write_all(&bytes).await?;
    stream.flush().await?;

    let len = stream.read_u32().await?;
    let mut buf = vec![0u8; len as usize];
    stream.read_exact(&mut buf).await?;
    Ok(rmp_serde::from_slice(&buf)?)
}

async fn auth_handshake<S>(stream: &mut S, token: &str) -> anyhow::Result<()>
where
    S: AsyncReadExt + AsyncWriteExt + Unpin,
{
    stream.write_u32(token.len() as u32).await?;
    stream.write_all(token.as_bytes()).await?;
    stream.flush().await?;
    let status = stream.read_u8().await?;
    if status != 1 {
        anyhow::bail!("auth failed: status {}", status);
    }
    Ok(())
}

async fn connect_with_retry(addr: SocketAddr) -> anyhow::Result<TcpStream> {
    let mut last_err = None;
    for _ in 0..12 {
        match TcpStream::connect(addr).await {
            Ok(stream) => return Ok(stream),
            Err(err) => {
                last_err = Some(err);
                sleep(Duration::from_millis(50)).await;
            }
        }
    }
    Err(anyhow::anyhow!(
        "failed to connect to {}: {:?}",
        addr,
        last_err
    ))
}

#[tokio::test]
async fn test_hmac_auth_plaintext_healthcheck() {
    let _guard = lock_env();

    let auth = AuthManager::new_hmac("test-secret-key-32-chars-long-here".to_string(), 3600);
    let token = auth
        .generate_token("test-client", vec![Role::Reader])
        .unwrap();

    let (addr, shutdown_tx, handle, _temp_dir) =
        start_secure_server(Some(auth), false, None, None).await;

    let mut stream = connect_with_retry(addr).await.unwrap();
    auth_handshake(&mut stream, &token).await.unwrap();

    let response = send_request(&mut stream, &StorageRequest::HealthCheck)
        .await
        .unwrap();

    match response {
        StorageResponse::HealthCheckOk { healthy, .. } => assert!(healthy),
        other => panic!("Unexpected response: {:?}", other),
    }

    let _ = shutdown_tx.send(());
    let _ = handle.await;
}

#[tokio::test]
async fn test_hmac_auth_rejects_invalid_token() {
    let _guard = lock_env();

    let auth = AuthManager::new_hmac("test-secret-key-32-chars-long-here".to_string(), 3600);
    let (addr, shutdown_tx, handle, _temp_dir) =
        start_secure_server(Some(auth), false, None, None).await;

    let mut stream = connect_with_retry(addr).await.unwrap();
    stream.write_u32(5).await.unwrap();
    stream.write_all(b"bogus").await.unwrap();
    stream.flush().await.unwrap();

    let result = stream.read_u8().await;
    assert!(result.is_err());

    let _ = shutdown_tx.send(());
    let _ = handle.await;
}

#[tokio::test]
async fn test_tls_auth_handshake() {
    let _guard = lock_env();

    let cert_dir = TempDir::new().unwrap();
    let cert_path = cert_dir.path().join("cert.pem");
    let key_path = cert_dir.path().join("key.pem");

    let cert = rcgen::generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();
    std::fs::write(&cert_path, cert.serialize_pem().unwrap()).unwrap();
    std::fs::write(&key_path, cert.serialize_private_key_pem()).unwrap();

    let auth = AuthManager::new_hmac("test-secret-key-32-chars-long-here".to_string(), 3600);
    let token = auth
        .generate_token("tls-client", vec![Role::Reader])
        .unwrap();

    let (addr, shutdown_tx, handle, _temp_dir) =
        start_secure_server(Some(auth), true, cert_path.to_str(), key_path.to_str()).await;

    // Build rustls client config trusting the generated cert
    let cert_der = cert.serialize_der().unwrap();
    let mut root_store = rustls::RootCertStore::empty();
    root_store.add(&rustls::Certificate(cert_der)).unwrap();
    let config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    let connector = tokio_rustls::TlsConnector::from(Arc::new(config));
    let stream = connect_with_retry(addr).await.unwrap();
    let domain = rustls::ServerName::try_from("localhost").unwrap();
    let mut tls_stream = connector.connect(domain, stream).await.unwrap();

    auth_handshake(&mut tls_stream, &token).await.unwrap();

    let response = send_request(&mut tls_stream, &StorageRequest::HealthCheck)
        .await
        .unwrap();

    match response {
        StorageResponse::HealthCheckOk { healthy, .. } => assert!(healthy),
        other => panic!("Unexpected response: {:?}", other),
    }

    let _ = shutdown_tx.send(());
    let _ = handle.await;
}
