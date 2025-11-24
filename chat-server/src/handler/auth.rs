use anyhow::Result;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};
use chat_core::{models::user::CurUser, utils::jwt::EncodingKey};
use serde::{Deserialize, Serialize};

use crate::{error::AppError, models::user::User, AppState};

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SigninPayload {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SigninResp {
    pub token: String,
}
impl SigninResp {
    pub fn new(token: String) -> Self {
        Self { token }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SignupPayload {
    pub ws_id: i64,
    pub email: String,
    pub password: String,
    pub fullname: String,
}

pub(crate) async fn signin(
    State(state): State<AppState>,
    Json(payload): Json<SigninPayload>,
) -> Result<impl IntoResponse, AppError> {
    let user = state.user_service.find_by_email(&payload.email).await?;
    if user.is_none() {
        return Err(AppError::EmailOrPasswordIncorrect);
    }
    let user = user.unwrap();
    // 验证密码
    if !User::verify_password(&payload.password, &user.password)? {
        return Err(AppError::EmailOrPasswordIncorrect);
    }
    // 使用jwt生成token，包含用户数据
    let encoding_key = EncodingKey::load(state.app_config.auth.secret_key.as_str())?;

    let user: CurUser = user.into();
    let token = encoding_key.sign(user)?;

    Ok((StatusCode::OK, Json(SigninResp::new(token))).into_response())
}

pub(crate) async fn signup(
    State(state): State<AppState>,
    Json(payload): Json<SignupPayload>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = state
        .user_service
        .create(
            payload.ws_id,
            payload.fullname,
            payload.password,
            payload.email,
        )
        .await?;
    Ok((StatusCode::OK, user_id.to_string()).into_response())
}

pub(crate) async fn logout(
    Extension(_user): Extension<CurUser>,
    State(_state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    // 用户登出，删除token
    Ok(())
}
