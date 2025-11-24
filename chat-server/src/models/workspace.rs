use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, MySql, Pool, QueryBuilder};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub(crate) struct Workspace {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub owner_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Workspace {
    // 创建空间
    pub(crate) async fn create(
        name: String,
        description: String,
        owner_id: i64,
        pool: &Pool<MySql>,
    ) -> Result<i64> {
        let res =
            sqlx::query("INSERT INTO workspaces (name, description, owner_id) VALUES (?, ?, ?)")
                .bind(name)
                .bind(description)
                .bind(owner_id)
                .execute(pool)
                .await?;
        Ok(res.last_insert_id() as i64)
    }
    // 根据名字查询工作空间（精确查询）
    pub(crate) async fn find_by_name(
        name: &String,
        pool: &Pool<MySql>,
    ) -> Result<Option<Workspace>> {
        let res = sqlx::query_as::<_, Workspace>("SELECT * FROM workspaces WHERE name = ?")
            .bind(name)
            .fetch_optional(pool)
            .await?;
        Ok(res)
    }
    // 变更工作空间拥有者
    pub(crate) async fn change_owner(id: i64, owner_id: i64, pool: &Pool<MySql>) -> Result<bool> {
        let res = sqlx::query("UPDATE workspaces SET owner_id = ? WHERE id = ?")
            .bind(owner_id)
            .bind(id)
            .execute(pool)
            .await?;
        Ok(res.rows_affected() > 0)
    }
    #[allow(dead_code)]
    // 删除工作空间
    pub(crate) async fn drop(id: i64, pool: &Pool<MySql>) -> Result<bool> {
        let res = sqlx::query("DELETE FROM workspaces WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(res.rows_affected() > 0)
    }

    // 根据ID查询工作空间
    pub(crate) async fn find_by_id<'a, E>(
        ws_id: i64,
        lock: bool,
        executor: E,
    ) -> Result<Option<Workspace>>
    where
        E: sqlx::Executor<'a, Database = MySql>,
    {
        let mut query_builder = QueryBuilder::new("SELECT * FROM workspaces WHERE id = ");
        query_builder.push_bind(ws_id);
        if lock {
            query_builder.push(" FOR UPDATE");
        }

        let res = query_builder
            .build_query_as()
            .fetch_optional(executor)
            .await?;
        Ok(res)
    }
}
