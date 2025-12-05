use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::{
    error::AppError,
    middleware::AuthUser,
    state::AppState,
};

/// Middleware to check if user is an admin
pub async fn admin_middleware<B>(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, AppError> {
    // Get user from database
    let user = state
        .user_repository
        .find_by_id(user_id)
        .await?
        .ok_or(AppError::Unauthorized("User not found".to_string()))?;

    // Check if user is active
    if !user.is_active {
        return Err(AppError::Forbidden(
            "Account is deactivated".to_string(),
        ));
    }

    // Check if user is admin
    if !user.is_admin {
        return Err(AppError::Forbidden(
            "Admin access required".to_string(),
        ));
    }

    Ok(next.run(request).await)
}
