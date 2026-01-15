use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsMessage {
    ChatMessage(ChatMessagePayload),
    TypingIndicator(TypingIndicatorPayload),
    UserStatus(UserStatusPayload),
    TaskUpdated(TaskUpdatedPayload),
    TaskShared(TaskSharedPayload),
    TaskMemberRemoved(TaskMemberRemovedPayload),
    MessageDelivered(MessageDeliveredPayload),
    Error(ErrorPayload),
    Ping,
    Pong,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChatMessagePayload {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub receiver_id: Uuid,
    pub content: String,
    pub image_url: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TypingIndicatorPayload {
    pub user_id: Uuid,
    pub is_typing: bool,
    pub conversation_with: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserStatusPayload {
    pub user_id: Uuid,
    pub is_online: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TaskUpdatedPayload {
    pub task_id: Uuid,
    pub updated_by: Uuid,
    pub field: String,
    pub old_value: Option<String>,
    pub new_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TaskSharedPayload {
    pub task_id: Uuid,
    pub task_title: String,
    pub shared_by: Uuid,
    pub shared_by_username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TaskMemberRemovedPayload {
    pub task_id: Uuid,
    pub task_title: String,
    pub removed_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MessageDeliveredPayload {
    pub message_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ErrorPayload {
    pub message: String,
}

// Client-to-server messages
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    SendMessage {
        receiver_id: Uuid,
        content: String,
        image_url: Option<String>,
    },
    TypingIndicator {
        conversation_with: Uuid,
        is_typing: bool,
    },
    MarkMessageDelivered {
        message_id: Uuid,
    },
    Ping,
}
