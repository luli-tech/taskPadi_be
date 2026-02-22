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
    CallInitiated(CallInitiatedPayload),
    CallAccepted(CallAcceptedPayload),
    CallRejected(CallRejectedPayload),
    CallEnded(CallEndedPayload),
    CallOffer(CallOfferPayload),
    CallAnswer(CallAnswerPayload),
    IceCandidate(IceCandidatePayload),
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

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CallInitiatedPayload {
    pub call_id: Uuid,
    pub caller_id: Uuid,
    pub receiver_id: Uuid,
    pub call_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CallAcceptedPayload {
    pub call_id: Uuid,
    pub caller_id: Uuid,
    pub receiver_id: Uuid,
    pub call_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CallRejectedPayload {
    pub call_id: Uuid,
    pub caller_id: Uuid,
    pub receiver_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CallEndedPayload {
    pub call_id: Uuid,
    pub ended_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CallOfferPayload {
    pub call_id: Uuid,
    pub from_user_id: Uuid,
    pub to_user_id: Uuid,
    pub sdp: String, // WebRTC SDP offer
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CallAnswerPayload {
    pub call_id: Uuid,
    pub from_user_id: Uuid,
    pub to_user_id: Uuid,
    pub sdp: String, // WebRTC SDP answer
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct IceCandidatePayload {
    pub call_id: Uuid,
    pub from_user_id: Uuid,
    pub to_user_id: Uuid,
    pub candidate: String, // ICE candidate JSON string
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
    AcceptCall {
        call_id: Uuid,
    },
    RejectCall {
        call_id: Uuid,
    },
    EndCall {
        call_id: Uuid,
    },
    SendCallOffer {
        call_id: Uuid,
        to_user_id: Uuid,
        sdp: String,
    },
    SendCallAnswer {
        call_id: Uuid,
        to_user_id: Uuid,
        sdp: String,
    },
    SendIceCandidate {
        call_id: Uuid,
        to_user_id: Uuid,
        candidate: String,
    },
    Ping,
}
