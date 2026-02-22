use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    error::{AppError, Result},
    middleware::AuthUser,
    state::AppState,
    websocket::types::{ClientMessage, ErrorPayload, UserStatusPayload, WsMessage},
};

use super::connection::WsSender;

/// General-purpose JSON signaling WebSocket handler.
///
/// Handles:
///  - Chat messages / typing indicators
///  - Call control signals (CallInitiated, CallAccepted, CallRejected, CallEnded)
///  - User presence (online/offline)
///
/// **Media (audio/video) does NOT flow through here.**
/// Media is relayed via the binary NATS-backed WebSocket at
/// `GET /api/video-calls/{call_id}/ws`.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, user_id, state))
}

/// Handle individual WebSocket connection
async fn handle_socket(socket: WebSocket, user_id: Uuid, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<WsMessage>();

    // Register connection for signaling messages
    state.ws_connections.add_connection(user_id, tx.clone());

    // Broadcast user online status
    let online_status = WsMessage::UserStatus(UserStatusPayload {
        user_id,
        is_online: true,
    });
    state.ws_connections.broadcast(online_status);

    // Task: send messages from channel to WebSocket
    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&msg) {
                if sender.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
        }
    });

    // Task: receive messages from WebSocket
    let state_clone = state.clone();
    let tx_clone = tx.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                if let Err(e) =
                    process_client_message(&text, user_id, &state_clone, &tx_clone).await
                {
                    tracing::error!("Error processing signaling message: {:?}", e);
                    let error_msg = WsMessage::Error(ErrorPayload {
                        message: e.to_string(),
                    });
                    let _ = tx_clone.send(error_msg);
                }
            } else if let Message::Close(_) = msg {
                break;
            }
            // Binary frames on the signaling WS are ignored — they should
            // go to the dedicated media relay WS (/video-calls/{id}/ws).
        }
    });

    // Heartbeat task
    let tx_heartbeat = tx.clone();
    let mut heartbeat_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            if tx_heartbeat.send(WsMessage::Ping).is_err() {
                break;
            }
        }
    });

    // Stop all tasks when any one finishes
    tokio::select! {
        _ = &mut send_task => {
            recv_task.abort();
            heartbeat_task.abort();
        },
        _ = &mut recv_task => {
            send_task.abort();
            heartbeat_task.abort();
        },
        _ = &mut heartbeat_task => {
            send_task.abort();
            recv_task.abort();
        }
    }

    // Cleanup
    state.ws_connections.remove_connection(&user_id);
    let offline_status = WsMessage::UserStatus(UserStatusPayload {
        user_id,
        is_online: false,
    });
    state.ws_connections.broadcast(offline_status);

    tracing::info!("Signaling WebSocket closed for user {}", user_id);
}

/// Process incoming client signaling messages.
///
/// Only JSON call-control and chat messages are handled here.
/// No WebRTC SDP/ICE — those variants have been removed from ClientMessage.
async fn process_client_message(
    text: &str,
    user_id: Uuid,
    state: &AppState,
    _tx: &WsSender,
) -> Result<()> {
    let client_msg: ClientMessage = serde_json::from_str(text)
        .map_err(|e| AppError::BadRequest(format!("Invalid message format: {}", e)))?;

    match client_msg {
        // ── Chat ──────────────────────────────────────────────────────────────
        ClientMessage::SendMessage {
            receiver_id,
            content,
            image_url,
        } => {
            let _receiver = state
                .user_repository
                .find_by_id(receiver_id)
                .await?
                .ok_or(AppError::NotFound("Receiver not found".to_string()))?;

            state
                .message_service
                .send_message(
                    user_id,
                    crate::message::message_dto::SendMessageRequest {
                        receiver_id: Some(receiver_id),
                        group_id: None,
                        content,
                        image_url,
                    },
                )
                .await?;
        }

        ClientMessage::TypingIndicator {
            conversation_with,
            is_typing,
        } => {
            let typing_msg =
                WsMessage::TypingIndicator(crate::websocket::types::TypingIndicatorPayload {
                    user_id,
                    is_typing,
                    conversation_with,
                });
            state.ws_connections.send_to_user(&conversation_with, typing_msg);
        }

        ClientMessage::MarkMessageDelivered { message_id } => {
            let _ = state.message_service.mark_read(user_id, message_id).await;
        }

        // ── Call control (thin wrapper over the REST service layer) ───────────
        ClientMessage::AcceptCall { call_id } => {
            if let Err(e) = state.video_call_service.accept_call(call_id, user_id).await {
                let _ = _tx.send(WsMessage::Error(ErrorPayload {
                    message: e.to_string(),
                }));
            }
        }

        ClientMessage::RejectCall { call_id } => {
            if let Err(e) = state.video_call_service.reject_call(call_id, user_id).await {
                let _ = _tx.send(WsMessage::Error(ErrorPayload {
                    message: e.to_string(),
                }));
            }
        }

        ClientMessage::EndCall { call_id } => {
            if let Err(e) = state.video_call_service.end_call(call_id, user_id).await {
                let _ = _tx.send(WsMessage::Error(ErrorPayload {
                    message: e.to_string(),
                }));
            }
        }

        // ── Keep-alive ────────────────────────────────────────────────────────
        ClientMessage::Ping => {
            let _ = _tx.send(WsMessage::Pong);
        }
    }

    Ok(())
}
