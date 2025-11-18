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
    
    let state = state.read().await;
    let messages = state.mbo_messages.clone();
    
    info!("Streaming {} MBO messages as Server-Sent Events", messages.len());
    
    // Create a stream that yields each message as an SSE event
    let stream = stream::iter(messages)
        .map(|msg| {
            // Serialize each MboMsg to JSON
            match serde_json::to_string(&msg) {
                Ok(json) => Ok::<_, std::convert::Infallible>(Event::default().data(json)),
                Err(e) => {
                    error!("Failed to serialize MboMsg: {}", e);
                    Ok(Event::default().data(format!("{{\"error\": \"{}\"}}", e)))
                }
            }
        });
    
    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(15))
    )
}

