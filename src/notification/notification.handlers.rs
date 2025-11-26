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
use std::convert::Infallible;
use uuid::Uuid;

use crate::{
    error::{AppError, Result},
    state::AppState,
};
use super::{
    dto::UpdateNotificationPreferencesRequest,
    models::Notification,
};

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
    let notifications = state.notification_repository.find_all_by_user(user_id).await?;

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
) -> Sse<impl Stream<Item = std::result::Result<Event, Infallible>>> {
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
    let notification = state.notification_repository.mark_as_read(notification_id, user_id)
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
    let rows_affected = state.notification_repository.delete(notification_id, user_id).await?;

    if rows_affected == 0 {
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
    state.user_repository.update_notification_preferences(user_id, payload.notification_enabled).await?;

    Ok(StatusCode::OK)
}
