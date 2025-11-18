mod datatypes;
mod api;
mod storage;
mod metrics;

use std::{path::Path, sync::Arc, time::Duration};

use databento::{HistoricalClient, dbn::MboMsg};
use anyhow::{Result, Context};
use tokio::sync::RwLock;
use tracing::info;

use self::datatypes::market::Market;
use self::storage::Storage;
use self::metrics::Metrics;


pub struct State {
    pub dbn_client: HistoricalClient,
    pub market: Market,
    pub mbo_messages: Vec<MboMsg>,
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
        let (market, mbo_messages) = {
            let dbn_file_path_st = std::env::var("DBN_FILE_PATH")
                .unwrap_or("assets/CLX5_mbo.dbn".to_string());
            let path = Path::new(&dbn_file_path_st);

            Market::load_from_path_with_messages(path, Some(&storage))
                .context("...while loading market from DBN file")?
        };

        // Initialize metrics
        let metrics = Metrics::new()
            .context("...while initializing metrics")?;

        Ok(Self {
            dbn_client,
            market,
            mbo_messages,
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
