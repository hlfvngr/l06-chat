use axum::{
    Router,
    http::{Method, StatusCode},
    middleware::from_fn_with_state,
    response::{Html, IntoResponse},
    routing::{any, get},
};
use chat_core::middlewares::auth::verify_token;
use tower_http::cors::{self, CorsLayer};

use crate::{AppState, handler::sse};

pub fn get_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::DELETE,
            Method::PUT,
        ])
        .allow_origin(cors::Any)
        .allow_headers(cors::Any);
    // build our application with a route
    Router::new()
        .route("/events", any(sse::sse_handler))
        .layer(from_fn_with_state(state.clone(), verify_token::<AppState>))
        .layer(cors)
        .route("/favicon.ico", get(favicon))
        .route("/", get(index_handler))
        .with_state(state)
}

async fn index_handler() -> Html<&'static str> {
    Html(include_str!("../index.html"))
}

// 处理 favicon 请求（返回空 ico 或真实图标）
async fn favicon() -> impl IntoResponse {
    (
        StatusCode::OK,
        [("Content-Type", "image/x-icon")],
        // 可以返回真实图标 bytes，这里用最小合法 ICO（227 字节）
        include_bytes!("../assets/favicon.ico").as_slice(),
    )
}
