use axum::{
    extract::State,
    Json,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::instrument;

/// Export complete market state as JSON
///
/// Returns the entire order book state including all price levels,
/// orders, and market data for all instruments and publishers.
#[utoipa::path(
    get,
    path = "/api/market/export",
    responses(
        (status = 200, description = "Market state exported successfully", body = serde_json::Value),
        (status = 500, description = "Failed to serialize market state")
    ),
    tag = "market"
)]
#[instrument(skip(state))]
pub async fn handler(
    State(state): State<Arc<RwLock<crate::State>>>,
) -> Json<serde_json::Value> {
    let state = state.read().await;
    
    // Serialize the market to JSON
    match serde_json::to_value(&state.market_snapshots) {
        Ok(json) => Json(json),
        Err(e) => {
            tracing::error!("Failed to serialize market: {}", e);
            Json(serde_json::json!({
                "error": "Failed to serialize market state"
            }))
        }
    }
}
