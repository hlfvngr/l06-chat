use axum::response::IntoResponse;
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum AppError {
    // TODO: 更好的处理anyhow的错误
    // #[error("service error")]
    // ServiceError(#[from] anyhow::Error),
    #[error("jwt error")]
    JwtError(#[from] jwt_simple::Error),

    #[error("db execute error")]
    DbError(#[from] sqlx::Error),

    #[error("password hash error")]
    PasswordHashError(#[from] argon2::password_hash::Error),

    #[error("sedre error")]
    SedreError(#[from] serde_json::Error),

    #[error("chat member is empty")]
    ChatMemberIsEmpty,

    #[error("user not found")]
    UserNotFound,

    #[error("chat not found")]
    ChatNotFound,

    #[error("email already exists")]
    EmailAlreadyExists,

    #[error("user not in chat")]
    UserNotInChat,

    #[error("workspace already exists")]
    WorkspaceNameAlreadyExists,

    #[error("workspace not found")]
    WorkspaceNotFound,

    #[error("email or password incorrect")]
    EmailOrPasswordIncorrect,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("{}", self),
        )
            .into_response()
    }
}
