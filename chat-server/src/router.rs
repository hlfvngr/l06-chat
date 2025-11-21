use crate::handler::{auth, chat, message, user, workspace};
use crate::AppConfig;
use axum::{routing::post, Router};

pub fn get_router(_app_config: &AppConfig) -> Router {
    let auth = Router::new().nest(
        "/auth",
        Router::new()
            .route("/signin", post(auth::signin))
            .route("/signup", post(auth::signup))
            .route("/logout", post(auth::logout)),
    );

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

    let user = Router::new().nest(
        "/user",
        Router::new()
            .route("/create", post(user::friends))
            .route("/join_workspace", post(user::join_workspace)),
    );

    let workspace = Router::new().nest(
        "/workspace",
        Router::new()
            .route("/create", post(workspace::create))
            .route("/change_owner", post(workspace::change_owner)),
    );

    Router::new().nest(
        "/api",
        auth.merge(chat).merge(message).merge(user).merge(workspace),
    )
}
