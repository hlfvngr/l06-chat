use anyhow::{Ok, Result};
use chat_core::chat_type::ChatType;
use serde::{Deserialize, Serialize};

use sqlx::{
    types::chrono::{DateTime, Utc},
    Executor, FromRow, MySql, QueryBuilder,
};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub(crate) struct Chat {
    pub id: i64,
    pub title: String,
    pub r#type: ChatType,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ChatDetails {
    pub id: i64,
    pub title: String,
    pub r#type: ChatType,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub members: Vec<i64>,
}

impl Chat {
    // 根据聊天ID查询一个聊天
    pub(crate) async fn find_by_id<'a, E>(id: i64, lock: bool, executor: E) -> Result<Option<Chat>>
    where
        E: Executor<'a, Database = MySql>,
    {
        let mut query_builder = QueryBuilder::new("SELECT * FROM chats WHERE id = ");
        query_builder.push_bind(id);
        if lock {
            query_builder.push(" FOR UPDATE");
        }

        Ok(query_builder
            .build_query_as()
            .fetch_optional(executor)
            .await?)
    }
    // 创建聊天
    pub(crate) async fn create<'a, E>(title: String, r#type: ChatType, executor: E) -> Result<i64>
    where
        E: Executor<'a, Database = MySql>,
    {
        let res = sqlx::query("INSERT INTO chats (title, type) VALUES (?, ?)")
            .bind(title)
            .bind(r#type)
            .execute(executor)
            .await?;
        Ok(res.last_insert_id() as i64)
    }
    // 删除聊天
    pub(crate) async fn drop<'a, E>(id: i64, executor: E) -> Result<i64>
    where
        E: Executor<'a, Database = MySql>,
    {
        let res = sqlx::query("DELETE FROM chats WHERE id = ?")
            .bind(id)
            .execute(executor)
            .await?;
        Ok(res.last_insert_id() as i64)
    }
    // 列出用户所有聊天
    pub(crate) async fn list_by_user_id<'a, E>(user_id: i64, executor: E) -> Result<Vec<Chat>>
    where
        E: Executor<'a, Database = MySql>,
    {
        let mut query_builder = QueryBuilder::new(
            "SELECT DISTINCT chats.* FROM chats INNER JOIN chat_members ON chats.id = chat_members.chat_id WHERE chat_members.user_id = ");
        query_builder.push_bind(user_id);

        let res: Vec<Chat> = query_builder.build_query_as().fetch_all(executor).await?;
        Ok(res)
    }
}
