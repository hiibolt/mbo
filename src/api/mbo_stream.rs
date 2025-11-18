use axum::{
    extract::State,
    response::IntoResponse,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{instrument, info, error};

/// Stream MBO messages over TCP
/// This endpoint provides information about TCP streaming
#[instrument(skip(state))]
pub async fn mbo_stream(
    State(state): State<Arc<RwLock<crate::State>>>,
) -> impl IntoResponse {
    info!("Client requested MBO stream info");
    
    let state = state.read().await;
    let message_count = state.mbo_messages.len();
    
    let response = serde_json::json!({
        "message": "Use /api/mbo/stream/json for JSON line streaming",
        "message_count": message_count,
    });
    
    axum::Json(response)
}

/// Stream MBO messages as newline-delimited JSON using Server-Sent Events
/// This works well over HTTP and is compatible with Cloudflare tunnels
#[instrument(skip(state))]
pub async fn mbo_stream_json(
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

