use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum CallType {
    #[serde(rename = "video")]
    Video,
    #[serde(rename = "voice")]
    Voice,
}

impl std::fmt::Display for CallType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CallType::Video => write!(f, "video"),
            CallType::Voice => write!(f, "voice"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum CallStatus {
    #[serde(rename = "initiating")]
    Initiating,
    #[serde(rename = "ringing")]
    Ringing,
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "ended")]
    Ended,
    #[serde(rename = "missed")]
    Missed,
    #[serde(rename = "rejected")]
    Rejected,
}

impl std::fmt::Display for CallStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CallStatus::Initiating => write!(f, "initiating"),
            CallStatus::Ringing => write!(f, "ringing"),
            CallStatus::Active => write!(f, "active"),
            CallStatus::Ended => write!(f, "ended"),
            CallStatus::Missed => write!(f, "missed"),
            CallStatus::Rejected => write!(f, "rejected"),
        }
    }
}

impl CallStatus {
    pub fn as_str(&self) -> &str {
        match self {
            CallStatus::Initiating => "initiating",
            CallStatus::Ringing => "ringing",
            CallStatus::Active => "active",
            CallStatus::Ended => "ended",
            CallStatus::Missed => "missed",
            CallStatus::Rejected => "rejected",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct VideoCall {
    pub id: Uuid,
    pub caller_id: Uuid,
    pub receiver_id: Option<Uuid>,
    pub group_id: Option<Uuid>,
    pub call_type: String, // "video" or "voice"
    #[sqlx(rename = "status")]
    pub status: String,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub duration_seconds: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct CallParticipant {
    pub id: Uuid,
    pub call_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub status: String,
    pub joined_at: DateTime<Utc>,
    pub left_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VideoCallResponse {
    pub id: Uuid,
    pub caller_id: Uuid,
    pub receiver_id: Option<Uuid>,
    pub group_id: Option<Uuid>,
    pub call_type: CallType,
    pub status: CallStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub duration_seconds: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub participants: Vec<CallParticipantResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct CallParticipantResponse {
    pub user_id: Uuid,
    pub username: String,
    pub avatar_url: Option<String>,
    pub role: String,
    pub status: String,
    pub joined_at: DateTime<Utc>,
}

impl From<VideoCall> for VideoCallResponse {
    fn from(call: VideoCall) -> Self {
        let status = match call.status.as_str() {
            "initiating" => CallStatus::Initiating,
            "ringing" => CallStatus::Ringing,
            "active" => CallStatus::Active,
            "ended" => CallStatus::Ended,
            "missed" => CallStatus::Missed,
            "rejected" => CallStatus::Rejected,
            _ => CallStatus::Ended,
        };

        let call_type = match call.call_type.as_str() {
            "voice" => CallType::Voice,
            _ => CallType::Video,
        };

        Self {
            id: call.id,
            caller_id: call.caller_id,
            receiver_id: call.receiver_id,
            group_id: call.group_id,
            call_type,
            status,
            started_at: call.started_at,
            ended_at: call.ended_at,
            duration_seconds: call.duration_seconds,
            created_at: call.created_at,
            updated_at: call.updated_at,
            participants: Vec::new(), // To be filled by service
        }
    }
}
