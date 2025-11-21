use crate::error::AppError;
use anyhow::Result;
use axum::response::IntoResponse;

pub(crate) async fn create() -> Result<impl IntoResponse, AppError> {
    Ok(())
}

pub(crate) async fn drop() -> Result<impl IntoResponse, AppError> {
    Ok(())
}

pub(crate) async fn list_by_user_id() -> Result<impl IntoResponse, AppError> {
    Ok("chat")
}

pub(crate) async fn user_join() -> Result<impl IntoResponse, AppError> {
    Ok(())
}

pub(crate) async fn user_leave() -> Result<impl IntoResponse, AppError> {
    Ok(())
}
