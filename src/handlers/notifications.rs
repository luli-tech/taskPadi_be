use crate::{
    dto::UpdateNotificationPreferencesRequest,
    error::{AppError, Result},
    models::Notification,
    state::AppState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive},
        Sse,
    },
    Extension, Json,
};
use futures::stream::{Stream, StreamExt};
use sqlx::query_as;
use std::convert::Infallible;
use uuid::Uuid;

/// Get all notifications for the authenticated user
#[utoipa::path(
    get,
    path = "/api/notifications",
    responses(
        (status = 200, description = "List of notifications", body = Vec<Notification>),
        (status = 401, description = "Unauthorized")
    ),
    tag = "notifications",
    security(("bearer_auth" = []))
)]
pub async fn get_notifications(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<Vec<Notification>>> {
    let notifications = query_as::<_, Notification>(
        "SELECT * FROM notifications WHERE user_id = $1 ORDER BY created_at DESC"
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(notifications))
}

/// Subscribe to real-time notifications via Server-Sent Events
#[utoipa::path(
    get,
    path = "/api/notifications/stream",
    responses(
        (status = 200, description = "SSE stream of notifications"),
        (status = 401, description = "Unauthorized")
    ),
    tag = "notifications",
    security(("bearer_auth" = []))
)]
pub async fn notification_stream(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.notification_tx.subscribe();

    let stream = tokio_stream::wrappers::BroadcastStream::new(rx)
        .filter_map(move |msg| async move {
            match msg {
                Ok(notification) => {
                    if notification.contains(&user_id.to_string()) {
                        Some(Ok(Event::default().data(notification)))
                    } else {
                        None
                    }
                }
                Err(_) => None,
            }
        });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// Mark notification as read
#[utoipa::path(
    patch,
    path = "/api/notifications/{id}/read",
    params(
        ("id" = Uuid, Path, description = "Notification ID")
    ),
    responses(
        (status = 200, description = "Notification marked as read", body = Notification),
        (status = 404, description = "Notification not found"),
        (status = 401, description = "Unauthorized")
    ),
    tag = "notifications",
    security(("bearer_auth" = []))
)]
pub async fn mark_notification_read(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(notification_id): Path<Uuid>,
) -> Result<Json<Notification>> {
    let notification = query_as::<_, Notification>(
        "UPDATE notifications SET is_read = true WHERE id = $1 AND user_id = $2 RETURNING *"
    )
    .bind(notification_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Notification not found".to_string()))?;

    Ok(Json(notification))
}

/// Delete a notification
#[utoipa::path(
    delete,
    path = "/api/notifications/{id}",
    params(
        ("id" = Uuid, Path, description = "Notification ID")
    ),
    responses(
        (status = 204, description = "Notification deleted"),
        (status = 404, description = "Notification not found"),
        (status = 401, description = "Unauthorized")
    ),
    tag = "notifications",
    security(("bearer_auth" = []))
)]
pub async fn delete_notification(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(notification_id): Path<Uuid>,
) -> Result<StatusCode> {
    let result = sqlx::query("DELETE FROM notifications WHERE id = $1 AND user_id = $2")
        .bind(notification_id)
        .bind(user_id)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Notification not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Update notification preferences
#[utoipa::path(
    put,
    path = "/api/notifications/preferences",
    request_body = UpdateNotificationPreferencesRequest,
    responses(
        (status = 200, description = "Preferences updated"),
        (status = 401, description = "Unauthorized")
    ),
    tag = "notifications",
    security(("bearer_auth" = []))
)]
pub async fn update_notification_preferences(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Json(payload): Json<UpdateNotificationPreferencesRequest>,
) -> Result<StatusCode> {
    sqlx::query("UPDATE users SET notification_enabled = $1 WHERE id = $2")
        .bind(payload.notification_enabled)
        .bind(user_id)
        .execute(&state.db)
        .await?;

    Ok(StatusCode::OK)
}
