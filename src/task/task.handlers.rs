use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, sse::{Event, KeepAlive, Sse}},
    Extension, Json,
};
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
    status: Option<String>,
    priority: Option<String>,
    search: Option<String>,
    sort_by: Option<String>,
    sort_order: Option<String>,
    page: Option<u32>,
    limit: Option<u32>,
}

/// Get all tasks for the authenticated user
#[utoipa::path(
    get,
    path = "/api/tasks",
    params(
        ("status" = Option<String>, Query, description = "Filter by status"),
        ("priority" = Option<String>, Query, description = "Filter by priority"),
        ("search" = Option<String>, Query, description = "Search by title or description"),
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

    let repo_filters = crate::task::repository::TaskFilters {
        status: filters.status,
        priority: filters.priority,
        search: filters.search,
        sort_by: filters.sort_by,
        sort_order: filters.sort_order,
        page: Some(page),
        limit: Some(limit),
    };

    let (tasks, total) = state.task_repository.find_all(user_id, repo_filters).await?;

    let total_pages = (total as f64 / limit as f64).ceil() as u32;

    Ok(Json(PaginatedResponse {
        data: tasks,
        total,
        page,
        limit,
        total_pages,
    }))
}

/// Get a single task by ID
#[utoipa::path(
    get,
    path = "/api/tasks/{id}",
    params(
        ("id" = Uuid, Path, description = "Task ID")
    ),
    responses(
        (status = 200, description = "Task found", body = Task),
        (status = 404, description = "Task not found"),
        (status = 401, description = "Unauthorized")
    ),
    tag = "tasks",
    security(("bearer_auth" = []))
)]
pub async fn get_task(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<Task>> {
    let task = state.task_repository.find_by_id(task_id, user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Task not found".to_string()))?;

    Ok(Json(task))
}

/// Create a new task
#[utoipa::path(
    post,
    path = "/api/tasks",
    request_body = CreateTaskRequest,
    responses(
        (status = 201, description = "Task created", body = Task),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized")
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

    let priority = payload.priority.unwrap_or_else(|| "Medium".to_string());

    let task = state.task_repository.create(
        user_id,
        &payload.title,
        payload.description.as_deref(),
        &priority,
        payload.due_date,
        payload.reminder_time,
    ).await?;

    // Broadcast task creation
    let _ = state.task_tx.send((user_id, task.clone()));

    Ok((StatusCode::CREATED, Json(task)))
}

/// Update a task
#[utoipa::path(
    put,
    path = "/api/tasks/{id}",
    params(
        ("id" = Uuid, Path, description = "Task ID")
    ),
    request_body = UpdateTaskRequest,
    responses(
        (status = 200, description = "Task updated", body = Task),
        (status = 404, description = "Task not found"),
        (status = 401, description = "Unauthorized")
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

    let existing_task = state.task_repository.find_by_id(task_id, user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Task not found".to_string()))?;

    let task = state.task_repository.update(
        task_id,
        user_id,
        payload.title.as_deref(),
        payload.description.as_deref(),
        payload.status.as_deref(),
        payload.priority.as_deref(),
        payload.due_date,
        payload.reminder_time,
    ).await?;

    // Broadcast task update
    let _ = state.task_tx.send((user_id, task.clone()));

    Ok(Json(task))
}

/// Delete a task
#[utoipa::path(
    delete,
    path = "/api/tasks/{id}",
    params(
        ("id" = Uuid, Path, description = "Task ID")
    ),
    responses(
        (status = 204, description = "Task deleted"),
        (status = 404, description = "Task not found"),
        (status = 401, description = "Unauthorized")
    ),
    tag = "tasks",
    security(("bearer_auth" = []))
)]
pub async fn delete_task(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(task_id): Path<Uuid>,
) -> Result<StatusCode> {
    let rows_affected = state.task_repository.delete(task_id, user_id).await?;

    if rows_affected == 0 {
        return Err(AppError::NotFound("Task not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Update task status
#[utoipa::path(
    patch,
    path = "/api/tasks/{id}/status",
    params(
        ("id" = Uuid, Path, description = "Task ID")
    ),
    request_body = UpdateTaskStatusRequest,
    responses(
        (status = 200, description = "Status updated", body = Task),
        (status = 404, description = "Task not found"),
        (status = 401, description = "Unauthorized")
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
    let task = state.task_repository.update_status(task_id, user_id, &payload.status)
        .await?
    .ok_or_else(|| AppError::NotFound("Task not found".to_string()))?;

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
