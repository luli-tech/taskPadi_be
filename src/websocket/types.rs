use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Server-to-client WebSocket messages (signaling only).
///
/// These messages handle call control signaling (ringing, accepted, ended, etc.)
/// over the existing JSON WebSocket channel.
///
/// Media (audio/video frames) is NOT sent through here — it flows through the
/// dedicated binary WebSocket at `GET /api/video-calls/{call_id}/ws` via NATS.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum WsMessage {
    // ── Chat ──────────────────────────────────────────────────────────────────
    ChatMessage(ChatMessagePayload),
    TypingIndicator(TypingIndicatorPayload),
    MessageDelivered(MessageDeliveredPayload),

    // ── Presence ──────────────────────────────────────────────────────────────
    UserStatus(UserStatusPayload),

    // ── Tasks ─────────────────────────────────────────────────────────────────
    TaskUpdated(TaskUpdatedPayload),
    TaskShared(TaskSharedPayload),
    TaskMemberRemoved(TaskMemberRemovedPayload),

    // ── Call signaling (control plane only — no media) ────────────────────────
    /// Sent to the receiver when a new call is initiated.
    /// The receiver should prompt the user to accept or reject.
    /// On accept, both sides connect to `GET /api/video-calls/{call_id}/ws`.
    CallInitiated(CallInitiatedPayload),

    /// Sent to the caller when the receiver accepts.
    /// Both sides should now open the media WebSocket.
    CallAccepted(CallAcceptedPayload),

    /// Sent to the caller when the receiver declines.
    CallRejected(CallRejectedPayload),

    /// Sent to all participants when the call ends.
    CallEnded(CallEndedPayload),

    // ── System ────────────────────────────────────────────────────────────────
    Error(ErrorPayload),
    Ping,
    Pong,
}

// ─────────────────────────────────────────────────────────────────────────────
// Payload structs
// ─────────────────────────────────────────────────────────────────────────────

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

/// Payload sent to the receiver when a call is initiated.
///
/// On receipt, the receiver should:
/// 1. Show an incoming call UI.
/// 2. Accept via `POST /api/video-calls/{call_id}/accept`
///    or reject via `POST /api/video-calls/{call_id}/reject`.
/// 3. If accepted, open the media WebSocket at
///    `GET /api/video-calls/{call_id}/ws` with a Bearer token.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CallInitiatedPayload {
    pub call_id: Uuid,
    pub caller_id: Uuid,
    pub receiver_id: Uuid,
    pub call_type: String, // "video" or "voice"
    /// Relative URL for the media relay WebSocket.
    /// Connect here (with Authorization header) after accepting.
    pub media_ws_path: String,
}

/// Payload sent to the caller when the receiver accepts.
///
/// On receipt, the caller should open the media WebSocket at
/// `GET /api/video-calls/{call_id}/ws`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CallAcceptedPayload {
    pub call_id: Uuid,
    pub caller_id: Uuid,
    pub receiver_id: Uuid,
    pub call_type: String,
    /// Relative URL for the media relay WebSocket.
    pub media_ws_path: String,
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

// ─────────────────────────────────────────────────────────────────────────────
// Client-to-server messages (signaling only)
//
// NOTE: There are NO WebRTC messages here (no CallOffer/CallAnswer/IceCandidate).
// WebRTC has been replaced by the NATS server-relay architecture from videocall-rs.
// Clients send raw binary media frames directly to the media WebSocket endpoint.
// ─────────────────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum ClientMessage {
    // Chat
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

    // Call control (these trigger REST-equivalent logic via WebSocket for convenience)
    AcceptCall {
        call_id: Uuid,
    },
    RejectCall {
        call_id: Uuid,
    },
    EndCall {
        call_id: Uuid,
    },

    // Keep-alive
    Ping,
}
