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
        message_dto::{ConversationUser, SendMessageRequest},
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

    // If 1-on-1 message, verify receiver exists
    if let Some(receiver_id) = payload.receiver_id {
        let _receiver = state
            .user_repository
            .find_by_id(receiver_id)
            .await?
            .ok_or(AppError::NotFound("Receiver not found".to_string()))?;
    }

    // If group message, verify user is a member of the group
    if let Some(group_id) = payload.group_id {
        state.group_service.verify_membership(group_id, user_id).await?;
    }

    // Create and broadcast message
    let message = state
        .message_service
        .send_message(user_id, payload)
        .await?;

    Ok((StatusCode::CREATED, Json(MessageResponse::from(message))))
}

/// Get conversation messages with a specific user
#[utoipa::path(
    get,
    path = "/api/messages/{user_id}",
    tag = "messages",
    params(
        ("user_id" = Uuid, Path, description = "Other user ID to get conversation with"),
        ("page" = Option<u32>, Query, description = "Page number (default: 1)"),
        ("limit" = Option<u32>, Query, description = "Items per page (default: 50)")
    ),
    responses(
        (status = 200, description = "Paginated conversation messages", body = PaginatedResponse<MessageResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "User not found")
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

    let (messages, total) = state
        .message_service
        .get_conversation_with_count(user_id, other_user_id, limit as i64, offset)
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

/// Get all conversations for the authenticated user (1-on-1 only)
#[utoipa::path(
    get,
    path = "/api/messages/conversations",
    tag = "messages",
    responses(
        (status = 200, description = "List of conversations", body = Vec<ConversationUser>),
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

/// Get group messages
#[utoipa::path(
    get,
    path = "/api/messages/groups/{group_id}",
    tag = "messages",
    params(
        ("group_id" = Uuid, Path, description = "Group ID to get messages from"),
        ("page" = Option<u32>, Query, description = "Page number (default: 1)"),
        ("limit" = Option<u32>, Query, description = "Items per page (default: 50)")
    ),
    responses(
        (status = 200, description = "Paginated group messages", body = PaginatedResponse<MessageResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Not a member"),
        (status = 404, description = "Group not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_group_messages(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(group_id): Path<Uuid>,
    Query(query): Query<MessageQuery>,
) -> Result<impl IntoResponse> {
    // Verify user is a member of the group
    state.group_service.verify_membership(group_id, user_id).await?;

    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(50);
    let offset = ((page - 1) * limit) as i64;

    let (messages, total) = state
        .message_service
        .get_group_messages_with_count(group_id, limit as i64, offset)
        .await?;

    // Mark messages as read
    let _ = state
        .message_service
        .mark_group_messages_as_read(user_id, group_id)
        .await;

    let message_responses: Vec<MessageResponse> = messages
        .into_iter()
        .map(MessageResponse::from)
        .collect();

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

/// Mark a message as read
#[utoipa::path(
    patch,
    path = "/api/messages/{id}/read",
    tag = "messages",
    params(
        ("id" = Uuid, Path, description = "Message ID to mark as read")
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
