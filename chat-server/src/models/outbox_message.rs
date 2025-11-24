use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{
    encode::IsNull,
    mysql::MySqlTypeInfo,
    prelude::{FromRow, Type},
    Decode, Encode, MySql,
};
use tracing::error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SendStatus {
    Pending,
    Sending,
    Success,
    Failed,
}

// 实现 Type：声明这个类型在 MySQL 中对应 VARCHAR/TEXT
impl Type<MySql> for SendStatus {
    fn type_info() -> MySqlTypeInfo {
        // 对应 MySQL 的 TEXT 类型（也兼容 VARCHAR）
        <&str as Type<MySql>>::type_info()
    }

    fn compatible(ty: &MySqlTypeInfo) -> bool {
        // 允许与字符串类型兼容
        <&str as Type<MySql>>::compatible(ty)
    }
}

// 实现 Encode：如何把 ChatType 写入数据库（转为字符串）
impl Encode<'_, MySql> for SendStatus {
    fn encode_by_ref(
        &self,
        buf: &mut <MySql as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> std::result::Result<IsNull, sqlx::error::BoxDynError> {
        let s: &str = self.into();
        <&str as Encode<MySql>>::encode(s, buf)
    }
}

// 实现 Decode：如何从数据库读取（从字符串解析）
impl<'r> Decode<'r, MySql> for SendStatus {
    fn decode(
        value: <MySql as sqlx::Database>::ValueRef<'r>,
    ) -> std::result::Result<Self, sqlx::error::BoxDynError> {
        let s: &str = <&str as Decode<MySql>>::decode(value)?;
        match s {
            "pending" => std::result::Result::Ok(SendStatus::Pending),
            "sending" => std::result::Result::Ok(SendStatus::Sending),
            "success" => std::result::Result::Ok(SendStatus::Success),
            "failed" => std::result::Result::Ok(SendStatus::Failed),
            _ => Err(format!("unknown SendStatus variant: '{}'", s).into()),
        }
    }
}

// 辅助：ChatType → &str
impl From<&SendStatus> for &'static str {
    fn from(t: &SendStatus) -> Self {
        match t {
            SendStatus::Pending => "pending",
            SendStatus::Sending => "sending",
            SendStatus::Success => "success",
            SendStatus::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub(crate) struct OutboxMessage {
    pub id: i64,
    pub chat_id: i64,
    pub sender_id: i64,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub send_status: SendStatus,
    pub retry_count: i32,
    pub last_retry_time: Option<DateTime<Utc>>,
    pub send_fail_reason: Option<String>,
    pub next_retry_time: DateTime<Utc>,
    pub send_success_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub(crate) struct OutboxMessageSendFail {
    pub id: i64,
    pub last_retry_time: DateTime<Utc>,
    pub send_fail_reason: String,
    pub next_retry_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub(crate) struct OutboxMessageSendSuccess {
    pub id: i64,
    pub send_success_time: DateTime<Utc>,
}

impl OutboxMessage {
    // 创建出站消息
    pub(crate) async fn create<'a, E>(
        chat_id: i64,
        sender_id: i64,
        content: String,
        executor: E,
    ) -> Result<i64>
    where
        E: sqlx::Executor<'a, Database = MySql>,
    {
        let now = Utc::now();
        let res = sqlx::query(
            r#"
            INSERT INTO outbox_messages (chat_id, sender_id, content, send_status, retry_count, next_retry_time)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        ).bind(chat_id)
        .bind(sender_id)
        .bind(content)
        .bind(SendStatus::Pending)
        .bind(0)
        .bind(now)
        .execute(executor)
        .await?;
        Ok(res.last_insert_id() as i64)
    }

    #[allow(dead_code)]
    // 查询已经发送成功的消息的最小ID
    pub(crate) async fn get_min_success_message_id<'a, E>(
        executor: E,
    ) -> Result<Option<OutboxMessage>>
    where
        E: sqlx::Executor<'a, Database = MySql>,
    {
        let res = sqlx::query_as::<_, OutboxMessage>(
            r#"
            SELECT MIN(id) FROM outbox_messages WHERE send_status = ?
            "#,
        )
        .bind(SendStatus::Success)
        .fetch_optional(executor)
        .await?;
        Ok(res)
    }

    #[allow(dead_code)]
    // 删除已经发送成功的消息
    pub(crate) async fn delete_success<'a, E>(
        start_id: i64,
        limit: i64,
        end_time: DateTime<Utc>,
        executor: E,
    ) -> Result<u64>
    where
        E: sqlx::Executor<'a, Database = MySql>,
    {
        let res = sqlx::query(
            r#"
            DELETE FROM outbox_messages WHERE id >= ? AND send_success_time <= ? LIMIT ?
            "#,
        )
        .bind(start_id)
        .bind(end_time)
        .bind(limit)
        .execute(executor)
        .await?;
        Ok(res.rows_affected())
    }

    // 获取所有需要发送的消息
    pub(crate) async fn get_pending_and_failed<'a, E>(executor: E) -> Result<Vec<OutboxMessage>>
    where
        E: sqlx::Executor<'a, Database = MySql>,
    {
        let res = match sqlx::query_as::<_, OutboxMessage>(
            r#"
            SELECT * FROM outbox_messages WHERE send_status in(?, ?) and next_retry_time < ?
            "#,
        )
        .bind(SendStatus::Pending)
        .bind(SendStatus::Failed)
        .bind(Utc::now())
        .fetch_all(executor)
        .await
        {
            Ok(res) => res,
            Err(e) => {
                error!("get_pending_and_failed error: {}", e);
                return Err(e.into());
            }
        };
        Ok(res)
    }
    // 消息发送成功时，批量更新消息状态
    pub(crate) async fn update_on_success<'a, E>(
        data: OutboxMessageSendSuccess,
        executor: E,
    ) -> Result<u64>
    where
        E: sqlx::Executor<'a, Database = MySql>,
    {
        let res = sqlx::query(
            r#"UPDATE outbox_messages SET send_status = ?, send_success_time = ? WHERE id = ?
            "#,
        )
        .bind(SendStatus::Success)
        .bind(data.send_success_time)
        .bind(data.id)
        .execute(executor)
        .await?;

        Ok(res.rows_affected())
    }

    // 消息发送失败时，批量更新消息状态、失败原因、下次重试时间、重试次数、最后重试时间
    pub(crate) async fn update_on_fail<'a, E>(
        data: OutboxMessageSendFail,
        executor: E,
    ) -> Result<u64>
    where
        E: sqlx::Executor<'a, Database = MySql>,
    {
        let res = sqlx::query(
            r#"
            UPDATE outbox_messages SET  send_status = ?, send_fail_reason = ?, next_retry_time = ?, retry_count = retry_count + 1, last_retry_time = ? WHERE id = ?
            "#,
        )
        .bind(SendStatus::Failed)
        .bind(data.send_fail_reason)
        .bind(data.next_retry_time)
        .bind(data.last_retry_time)
        .bind(data.id)
        .execute(executor).await?;

        Ok(res.rows_affected())
    }
}
