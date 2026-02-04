use clap::Parser;
use sutra_bulk_ingester::{server, BulkIngester, IngesterConfig};
use tracing::{error, info};

#[derive(Parser)]
#[command(name = "sutra-bulk-ingester")]
#[command(about = "High-performance bulk data ingestion service")]
struct Args {
    /// Storage server address
    #[arg(long, default_value = "storage-server:50051")]
    storage_server: String,

    /// Server port
    #[arg(long, default_value = "8005")]
    port: u16,

    /// Plugin directory
    #[arg(long, default_value = "./plugins")]
    plugin_dir: String,

    /// Maximum concurrent jobs
    #[arg(long, default_value = "4")]
    max_concurrent_jobs: usize,

    /// Batch size for processing
    #[arg(long, default_value = "100")]
    batch_size: usize,

    /// Memory limit in MB
    #[arg(long, default_value = "4096")]
    memory_limit_mb: usize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    info!("Starting Sutra Bulk Ingester");
    info!("Storage server: {}", args.storage_server);
    info!("Server port: {}", args.port);
    info!("Plugin directory: {}", args.plugin_dir);

    // Create ingester configuration
    let config = IngesterConfig {
        storage_server: args.storage_server,
        max_concurrent_jobs: args.max_concurrent_jobs,
        batch_size: args.batch_size,
        memory_limit_mb: args.memory_limit_mb,
        plugin_dir: args.plugin_dir,
        compression_enabled: true,
        metrics_enabled: true,
    };

    // Initialize bulk ingester
    match BulkIngester::new(config).await {
        Ok(ingester) => {
            info!("Bulk ingester initialized successfully");

            // Start the web server
            if let Err(e) = server::run_server(ingester, args.port).await {
                error!("Server error: {}", e);
                std::process::exit(1);
            }
        }
        Err(e) => {
            error!("Failed to initialize bulk ingester: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
