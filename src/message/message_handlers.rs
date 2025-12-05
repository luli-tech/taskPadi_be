use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::{AppError, Result},
    middleware::AuthUser,
    state::AppState,
    task::task_dto::PaginatedResponse,
    message::{
        message_dto::SendMessageRequest,
        message_models::MessageResponse,
    },
};

#[derive(Debug, Deserialize)]
pub struct MessageQuery {
    page: Option<u32>,
    limit: Option<u32>,
}

/// Send a message to another user
#[utoipa::path(
    post,
    path = "/api/messages",
    tag = "messages",
    request_body = SendMessageRequest,
    responses(
        (status = 201, description = "Message sent successfully", body = MessageResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Receiver not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn send_message(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(payload): Json<SendMessageRequest>,
) -> Result<impl IntoResponse> {
    payload.validate()?;

    // Verify receiver exists
    let _receiver = state
        .user_repository
        .find_by_id(payload.receiver_id)
        .await?
        .ok_or(AppError::NotFound("Receiver not found".to_string()))?;

    // Create message
    let message = state
        .message_service
        .send_message(user_id, payload.clone())
        .await?;

    // Broadcast message via WebSocket
    let ws_message = crate::websocket::types::WsMessage::ChatMessage(crate::websocket::types::ChatMessagePayload {
        id: message.id,
        sender_id: user_id,
        receiver_id: payload.receiver_id,
        content: message.content.clone(),
        image_url: message.image_url.clone(),
        created_at: message.created_at.to_rfc3339(),
    });
    state.ws_connections.send_to_user(&payload.receiver_id, ws_message);

    // Create notification for receiver
    let notification_message = if message.image_url.is_some() {
        format!("New message with image from user")
    } else {
        format!("New message: {}", &message.content)
    };

    let _ = state
        .notification_repository
        .create(payload.receiver_id, None, &notification_message)
        .await;

    Ok((StatusCode::CREATED, Json(MessageResponse::from(message))))
}

// ... (get_conversation)
#[utoipa::path(
    get,
    path = "/api/messages/conversations/{other_user_id}",
    tag = "messages",
    params(
        ("other_user_id" = Uuid, Path, description = "Other user ID")
    ),
    responses(
        (status = 200, description = "Conversation messages", body = Vec<MessageResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Receiver not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_conversation(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(other_user_id): Path<Uuid>,
    Query(query): Query<MessageQuery>,
) -> Result<impl IntoResponse> {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(50);
    let offset = ((page - 1) * limit) as i64;

    let messages = state
        .message_service
        .get_conversation(user_id, other_user_id, limit as i64, offset)
        .await?;

    // Mark messages from other user as read
    let _ = state
        .message_service
        .mark_conversation_as_read(user_id, other_user_id)
        .await;

    let message_responses: Vec<MessageResponse> = messages
        .into_iter()
        .map(MessageResponse::from)
        .collect();

    let total = message_responses.len() as i64;
    let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;

    let response = PaginatedResponse {
        data: message_responses,
        total,
        page,
        limit,
        total_pages,
    };

    Ok((StatusCode::OK, Json(response)))
}

// ... (get_conversations)
#[utoipa::path(
    get,
    path = "/api/messages/conversations",
    tag = "messages",
    responses(
        (status = 200, description = "List of conversations", body = Vec<MessageResponse>),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_conversations(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Result<impl IntoResponse> {
    let conversations = state
        .message_service
        .get_conversations(user_id)
        .await?;

    Ok((StatusCode::OK, Json(conversations)))
}

// ... (mark_message_read)
#[utoipa::path(
    put,
    path = "/api/messages/{message_id}/read",
    tag = "messages",
    params(
        ("message_id" = Uuid, Path, description = "Message ID")
    ),
    responses(
        (status = 200, description = "Message marked as read"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Message not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn mark_message_read(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(message_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    state
        .message_service
        .mark_read(user_id, message_id)
        .await?;

    Ok(StatusCode::OK)
}
