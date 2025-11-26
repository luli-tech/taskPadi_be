use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive},
        IntoResponse, Sse,
    },
    Json,
};
use futures::stream::Stream;
use serde::Deserialize;
use std::convert::Infallible;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::{AppError, Result},
    middleware::AuthUser,
    state::AppState,
    task::task_dto::PaginatedResponse,
    message::{
        message_dto::{ConversationUser, SendMessageRequest},
        message_models::{Message, MessageResponse},
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
    let receiver = state
        .user_repository
        .find_by_id(payload.receiver_id)
        .await?
        .ok_or(AppError::NotFound("Receiver not found".to_string()))?;

    // Create message
    let message = state
        .message_repository
        .create(
            user_id,
            payload.receiver_id,
            &payload.content,
            payload.image_url.as_deref(),
        )
        .await?;

    // Broadcast message to SSE subscribers
    let _ = state.message_tx.send((payload.receiver_id, message.clone()));

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

/// Get conversation with a specific user
#[utoipa::path(
    get,
    path = "/api/messages/{user_id}",
    tag = "messages",
    params(
        ("user_id" = Uuid, Path, description = "User ID to get conversation with")
    ),
    responses(
        (status = 200, description = "Conversation retrieved successfully", body = PaginatedResponse<MessageResponse>),
        (status = 401, description = "Unauthorized")
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
        .message_repository
        .find_conversation(user_id, other_user_id, limit as i64, offset)
        .await?;

    // Mark messages from other user as read
    let _ = state
        .message_repository
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

/// Get list of conversations
#[utoipa::path(
    get,
    path = "/api/messages/conversations",
    tag = "messages",
    responses(
        (status = 200, description = "Conversations retrieved successfully", body = Vec<ConversationUser>),
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
        .message_repository
        .find_user_conversations(user_id)
        .await?;

    Ok((StatusCode::OK, Json(conversations)))
}

/// Mark a message as read
#[utoipa::path(
    patch,
    path = "/api/messages/{id}/read",
    tag = "messages",
    params(
        ("id" = Uuid, Path, description = "Message ID")
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
        .message_repository
        .mark_as_read(message_id, user_id)
        .await?;

    Ok(StatusCode::OK)
}

/// Real-time message stream (SSE)
#[utoipa::path(
    get,
    path = "/api/messages/stream",
    tag = "messages",
    responses(
        (status = 200, description = "Message stream established"),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn message_stream(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Sse<impl Stream<Item = std::result::Result<Event, Infallible>>> {
    let rx = state.message_tx.subscribe();
    let stream = BroadcastStream::new(rx)
        .filter_map(move |result| match result {
            Ok((receiver_id, message)) if receiver_id == user_id => {
                let json = serde_json::to_string(&MessageResponse::from(message)).ok()?;
                Some(Ok(Event::default().data(json)))
            }
            _ => None,
        });

    Sse::new(stream).keep_alive(KeepAlive::default())
}
