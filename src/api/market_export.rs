use axum::{
    extract::State,
    Json,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::instrument;

/// Export full market state as JSON
#[instrument(skip(state))]
pub async fn market_export(
    State(state): State<Arc<RwLock<crate::State>>>,
) -> Json<serde_json::Value> {
    let state = state.read().await;
    
    // Serialize the market to JSON
    match serde_json::to_value(&state.market) {
        Ok(json) => Json(json),
        Err(e) => {
            tracing::error!("Failed to serialize market: {}", e);
            Json(serde_json::json!({
                "error": "Failed to serialize market state"
            }))
        }
    }
}
