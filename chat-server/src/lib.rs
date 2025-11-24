use crate::{
    mq::producers::outbox_message_producer::OutboxMessageProducer,
    services::{
        chat_service::ChatService, message_publish_service::MessagePublishService,
        message_service::MessageService, outbox_message_service::OutboxMessageService,
        user_service::UserService, workspace_service::WorkspaceService,
    },
};

use anyhow::Result;
use chat_core::{
    models::user::CurUser,
    utils::{jwt::DecodingKey, redis_lock::RedisLock},
    TokenVerify,
};
use sqlx::{MySql, MySqlPool, Pool};
use std::{ops::Deref, sync::Arc, time::Duration};
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::info;

mod config;
mod error;
mod handler;
mod middlewares;
mod models;
mod mq;
mod router;
mod schedules;
mod services;

pub use config::*;
pub use router::*;

#[derive(Debug, Clone)]
pub struct AppState {
    pub(crate) inner: Arc<AppStateInner>,
}

impl AppState {
    pub async fn try_new(app_config: AppConfig) -> Result<Self> {
        Ok(Self {
            inner: Arc::new(AppStateInner::try_new(app_config).await?),
        })
    }
}

impl TokenVerify for AppState {
    fn verify_token(&self, token: &str) -> Result<CurUser, jwt_simple::Error> {
        DecodingKey::load(&self.app_config.auth.public_key)?.verify(token)
    }
}

#[derive(Debug, Clone)]
pub struct AppStateInner {
    pub(crate) app_config: AppConfig,
    pub(crate) user_service: Arc<UserService>,
    pub(crate) chat_service: Arc<ChatService>,
    pub(crate) message_service: Arc<MessageService>,
    pub(crate) workspace_service: Arc<WorkspaceService>,
    pub(crate) outbox_message_service: Arc<OutboxMessageService>,
    pub(crate) message_publish_service: Arc<MessagePublishService>,
    #[allow(unused)]
    pub(crate) db_pool: Pool<MySql>,
    pub(crate) redis_client: redis::Client,
}

impl AppStateInner {
    pub(crate) async fn try_new(app_config: AppConfig) -> Result<Self> {
        let db_pool = MySqlPool::connect(&app_config.server.db_url)
            .await
            .expect(" can't connect to mysql");

        let redis_client =
            redis::Client::open(&*app_config.redis.url).expect("can't connect to redis");

        let user_service = Arc::new(UserService::new(db_pool.clone()));
        let chat_service = Arc::new(ChatService::new(db_pool.clone(), Arc::clone(&user_service)));
        let message_service = Arc::new(MessageService::new(
            db_pool.clone(),
            Arc::clone(&user_service),
            Arc::clone(&chat_service),
        ));
        let workspace_service = Arc::new(WorkspaceService::new(
            db_pool.clone(),
            Arc::clone(&user_service),
        ));
        let outbox_message_service = Arc::new(OutboxMessageService::new(db_pool.clone()));

        let outbox_message_producer = Arc::new(OutboxMessageProducer::new(redis_client.clone()));
        let message_publish_service = Arc::new(MessagePublishService::new(
            Arc::clone(&outbox_message_producer),
            Arc::clone(&outbox_message_service),
        ));
        Ok(Self {
            app_config,
            user_service,
            chat_service,
            message_service,
            workspace_service,
            outbox_message_service,
            message_publish_service,
            db_pool,
            redis_client,
        })
    }
}

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub async fn start_background_scheduler(state: AppState) -> Result<()> {
    tokio::spawn(async move {
        let sched = JobScheduler::new().await.expect("can't create scheduler");

        // Add async job
        sched
            .add(
                Job::new_async("0/1 * * * * *", move |_uuid, mut _l| {
                    let state_cloned = state.clone();
                    Box::pin(async move {
                        // info!("background scheduler execute: {}", _uuid);
                        // 从出站信息表中查询未发送和发送失败的消息，向redis pub/sub中发送
                        let _ = send_messages(state_cloned).await;
                    })
                })
                .expect("can't create job"),
            )
            .await
            .expect("can't add job");

        // Start the scheduler
        sched.start().await.expect("can't start scheduler");

        // Wait while the jobs run
        tokio::time::sleep(Duration::from_secs(100)).await;
    });

    Ok(())
}

async fn send_messages(state: AppState) -> Result<()> {
    // 先从redis中获取一把发送出站信息的锁
    let mut conn = state
        .redis_client
        .get_connection()
        .expect("can't get redis connection");

    let mut lock = RedisLock::new(&mut conn, "lock:outbox_message_send");
    match lock.acquire(Some(Duration::from_secs(30))) {
        Ok(a) => a,
        Err(e) => {
            info!("can't get lock: {}", e);
            return Ok(());
        }
    };

    // info!("staert to send messages");
    let msgs = state.outbox_message_service.get_pending_messages().await?;
    if msgs.is_empty() {
        return Ok(());
    }
    info!("start to send {} messages", msgs.len());
    for m in msgs.iter() {
        let _ = state.message_publish_service.send_message(m).await;
    }

    Ok(())
}
