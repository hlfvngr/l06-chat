use anyhow::Result;
use sqlx::{MySql, Pool};
use std::sync::Arc;
use tracing::info;

use crate::{error::AppError, models::workspace::Workspace, services::user_service::UserService};

#[derive(Debug)]
pub(crate) struct WorkspaceService {
    pub(crate) pool: Pool<MySql>,
    pub(crate) user_service: Arc<UserService>,
}

impl WorkspaceService {
    pub fn new(pool: Pool<MySql>, user_service: Arc<UserService>) -> Self {
        Self { pool, user_service }
    }

    // 创建工作空间
    pub async fn create(
        &self,
        name: String,
        description: String,
        owner_id: i64,
    ) -> Result<i64, AppError> {
        // 校验db中是否是否存在同名工作空间
        if self.find_by_name(&name).await?.is_some() {
            return Err(AppError::WorkspaceNameAlreadyExists);
        }
        // 校验用户ID合法性
        self.user_service
            .find_by_id(owner_id)
            .await?
            .ok_or(AppError::UserNotFound)?;

        Ok(Workspace::create(name, description, owner_id, &self.pool).await?)
    }

    // 根据名字查询工作空间（精确查询）
    pub async fn find_by_name(&self, name: &String) -> Result<Option<Workspace>, AppError> {
        Ok(Workspace::find_by_name(name, &self.pool).await?)
    }

    // 变更工作空间拥有者
    pub async fn change_owner(&self, id: i64, owner_id: i64) -> Result<bool, AppError> {
        // 校验工作空间ID合法性
        let workspace = self
            .find_by_id(id)
            .await?
            .ok_or(AppError::WorkspaceNotFound)?;
        if workspace.owner_id == owner_id {
            info!("工作空间拥有者未改变");
            return Ok(true);
        }
        // 校验用户ID合法性
        self.user_service
            .find_by_id(owner_id)
            .await?
            .ok_or(AppError::UserNotFound)?;

        Ok(Workspace::change_owner(id, owner_id, &self.pool).await?)
    }

    // 根据ID查询工作空间
    pub async fn find_by_id(&self, id: i64) -> Result<Option<Workspace>, AppError> {
        Ok(Workspace::find_by_id(id, false, &self.pool).await?)
    }
}
