pub mod market_export;
pub mod mbo_stream;

use axum::{Router, routing::get};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::State;

pub fn router(state: Arc<RwLock<State>>) -> Router {
    let api_router = Router::new()
        .route("/market/export", get(market_export::market_export))
        .route("/mbo/stream", get(mbo_stream::mbo_stream))
        .route("/mbo/stream/json", get(mbo_stream::mbo_stream_json));

    Router::new()
        .nest("/api", api_router)
        .with_state(state)
}
