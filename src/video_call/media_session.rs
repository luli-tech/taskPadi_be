//! Redis-based media relay session for video calls.
//!
//! This is the Axum equivalent of `WsChatSession` in videocall-rs.
//! Instead of Actix actors, it uses plain Tokio tasks with Redis pub/sub.
//!
//! ## How it works
//!
//! ```text
//!  Client A (WebSocket)               Redis                Client B (WebSocket)
//!      │                               │                           │
//!      │── Binary frame (protobuf) ───►│── call.{id}.A ──────────►│
//!      │                               │                           │
//!      │◄─ Binary frame ───────────────│◄── call.{id}.B ──────────│
//! ```
//!
//! Each participant:
//! 1. Subscribes to `call.{call_id}.*`
//! 2. Publishes their own media to `call.{call_id}.{user_id}`
//! 3. Receives all messages except their own (echo-filtered by subject)
//!
//! This is transport-agnostic — the server relays raw bytes without
//! inspecting or decrypting them, enabling client-side E2E encryption.

use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use tracing::{error, info, warn};
use uuid::Uuid;

/// Redis subject prefix for call rooms.
/// Full publish subject: `call.{call_id}.{user_id}`
/// Subscribe pattern:    `call.{call_id}.*`
const CALL_SUBJECT_PREFIX: &str = "call";

/// Configuration for a single participant's media relay session.
pub struct MediaSessionConfig {
    /// The call this participant belongs to.
    pub call_id: Uuid,
    /// The authenticated user connecting.
    pub user_id: Uuid,
    /// Redis client for pub/sub routing.
    pub redis_client: redis::Client,
}

/// Run the media relay session for one WebSocket connection.
///
/// Drives two concurrent Tokio tasks:
/// - **Redis → WS**: Forwards all messages from other participants to this client.
/// - **WS → Redis**: Publishes all binary frames from this client into the call room.
///
/// Exits when either task completes (client disconnects or Redis subscription ends).
pub async fn run_media_session(config: MediaSessionConfig, socket: WebSocket) {
    let call_id = config.call_id;
    let user_id = config.user_id;

    // The subject this session publishes to.
    // All other participants subscribe to `call.{call_id}.*` and receive this.
    let my_publish_subject = format!("{CALL_SUBJECT_PREFIX}.{call_id}.{user_id}");

    // Wildcard subscription — receives all media published in this call room.
    let subscribe_subject = format!("{CALL_SUBJECT_PREFIX}.{call_id}.*");

    info!(
        "Media session starting: call={call_id}, user={user_id}, subject={subscribe_subject}"
    );

    // Subscribe to the call room on Redis
    let mut pubsub_conn = match config.redis_client.get_async_pubsub().await {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to get Redis pubsub connection for call {call_id}: {e}");
            return;
        }
    };

    if let Err(e) = pubsub_conn.psubscribe(&subscribe_subject).await {
        error!("Failed to subscribe to Redis call room {call_id}: {e}");
        return;
    }

    let mut redis_sub = pubsub_conn.into_on_message();

    let (mut ws_sender, mut ws_receiver) = socket.split();
    let my_subject_for_filter = my_publish_subject.clone();

    // -------------------------------------------------------------------------
    // Task 1: Redis → WebSocket
    // Receives all messages from other call participants and forwards them to
    // this client's WebSocket as raw binary frames.
    // -------------------------------------------------------------------------
    let mut redis_to_ws = tokio::spawn(async move {
        while let Some(redis_msg) = redis_sub.next().await {
            let channel_name = redis_msg.get_channel_name();
            // Echo filter: skip messages this session published itself.
            if channel_name == my_subject_for_filter.as_str() {
                continue;
            }

            let payload: Vec<u8> = match redis_msg.get_payload() {
                Ok(p) => p,
                Err(_) => continue,
            };

            if ws_sender.send(Message::Binary(payload)).await.is_err() {
                // Client disconnected
                break;
            }
        }
    });

    // -------------------------------------------------------------------------
    // Task 2: WebSocket → Redis
    // Receives binary frames from this client and publishes them to the Redis
    // call room subject so all other participants receive them.
    // -------------------------------------------------------------------------
    let mut redis_pub = match config.redis_client.get_multiplexed_async_connection().await {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to get Redis multiplexed connection for {call_id}: {e}");
            return;
        }
    };
    
    let pub_subject = my_publish_subject.clone();

    let mut ws_to_redis = tokio::spawn(async move {
        use redis::AsyncCommands;
        while let Some(msg_result) = ws_receiver.next().await {
            match msg_result {
                Ok(Message::Binary(data)) => {
                    // Publish raw binary frame to call room
                    if let Err(e) = redis_pub.publish::<_, _, ()>(&pub_subject, data).await {
                        error!("Redis publish error in call {call_id}: {e}");
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
        _ = &mut redis_to_ws => {
            ws_to_redis.abort();
        }
        _ = &mut ws_to_redis => {
            redis_to_ws.abort();
        }
    }

    info!("Media session ended: call={call_id}, user={user_id}");
}
