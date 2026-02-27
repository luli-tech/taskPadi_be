//! Redis-based WebRTC signaling relay session for video calls.
//!
//! This replaces the previous binary media relay. Instead of streaming raw video
//! bytes over WebSockets (which causes TCP head-of-line blocking), we use this
//! WebSocket only to exchange WebRTC signaling messages (SDP offers, answers,
//! and ICE candidates) as JSON text.
//!
//! Once signaling is exchanged here, the clients establish a direct P2P UDP
//! connection for the actual video/audio media.
//!
//! ## How it works
//!
//! ```text
//!  Client A (WebSocket)               Redis                Client B (WebSocket)
//!      │                               │                           │
//!      │── JSON Signaling (Text) ─────►│── call.{id}.A ───────────►│
//!      │                               │                           │
//!      │◄─ JSON Signaling (Text) ──────│◄── call.{id}.B ───────────│
//! ```

use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use tracing::{error, info, warn};
use uuid::Uuid;

/// Redis subject prefix for call rooms.
/// Full publish subject: `call.{call_id}.{user_id}`
/// Subscribe pattern:    `call.{call_id}.*`
const CALL_SUBJECT_PREFIX: &str = "webrtc_call";

/// Configuration for a single participant's signaling session.
pub struct MediaSessionConfig {
    pub call_id: Uuid,
    pub user_id: Uuid,
    pub redis_client: redis::Client,
}

/// Run the WebRTC signaling relay session for one WebSocket connection.
pub async fn run_media_session(config: MediaSessionConfig, socket: WebSocket) {
    let call_id = config.call_id;
    let user_id = config.user_id;

    let my_publish_subject = format!("{CALL_SUBJECT_PREFIX}.{call_id}.{user_id}");
    let subscribe_subject = format!("{CALL_SUBJECT_PREFIX}.{call_id}.*");

    info!(
        "WebRTC Signaling session starting: call={call_id}, user={user_id}, subject={subscribe_subject}"
    );

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
    // Receives WebRTC signaling text from other participants and forwards it
    // -------------------------------------------------------------------------
    let mut redis_to_ws = tokio::spawn(async move {
        while let Some(redis_msg) = redis_sub.next().await {
            let channel_name = redis_msg.get_channel_name();
            // Echo filter: skip messages this session published itself.
            if channel_name == my_subject_for_filter.as_str() {
                continue;
            }

            let payload: String = match redis_msg.get_payload() {
                Ok(p) => p,
                Err(_) => continue,
            };

            if ws_sender.send(Message::Text(payload)).await.is_err() {
                break;
            }
        }
    });

    // -------------------------------------------------------------------------
    // Task 2: WebSocket → Redis
    // Receives WebRTC signaling text from this client and publishes it
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
                Ok(Message::Text(text)) => {
                    // Ignore keep-alive pings from the frontend
                    if text.contains("\"ping\"") {
                        continue;
                    }
                    
                    // Publish WebRTC JSON signaling
                    if let Err(e) = redis_pub.publish::<_, _, ()>(&pub_subject, text).await {
                        error!("Redis publish error in call {call_id}: {e}");
                    }
                }
                Ok(Message::Binary(_)) => {
                    warn!("Received unexpected binary frame on WebRTC signaling ws {call_id}; ignoring, client shouldn't send binary media here.");
                }
                Ok(Message::Close(_)) | Err(_) => {
                    break;
                }
                Ok(Message::Ping(_)) | Ok(Message::Pong(_)) => {}
            }
        }
    });

    tokio::select! {
        _ = &mut redis_to_ws => {
            ws_to_redis.abort();
        }
        _ = &mut ws_to_redis => {
            redis_to_ws.abort();
        }
    }

    info!("WebRTC Signaling session ended: call={call_id}, user={user_id}");
}
