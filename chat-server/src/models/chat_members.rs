use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, Executor, MySql, Pool, QueryBuilder};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub(crate) struct ChatMembers {
    pub id: i64,
    pub chat_id: i64,
    pub user_id: i64,
}

impl ChatMembers {
    // 根据聊天ID查询所有成员
    pub(crate) async fn list_by_chat_id<'a, E>(chat_id: i64, executor: E) -> Result<Vec<i64>>
    where
        E: Executor<'a, Database = MySql>,
    {
        let res: Vec<i64> =
            sqlx::query_scalar("SELECT user_id FROM chat_members WHERE chat_id = ?")
                .bind(chat_id)
                .fetch_all(executor)
                .await?;
        Ok(res)
    }
    // 创建聊天成员
    // 此处仅仅考虑插入操作，其他的逻辑在上层实现
    pub(crate) async fn add_members<'a, E>(
        chat_id: i64,
        user_ids: Vec<i64>,
        executor: E,
    ) -> Result<u64>
    where
        E: Executor<'a, Database = MySql>,
    {
        if user_ids.is_empty() {
            return Ok(0);
        }

        let mut query_builder = QueryBuilder::new("INSERT INTO chat_members (chat_id, user_id) ");
        query_builder.push_values(user_ids, |mut b, user_id| {
            b.push_bind(chat_id).push_bind(user_id);
        });

        let a = query_builder.build().execute(executor).await?;

        Ok(a.rows_affected())
    }
    // 删除聊天成员
    // 此处仅仅考虑移除操作，其他的逻辑在上层实现
    pub(crate) async fn remove_members(
        chat_id: i64,
        user_ids: Vec<i64>,
        pool: &Pool<MySql>,
    ) -> Result<u64> {
        if user_ids.is_empty() {
            return Ok(0);
        }

        let mut query_builder = QueryBuilder::new("DELETE FROM chat_members WHERE chat_id = ");
        query_builder.push_bind(chat_id);
        query_builder.push(" AND user_id IN (");
        let mut separated = query_builder.separated(',');
        for user_id in user_ids {
            separated.push_bind(user_id);
        }
        separated.push_unseparated(")");

        let sql = query_builder.build();
        let res = sql.execute(pool).await?;
        Ok(res.rows_affected())
    }

    //根据聊天室ID删除聊天室成员
    pub(crate) async fn delete_members_by_chat_id<'a, E>(chat_id: i64, executor: E) -> Result<u64>
    where
        E: Executor<'a, Database = MySql>,
    {
        let res = sqlx::query("DELETE FROM chat_members WHERE chat_id = ?")
            .bind(chat_id)
            .execute(executor)
            .await?;
        Ok(res.rows_affected())
    }
}
