use crate::error::AppError;
use anyhow::Result;
use axum::response::IntoResponse;

pub(crate) async fn friends() -> Result<impl IntoResponse, AppError> {
    Ok(())
}

pub(crate) async fn join_workspace() -> Result<impl IntoResponse, AppError> {
    Ok(())
}
