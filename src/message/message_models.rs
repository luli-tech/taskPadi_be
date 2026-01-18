use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Message {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub receiver_id: Option<Uuid>, // Nullable for group messages
    pub group_id: Option<Uuid>, // Nullable for 1-on-1 messages
    pub content: String,
    pub image_url: Option<String>,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone,Debug, Serialize, Deserialize, ToSchema)]
pub struct MessageResponse {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub receiver_id: Option<Uuid>, // Nullable for group messages
    pub group_id: Option<Uuid>, // Nullable for 1-on-1 messages
    pub content: String,
    pub image_url: Option<String>,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Message> for MessageResponse {
    fn from(message: Message) -> Self {
        Self {
            id: message.id,
            sender_id: message.sender_id,
            receiver_id: message.receiver_id,
            group_id: message.group_id,
            content: message.content,
            image_url: message.image_url,
            is_read: message.is_read,
            created_at: message.created_at,
            updated_at: message.updated_at,
        }
    }
}
