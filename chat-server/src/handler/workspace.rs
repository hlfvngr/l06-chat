use axum::response::IntoResponse;

use crate::error::AppError;

pub(crate) async fn create() -> Result<impl IntoResponse, AppError> {
    Ok(())
}
pub(crate) async fn change_owner() -> Result<impl IntoResponse, AppError> {
    Ok(())
}
