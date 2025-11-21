use crate::error::AppError;
use anyhow::Result;
use axum::response::IntoResponse;

pub(crate) async fn recent() -> Result<impl IntoResponse, AppError> {
    Ok(())
}

pub(crate) async fn send() -> Result<impl IntoResponse, AppError> {
    Ok(())
}

pub(crate) async fn upload() -> Result<impl IntoResponse, AppError> {
    Ok(())
}
