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
    websocket::types::{
        ClientMessage, ErrorPayload, UserStatusPayload, WsMessage,
        CallOfferPayload, CallAnswerPayload, IceCandidatePayload,
    },
};

use super::connection::WsSender;

/// WebSocket upgrade handler
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

    // Add connection to manager
    state.ws_connections.add_connection(user_id, tx.clone());

    // Broadcast user online status
    let online_status = WsMessage::UserStatus(UserStatusPayload {
        user_id,
        is_online: true,
    });
    state.ws_connections.broadcast(online_status);

    // Spawn task to send messages from channel to WebSocket
    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&msg) {
                if sender.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
        }
    });

    // Spawn task to receive messages from WebSocket
    let state_clone = state.clone();
    let tx_clone = tx.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                if let Err(e) = process_client_message(&text, user_id, &state_clone, &tx_clone).await {
                    tracing::error!("Error processing message: {:?}", e);
                    let error_msg = WsMessage::Error(ErrorPayload {
                        message: e.to_string(),
                    });
                    let _ = tx_clone.send(error_msg);
                }
            } else if let Message::Close(_) = msg {
                break;
            }
        }
    });

    // Spawn heartbeat task
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

    // Wait for either task to finish
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

    // Remove connection and broadcast offline status
    state.ws_connections.remove_connection(&user_id);
    let offline_status = WsMessage::UserStatus(UserStatusPayload {
        user_id,
        is_online: false,
    });
    state.ws_connections.broadcast(offline_status);

    tracing::info!("WebSocket connection closed for user {}", user_id);
}

/// Process incoming client messages
async fn process_client_message(
    text: &str,
    user_id: Uuid,
    state: &AppState,
    _tx: &WsSender,
) -> Result<()> {
    let client_msg: ClientMessage = serde_json::from_str(text)
        .map_err(|e| AppError::BadRequest(format!("Invalid message format: {}", e)))?;

    match client_msg {
        ClientMessage::SendMessage {
            receiver_id,
            content,
            image_url,
        } => {
            // Verify receiver exists
            let _receiver = state
                .user_repository
                .find_by_id(receiver_id)
                .await?
                .ok_or(AppError::NotFound("Receiver not found".to_string()))?;

            // Use MessageService for consistent behavior
            state.message_service.send_message(user_id, crate::message::message_dto::SendMessageRequest {
                receiver_id: Some(receiver_id),
                group_id: None,
                content,
                image_url,
            }).await?;
        }
        ClientMessage::TypingIndicator {
            conversation_with,
            is_typing,
        } => {
            let typing_msg = WsMessage::TypingIndicator(crate::websocket::types::TypingIndicatorPayload {
                user_id,
                is_typing,
                conversation_with,
            });
            state.ws_connections.send_to_user(&conversation_with, typing_msg);
        }
        ClientMessage::MarkMessageDelivered { message_id } => {
            // Mark message as read
            let _ = state.message_service.mark_read(user_id, message_id).await;
        }
        ClientMessage::AcceptCall { call_id } => {
            // Accept call via service
            if let Err(e) = state.video_call_service.accept_call(call_id, user_id).await {
                let error_msg = WsMessage::Error(ErrorPayload {
                    message: e.to_string(),
                });
                let _ = _tx.send(error_msg);
            }
        }
        ClientMessage::RejectCall { call_id } => {
            // Reject call via service
            if let Err(e) = state.video_call_service.reject_call(call_id, user_id).await {
                let error_msg = WsMessage::Error(ErrorPayload {
                    message: e.to_string(),
                });
                let _ = _tx.send(error_msg);
            }
        }
        ClientMessage::EndCall { call_id } => {
            // End call via service
            if let Err(e) = state.video_call_service.end_call(call_id, user_id).await {
                let error_msg = WsMessage::Error(ErrorPayload {
                    message: e.to_string(),
                });
                let _ = _tx.send(error_msg);
            }
        }
        ClientMessage::SendCallOffer {
            call_id,
            to_user_id,
            sdp,
        } => {
            // Forward WebRTC offer to recipient
            let offer_msg = WsMessage::CallOffer(CallOfferPayload {
                call_id,
                from_user_id: user_id,
                to_user_id,
                sdp,
            });
            state.ws_connections.send_to_user(&to_user_id, offer_msg);
        }
        ClientMessage::SendCallAnswer {
            call_id,
            to_user_id,
            sdp,
        } => {
            // Forward WebRTC answer to recipient
            let answer_msg = WsMessage::CallAnswer(CallAnswerPayload {
                call_id,
                from_user_id: user_id,
                to_user_id,
                sdp,
            });
            state.ws_connections.send_to_user(&to_user_id, answer_msg);
        }
        ClientMessage::SendIceCandidate {
            call_id,
            to_user_id,
            candidate,
        } => {
            // Forward ICE candidate to recipient
            let ice_msg = WsMessage::IceCandidate(IceCandidatePayload {
                call_id,
                from_user_id: user_id,
                to_user_id,
                candidate,
            });
            state.ws_connections.send_to_user(&to_user_id, ice_msg);
        }
        ClientMessage::Ping => {
            let _ = _tx.send(WsMessage::Pong);
        }
    }

    Ok(())
}
