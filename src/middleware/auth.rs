use crate::{auth::verify_jwt, error::AppError, state::AppState};
use axum::{
    body::Body,
    extract::{State, FromRequestParts},
    http::{Request, request::Parts},
    middleware::Next,
    response::Response,
    async_trait,
};
use uuid::Uuid;

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
       .ok_or(AppError::Unauthorized("Invalid credentials".to_string()))?


    let token = auth_header
        .strip_prefix("Bearer ")
       .ok_or(AppError::Unauthorized("Invalid credentials".to_string()))?


    let claims = verify_jwt(token, &state.config.jwt_secret)?;
    
   let user_id = Uuid::parse_str(&claims.sub)
    .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))?;


    req.extensions_mut().insert(user_id);
    
    Ok(next.run(req).await)
}

// Extractor for getting user_id from request extensions
pub struct AuthUser(pub Uuid);

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Uuid>()
            .copied()
            .map(AuthUser)
           .ok_or(AppError::Unauthorized("Invalid credentials".to_string()))?

    }
}
