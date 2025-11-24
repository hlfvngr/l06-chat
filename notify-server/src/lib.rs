use anyhow::Result;
use chat_core::{TokenVerify, event::ChatEvent, models::user::CurUser, utils::jwt::DecodingKey};
pub use config::*;
use dashmap::DashMap;
pub use router::*;
use std::{ops::Deref, sync::Arc};
use tokio::sync::broadcast;

mod config;
mod handler;
mod router;
pub mod task;

pub use task::*;

pub type UserMap = Arc<DashMap<i64, broadcast::Sender<Arc<ChatEvent>>>>;

#[derive(Debug, Clone)]
pub struct AppState {
    pub inner: Arc<AppStateInner>,
}

#[derive(Debug, Clone)]
pub struct AppStateInner {
    pub app_config: AppConfig,
    pub users: UserMap,
    pub redis_client: Arc<redis::Client>,
}

impl AppState {
    pub async fn try_new(app_config: AppConfig) -> Result<Self> {
        let inner = AppStateInner::try_new(app_config)?;
        Ok(Self {
            inner: Arc::new(inner),
        })
    }
}

impl Deref for AppState {
    type Target = AppStateInner;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AppStateInner {
    pub fn try_new(app_config: AppConfig) -> Result<Self> {
        let redis_client = redis::Client::open(app_config.redis.url.clone())?;
        Ok(Self {
            app_config,
            users: Arc::new(DashMap::new()),
            redis_client: Arc::new(redis_client),
        })
    }
}

impl TokenVerify for AppState {
    fn verify_token(&self, token: &str) -> std::result::Result<CurUser, jwt_simple::Error> {
        DecodingKey::load(&self.inner.app_config.auth.public_key)?.verify(token)
    }
}
