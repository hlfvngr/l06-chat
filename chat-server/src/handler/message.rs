use crate::{error::AppError, AppState};
use anyhow::Result;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};
use chat_core::models::user::CurUser;
use serde::Deserialize;
use tracing::info;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct MessageQuery {
    pub chat_id: i64,
    pub limit: i64,
    pub start_message_id: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct MessageSend {
    pub chat_id: i64,
    pub content: String,
    pub attachments: Option<Vec<String>>,
}

pub(crate) async fn recent(
    Extension(_user): Extension<CurUser>,
    State(state): State<AppState>,
    Json(message_query): Json<MessageQuery>,
) -> Result<impl IntoResponse, AppError> {
    info!("recent");
    let messages = state
        .message_service
        .recent(
            message_query.chat_id,
            message_query.start_message_id,
            message_query.limit,
        )
        .await?;
    info!("recent messages: {:?}", messages);
    Ok(Json(messages))
}

pub(crate) async fn send(
    Extension(user): Extension<CurUser>,
    State(state): State<AppState>,
    Json(message_send): Json<MessageSend>,
) -> Result<impl IntoResponse, AppError> {
    state
        .message_service
        .send(
            message_send.chat_id,
            user.id,
            message_send.content,
            message_send.attachments,
        )
        .await?;

    Ok(StatusCode::OK)
}

pub(crate) async fn upload() -> Result<impl IntoResponse, AppError> {
    Ok(())
}
