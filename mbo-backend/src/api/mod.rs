pub mod market;
pub mod mbo;

use axum::{Router, routing::get, Json, response::Html, http::StatusCode};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::State;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        market::export::handler,
        mbo::stream::json::handler,
    ),
    tags(
        (name = "market", description = "Market data export endpoints"),
        (name = "mbo", description = "Market-By-Order message streaming endpoints")
    ),
    info(
        title = "MBO Order Book API",
        version = "0.1.0",
        description = "Real-time market data order book system with MBO message streaming",
        contact(
            name = "API Support"
        )
    )
)]
struct ApiDoc;

async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

async fn swagger_ui() -> Html<&'static str> {
    Html(r#"<!DOCTYPE html>
<html>
<head>
    <title>MBO Order Book API</title>
    <link rel="stylesheet" type="text/css" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css" />
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
    <script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-standalone-preset.js"></script>
    <script>
        window.onload = function() {
            SwaggerUIBundle({
                url: "/openapi.json",
                dom_id: '#swagger-ui',
                presets: [
                    SwaggerUIBundle.presets.apis,
                    SwaggerUIStandalonePreset
                ],
                layout: "StandaloneLayout"
            });
        };
    </script>
</body>
</html>"#)
}

/// Health check endpoint - returns 200 OK if service is running
async fn health_check() -> StatusCode {
    StatusCode::OK
}

/// Readiness check endpoint - verifies all dependencies are ready
async fn ready_check(axum::extract::State(state): axum::extract::State<Arc<RwLock<State>>>) -> StatusCode {
    // Check if we can read state (simple readiness check)
    let _ = state.read().await;
    StatusCode::OK
}

/// Prometheus metrics endpoint
async fn metrics_handler(
    axum::extract::State(state): axum::extract::State<Arc<RwLock<State>>>
) -> Result<(StatusCode, String), (StatusCode, String)> {
    let state = state.read().await;
    state.metrics.encode()
        .map(|bytes| (StatusCode::OK, String::from_utf8_lossy(&bytes).to_string()))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to encode metrics: {}", e)))
}

pub fn router(state: Arc<RwLock<State>>) -> Router {
    let api_router = Router::new()
        .route("/market/export", get(market::export::handler))
        .route("/mbo/stream/json", get(mbo::stream::json::handler))
        .with_state(Arc::clone(&state));

    Router::new()
        .route("/openapi.json", get(openapi_json))
        .route("/", get(swagger_ui))
        .route("/health", get(health_check))
        .route("/ready", get(ready_check))
        .route("/metrics", get(metrics_handler))
        .nest("/api", api_router)
        .with_state(state)
}
