use crate::handler::{auth, chat, message, workspace};
use crate::middlewares::set_common_layer;
use crate::AppState;
use anyhow::Result;
use axum::http::Method;
use axum::middleware::from_fn_with_state;
use axum::{routing::post, Router};
use chat_core::middlewares::auth::verify_token;
use tower_http::cors::{self, CorsLayer};

pub fn get_router(state: AppState) -> Result<Router> {
    let chat = Router::new().nest(
        "/chat",
        Router::new()
            .route("/create", post(chat::create))
            .route("/drop", post(chat::drop))
            .route("/list_by_user_id", post(chat::list_by_user_id))
            .route("/user_join", post(chat::user_join))
            .route("/user_leave", post(chat::user_leave)),
    );
    let message = Router::new().nest(
        "/message",
        Router::new()
            .route("/recent", post(message::recent))
            .route("/send", post(message::send))
            .route("/upload", post(message::upload)),
    );

    let workspace = Router::new().nest(
        "/workspace",
        Router::new()
            .route("/create", post(workspace::create))
            .route("/change_owner", post(workspace::change_owner)),
    );

    let protected_routes = chat
        .merge(message)
        .merge(workspace)
        .layer(from_fn_with_state(state.clone(), verify_token::<AppState>));

    let auth = Router::new().nest(
        "/auth",
        Router::new()
            .route("/logout", post(auth::logout))
            .layer(from_fn_with_state(state.clone(), verify_token::<AppState>))
            .route("/signin", post(auth::signin))
            .route("/signup", post(auth::signup)),
    );

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

    let router = Router::new()
        .nest("/api", protected_routes.merge(auth))
        .layer(cors)
        .with_state(state);

    Ok(set_common_layer(router))
}
