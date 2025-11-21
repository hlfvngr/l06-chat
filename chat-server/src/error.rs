use axum::response::IntoResponse;
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        todo!()
    }
}
