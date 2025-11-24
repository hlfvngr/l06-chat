use crate::{error::AppError, AppState};
use anyhow::Result;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};
use chat_core::{chat_type::ChatType, models::user::CurUser};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ChatCreate {
    pub title: String,
    pub r#type: ChatType,
    pub members: Vec<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ChatDrop {
    pub chat_id: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ChatJoin {
    pub chat_id: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ChatLeave {
    pub chat_id: i64,
}

pub(crate) async fn create(
    Extension(_user): Extension<CurUser>,
    State(state): State<AppState>,
    Json(chat_create): Json<ChatCreate>,
) -> Result<impl IntoResponse, AppError> {
    let chat_id = state
        .chat_service
        .create(chat_create.title, chat_create.r#type, chat_create.members)
        .await?;
    Ok((StatusCode::CREATED, Json(chat_id)))
}

pub(crate) async fn drop(
    Extension(_user): Extension<CurUser>,
    State(state): State<AppState>,
    Json(chat_drop): Json<ChatDrop>,
) -> Result<impl IntoResponse, AppError> {
    state.chat_service.delete(chat_drop.chat_id).await?;
    Ok((StatusCode::CREATED, Json(true)))
}

pub(crate) async fn list_by_user_id(
    Extension(user): Extension<CurUser>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let chats = state.chat_service.list_by_user_id(user.id).await?;
    Ok(Json(chats))
}

pub(crate) async fn user_join(
    Extension(user): Extension<CurUser>,
    State(state): State<AppState>,
    Json(chat_join): Json<ChatJoin>,
) -> Result<impl IntoResponse, AppError> {
    state
        .chat_service
        .add_members(chat_join.chat_id, vec![user.id])
        .await?;
    Ok(StatusCode::OK)
}

pub(crate) async fn user_leave(
    Extension(user): Extension<CurUser>,
    State(state): State<AppState>,
    Json(chat_leave): Json<ChatLeave>,
) -> Result<impl IntoResponse, AppError> {
    state
        .chat_service
        .remove_members(chat_leave.chat_id, vec![user.id])
        .await?;
    Ok(StatusCode::OK)
}
