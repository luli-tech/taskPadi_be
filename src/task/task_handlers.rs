use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, sse::{Event, KeepAlive, Sse}},
    Extension, Json,
};
use chrono::{DateTime, Utc};
use futures::stream::Stream;
use serde::Deserialize;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::{AppError, Result},
    state::AppState,
};
use super::{
    task_dto::{CreateTaskRequest, UpdateTaskRequest, UpdateTaskStatusRequest, PaginatedResponse},
    task_models::Task,
};

#[derive(Deserialize)]
pub struct TaskFilters {
    pub status: Option<String>,
    pub statuses: Option<Vec<String>>,
    pub priority: Option<String>,
    pub priorities: Option<Vec<String>>,
    pub search: Option<String>,
    pub created_from: Option<DateTime<Utc>>,
    pub created_to: Option<DateTime<Utc>>,
    pub due_from: Option<DateTime<Utc>>,
    pub due_to: Option<DateTime<Utc>>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

/// Get all tasks for the authenticated user
#[utoipa::path(
    get,
    path = "/api/tasks",
    params(
        ("status" = Option<String>, Query, description = "Filter by status"),
        ("statuses" = Option<[String]>, Query, description = "Multiple statuses"),
        ("priority" = Option<String>, Query, description = "Filter by priority"),
        ("priorities" = Option<[String]>, Query, description = "Multiple priorities"),
        ("search" = Option<String>, Query, description = "Search by title or description"),
        ("created_from" = Option<DateTime<Utc>>, Query, description = "Filter by creation date (from)"),
        ("created_to" = Option<DateTime<Utc>>, Query, description = "Filter by creation date (to)"),
        ("due_from" = Option<DateTime<Utc>>, Query, description = "Filter by due date (from)"),
        ("due_to" = Option<DateTime<Utc>>, Query, description = "Filter by due date (to)"),
        ("sort_by" = Option<String>, Query, description = "Sort by field (priority, due_date, created_at)"),
        ("sort_order" = Option<String>, Query, description = "Sort order (asc, desc)"),
        ("page" = Option<u32>, Query, description = "Page number"),
        ("limit" = Option<u32>, Query, description = "Items per page")
    ),
    responses(
        (status = 200, description = "List of tasks", body = PaginatedResponse<Task>),
        (status = 401, description = "Unauthorized")
    ),
    tag = "tasks",
    security(("bearer_auth" = []))
)]
pub async fn get_tasks(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Query(filters): Query<TaskFilters>,
) -> Result<Json<PaginatedResponse<Task>>> {
    let page = filters.page.unwrap_or(1);
    let limit = filters.limit.unwrap_or(10);

    let repo_filters = crate::task::task_repository::TaskFilters {
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
        user_id: None,
    };

    let (tasks, total) = state.task_service.list_tasks(user_id, repo_filters).await?;

    let total_pages = (total as f64 / limit as f64).ceil() as u32;

    Ok(Json(PaginatedResponse {
        data: tasks,
        total,
        page,
        limit,
        total_pages,
    }))
}

// ... (get_task)


// ... (get_task)
#[utoipa::path(
    get,
    path = "/api/tasks/{task_id}",
    params(
        ("task_id" = Uuid, Path, description = "Task ID")
    ),
    responses(
        (status = 200, description = "Task details", body = Task),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Task not found")
    ),
    tag = "tasks",
    security(("bearer_auth" = []))
)]
pub async fn get_task(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<Task>> {
    let task = state.task_service.get_task(user_id, task_id).await?;
    Ok(Json(task))
}

// ... (create_task)
#[utoipa::path(
    post,
    path = "/api/tasks",
    request_body = CreateTaskRequest,
    responses(
        (status = 201, description = "Task created", body = Task),
        (status = 401, description = "Unauthorized"),
        (status = 400, description = "Validation error")
    ),
    tag = "tasks",
    security(("bearer_auth" = []))
)]
pub async fn create_task(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<impl IntoResponse> {
    payload.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let task = state.task_service.create_task(user_id, payload).await?;

    // Broadcast task creation
    let _ = state.task_tx.send((user_id, task.clone()));

    Ok((StatusCode::CREATED, Json(task)))
}

// ... (update_task)
#[utoipa::path(
    put,
    path = "/api/tasks/{task_id}",
    params(
        ("task_id" = Uuid, Path, description = "Task ID")
    ),
    request_body = UpdateTaskRequest,
    responses(
        (status = 200, description = "Task updated", body = Task),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Task not found"),
        (status = 400, description = "Validation error")
    ),
    tag = "tasks",
    security(("bearer_auth" = []))
)]
pub async fn update_task(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(task_id): Path<Uuid>,
    Json(payload): Json<UpdateTaskRequest>,
) -> Result<Json<Task>> {
    payload.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let task = state.task_service.update_task(user_id, task_id, payload).await?;

    // Broadcast task update
    let _ = state.task_tx.send((user_id, task.clone()));

    Ok(Json(task))
}

// ... (delete_task)
#[utoipa::path(
    delete,
    path = "/api/tasks/{task_id}",
    params(
        ("task_id" = Uuid, Path, description = "Task ID")
    ),
    responses(
        (status = 204, description = "Task deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Task not found")
    ),
    tag = "tasks",
    security(("bearer_auth" = []))
)]
pub async fn delete_task(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(task_id): Path<Uuid>,
) -> Result<StatusCode> {
    let rows_affected = state.task_service.delete_task(user_id, task_id).await?;

    if rows_affected == 0 {
        return Err(AppError::NotFound("Task not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

// ... (update_task_status)
#[utoipa::path(
    patch,
    path = "/api/tasks/{task_id}/status",
    params(
        ("task_id" = Uuid, Path, description = "Task ID")
    ),
    request_body = UpdateTaskStatusRequest,
    responses(
        (status = 200, description = "Task status updated", body = Task),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Task not found"),
        (status = 400, description = "Validation error")
    ),
    tag = "tasks",
    security(("bearer_auth" = []))
)]
pub async fn update_task_status(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(task_id): Path<Uuid>,
    Json(payload): Json<UpdateTaskStatusRequest>,
) -> Result<Json<Task>> {
    let task = state.task_service.update_status(user_id, task_id, payload).await?;

    // Broadcast task status update
    let _ = state.task_tx.send((user_id, task.clone()));

    Ok(Json(task))
}

/// Real-time task stream (SSE)
#[utoipa::path(
    get,
    path = "/api/tasks/stream",
    tag = "tasks",
    responses(
        (status = 200, description = "Task stream established"),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn task_stream(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
) -> Sse<impl Stream<Item = std::result::Result<Event, std::convert::Infallible>>> {
    let rx = state.task_tx.subscribe();
    let stream = BroadcastStream::new(rx)
        .filter_map(move |result| match result {
            Ok((task_user_id, task)) if task_user_id == user_id => {
                let json = serde_json::to_string(&task).ok()?;
                Some(Ok(Event::default().data(json)))
            }
            _ => None,
        });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

// Collaboration endpoints

/// Share task with other users
#[utoipa::path(
    post,
    path = "/api/tasks/{task_id}/share",
    params(
        ("task_id" = Uuid, Path, description = "Task ID")
    ),
    request_body = super::task_dto::ShareTaskRequest,
    responses(
        (status = 200, description = "Task shared successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Only task owner can share"),
        (status = 404, description = "Task not found")
    ),
    tag = "tasks",
    security(("bearer_auth" = []))
)]
pub async fn share_task(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(task_id): Path<Uuid>,
    Json(payload): Json<super::task_dto::ShareTaskRequest>,
) -> Result<impl IntoResponse> {
    payload.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    state.task_service.share_task(task_id, payload.user_ids.clone(), user_id).await?;

    // Broadcast task shared event via WebSocket
    for shared_user_id in payload.user_ids {
        let task = state.task_service.get_task(user_id, task_id).await?;
        let ws_message = crate::websocket::types::WsMessage::TaskShared(
            crate::websocket::types::TaskSharedPayload {
                task_id,
                task_title: task.title.clone(),
                shared_by: user_id,
                shared_by_username: state.user_repository.find_by_id(user_id).await?
                    .map(|u| u.username)
                    .unwrap_or_else(|| "Unknown".to_string()),
            }
        );
        state.ws_connections.send_to_user(&shared_user_id, ws_message);
    }

    Ok(StatusCode::OK)
}

/// Remove collaborator from task
#[utoipa::path(
    delete,
    path = "/api/tasks/{task_id}/members/{user_id}",
    params(
        ("task_id" = Uuid, Path, description = "Task ID"),
        ("user_id" = Uuid, Path, description = "User ID to remove")
    ),
    responses(
        (status = 204, description = "Collaborator removed successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Only task owner can remove collaborators"),
        (status = 404, description = "Task not found")
    ),
    tag = "tasks",
    security(("bearer_auth" = []))
)]
pub async fn remove_task_member(
    State(state): State<AppState>,
    Extension(requesting_user): Extension<Uuid>,
    Path((task_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode> {
    state.task_service.remove_collaborator(task_id, user_id, requesting_user).await?;

    // Broadcast member removed event via WebSocket
    let task = state.task_service.get_task(requesting_user, task_id).await?;
    let ws_message = crate::websocket::types::WsMessage::TaskMemberRemoved(
        crate::websocket::types::TaskMemberRemovedPayload {
            task_id,
            task_title: task.title,
            removed_by: requesting_user,
        }
    );
    state.ws_connections.send_to_user(&user_id, ws_message);

    Ok(StatusCode::NO_CONTENT)
}

/// Get task members
#[utoipa::path(
    get,
    path = "/api/tasks/{task_id}/members",
    params(
        ("task_id" = Uuid, Path, description = "Task ID")
    ),
    responses(
        (status = 200, description = "Task members retrieved successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Access denied"),
        (status = 404, description = "Task not found")
    ),
    tag = "tasks",
    security(("bearer_auth" = []))
)]
pub async fn get_task_members(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<Vec<super::task_models::TaskMemberInfo>>> {
    let members = state.task_service.get_task_members(task_id, user_id).await?;
    Ok(Json(members))
}

/// Get task activity log
#[utoipa::path(
    get,
    path = "/api/tasks/{task_id}/activity",
    params(
        ("task_id" = Uuid, Path, description = "Task ID")
    ),
    responses(
        (status = 200, description = "Task activity retrieved successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Access denied"),
        (status = 404, description = "Task not found")
    ),
    tag = "tasks",
    security(("bearer_auth" = []))
)]
pub async fn get_task_activity(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<Vec<super::task_dto::TaskActivityResponse>>> {
    let activity = state.task_service.get_task_activity(task_id, user_id).await?;
    Ok(Json(activity))
}
