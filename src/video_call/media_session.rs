//! NATS-based media relay session for video calls.
//!
//! This is the Axum equivalent of `WsChatSession` in videocall-rs.
//! Instead of Actix actors, it uses plain Tokio tasks with NATS pub/sub.
//!
//! ## How it works
//!
//! ```text
//!  Client A (WebSocket)               NATS                 Client B (WebSocket)
//!      │                               │                           │
//!      │── Binary frame (protobuf) ───►│── call.{id}.A ──────────►│
//!      │                               │                           │
//!      │◄─ Binary frame ───────────────│◄── call.{id}.B ──────────│
//! ```
//!
//! Each participant:
//! 1. Subscribes to `call.{call_id}.*` with a unique queue group
//! 2. Publishes their own media to `call.{call_id}.{user_id}`
//! 3. Receives all messages except their own (echo-filtered by subject)
//!
//! This is transport-agnostic — the server relays raw bytes without
//! inspecting or decrypting them, enabling client-side E2E encryption.

use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use tracing::{error, info, warn};
use uuid::Uuid;

/// NATS subject prefix for call rooms.
/// Full publish subject: `call.{call_id}.{user_id}`
/// Subscribe pattern:    `call.{call_id}.*`
const CALL_SUBJECT_PREFIX: &str = "call";

/// Configuration for a single participant's media relay session.
pub struct MediaSessionConfig {
    /// The call this participant belongs to.
    pub call_id: Uuid,
    /// The authenticated user connecting.
    pub user_id: Uuid,
    /// NATS client for pub/sub routing.
    pub nats: async_nats::Client,
}

/// Run the media relay session for one WebSocket connection.
///
/// Drives two concurrent Tokio tasks:
/// - **NATS → WS**: Forwards all messages from other participants to this client.
/// - **WS → NATS**: Publishes all binary frames from this client into the call room.
///
/// Exits when either task completes (client disconnects or NATS subscription ends).
pub async fn run_media_session(config: MediaSessionConfig, socket: WebSocket) {
    let call_id = config.call_id;
    let user_id = config.user_id;
    let nats = config.nats;

    // The subject this session publishes to.
    // All other participants subscribe to `call.{call_id}.*` and receive this.
    let my_publish_subject = format!("{CALL_SUBJECT_PREFIX}.{call_id}.{user_id}");

    // Wildcard subscription — receives all media published in this call room.
    let subscribe_subject = format!("{CALL_SUBJECT_PREFIX}.{call_id}.*");

    // Queue group: ensures each connection gets exactly one copy of each message,
    // even when multiple actix-api / task-manager instances run in parallel.
    // Format: media-{call_id}-{user_id}  (unique per connection)
    let queue_group = format!("media-{call_id}-{user_id}");

    info!(
        "Media session starting: call={call_id}, user={user_id}, subject={subscribe_subject}"
    );

    // Subscribe to the call room on NATS
    let mut nats_sub = match nats
        .queue_subscribe(subscribe_subject.clone(), queue_group)
        .await
    {
        Ok(sub) => sub,
        Err(e) => {
            error!("Failed to subscribe to NATS call room {call_id}: {e}");
            return;
        }
    };

    let (mut ws_sender, mut ws_receiver) = socket.split();
    let my_subject_for_filter = my_publish_subject.clone();

    // -------------------------------------------------------------------------
    // Task 1: NATS → WebSocket
    // Receives all messages from other call participants and forwards them to
    // this client's WebSocket as raw binary frames.
    // -------------------------------------------------------------------------
    let mut nats_to_ws = tokio::spawn(async move {
        while let Some(nats_msg) = nats_sub.next().await {
            // Echo filter: skip messages this session published itself.
            // (Same logic as `handle_msg` in videocall-rs chat_server.rs)
            if nats_msg.subject.as_str() == my_subject_for_filter.as_str() {
                continue;
            }

            let payload = nats_msg.payload.to_vec();

            if ws_sender.send(Message::Binary(payload)).await.is_err() {
                // Client disconnected
                break;
            }
        }
    });

    // -------------------------------------------------------------------------
    // Task 2: WebSocket → NATS
    // Receives binary frames from this client and publishes them to the NATS
    // call room subject so all other participants receive them.
    // -------------------------------------------------------------------------
    let nats_pub = nats.clone();
    let pub_subject = my_publish_subject.clone();

    let mut ws_to_nats = tokio::spawn(async move {
        while let Some(msg_result) = ws_receiver.next().await {
            match msg_result {
                Ok(Message::Binary(data)) => {
                    // Publish raw binary frame to call room
                    if let Err(e) = nats_pub.publish(pub_subject.clone(), data.into()).await {
                        error!("NATS publish error in call {call_id}: {e}");
                        // Non-fatal — keep running unless disconnect
                    }
                }
                Ok(Message::Close(_)) | Err(_) => {
                    // Client closed the connection
                    break;
                }
                Ok(Message::Ping(_)) | Ok(Message::Pong(_)) => {
                    // WebSocket keep-alive frames — no action needed
                }
                Ok(Message::Text(_)) => {
                    // Text frames are not expected on the media endpoint.
                    // The media WS only carries binary protobuf packets.
                    warn!("Unexpected text frame on media WS for call {call_id}; ignoring");
                }
            }
        }
    });

    // Race: stop as soon as either side finishes
    tokio::select! {
        _ = &mut nats_to_ws => {
            ws_to_nats.abort();
        }
        _ = &mut ws_to_nats => {
            nats_to_ws.abort();
        }
    }

    info!("Media session ended: call={call_id}, user={user_id}");
}
