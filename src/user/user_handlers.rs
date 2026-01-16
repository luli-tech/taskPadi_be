use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use validator::Validate;

use crate::{
    error::Result,
    middleware::AuthUser,
    state::AppState,
    user::user_dto::UpdateProfileRequest,
};

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

/// Get current user profile
#[utoipa::path(
    get,
    path = "/api/users/me",
    tag = "users",
    responses(
        (status = 200, description = "User profile retrieved successfully"),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_current_user(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Result<impl IntoResponse> {
    let user = state
        .user_service
        .get_current_user(user_id)
        .await?;

    Ok((StatusCode::OK, Json(user)))
}

/// Update current user profile
#[utoipa::path(
    put,
    path = "/api/users/me",
    tag = "users",
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, description = "Profile updated successfully"),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_current_user(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<impl IntoResponse> {
    payload.validate()?;

    let user = state
        .user_service
        .update_current_user(user_id, payload)
        .await?;

    Ok((StatusCode::OK, Json(user)))
}

/// Get user task statistics
#[utoipa::path(
    get,
    path = "/api/users/me/stats",
    tag = "users",
    responses(
        (status = 200, description = "User statistics retrieved successfully"),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_user_stats(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Result<impl IntoResponse> {
    let stats = state.user_service.get_user_stats(user_id).await?;

    Ok((StatusCode::OK, Json(stats)))
}


