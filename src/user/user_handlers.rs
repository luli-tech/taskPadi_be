use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use validator::Validate;

use crate::{
    error::Result,
    middleware::AuthUser,
    state::AppState,
    user::user_dto::UpdateProfileRequest,
};

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

// Admin endpoints

/// Get all users (admin only)
#[utoipa::path(
    get,
    path = "/api/admin/users",
    tag = "admin",
    responses(
        (status = 200, description = "Users retrieved successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin access required")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_all_users(
    State(state): State<AppState>,
    Query(query): Query<crate::task::task_dto::PaginatedResponse<()>>,
) -> Result<impl IntoResponse> {
    let page = query.page.max(1);
    let limit = query.limit.min(100).max(1);
    let offset = ((page - 1) * limit) as i64;

    let users = state
        .user_repository
        .find_all(limit as i64, offset)
        .await?;

    let total = state.user_repository.count_all().await?;
    let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;

    let user_responses: Vec<crate::user::user_models::UserResponse> = users
        .into_iter()
        .map(|u| u.into())
        .collect();

    let response = crate::task::task_dto::PaginatedResponse {
        data: user_responses,
        total,
        page,
        limit,
        total_pages,
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Get specific user by ID (admin only)
#[utoipa::path(
    get,
    path = "/api/admin/users/{user_id}",
    tag = "admin",
    params(
        ("user_id" = uuid::Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User retrieved successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin access required"),
        (status = 404, description = "User not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_user_by_id(
    State(state): State<AppState>,
    Path(user_id): Path<uuid::Uuid>,
) -> Result<impl IntoResponse> {
    let user = state
        .user_repository
        .find_by_id(user_id)
        .await?
        .ok_or(crate::error::AppError::NotFound("User not found".to_string()))?;

    Ok((StatusCode::OK, Json(crate::user::user_models::UserResponse::from(user))))
}

/// Update user (admin only)
#[utoipa::path(
    put,
    path = "/api/admin/users/{user_id}",
    tag = "admin",
    params(
        ("user_id" = uuid::Uuid, Path, description = "User ID")
    ),
    request_body = crate::user::user_dto::AdminUpdateUserRequest,
    responses(
        (status = 200, description = "User updated successfully"),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin access required"),
        (status = 404, description = "User not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn admin_update_user(
    State(state): State<AppState>,
    Path(user_id): Path<uuid::Uuid>,
    Json(payload): Json<crate::user::user_dto::AdminUpdateUserRequest>,
) -> Result<impl IntoResponse> {
    payload.validate()?;

    let user = state
        .user_repository
        .admin_update_user(
            user_id,
            payload.username,
            payload.email,
            payload.bio,
            payload.theme,
            payload.avatar_url,
            payload.is_admin,
            payload.is_active,
        )
        .await?;

    Ok((StatusCode::OK, Json(crate::user::user_models::UserResponse::from(user))))
}

/// Delete user (admin only)
#[utoipa::path(
    delete,
    path = "/api/admin/users/{user_id}",
    tag = "admin",
    params(
        ("user_id" = uuid::Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 204, description = "User deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin access required"),
        (status = 404, description = "User not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_user(
    State(state): State<AppState>,
    Path(user_id): Path<uuid::Uuid>,
) -> Result<impl IntoResponse> {
    // Verify user exists
    let _ = state
        .user_repository
        .find_by_id(user_id)
        .await?
        .ok_or(crate::error::AppError::NotFound("User not found".to_string()))?;

    state.user_repository.delete_user(user_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Update user active status (admin only)
#[utoipa::path(
    patch,
    path = "/api/admin/users/{user_id}/status",
    tag = "admin",
    params(
        ("user_id" = uuid::Uuid, Path, description = "User ID")
    ),
    request_body = crate::user::user_dto::UpdateUserStatusRequest,
    responses(
        (status = 200, description = "User status updated successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin access required"),
        (status = 404, description = "User not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_user_status(
    State(state): State<AppState>,
    Path(user_id): Path<uuid::Uuid>,
    Json(payload): Json<crate::user::user_dto::UpdateUserStatusRequest>,
) -> Result<impl IntoResponse> {
    let user = state
        .user_repository
        .update_active_status(user_id, payload.is_active)
        .await?;

    Ok((StatusCode::OK, Json(crate::user::user_models::UserResponse::from(user))))
}

/// Update user admin status (admin only)
#[utoipa::path(
    patch,
    path = "/api/admin/users/{user_id}/admin",
    tag = "admin",
    params(
        ("user_id" = uuid::Uuid, Path, description = "User ID")
    ),
    request_body = crate::user::user_dto::UpdateAdminStatusRequest,
    responses(
        (status = 200, description = "User admin status updated successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin access required"),
        (status = 404, description = "User not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_admin_status(
    State(state): State<AppState>,
    Path(user_id): Path<uuid::Uuid>,
    Json(payload): Json<crate::user::user_dto::UpdateAdminStatusRequest>,
) -> Result<impl IntoResponse> {
    let user = state
        .user_repository
        .update_admin_status(user_id, payload.is_admin)
        .await?;

    Ok((StatusCode::OK, Json(crate::user::user_models::UserResponse::from(user))))
}
