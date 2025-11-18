pub mod market;
pub mod mbo;

use axum::{Router, routing::get};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::State;

pub fn router(state: Arc<RwLock<State>>) -> Router {
    let api_router = Router::new()
        .route("/market/export", get(market::export::handler))
        .route("/mbo/stream/json", get(mbo::stream::json::handler));

    Router::new()
        .nest("/api", api_router)
        .with_state(state)
}
