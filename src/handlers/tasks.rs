use crate::{
    dto::{CreateTaskRequest, UpdateTaskRequest, UpdateTaskStatusRequest},
    error::{AppError, Result},
    models::Task,
    state::AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::{Request, StatusCode},
    response::IntoResponse,
    Extension, Json,
};
use serde::Deserialize;
use sqlx::query_as;
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize)]
pub struct TaskFilters {
    status: Option<String>,
    priority: Option<String>,
}

/// Get all tasks for the authenticated user
#[utoipa::path(
    get,
    path = "/api/tasks",
    params(
        ("status" = Option<String>, Query, description = "Filter by status"),
        ("priority" = Option<String>, Query, description = "Filter by priority")
    ),
    responses(
        (status = 200, description = "List of tasks", body = Vec<Task>),
        (status = 401, description = "Unauthorized")
    ),
    tag = "tasks",
    security(("bearer_auth" = []))
)]
pub async fn get_tasks(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Query(filters): Query<TaskFilters>,
) -> Result<Json<Vec<Task>>> {
    let mut query = "SELECT * FROM tasks WHERE user_id = $1".to_string();
    let mut params_count = 1;

    if filters.status.is_some() {
        params_count += 1;
        query.push_str(&format!(" AND status = ${}", params_count));
    }

    if filters.priority.is_some() {
        params_count += 1;
        query.push_str(&format!(" AND priority = ${}", params_count));
    }

    query.push_str(" ORDER BY created_at DESC");

    let mut db_query = sqlx::query_as::<_, Task>(&query).bind(user_id);

    if let Some(status) = filters.status {
        db_query = db_query.bind(status);
    }

    if let Some(priority) = filters.priority {
        db_query = db_query.bind(priority);
    }

    let tasks = db_query.fetch_all(&state.db).await?;

    Ok(Json(tasks))
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
    let task = query_as::<_, Task>("SELECT * FROM tasks WHERE id = $1 AND user_id = $2")
        .bind(task_id)
        .bind(user_id)
        .fetch_optional(&state.db)
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

    let task = query_as::<_, Task>(
        "INSERT INTO tasks (user_id, title, description, priority, due_date, reminder_time)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING *"
    )
    .bind(user_id)
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(&priority)
    .bind(&payload.due_date)
    .bind(&payload.reminder_time)
    .fetch_one(&state.db)
    .await?;

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

    let existing_task = query_as::<_, Task>("SELECT * FROM tasks WHERE id = $1 AND user_id = $2")
        .bind(task_id)
        .bind(user_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Task not found".to_string()))?;

    let task = query_as::<_, Task>(
        "UPDATE tasks SET
            title = COALESCE($1, title),
            description = COALESCE($2, description),
            status = COALESCE($3, status),
            priority = COALESCE($4, priority),
            due_date = COALESCE($5, due_date),
            reminder_time = COALESCE($6, reminder_time),
            notified = CASE WHEN $6 IS NOT NULL THEN false ELSE notified END,
            updated_at = NOW()
         WHERE id = $7 AND user_id = $8
         RETURNING *"
    )
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(&payload.status)
    .bind(&payload.priority)
    .bind(&payload.due_date)
    .bind(&payload.reminder_time)
    .bind(task_id)
    .bind(user_id)
    .fetch_one(&state.db)
    .await?;

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
    let result = sqlx::query("DELETE FROM tasks WHERE id = $1 AND user_id = $2")
        .bind(task_id)
        .bind(user_id)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
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
    let task = query_as::<_, Task>(
        "UPDATE tasks SET status = $1, updated_at = NOW()
         WHERE id = $2 AND user_id = $3
         RETURNING *"
    )
    .bind(&payload.status)
    .bind(task_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Task not found".to_string()))?;

    Ok(Json(task))
}
