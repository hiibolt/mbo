mod datatypes;
mod api;
mod storage;
mod metrics;

use std::{path::Path, sync::Arc, time::Duration};

use databento::HistoricalClient;
use anyhow::{Result, Context};
use tokio::sync::RwLock;
use tracing::{info, warn};
use std::io::Write;

use crate::datatypes::market::{MarketSnapshot, load_market_snapshots};

use self::storage::Storage;
use self::metrics::Metrics;


pub struct State {
    pub dbn_client: HistoricalClient,
    pub market_snapshots: Vec<MarketSnapshot>,
    pub storage: Storage,
    pub metrics: Arc<Metrics>,
}
impl State {
    #[tracing::instrument]
    fn from_env() -> Result<Self> {
        // Load DBN API key from environment variable and
        //  initialize the DBN client
        let dbn_client = {
            let dbn_api_key = std::env::var("DBN_KEY")
                .context("DBN_KEY environment variable not set")?;

            HistoricalClient::builder()
                .key(dbn_api_key)
                .context("...while building DBN client")?
                .build()
                .context("...while building DBN client")?
        };

        // Initialize SQLite storage
        let storage = {
            let db_path = std::env::var("DB_PATH")
                .unwrap_or("mbo_data.db".to_string());
            
            Storage::new(db_path)
                .context("...while initializing SQLite storage")?
        };

        // Build the market from a DBN file path specified
        let market_snapshots = {
            let dbn_file_path_st = std::env::var("DBN_FILE_PATH")
                .unwrap_or("assets/CLX5_mbo.dbn".to_string());
            let path = Path::new(&dbn_file_path_st);

            load_market_snapshots(path, Some(&storage))
                .context("...while loading market from DBN file")?
        };

        // Write each snapshot to `assets/snapshots/<index>.json`
        std::fs::create_dir_all("assets/snapshots")
            .context("...while creating snapshots directory")?;
        for (i, snapshot) in market_snapshots.iter().enumerate() {
            let snapshot_path = Path::new("assets/snapshots")
                .join(format!("snapshot_{}.json", i));

            // Check that the file doesn't already exist
            if snapshot_path.exists() {
                info!("Snapshot file {:?} already exists, skipping write", snapshot_path);
                continue;
            }
            
            // Serialize to JSON
            let snapshot_json = serde_json::to_string(&snapshot)
                .context("...while serializing snapshot to JSON")?;

            // Write to file
            std::fs::write(&snapshot_path, snapshot_json)
                .context(format!("...while writing snapshot to {:?}", snapshot_path))?;

            warn!("First generation of snapshot {} to {:?}", i, snapshot_path);
        }

        // Zip the entire thing into `assets/feed.zip`
        {
            let zip_path = Path::new("assets/feed.zip");
            // Check that the zip file doesn't already exist
            if !zip_path.exists() {
                info!("ZIP file {:?} already exists, skipping creation", zip_path);

                let file = std::fs::File::create(&zip_path)
                    .context("...while creating zip file")?;
                let mut zip = zip::ZipWriter::new(file);

                let options: zip::write::FileOptions<'_, ()> = zip::write::FileOptions::default()
                    .compression_method(zip::CompressionMethod::Stored)
                    .unix_permissions(0o644);

                for entry in std::fs::read_dir("assets/snapshots")
                    .context("...while reading snapshots directory")? 
                {
                    let entry = entry.context("...while reading snapshot entry")?;
                    let path = entry.path();
                    if path.is_file() {
                        let name = path.file_name()
                            .and_then(|n| n.to_str())
                            .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?;
                        
                        zip.start_file(name, options)
                            .context("...while adding file to zip")?;
                        
                        let data = std::fs::read(&path)
                            .context("...while reading snapshot file for zipping")?;
                        
                        zip.write_all(&data)
                            .context("...while writing file data to zip")?;
                    }
                }

                zip.finish()
                    .context("...while finalizing zip file")?;
            }
        }

        // Initialize metrics
        let metrics = Metrics::new()
            .context("...while initializing metrics")?;

        Ok(Self {
            dbn_client,
            market_snapshots,
            storage,
            metrics,
        })
    }
}

#[tokio::main]
#[tracing::instrument]
async fn main() -> Result<()> {
    // Initialize the logger - to view all logs, set the 
    //  RUST_LOG environment variable to "debug" or "trace"
    tracing_subscriber::fmt()
        .compact()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(false)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Initialize state from environment variables
    println!("Loading application state...");
    let state = Arc::new(RwLock::new(State::from_env()
        .context("...while loading configuration from environment")?));
    println!("State loaded successfully!");

    // Build the API router
    let app = api::router(Arc::clone(&state));

    // Configure the server address
    let addr = std::env::var("BIND_ADDRESS")
        .unwrap_or_else(|_| "0.0.0.0:3000".to_string());
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .context(format!("Failed to bind to {}", addr))?;

    // Start the server with graceful shutdown
    println!("\nStarting server on {}", addr);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("Server error")?;

    println!("Server shut down gracefully");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    // Define what the terminate signal **actually** means
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C, shutting down gracefully...");
        },
        _ = terminate => {
            info!("Received SIGTERM, shutting down gracefully...");
        },
    }
    
    // Give in-flight requests time to complete
    tokio::time::sleep(Duration::from_secs(1)).await;
}
