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
    
    info!("Client connected to MBO JSON stream");
    
    let state_read = state.read().await;
    
    // Increment active connections metric
    state_read.metrics.active_connections.inc();
    state_read.metrics.http_requests_total.inc();
    
    let messages = state_read.mbo_messages.clone();
    let metrics = Arc::clone(&state_read.metrics);
    
    // Drop the read lock before streaming
    drop(state_read);
    
    info!("Streaming {} MBO messages as Server-Sent Events", messages.len());
    
    // Clone metrics for use in stream and cleanup
    let metrics_for_stream = Arc::clone(&metrics);
    let metrics_for_cleanup = Arc::clone(&metrics);
    
    // Create a stream that yields each message as an SSE event
    let stream = stream::iter(messages)
        .map(move |msg| {
            // Increment messages processed counter
            metrics_for_stream.messages_processed.inc();
            
            // Serialize each MboMsg to JSON
            match serde_json::to_string(&msg) {
                Ok(json) => Ok::<_, std::convert::Infallible>(Event::default().data(json)),
                Err(e) => {
                    error!("Failed to serialize MboMsg: {}", e);
                    metrics_for_stream.messages_processing_errors.inc();
                    Ok(Event::default().data(format!("{{\"error\": \"{}\"}}", e)))
                }
            }
        });
    
    // Decrement active connections when stream ends
    let stream_with_cleanup = stream.chain(stream::once(async move {
        metrics_for_cleanup.active_connections.dec();
        Ok(Event::default().comment("stream_end"))
    }));
    
    Sse::new(stream_with_cleanup).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(15))
    )
}

