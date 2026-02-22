use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct InitiateCallRequest {
    pub receiver_id: Option<Uuid>,
    pub group_id: Option<Uuid>,
    pub call_type: Option<String>, // "video" or "voice"
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct AddParticipantRequest {
    #[validate(required)]
    pub user_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct EndCallRequest {
    #[validate(required)]
    pub call_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CallHistoryParams {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default = "default_offset")]
    pub offset: i64,
}

fn default_limit() -> i64 {
    20
}

fn default_offset() -> i64 {
    0
}
