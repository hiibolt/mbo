use axum::{
    body::Body,
    http::{HeaderMap, HeaderValue},
    response::IntoResponse,
};
use tokio_util::io::ReaderStream;
use tracing::instrument;

/// Export complete market state as a ZIP, loading from `assets/feed.zip`
/// 
/// This file is pre-made, this route does not generate it on-the-fly.
#[utoipa::path(
    get,
    path = "/api/market/export",
    responses(
        (status = 200, description = "Market state exported successfully (ZIP stream)", body = Vec<u8>),
        (status = 500, description = "Failed to stream market state")
    ),
    tag = "market"
)]
#[instrument]
pub async fn handler( ) -> impl IntoResponse {
    let file = tokio::fs::File::open("assets/feed.zip").await;

    match file {
        Ok(file) => {
            let stream = ReaderStream::new(file);
            let body = Body::from_stream(stream);

            let mut headers = HeaderMap::new();
            headers.insert("Content-Type", HeaderValue::from_static("application/zip"));
            headers.insert("Content-Disposition", HeaderValue::from_static("attachment; filename=\"market_export.zip\""));

            (headers, body)
        }
        Err(e) => {
            tracing::error!("Failed to open market export ZIP file: {}", e);
            (
                HeaderMap::new(),
                Body::from("Failed to stream market state"),
            )
        }
    }
}