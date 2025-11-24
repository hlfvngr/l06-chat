use chrono::{DateTime, Utc};
use sqlx::{MySql, Pool};

use crate::{
    error::AppError,
    models::outbox_message::{OutboxMessage, OutboxMessageSendFail, OutboxMessageSendSuccess},
};

#[derive(Debug)]
pub(crate) struct OutboxMessageService {
    pub(crate) pool: Pool<MySql>,
}

impl OutboxMessageService {
    pub fn new(pool: Pool<MySql>) -> Self {
        Self { pool }
    }

    // 获取待发送和发送失败的消息
    pub async fn get_pending_messages(&self) -> Result<Vec<OutboxMessage>, AppError> {
        let messages = OutboxMessage::get_pending_and_failed(&self.pool).await?;
        Ok(messages)
    }

    #[allow(dead_code)]
    // 删除已经发送成功的消息
    pub async fn delete_success_messages(
        &self,
        start_id: i64,
        limit: i64,
        end_time: DateTime<Utc>,
    ) -> Result<u64, AppError> {
        let count = OutboxMessage::delete_success(start_id, limit, end_time, &self.pool).await?;
        Ok(count)
    }
    // 开启一个事务，然后有一个消息发送成功的列表，根据这个列表进行更新
    pub async fn update_success_messages(
        &self,
        success_messages: Vec<OutboxMessageSendSuccess>,
    ) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await?;
        for message in success_messages {
            OutboxMessage::update_on_success(message, &mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    // 开启一个事务，然后有一个消息发送失败的列表，根据这个列表进行更新
    pub async fn update_failed_messages(
        &self,
        failed_messages: Vec<OutboxMessageSendFail>,
    ) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await?;
        for message in failed_messages {
            OutboxMessage::update_on_fail(message, &mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }
}
