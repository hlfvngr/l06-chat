use axum::{extract::State, response::IntoResponse, Json};
use serde::Deserialize;

use crate::{error::AppError, AppState};

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct WorkspaceCreate {
    pub name: String,
    pub description: String,
    pub owner_id: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct WorkspaceChangeOwner {
    pub workspace_id: i64,
    pub owner_id: i64,
}

pub(crate) async fn create(
    State(state): State<AppState>,
    Json(workspace_create): Json<WorkspaceCreate>,
) -> Result<impl IntoResponse, AppError> {
    state
        .workspace_service
        .create(
            workspace_create.name,
            workspace_create.description,
            workspace_create.owner_id,
        )
        .await?;
    Ok(())
}
pub(crate) async fn change_owner(
    State(state): State<AppState>,
    Json(workspace_change_owner): Json<WorkspaceChangeOwner>,
) -> Result<impl IntoResponse, AppError> {
    state
        .workspace_service
        .change_owner(
            workspace_change_owner.workspace_id,
            workspace_change_owner.owner_id,
        )
        .await?;
    Ok(())
}
