pub mod market;
pub mod mbo;

use axum::{Router, routing::get, Json, response::Html};
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

pub fn router(state: Arc<RwLock<State>>) -> Router {
    let api_router = Router::new()
        .route("/market/export", get(market::export::handler))
        .route("/mbo/stream/json", get(mbo::stream::json::handler))
        .with_state(state);

    Router::new()
        .route("/openapi.json", get(openapi_json))
        .route("/", get(swagger_ui))
        .nest("/api", api_router)
}
