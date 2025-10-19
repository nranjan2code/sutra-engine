//! FastAPI-like server for job management

use crate::BulkIngester;
use axum::{
    extract::{Json, Path, State}, 
    http::StatusCode, 
    response::{IntoResponse, Json as ResponseJson},
    routing::{get, post}, 
    Router
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::info;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateJobRequest {
    pub source_type: String,
    pub source_config: Value,
    pub adapter_name: String,
}

#[derive(Debug, Serialize)]
pub struct JobResponse {
    pub id: String,
    pub status: String,
    pub progress: Value,
}

pub type SharedIngester = Arc<Mutex<BulkIngester>>;

pub async fn create_server(ingester: BulkIngester) -> Router {
    let shared_ingester = Arc::new(Mutex::new(ingester));
    
    Router::new()
        .route("/health", get(health_check))
        .route("/jobs", post(create_job))
        .route("/jobs/:id", get(get_job))
        .route("/jobs", get(list_jobs))
        .route("/adapters", get(list_adapters))
        .layer(CorsLayer::permissive())
        .with_state(shared_ingester)
}

async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "healthy",
        "service": "sutra-bulk-ingester",
        "version": "0.1.0"
    }))
}

async fn create_job(
    State(_ingester): State<SharedIngester>,
    Json(request): Json<CreateJobRequest>,
) -> impl IntoResponse {
    // Simplified implementation for testing
    let job_id = uuid::Uuid::new_v4().to_string();
    
    // Log the request
    println!("Received job request: source_type={}, adapter={}, config={:?}", 
        request.source_type, request.adapter_name, request.source_config);
    
    // Return immediate success response
    let response = JobResponse {
        id: job_id.clone(),
        status: "submitted".to_string(),
        progress: json!({
            "processed_items": 0,
            "total_items": null
        }),
    };
    
    ResponseJson(response)
}

async fn get_job(
    State(ingester): State<SharedIngester>,
    Path(job_id): Path<String>,
) -> impl IntoResponse {
    match ingester.lock() {
        Ok(ing) => {
            match ing.get_job(&job_id) {
                Some(job) => {
                    Json(json!({
                        "id": job.id,
                        "status": format!("{:?}", job.status).to_lowercase(),
                        "progress": {
                            "processed_items": job.progress.processed_items,
                            "total_items": job.progress.total_items,
                            "failed_items": job.progress.failed_items,
                            "concepts_created": job.progress.concepts_created,
                            "bytes_processed": job.progress.bytes_processed,
                            "current_rate": job.progress.current_rate
                        },
                        "started_at": job.started_at,
                        "completed_at": job.completed_at,
                        "error": job.error
                    })).into_response()
                }
                None => StatusCode::NOT_FOUND.into_response()
            }
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

async fn list_jobs(
    State(ingester): State<SharedIngester>,
) -> impl IntoResponse {
    match ingester.lock() {
        Ok(ing) => {
            let jobs: Vec<Value> = ing.list_jobs()
                .iter()
                .map(|job| json!({
                    "id": job.id,
                    "status": format!("{:?}", job.status).to_lowercase(),
                    "adapter_name": job.adapter_name,
                    "started_at": job.started_at,
                    "progress": {
                        "processed_items": job.progress.processed_items,
                        "concepts_created": job.progress.concepts_created
                    }
                }))
                .collect();
                
            Json(json!({
                "jobs": jobs,
                "total": jobs.len()
            })).into_response()
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

async fn list_adapters(
    State(ingester): State<SharedIngester>,
) -> impl IntoResponse {
    match ingester.lock() {
        Ok(ing) => {
            let adapters = ing.plugin_registry.list_adapters();
            Json(json!({
                "adapters": adapters,
                "total": adapters.len()
            })).into_response()
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

pub async fn run_server(ingester: BulkIngester, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let app = create_server(ingester).await;
    let addr = format!("0.0.0.0:{}", port);
    
    info!("Starting bulk ingester server on {}", addr);
    
    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}