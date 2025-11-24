use serde::{Deserialize, Serialize};

use crate::chat_type::ChatType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChatEvent {
    ChatCreate(ChatCreateEvent),
    ChatDrop(ChatDropEvent),
    UserJoin(UserJoinEvent),
    UserLeave(UserLeaveEvent),
    MessageSend(MessageSendEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCreateEvent {
    pub chat_id: i64,
    pub title: String,
    pub r#type: ChatType,
    pub members: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatDropEvent {
    pub chat_id: i64,
    pub title: String,
    pub r#type: ChatType,
    pub members: Vec<i64>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserJoinEvent {
    pub chat_id: i64,
    pub title: String,
    pub members: Vec<i64>,
    pub user_id: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLeaveEvent {
    pub chat_id: i64,
    pub title: String,
    pub members: Vec<i64>,
    pub user_id: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageSendEvent {
    pub message_id: i64,
    pub chat_id: i64,
    pub sender_id: i64,
    pub content: String,
    pub members: Vec<i64>,
    pub attachments: Option<Vec<String>>,
}

impl ChatCreateEvent {
    pub fn new(
        chat_id: i64,
        title: impl Into<String>,
        r#type: ChatType,
        members: Vec<i64>,
    ) -> Self {
        Self {
            chat_id,
            title: title.into(),
            r#type,
            members,
        }
    }
}

impl ChatDropEvent {
    pub fn new(
        chat_id: i64,
        title: impl Into<String>,
        r#type: ChatType,
        members: Vec<i64>,
    ) -> Self {
        Self {
            chat_id,
            title: title.into(),
            r#type,
            members,
        }
    }
}

impl UserJoinEvent {
    pub fn new(chat_id: i64, title: impl Into<String>, members: Vec<i64>, user_id: i64) -> Self {
        Self {
            chat_id,
            title: title.into(),
            members,
            user_id,
        }
    }
}

impl UserLeaveEvent {
    pub fn new(chat_id: i64, title: impl Into<String>, members: Vec<i64>, user_id: i64) -> Self {
        Self {
            chat_id,
            title: title.into(),
            members,
            user_id,
        }
    }
}

impl MessageSendEvent {
    pub fn new(
        message_id: i64,
        chat_id: i64,
        sender_id: i64,
        content: impl Into<String>,
        members: Vec<i64>,
        attachments: Option<Vec<String>>,
    ) -> Self {
        Self {
            message_id,
            chat_id,
            sender_id,
            content: content.into(),
            members,
            attachments,
        }
    }
}

impl From<ChatCreateEvent> for ChatEvent {
    fn from(value: ChatCreateEvent) -> Self {
        Self::ChatCreate(value)
    }
}
impl From<ChatDropEvent> for ChatEvent {
    fn from(value: ChatDropEvent) -> Self {
        Self::ChatDrop(value)
    }
}
impl From<UserJoinEvent> for ChatEvent {
    fn from(value: UserJoinEvent) -> Self {
        Self::UserJoin(value)
    }
}
impl From<UserLeaveEvent> for ChatEvent {
    fn from(value: UserLeaveEvent) -> Self {
        Self::UserLeave(value)
    }
}
impl From<MessageSendEvent> for ChatEvent {
    fn from(value: MessageSendEvent) -> Self {
        Self::MessageSend(value)
    }
}
