use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use crate::{
    error::{AppError, Result},
    middleware::auth::AuthUser,
};

pub async fn admin_authorization(
    AuthUser(user_id): AuthUser,
    request: Request,
    next: Next,
) -> Result<Response> {
    // We need to fetch the user to check the role.
    // Since AuthUser only gives us the ID (from the token claims), we might need to query the DB
    // OR we could have included the role in the token claims.
    //
    // In our updated JWT implementation, we DID include the role in the claims.
    // However, AuthUser extractor currently only extracts the ID (sub).
    //
    // To avoid a DB call on every admin request, we should update AuthUser to extract the role too.
    // BUT, for now, let's assume we query the DB or update AuthUser later.
    //
    // Actually, checking the DB is safer for role revocation.
    // Let's retrieve the state from the request extensions to get the repository.
    
    let state = request
        .extensions()
        .get::<crate::state::AppState>()
        .ok_or(AppError::InternalError)?;

    let user = state.user_repository.find_by_id(user_id).await?
      .ok_or(AppError::Unauthorized("Invalid credentials".to_string()))?;


    if user.role != "admin" {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    Ok(next.run(request).await)
}
