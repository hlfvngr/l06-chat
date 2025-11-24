use std::sync::Arc;

use sqlx::{MySql, Pool};

use chat_core::event::{ChatEvent, MessageSendEvent};

use crate::{
    error::AppError,
    models::{message::Message, outbox_message::OutboxMessage},
    services::{chat_service::ChatService, user_service::UserService},
};

#[derive(Debug)]
pub(crate) struct MessageService {
    pub(crate) pool: Pool<MySql>,
    pub(crate) user_service: Arc<UserService>,
    pub(crate) chat_service: Arc<ChatService>,
}

impl MessageService {
    pub(crate) fn new(
        pool: Pool<MySql>,
        user_service: Arc<UserService>,
        chat_service: Arc<ChatService>,
    ) -> Self {
        Self {
            pool,
            user_service,
            chat_service,
        }
    }
    // 发送消息
    pub(crate) async fn send(
        &self,
        chat_id: i64,
        sender_id: i64,
        content: String,
        files: Option<Vec<String>>,
    ) -> Result<i64, AppError> {
        // 校验聊天ID合法性
        self.chat_service
            .find_by_id(chat_id)
            .await?
            .ok_or(AppError::ChatNotFound)?;
        // 校验发送者ID合法性
        self.user_service
            .find_by_id(sender_id)
            .await?
            .ok_or(AppError::UserNotFound)?;
        // 校验发送者在聊天室里
        let _ = match self.chat_service.is_member(chat_id, sender_id).await {
            Ok(true) => Ok(()),
            Ok(false) => return Err(AppError::UserNotInChat),
            Err(e) => Err(e),
        };
        // 查询聊天室的成员ID
        let members = self.chat_service.get_members(chat_id).await?;

        // 最好是能校验一下内容长度以及文件名的长度，此处略过
        let file_str = if let Some(files) = files {
            files.join(",")
        } else {
            "".to_string()
        };
        // 开始事务
        let mut tx = self.pool.begin().await?;
        let message_id = Message::create(
            chat_id,
            sender_id,
            content.clone(),
            file_str.clone(),
            &mut *tx,
        )
        .await?;
        // 需要保存本地消息表（outbox message），后面用来做消息的pub/sub
        let event = MessageSendEvent::new(message_id, chat_id, sender_id, &content, members, None);
        let event: ChatEvent = event.into();
        let event_json = serde_json::to_string(&event)?;
        OutboxMessage::create(chat_id, sender_id, event_json, &mut *tx).await?;
        tx.commit().await?;
        Ok(message_id)
    }

    // 获取最近消息
    pub(crate) async fn recent(
        &self,
        chat_id: i64,
        start_message_id: i64,
        limit: i64,
    ) -> Result<Vec<Message>, AppError> {
        let messages = Message::recent(chat_id, start_message_id, limit, &self.pool).await?;
        Ok(messages)
    }

    #[allow(dead_code)]
    // 上传文件
    pub(crate) async fn upload(&self) -> Result<String, AppError> {
        // TODO: 上传文件
        todo!()
    }
}
