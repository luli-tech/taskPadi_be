use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;
use uuid::Uuid;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct SendMessageRequest {
    pub receiver_id: Uuid,
    #[validate(length(min = 1))]
    pub content: String,
    pub image_url: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ConversationUser {
    pub user_id: Uuid,
    pub username: String,
    pub avatar_url: Option<String>,
    pub last_message: String,
    pub last_message_time: DateTime<Utc>,
    pub unread_count: i64,
}
