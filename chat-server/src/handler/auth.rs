use anyhow::Result;
use axum::response::IntoResponse;

use crate::error::AppError;

pub(crate) async fn signin() -> Result<impl IntoResponse, AppError> {
    Ok(())
}

pub(crate) async fn signup() -> Result<impl IntoResponse, AppError> {
    Ok(())
}

pub(crate) async fn logout() -> Result<impl IntoResponse, AppError> {
    Ok(())
}
