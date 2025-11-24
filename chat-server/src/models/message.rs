use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, MySql, Pool, QueryBuilder};

#[derive(Debug, Clone, Default, Serialize, Deserialize, FromRow)]
pub(crate) struct Message {
    pub id: i64,
    pub chat_id: i64,
    pub sender_id: i64,
    pub content: String,
    pub files: String,
    pub created_at: DateTime<Utc>,
}

impl Message {
    // 创建消息
    pub(crate) async fn create<'a, E>(
        chat_id: i64,
        sender_id: i64,
        content: String,
        files: String,
        executor: E,
    ) -> Result<i64>
    where
        E: sqlx::Executor<'a, Database = MySql>,
    {
        let res = sqlx::query(
            "INSERT INTO messages (chat_id, sender_id, content, files) VALUES (?, ?, ?, ?)",
        )
        .bind(chat_id)
        .bind(sender_id)
        .bind(content)
        .bind(files)
        .execute(executor)
        .await?;

        Ok(res.last_insert_id() as i64)
    }

    // 获取最近消息
    pub(crate) async fn recent(
        chat_id: i64,
        start_message_id: i64,
        limit: i64,
        pool: &Pool<MySql>,
    ) -> Result<Vec<Message>> {
        let mut query_builder = QueryBuilder::new("SELECT * FROM messages WHERE chat_id = ");
        query_builder.push_bind(chat_id);
        if start_message_id > 0 {
            query_builder.push(" AND id < ").push_bind(start_message_id);
        }
        query_builder
            .push(" ORDER BY created_at DESC LIMIT ")
            .push_bind(limit);

        let res = query_builder.build_query_as().fetch_all(pool).await?;
        Ok(res)
    }
}
