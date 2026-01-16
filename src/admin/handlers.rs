use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;
use validator::Validate;
use crate::{
    error::{Result, AppError},
    state::AppState,
    task::{
        task_dto::PaginatedResponse,
        task_models::Task,
        task_repository::TaskFilters,
    },
    user::{
        user_models::UserResponse,
        user_handlers::PaginationParams,
    },
    admin::dto::{AdminUpdateUserRequest, UpdateUserStatusRequest, UpdateAdminStatusRequest},
};

#[derive(serde::Deserialize)]
pub struct AdminTaskFilters {
    pub status: Option<String>,
    pub statuses: Option<Vec<String>>,
    pub priority: Option<String>,
    pub priorities: Option<Vec<String>>,
    pub search: Option<String>,
    pub created_from: Option<chrono::DateTime<chrono::Utc>>,
    pub created_to: Option<chrono::DateTime<chrono::Utc>>,
    pub due_from: Option<chrono::DateTime<chrono::Utc>>,
    pub due_to: Option<chrono::DateTime<chrono::Utc>>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub user_id: Option<Uuid>,
}

/// Get all tasks (admin only)
#[utoipa::path(
    get,
    path = "/api/admin/tasks",
    tag = "admin",
    params(
        ("status" = Option<String>, Query, description = "Filter by status"),
        ("statuses" = Option<[String]>, Query, description = "Multiple statuses"),
        ("priority" = Option<String>, Query, description = "Filter by priority"),
        ("priorities" = Option<[String]>, Query, description = "Multiple priorities"),
        ("search" = Option<String>, Query, description = "Search by title"),
        ("created_from" = Option<DateTime<Utc>>, Query, description = "Created from"),
        ("created_to" = Option<DateTime<Utc>>, Query, description = "Created to"),
        ("due_from" = Option<DateTime<Utc>>, Query, description = "Due from"),
        ("due_to" = Option<DateTime<Utc>>, Query, description = "Due to"),
        ("user_id" = Option<Uuid>, Query, description = "Filter by user ID"),
        ("page" = Option<u32>, Query, description = "Page number"),
        ("limit" = Option<u32>, Query, description = "Items per page")
    ),
    responses(
        (status = 200, description = "Tasks retrieved successfully", body = PaginatedResponse<Task>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin access required")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_all_tasks(
    State(state): State<AppState>,
    Query(filters): Query<AdminTaskFilters>,
) -> Result<Json<PaginatedResponse<Task>>> {
    let page = filters.page.unwrap_or(1);
    let limit = filters.limit.unwrap_or(10);

    let repo_filters = TaskFilters {
        status: filters.status,
        statuses: filters.statuses,
        priority: filters.priority,
        priorities: filters.priorities,
        search: filters.search,
        created_from: filters.created_from,
        created_to: filters.created_to,
        due_from: filters.due_from,
        due_to: filters.due_to,
        sort_by: filters.sort_by,
        sort_order: filters.sort_order,
        page: Some(page),
        limit: Some(limit),
        user_id: filters.user_id,
    };

    let (tasks, total) = state.admin_service.list_tasks(repo_filters).await?;
    let total_pages = (total as f64 / limit as f64).ceil() as u32;

    Ok(Json(PaginatedResponse {
        data: tasks,
        total,
        page,
        limit,
        total_pages,
    }))
}

/// Get tasks for a specific user (admin only)
#[utoipa::path(
    get,
    path = "/api/admin/users/{user_id}/tasks",
    tag = "admin",
    params(
        ("user_id" = Uuid, Path, description = "User ID"),
        ("status" = Option<String>, Query, description = "Filter by status"),
        ("statuses" = Option<[String]>, Query, description = "Multiple statuses"),
        ("priority" = Option<String>, Query, description = "Filter by priority"),
        ("priorities" = Option<[String]>, Query, description = "Multiple priorities"),
        ("search" = Option<String>, Query, description = "Search by title"),
        ("created_from" = Option<DateTime<Utc>>, Query, description = "Created from"),
        ("created_to" = Option<DateTime<Utc>>, Query, description = "Created to"),
        ("due_from" = Option<DateTime<Utc>>, Query, description = "Due from"),
        ("due_to" = Option<DateTime<Utc>>, Query, description = "Due to"),
        ("page" = Option<u32>, Query, description = "Page number"),
        ("limit" = Option<u32>, Query, description = "Items per page")
    ),
    responses(
        (status = 200, description = "User tasks retrieved successfully", body = PaginatedResponse<Task>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin access required"),
        (status = 404, description = "User not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_user_tasks(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Query(filters): Query<AdminTaskFilters>,
) -> Result<Json<PaginatedResponse<Task>>> {
    // Verify user exists first
    if state.user_repository.find_by_id(user_id).await?.is_none() {
        return Err(AppError::NotFound("User not found".to_string()));
    }

    let page = filters.page.unwrap_or(1);
    let limit = filters.limit.unwrap_or(10);

    let repo_filters = TaskFilters {
        status: filters.status,
        statuses: filters.statuses,
        priority: filters.priority,
        priorities: filters.priorities,
        search: filters.search,
        created_from: filters.created_from,
        created_to: filters.created_to,
        due_from: filters.due_from,
        due_to: filters.due_to,
        sort_by: filters.sort_by,
        sort_order: filters.sort_order,
        page: Some(page),
        limit: Some(limit),
        user_id: Some(user_id),
    };

    let (tasks, total) = state.admin_service.list_tasks(repo_filters).await?;
    let total_pages = (total as f64 / limit as f64).ceil() as u32;

    Ok(Json(PaginatedResponse {
        data: tasks,
        total,
        page,
        limit,
        total_pages,
    }))
}

/// Delete task (admin only)
#[utoipa::path(
    delete,
    path = "/api/admin/tasks/{task_id}",
    tag = "admin",
    params(
        ("task_id" = Uuid, Path, description = "Task ID")
    ),
    responses(
        (status = 204, description = "Task deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Admin access required"),
        (status = 404, description = "Task not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_task(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
) -> Result<StatusCode> {
    state.admin_service.delete_task(task_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

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
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse> {
    let page = params.page.unwrap_or(1).max(1);
    let limit = params.limit.unwrap_or(10).min(100).max(1);
    let offset = ((page - 1) * limit) as i64;

    let users = state
        .admin_service
        .list_users(limit as i64, offset)
        .await?;

    let total = state.admin_service.count_users().await?;
    let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;

    let user_responses: Vec<UserResponse> = users
        .into_iter()
        .map(|u| u.into())
        .collect();

    let response = PaginatedResponse {
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
        ("user_id" = Uuid, Path, description = "User ID")
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
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let user = state
        .user_repository
        .find_by_id(user_id)
        .await?
        .ok_or(AppError::NotFound("User not found".to_string()))?;

    Ok((StatusCode::OK, Json(UserResponse::from(user))))
}

/// Update user (admin only)
#[utoipa::path(
    put,
    path = "/api/admin/users/{user_id}",
    tag = "admin",
    params(
        ("user_id" = Uuid, Path, description = "User ID")
    ),
    request_body = AdminUpdateUserRequest,
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
    Path(user_id): Path<Uuid>,
    Json(payload): Json<AdminUpdateUserRequest>,
) -> Result<impl IntoResponse> {
    payload.validate()?;

    let user = state
        .admin_service
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

    Ok((StatusCode::OK, Json(UserResponse::from(user))))
}

/// Delete user (admin only)
#[utoipa::path(
    delete,
    path = "/api/admin/users/{user_id}",
    tag = "admin",
    params(
        ("user_id" = Uuid, Path, description = "User ID")
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
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    // Verify user exists
    let _ = state
        .user_repository
        .find_by_id(user_id)
        .await?
        .ok_or(AppError::NotFound("User not found".to_string()))?;

    state.admin_service.delete_user(user_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Update user active status (admin only)
#[utoipa::path(
    patch,
    path = "/api/admin/users/{user_id}/status",
    tag = "admin",
    params(
        ("user_id" = Uuid, Path, description = "User ID")
    ),
    request_body = UpdateUserStatusRequest,
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
    Path(user_id): Path<Uuid>,
    Json(payload): Json<UpdateUserStatusRequest>,
) -> Result<impl IntoResponse> {
    let user = state
        .admin_service
        .update_user_active_status(user_id, payload.is_active)
        .await?;

    Ok((StatusCode::OK, Json(UserResponse::from(user))))
}

/// Update user admin status (admin only)
#[utoipa::path(
    patch,
    path = "/api/admin/users/{user_id}/admin",
    tag = "admin",
    params(
        ("user_id" = Uuid, Path, description = "User ID")
    ),
    request_body = UpdateAdminStatusRequest,
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
    Path(user_id): Path<Uuid>,
    Json(payload): Json<UpdateAdminStatusRequest>,
) -> Result<impl IntoResponse> {
    let user = state
        .admin_service
        .update_user_admin_status(user_id, payload.is_admin)
        .await?;

    Ok((StatusCode::OK, Json(UserResponse::from(user))))
}
