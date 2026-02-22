use axum::{
    extract::{Path, Query, State, WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::{AppError, Result},
    middleware::AuthUser,
    state::AppState,
    video_call::{
        video_call_dto::{CallHistoryParams, InitiateCallRequest, AddParticipantRequest},
        media_session::{run_media_session, MediaSessionConfig},
    },
};

// ─────────────────────────────────────────────────────────────────────────────
// REST handlers (call lifecycle)
// ─────────────────────────────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/video-calls",
    tag = "video-calls",
    request_body = InitiateCallRequest,
    responses(
        (status = 201, description = "Call initiated successfully", body = VideoCallResponse),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "User not found"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn initiate_call(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(payload): Json<InitiateCallRequest>,
) -> Result<impl IntoResponse> {
    payload
        .validate()
        .map_err(|e| AppError::BadRequest(format!("Validation error: {}", e)))?;

    let call_type = payload.call_type.unwrap_or_else(|| "video".to_string());

    // Validate that at least one target is given
    if payload.receiver_id.is_none() && payload.group_id.is_none() {
        return Err(AppError::BadRequest(
            "Either receiver_id or group_id must be provided".to_string(),
        ));
    }

    // Verify receiver exists (for direct calls)
    if let Some(receiver_id) = payload.receiver_id {
        let _receiver = state
            .user_repository
            .find_by_id(receiver_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Receiver not found".to_string()))?;
    }

    let call = state
        .video_call_service
        .initiate_call(user_id, payload.receiver_id, payload.group_id, call_type)
        .await?;

    Ok((StatusCode::CREATED, Json(call)))
}

#[utoipa::path(
    post,
    path = "/api/video-calls/{call_id}/accept",
    tag = "video-calls",
    params(
        ("call_id" = Uuid, Path, description = "Call ID")
    ),
    responses(
        (status = 200, description = "Call accepted successfully", body = VideoCallResponse),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Call not found"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn accept_call(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(call_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let call = state
        .video_call_service
        .accept_call(call_id, user_id)
        .await?;

    Ok(Json(call))
}

#[utoipa::path(
    post,
    path = "/api/video-calls/{call_id}/reject",
    tag = "video-calls",
    params(
        ("call_id" = Uuid, Path, description = "Call ID")
    ),
    responses(
        (status = 200, description = "Call rejected successfully", body = VideoCallResponse),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Call not found"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn reject_call(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(call_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let call = state
        .video_call_service
        .reject_call(call_id, user_id)
        .await?;

    Ok(Json(call))
}

#[utoipa::path(
    post,
    path = "/api/video-calls/{call_id}/end",
    tag = "video-calls",
    params(
        ("call_id" = Uuid, Path, description = "Call ID")
    ),
    responses(
        (status = 200, description = "Call ended successfully", body = VideoCallResponse),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Call not found"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn end_call(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(call_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let call = state
        .video_call_service
        .end_call(call_id, user_id)
        .await?;

    Ok(Json(call))
}

#[utoipa::path(
    get,
    path = "/api/video-calls/{call_id}",
    tag = "video-calls",
    params(
        ("call_id" = Uuid, Path, description = "Call ID")
    ),
    responses(
        (status = 200, description = "Call details", body = VideoCallResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Call not found"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_call(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(call_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let call = state
        .video_call_service
        .get_call(call_id, user_id)
        .await?;

    Ok(Json(call))
}

#[utoipa::path(
    get,
    path = "/api/video-calls",
    tag = "video-calls",
    params(
        ("limit" = Option<i64>, Query, description = "Limit for pagination (default: 20)"),
        ("offset" = Option<i64>, Query, description = "Offset for pagination (default: 0)"),
    ),
    responses(
        (status = 200, description = "Call history retrieved successfully", body = Vec<VideoCallResponse>),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_call_history(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Query(params): Query<CallHistoryParams>,
) -> Result<impl IntoResponse> {
    let limit = params.limit.min(100).max(1);
    let offset = params.offset.max(0);

    let (calls, total) = state
        .video_call_service
        .get_call_history(user_id, limit, offset)
        .await?;

    Ok(Json(serde_json::json!({
        "data": calls,
        "total": total,
        "limit": limit,
        "offset": offset,
        "total_pages": (total as f64 / limit as f64).ceil() as i64,
    })))
}

#[utoipa::path(
    get,
    path = "/api/video-calls/active",
    tag = "video-calls",
    responses(
        (status = 200, description = "Active calls retrieved successfully", body = Vec<VideoCallResponse>),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_active_calls(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Result<impl IntoResponse> {
    let calls = state
        .video_call_service
        .get_active_calls(user_id)
        .await?;

    Ok(Json(calls))
}

/// Add a participant to an ongoing call
#[utoipa::path(
    post,
    path = "/api/video-calls/{call_id}/participants",
    tag = "video-calls",
    params(
        ("call_id" = Uuid, Path, description = "Call ID")
    ),
    request_body = AddParticipantRequest,
    responses(
        (status = 200, description = "Participant added successfully", body = VideoCallResponse),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - not an active participant"),
        (status = 404, description = "Call or user not found"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn add_participant(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(call_id): Path<Uuid>,
    Json(payload): Json<AddParticipantRequest>,
) -> Result<impl IntoResponse> {
    payload
        .validate()
        .map_err(|e| AppError::BadRequest(format!("Validation error: {}", e)))?;

    let new_participant_id = payload.user_id.ok_or_else(|| {
        AppError::BadRequest("user_id is required".to_string())
    })?;

    // Verify the user to add exists
    let _user = state
        .user_repository
        .find_by_id(new_participant_id)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let call = state
        .video_call_service
        .add_participant(call_id, user_id, new_participant_id)
        .await?;

    Ok(Json(call))
}

// ─────────────────────────────────────────────────────────────────────────────
// WebSocket media relay endpoint (videocall-rs architecture)
//
// GET /api/video-calls/{call_id}/ws
//
// After a call is accepted (status = "active"), both participants connect here.
// This endpoint upgrades to WebSocket and enters the NATS-based media relay loop:
//
//   Client sends binary frames  →  published to NATS `call.{call_id}.{user_id}`
//   NATS delivers frames        →  forwarded as binary to this client's WebSocket
//
// No WebRTC. No SDP. No ICE. The client is responsible for encoding media into
// binary frames (e.g. protobuf-wrapped audio/video, compatible with videocall-rs
// client libraries).
//
// Returns 503 if NATS is not configured (NATS_URL env var not set).
// ─────────────────────────────────────────────────────────────────────────────
pub async fn join_call_media(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(call_id): Path<Uuid>,
) -> impl IntoResponse {
    // Require NATS to be available
    let nats = match state.nats_client.clone() {
        Some(n) => n,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                "Media relay is not configured (NATS_URL not set)",
            )
                .into_response();
        }
    };

    // Verify user is a participant of this call and it is in a joinable state
    match state.video_call_service.get_call(call_id, user_id).await {
        Ok(call) => {
            use crate::video_call::video_call_models::CallStatus;
            let joinable = matches!(
                call.status,
                CallStatus::Active | CallStatus::Ringing | CallStatus::Initiating
            );
            if !joinable {
                return (
                    StatusCode::BAD_REQUEST,
                    "Call is not active",
                )
                    .into_response();
            }
        }
        Err(e) => {
            let status = match &e {
                AppError::NotFound(_) => StatusCode::NOT_FOUND,
                AppError::Forbidden(_) => StatusCode::FORBIDDEN,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            return (status, e.to_string()).into_response();
        }
    }

    // Upgrade to WebSocket and hand off to the NATS relay session
    ws.on_upgrade(move |socket| {
        run_media_session(
            MediaSessionConfig {
                call_id,
                user_id,
                nats,
            },
            socket,
        )
    })
}

use uuid::Uuid;
use validator::Validate;

use crate::{
    error::{AppError, Result},
    middleware::AuthUser,
    state::AppState,
    video_call::{
        video_call_dto::{CallHistoryParams, InitiateCallRequest, AddParticipantRequest},
    },
};

#[utoipa::path(
    post,
    path = "/api/video-calls",
    tag = "video-calls",
    request_body = InitiateCallRequest,
    responses(
        (status = 201, description = "Call initiated successfully", body = VideoCallResponse),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "User not found"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn initiate_call(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(payload): Json<InitiateCallRequest>,
) -> Result<impl IntoResponse> {
    payload
        .validate()
        .map_err(|e| AppError::BadRequest(format!("Validation error: {}", e)))?;

    let call_type = payload.call_type.unwrap_or_else(|| "video".to_string());

    // Validate that at least one target is given
    if payload.receiver_id.is_none() && payload.group_id.is_none() {
        return Err(AppError::BadRequest(
            "Either receiver_id or group_id must be provided".to_string(),
        ));
    }

    // Verify receiver exists (for direct calls)
    if let Some(receiver_id) = payload.receiver_id {
        let _receiver = state
            .user_repository
            .find_by_id(receiver_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Receiver not found".to_string()))?;
    }

    let call = state
        .video_call_service
        .initiate_call(user_id, payload.receiver_id, payload.group_id, call_type)
        .await?;

    Ok((StatusCode::CREATED, Json(call)))
}

#[utoipa::path(
    post,
    path = "/api/video-calls/{call_id}/accept",
    tag = "video-calls",
    params(
        ("call_id" = Uuid, Path, description = "Call ID")
    ),
    responses(
        (status = 200, description = "Call accepted successfully", body = VideoCallResponse),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Call not found"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn accept_call(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(call_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let call = state
        .video_call_service
        .accept_call(call_id, user_id)
        .await?;

    Ok(Json(call))
}

#[utoipa::path(
    post,
    path = "/api/video-calls/{call_id}/reject",
    tag = "video-calls",
    params(
        ("call_id" = Uuid, Path, description = "Call ID")
    ),
    responses(
        (status = 200, description = "Call rejected successfully", body = VideoCallResponse),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Call not found"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn reject_call(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(call_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let call = state
        .video_call_service
        .reject_call(call_id, user_id)
        .await?;

    Ok(Json(call))
}

#[utoipa::path(
    post,
    path = "/api/video-calls/{call_id}/end",
    tag = "video-calls",
    params(
        ("call_id" = Uuid, Path, description = "Call ID")
    ),
    responses(
        (status = 200, description = "Call ended successfully", body = VideoCallResponse),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Call not found"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn end_call(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(call_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let call = state
        .video_call_service
        .end_call(call_id, user_id)
        .await?;

    Ok(Json(call))
}

#[utoipa::path(
    get,
    path = "/api/video-calls/{call_id}",
    tag = "video-calls",
    params(
        ("call_id" = Uuid, Path, description = "Call ID")
    ),
    responses(
        (status = 200, description = "Call details", body = VideoCallResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Call not found"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_call(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(call_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let call = state
        .video_call_service
        .get_call(call_id, user_id)
        .await?;

    Ok(Json(call))
}

#[utoipa::path(
    get,
    path = "/api/video-calls",
    tag = "video-calls",
    params(
        ("limit" = Option<i64>, Query, description = "Limit for pagination (default: 20)"),
        ("offset" = Option<i64>, Query, description = "Offset for pagination (default: 0)"),
    ),
    responses(
        (status = 200, description = "Call history retrieved successfully", body = Vec<VideoCallResponse>),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_call_history(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Query(params): Query<CallHistoryParams>,
) -> Result<impl IntoResponse> {
    let limit = params.limit.min(100).max(1);
    let offset = params.offset.max(0);

    let (calls, total) = state
        .video_call_service
        .get_call_history(user_id, limit, offset)
        .await?;

    Ok(Json(serde_json::json!({
        "data": calls,
        "total": total,
        "limit": limit,
        "offset": offset,
        "total_pages": (total as f64 / limit as f64).ceil() as i64,
    })))
}

#[utoipa::path(
    get,
    path = "/api/video-calls/active",
    tag = "video-calls",
    responses(
        (status = 200, description = "Active calls retrieved successfully", body = Vec<VideoCallResponse>),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_active_calls(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Result<impl IntoResponse> {
    let calls = state
        .video_call_service
        .get_active_calls(user_id)
        .await?;

    Ok(Json(calls))
}

/// Add a participant to an ongoing call
#[utoipa::path(
    post,
    path = "/api/video-calls/{call_id}/participants",
    tag = "video-calls",
    params(
        ("call_id" = Uuid, Path, description = "Call ID")
    ),
    request_body = AddParticipantRequest,
    responses(
        (status = 200, description = "Participant added successfully", body = VideoCallResponse),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - not an active participant"),
        (status = 404, description = "Call or user not found"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn add_participant(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(call_id): Path<Uuid>,
    Json(payload): Json<AddParticipantRequest>,
) -> Result<impl IntoResponse> {
    payload
        .validate()
        .map_err(|e| AppError::BadRequest(format!("Validation error: {}", e)))?;

    let new_participant_id = payload.user_id.ok_or_else(|| {
        AppError::BadRequest("user_id is required".to_string())
    })?;

    // Verify the user to add exists
    let _user = state
        .user_repository
        .find_by_id(new_participant_id)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let call = state
        .video_call_service
        .add_participant(call_id, user_id, new_participant_id)
        .await?;

    Ok(Json(call))
}
