use std::path::PathBuf;

use axum::{Router, routing::any};
use tower_http::services::ServeDir;

use crate::handler::sse;

pub fn get_router() -> Router {
    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
    let static_files_service = ServeDir::new(assets_dir).append_index_html_on_directories(true);
    // build our application with a route
    Router::new()
        .fallback_service(static_files_service)
        .route("/sse", any(sse::sse_handler))
}
