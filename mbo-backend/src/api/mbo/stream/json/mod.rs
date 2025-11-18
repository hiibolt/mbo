use axum::{
    extract::State,
    response::IntoResponse,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{instrument, info, error};

/// Stream MBO messages as Server-Sent Events
///
/// Streams all MBO (Market-By-Order) messages from the loaded dataset
/// as newline-delimited JSON events. Compatible with Cloudflare tunnels
/// and standard HTTP streaming.
///
/// The stream includes:
/// - All order book updates (Add, Cancel, Modify)
/// - Trade executions
/// - Timestamp and sequencing information
#[utoipa::path(
    get,
    path = "/api/mbo/stream/json",
    responses(
        (status = 200, description = "SSE stream of MBO messages", content_type = "text/event-stream"),
    ),
    tag = "mbo"
)]
#[instrument(skip(state))]
pub async fn handler(
    State(state): State<Arc<RwLock<crate::State>>>,
) -> impl IntoResponse {
    use axum::response::sse::{Event, Sse};
    use futures::stream::{self, StreamExt};
    
    // Start timing this request
    let start = std::time::Instant::now();
    
    info!("Client connected to MBO JSON stream");
    
    let state_read = state.read().await;
    
    // Activate metrics
    state_read.metrics.http_requests_total.inc();
    state_read.metrics.active_connections.inc();
    
    let market_snapshots = state_read.market_snapshots.clone();
    let metrics = Arc::clone(&state_read.metrics);
    let metrics_for_cleanup = Arc::clone(&state_read.metrics);
    
    // Record HTTP request setup duration
    let setup_duration = start.elapsed();
    metrics.http_request_duration.observe(setup_duration.as_secs_f64());
    
    // Drop the read lock before streaming
    drop(state_read);
    
    info!("Streaming {} MBO messages / Market Snapshots as Server-Sent Events", market_snapshots.len());
    
    // Create a channel to signal when the stream ends
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    let mut tx = Some(tx);
    
    // Spawn a background task that will decrement the counter when the stream ends or is dropped
    tokio::spawn(async move {
        let _ = rx.await;
        metrics_for_cleanup.active_connections.dec();
    });
    
    // Create a stream that yields each message as an SSE event
    let stream = stream::iter(market_snapshots)
        .map(move |snapshot| {
            // Increment messages processed counter
            metrics.messages_processed.inc();
            
            // Serialize each MboMsg to JSON
            match serde_json::to_string(&snapshot) {
                Ok(json) => Ok::<_, std::convert::Infallible>(Event::default().data(json)),
                Err(e) => {
                    error!("Failed to serialize MboMsg: {}", e);
                    metrics.messages_processing_errors.inc();
                    Ok(Event::default().data(format!("{{\"error\": \"{}\"}}", e)))
                }
            }
        })
        .chain(stream::once(async move {
            // Signal cleanup when stream ends
            if let Some(tx) = tx.take() {
                let _ = tx.send(());
            }
            Ok(Event::default().comment("stream_end"))
        }));
    
    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(15))
    )
}

