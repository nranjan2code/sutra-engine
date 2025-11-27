use axum::{
    extract::Query,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use tracing::{info, warn, error};

use crate::embedder::{Embedder, EmbedderConfig};
use crate::hardware::HardwareProfile;

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub model: String,
    pub dimensions: usize,
    pub uptime_seconds: u64,
}

#[derive(Debug, Serialize)]
pub struct InfoResponse {
    pub service: String,
    pub version: String,
    pub model: String,
    pub dimensions: usize,
    pub max_sequence_length: usize,
    pub quantization: String,
    pub batch_size: usize,
    pub fp16_enabled: bool,
    pub fused_ops_enabled: bool,
    pub hardware_profile: String,
}

#[derive(Debug, Deserialize)]
pub struct EmbedRequest {
    pub texts: Vec<String>,
    #[serde(default = "default_normalize")]
    pub normalize: bool,
    pub target_dimension: Option<usize>,
}

fn default_normalize() -> bool {
    true
}

#[derive(Debug, Serialize)]
pub struct EmbedResponse {
    pub embeddings: Vec<Vec<f32>>,
    pub processing_time_ms: f64,
    pub model_time_ms: f64,
    pub batch_size: usize,
    pub dimensions: usize,
}

#[derive(Debug, Serialize)]
pub struct MetricsResponse {
    pub requests_total: u64,
    pub requests_success: u64,
    pub requests_error: u64,
    pub avg_processing_time_ms: f64,
    pub avg_batch_size: f64,
    pub uptime_seconds: u64,
}

#[derive(Debug, Serialize)]
pub struct CacheStatsResponse {
    pub hits: u64,
    pub misses: u64,
    pub size: usize,
    pub hit_rate: f64,
}

#[derive(Clone)]
pub struct AppState {
    pub embedder: Arc<Mutex<Embedder>>,
    pub start_time: Instant,
    pub stats: Arc<Mutex<ServerStats>>,
    pub config: EmbedderConfig,
    pub hardware_profile: String,
}

#[derive(Debug, Default)]
pub struct ServerStats {
    pub requests_total: u64,
    pub requests_success: u64,
    pub requests_error: u64,
    pub total_processing_time_ms: f64,
    pub total_batch_size: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

impl ServerStats {
    pub fn avg_processing_time_ms(&self) -> f64 {
        if self.requests_total > 0 {
            self.total_processing_time_ms / self.requests_total as f64
        } else {
            0.0
        }
    }

    pub fn avg_batch_size(&self) -> f64 {
        if self.requests_total > 0 {
            self.total_batch_size as f64 / self.requests_total as f64
        } else {
            0.0
        }
    }

    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total > 0 {
            self.cache_hits as f64 / total as f64
        } else {
            0.0
        }
    }
}

pub async fn health_handler(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Json<HealthResponse> {
    let uptime = state.start_time.elapsed().as_secs();
    
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        model: state.config.name.clone(),
        dimensions: state.config.dimensions,
        uptime_seconds: uptime,
    })
}

pub async fn info_handler(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Json<InfoResponse> {
    let quantization = format!("{:?}", state.config.quantization);
    
    Json(InfoResponse {
        service: "sutra-embedder".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        model: state.config.name.clone(),
        dimensions: state.config.dimensions,
        max_sequence_length: state.config.max_sequence_length,
        quantization,
        batch_size: state.config.batch_size,
        fp16_enabled: state.config.use_fp16,
        fused_ops_enabled: state.config.use_fused_ops,
        hardware_profile: state.hardware_profile.clone(),
    })
}

pub async fn embed_handler(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(payload): Json<EmbedRequest>,
) -> Result<Json<EmbedResponse>, (StatusCode, Json<serde_json::Value>)> {
    let start = Instant::now();
    
    if payload.texts.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "No texts provided"}))
        ));
    }

    if payload.texts.len() > 1000 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Batch size too large (max 1000)"}))
        ));
    }

    let batch_size = payload.texts.len();
    
    // Update stats
    {
        let mut stats = state.stats.lock().await;
        stats.requests_total += 1;
    }

    let model_start = Instant::now();
    let mut embedder = state.embedder.lock().await;
    
    // Generate embeddings using the advanced Rust embedder
    let mut embeddings = match embedder.embed_batch(&payload.texts) {
        Ok(embeddings) => embeddings,
        Err(e) => {
            error!("Embedding generation failed: {}", e);
            let mut stats = state.stats.lock().await;
            stats.requests_error += 1;
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Embedding generation failed: {}", e)}))
            ));
        }
    };
    
    let model_time = model_start.elapsed().as_millis() as f64;
    
    // Apply target dimension truncation if requested
    if let Some(target_dim) = payload.target_dimension {
        if target_dim < embeddings[0].len() && target_dim > 0 {
            embeddings = embeddings.into_iter()
                .map(|emb| embedder.truncate_embedding(&emb, target_dim))
                .collect();
        }
    }
    
    // Apply normalization if requested (default: true)
    if payload.normalize {
        embeddings = embeddings.into_iter()
            .map(|mut emb| {
                let norm = emb.iter().map(|x| x * x).sum::<f32>().sqrt();
                if norm > 0.0 {
                    emb.iter_mut().for_each(|x| *x /= norm);
                }
                emb
            })
            .collect();
    }
    
    drop(embedder); // Release lock early
    
    let processing_time = start.elapsed().as_millis() as f64;
    let final_dimensions = embeddings.get(0).map(|e| e.len()).unwrap_or(0);
    
    // Update success stats
    {
        let mut stats = state.stats.lock().await;
        stats.requests_success += 1;
        stats.total_processing_time_ms += processing_time;
        stats.total_batch_size += batch_size as u64;
    }
    
    info!("Generated {} embeddings in {:.2}ms (model: {:.2}ms)", 
          batch_size, processing_time, model_time);
    
    Ok(Json(EmbedResponse {
        embeddings,
        processing_time_ms: processing_time,
        model_time_ms: model_time,
        batch_size,
        dimensions: final_dimensions,
    }))
}

pub async fn metrics_handler(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Json<MetricsResponse> {
    let stats = state.stats.lock().await;
    let uptime = state.start_time.elapsed().as_secs();
    
    Json(MetricsResponse {
        requests_total: stats.requests_total,
        requests_success: stats.requests_success,
        requests_error: stats.requests_error,
        avg_processing_time_ms: stats.avg_processing_time_ms(),
        avg_batch_size: stats.avg_batch_size(),
        uptime_seconds: uptime,
    })
}

pub async fn prometheus_metrics_handler(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> String {
    let stats = state.stats.lock().await;
    let uptime = state.start_time.elapsed().as_secs();
    
    format!(
        "# HELP embeddings_requests_total Total number of embedding requests\n\
         # TYPE embeddings_requests_total counter\n\
         embeddings_requests_total {}\n\
         # HELP embeddings_requests_success_total Successful embedding requests\n\
         # TYPE embeddings_requests_success_total counter\n\
         embeddings_requests_success_total {}\n\
         # HELP embeddings_requests_error_total Failed embedding requests\n\
         # TYPE embeddings_requests_error_total counter\n\
         embeddings_requests_error_total {}\n\
         # HELP embeddings_processing_time_ms Average processing time in milliseconds\n\
         # TYPE embeddings_processing_time_ms gauge\n\
         embeddings_processing_time_ms {:.2}\n\
         # HELP embeddings_uptime_seconds Service uptime in seconds\n\
         # TYPE embeddings_uptime_seconds gauge\n\
         embeddings_uptime_seconds {}\n",
        stats.requests_total,
        stats.requests_success,
        stats.requests_error,
        stats.avg_processing_time_ms(),
        uptime
    )
}

pub async fn cache_clear_handler(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Json<serde_json::Value> {
    {
        let mut stats = state.stats.lock().await;
        stats.cache_hits = 0;
        stats.cache_misses = 0;
    }
    
    info!("Cache statistics cleared");
    Json(serde_json::json!({"message": "Cache cleared", "status": "success"}))
}

pub async fn cache_stats_handler(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Json<CacheStatsResponse> {
    let stats = state.stats.lock().await;
    
    Json(CacheStatsResponse {
        hits: stats.cache_hits,
        misses: stats.cache_misses,
        size: 0, // TODO: Implement actual cache size tracking
        hit_rate: stats.cache_hit_rate(),
    })
}

pub async fn root_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "service": "sutra-embedder",
        "version": env!("CARGO_PKG_VERSION"),
        "status": "running",
        "description": "Production-ready multi-dimensional embedding system",
        "endpoints": {
            "health": "GET /health",
            "info": "GET /info", 
            "embed": "POST /embed",
            "metrics": "GET /metrics",
            "prometheus": "GET /prometheus",
            "cache": {
                "stats": "GET /cache/stats",
                "clear": "GET /cache/clear"
            }
        }
    }))
}

pub fn create_app(embedder: Embedder, config: EmbedderConfig, hardware_profile: String) -> Router {
    let state = AppState {
        embedder: Arc::new(Mutex::new(embedder)),
        start_time: Instant::now(),
        stats: Arc::new(Mutex::new(ServerStats::default())),
        config,
        hardware_profile,
    };

    Router::new()
        .route("/", get(root_handler))
        .route("/health", get(health_handler))
        .route("/info", get(info_handler))
        .route("/embed", post(embed_handler))
        .route("/metrics", get(metrics_handler))
        .route("/prometheus", get(prometheus_metrics_handler))
        .route("/cache/clear", get(cache_clear_handler))
        .route("/cache/stats", get(cache_stats_handler))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

pub async fn run_server(
    embedder: Embedder, 
    config: EmbedderConfig,
    hardware_profile: String,
    port: u16
) -> anyhow::Result<()> {
    let app = create_app(embedder, config, hardware_profile);
    
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    info!("🚀 Sutra Embedder HTTP server running on http://0.0.0.0:{}", port);
    info!("📊 Metrics available at http://0.0.0.0:{}/metrics", port);
    info!("🔍 Health check at http://0.0.0.0:{}/health", port);
    
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}
