use std::sync::Arc;

use sqlx::{MySql, Pool};
use tracing::info;

use chat_core::{
    chat_type::ChatType,
    event::{ChatCreateEvent, ChatDropEvent, ChatEvent, UserJoinEvent, UserLeaveEvent},
};

use crate::{
    error::AppError,
    models::{
        chat::{Chat, ChatDetails},
        chat_members::ChatMembers,
        outbox_message::OutboxMessage,
    },
    services::user_service::UserService,
};

#[derive(Debug)]
pub(crate) struct ChatService {
    pub(crate) pool: Pool<MySql>,
    pub(crate) user_service: Arc<UserService>,
}

impl ChatService {
    pub fn new(pool: Pool<MySql>, user_service: Arc<UserService>) -> Self {
        Self { pool, user_service }
    }
    // 获取聊天室信息
    pub async fn find_by_id(&self, chat_id: i64) -> Result<Option<Chat>, AppError> {
        let chat = Chat::find_by_id(chat_id, false, &self.pool).await?;
        Ok(chat)
    }

    // 校验用户是聊天室成员
    pub async fn is_member(&self, chat_id: i64, user_id: i64) -> Result<bool, AppError> {
        let members = ChatMembers::list_by_chat_id(chat_id, &self.pool).await?;
        Ok(members.contains(&user_id))
    }

    #[allow(unused)]
    // 获取聊天室详情
    pub async fn details(&self, chat_id: i64) -> Result<Option<ChatDetails>, AppError> {
        let chat = Chat::find_by_id(chat_id, false, &self.pool).await?;
        if chat.is_none() {
            return Ok(None);
        }

        let chat = chat.unwrap();
        let user_ids = ChatMembers::list_by_chat_id(chat.id, &self.pool).await?;
        Ok(Some(ChatDetails {
            id: chat.id,
            title: chat.title,
            r#type: chat.r#type,
            created_at: chat.created_at,
            updated_at: chat.updated_at,
            members: user_ids,
        }))
    }

    // 查询用户所有聊天列表
    pub async fn list_by_user_id(&self, user_id: i64) -> Result<Vec<Chat>, AppError> {
        let chats = Chat::list_by_user_id(user_id, &self.pool).await?;
        Ok(chats)
    }

    // 创建聊天
    pub async fn create(
        &self,
        title: String,
        r#type: ChatType,
        members: Vec<i64>,
    ) -> Result<i64, AppError> {
        // 创建聊天室前，聊天室成员数量不能为空
        if members.is_empty() {
            return Err(AppError::ChatMemberIsEmpty);
        }
        // 需要校验成员用户的ID的存在性
        self.user_service.validate_user_ids(&members).await?;
        // 开启一个事务
        let mut tx = self.pool.begin().await?;
        // 创建聊天室
        let chat_id = Chat::create(title.clone(), r#type.clone(), &mut *tx).await?;
        // 添加聊天室成员
        ChatMembers::add_members(chat_id, members.clone(), &mut *tx).await?;
        // 新增出站消息
        let event = ChatCreateEvent::new(chat_id, &title, r#type, members);
        let event: ChatEvent = event.into();
        let event_json = serde_json::to_string(&event)?;
        OutboxMessage::create(chat_id, 0, event_json, &mut *tx).await?;
        tx.commit().await?;
        Ok(chat_id)
    }

    // 删除聊天
    pub async fn delete(&self, chat_id: i64) -> Result<bool, AppError> {
        // 校验聊天室ID
        let chat = Chat::find_by_id(chat_id, false, &self.pool)
            .await?
            .ok_or(AppError::ChatNotFound)?;
        // 开启一个事务
        let mut tx = self.pool.begin().await?;
        let members = ChatMembers::list_by_chat_id(chat_id, &mut *tx).await?;
        // 删除聊天室
        let _res = Chat::drop(chat_id, &mut *tx).await?;
        // 删除聊天室成员
        ChatMembers::delete_members_by_chat_id(chat_id, &mut *tx).await?;
        // 新增出站消息
        let event = ChatDropEvent::new(chat_id, &chat.title, chat.r#type.clone(), members);
        let event: ChatEvent = event.into();
        let event_json = serde_json::to_string(&event)?;
        OutboxMessage::create(chat_id, 0, event_json, &mut *tx).await?;
        tx.commit().await?;
        Ok(true)
    }
    // 获取聊天成员
    pub async fn get_members(&self, chat_id: i64) -> Result<Vec<i64>, AppError> {
        let members = ChatMembers::list_by_chat_id(chat_id, &self.pool).await?;
        Ok(members)
    }
    // 添加聊天成员
    pub async fn add_members(&self, chat_id: i64, user_ids: Vec<i64>) -> Result<bool, AppError> {
        // 校验聊天室ID
        let chat = Chat::find_by_id(chat_id, false, &self.pool)
            .await?
            .ok_or(AppError::ChatNotFound)?;
        // 添加成员不能为空
        if user_ids.is_empty() {
            return Err(AppError::ChatMemberIsEmpty);
        }
        // 需要校验成员用户的ID的存在性
        self.user_service.validate_user_ids(&user_ids).await?;
        // 开启一个事务
        let mut tx = self.pool.begin().await?;
        // 根据聊天室ID实现悲观锁
        let _chat = Chat::find_by_id(chat_id, true, &mut *tx).await?;
        // 根据聊天室ID查询所有成员
        let members = ChatMembers::list_by_chat_id(chat_id, &mut *tx).await?;
        // 过滤传入的成员ID，如果已经在聊天室中，则过滤掉
        let members = members
            .into_iter()
            .collect::<std::collections::HashSet<_>>();
        let user_ids = user_ids
            .into_iter()
            .filter(|user_id| !members.contains(user_id))
            .collect::<Vec<_>>();
        if user_ids.is_empty() {
            info!("传入的成员已经在聊天室中，请勿重复添加");
            return Ok(true);
        }
        // 新增出站消息
        let user_join_events = create_user_join_event(
            chat.id,
            &chat.title,
            members.iter().copied().collect::<Vec<_>>(),
            user_ids.clone(),
        );
        for event in user_join_events {
            let event_json = serde_json::to_string(&event)?;
            OutboxMessage::create(chat_id, 0, event_json, &mut *tx).await?;
        }
        let _res = ChatMembers::add_members(chat_id, user_ids, &mut *tx).await?;
        tx.commit().await?;
        Ok(true)
    }

    // 移除聊天成员
    pub async fn remove_members(&self, chat_id: i64, user_ids: Vec<i64>) -> Result<bool, AppError> {
        // 校验聊天室ID
        let chat = Chat::find_by_id(chat_id, false, &self.pool)
            .await?
            .ok_or(AppError::ChatNotFound)?;
        // 移除的成员不能为空
        if user_ids.is_empty() {
            return Err(AppError::ChatMemberIsEmpty);
        }
        // 开启一个事务
        let mut tx = self.pool.begin().await?;
        let _res = ChatMembers::remove_members(chat_id, user_ids.clone(), &self.pool).await?;
        // 如果聊天室中已没有成员，则删除聊天室
        let members = ChatMembers::list_by_chat_id(chat_id, &mut *tx).await?;
        if members.is_empty() {
            self.delete(chat_id).await?;
            // 新增出站消息
            let event = ChatDropEvent::new(chat_id, &chat.title, chat.r#type.clone(), members);
            let event: ChatEvent = event.into();
            let event_json = serde_json::to_string(&event)?;
            OutboxMessage::create(chat_id, 0, event_json, &mut *tx).await?;
        } else {
            // 新增出站消息
            let user_leave_events =
                create_user_leave_event(chat_id, &chat.title, members.clone(), user_ids.clone());
            for event in user_leave_events {
                let event_json = serde_json::to_string(&event)?;
                OutboxMessage::create(chat_id, 0, event_json, &mut *tx).await?;
            }
        }
        tx.commit().await?;

        Ok(true)
    }
}

fn create_user_join_event(
    chat_id: i64,
    title: &str,
    members: Vec<i64>,
    user_ids: Vec<i64>,
) -> Vec<ChatEvent> {
    user_ids
        .into_iter()
        .map(|user_id| {
            ChatEvent::UserJoin(UserJoinEvent::new(chat_id, title, members.clone(), user_id))
        })
        .collect::<Vec<_>>()
}

fn create_user_leave_event(
    chat_id: i64,
    title: &str,
    user_ids: Vec<i64>,
    members: Vec<i64>,
) -> Vec<ChatEvent> {
    user_ids
        .into_iter()
        .map(|user_id| {
            ChatEvent::UserLeave(UserLeaveEvent::new(
                chat_id,
                title,
                members.clone(),
                user_id,
            ))
        })
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::user_service::UserService;
    use sqlx_db_tester::TestMySql;
    use std::path::Path;

    #[tokio::test]
    async fn test_create_chat() {
        let tdb = TestMySql::new(
            "mysql://root:123456@localhost:3306".to_string(),
            Path::new("../migrations"),
        );

        let pool = tdb.get_pool().await;

        let user_service = Arc::new(UserService::new(pool.clone()));
        let chat_service = ChatService::new(pool, user_service.clone());
        let chat_id = chat_service
            .create("测试聊天室".to_string(), ChatType::Group, vec![1])
            .await
            .unwrap();
        assert!(chat_id > 0);
    }

    #[tokio::test]
    async fn test_add_members() {
        let tdb = TestMySql::new(
            "mysql://root:123456@localhost:3306".to_string(),
            Path::new("../migrations"),
        );
        let pool = tdb.get_pool().await;
        let user_service = Arc::new(UserService::new(pool.clone()));
        let chat_service = ChatService::new(pool, user_service.clone());

        let chat_id = 1;
        let a = chat_service.add_members(chat_id, vec![4]).await.unwrap();
        assert!(a);
        let chat_details = chat_service.details(chat_id).await.unwrap();
        assert!(chat_details.is_some());
        let chat_details = chat_details.unwrap();
        assert_eq!(chat_details.members.len(), 4);
    }

    #[tokio::test]
    async fn test_remove_members() {
        let tdb = TestMySql::new(
            "mysql://root:123456@localhost:3306".to_string(),
            Path::new("../migrations"),
        );
        let pool = tdb.get_pool().await;
        let user_service = Arc::new(UserService::new(pool.clone()));
        let chat_service = ChatService::new(pool, user_service.clone());
        let chat_id = 1;
        let a = chat_service.remove_members(chat_id, vec![1]).await.unwrap();
        assert!(a);
        let chat_details = chat_service.details(chat_id).await.unwrap();
        assert!(chat_details.is_some());
        let chat_details = chat_details.unwrap();
        assert_eq!(chat_details.members.len(), 2);
    }

    #[tokio::test]
    async fn test_drop() {
        let tdb = TestMySql::new(
            "mysql://root:123456@localhost:3306".to_string(),
            Path::new("../migrations"),
        );
        let pool = tdb.get_pool().await;
        let user_service = Arc::new(UserService::new(pool.clone()));
        let chat_service = ChatService::new(pool, user_service.clone());
        let chat_id = 1;
        let a = chat_service.delete(chat_id).await.unwrap();
        assert!(a);
        let chat_details = chat_service.details(chat_id).await.unwrap();
        assert!(chat_details.is_none());
    }
}
