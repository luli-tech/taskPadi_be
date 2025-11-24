use crate::{
    auth::{create_jwt, create_oauth_client, hash_password, verify_password, GoogleUserInfo},
    dto::{AuthResponse, LoginRequest, RegisterRequest},
    error::{AppError, Result},
    models::{User, UserResponse},
    state::AppState,
};
use axum::{extract::State, http::StatusCode, response::{IntoResponse, Redirect}, Json, extract::Query};
use oauth2::{CsrfToken, PkceCodeChallenge, Scope, AuthorizationCode, TokenResponse};
use serde::Deserialize;
use sqlx::query_as;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize)]
pub struct GoogleCallback {
    code: String,
    state: String,
}

/// Register a new user
#[utoipa::path(
    post,
    path = "/api/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered successfully", body = AuthResponse),
        (status = 400, description = "Validation error"),
        (status = 500, description = "Internal server error")
    ),
    tag = "auth"
)]
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<impl IntoResponse> {
    payload.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let password_hash = hash_password(&payload.password)?;

    let user = query_as::<_, User>(
        "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3) RETURNING *"
    )
    .bind(&payload.username)
    .bind(&payload.email)
    .bind(&password_hash)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        if e.to_string().contains("duplicate key") {
            AppError::BadRequest("User already exists".to_string())
        } else {
            AppError::Database(e)
        }
    })?;

    let token = create_jwt(
        user.id,
        &user.email,
        &state.config.jwt_secret,
        state.config.jwt_expiration_hours,
    )?;

    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
            token,
            user: user.into(),
        }),
    ))
}

/// Login with email and password
#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 401, description = "Invalid credentials"),
        (status = 500, description = "Internal server error")
    ),
    tag = "auth"
)]
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse> {
    payload.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let user = query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(&payload.email)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::Authentication("Invalid credentials".to_string()))?;

    let password_hash = user.password_hash.as_ref()
        .ok_or_else(|| AppError::Authentication("Please use Google login".to_string()))?;

    if !verify_password(&payload.password, password_hash)? {
        return Err(AppError::Authentication("Invalid credentials".to_string()));
    }

    let token = create_jwt(
        user.id,
        &user.email,
        &state.config.jwt_secret,
        state.config.jwt_expiration_hours,
    )?;

    Ok(Json(AuthResponse {
        token,
        user: user.into(),
    }))
}

/// Initiate Google OAuth flow
#[utoipa::path(
    get,
    path = "/api/auth/google",
    responses(
        (status = 302, description = "Redirect to Google OAuth"),
    ),
    tag = "auth"
)]
pub async fn google_login(State(state): State<AppState>) -> impl IntoResponse {
    let (pkce_challenge, _pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let (auth_url, _csrf_token) = state
        .oauth_client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    Redirect::to(auth_url.as_str())
}

/// Handle Google OAuth callback
#[utoipa::path(
    get,
    path = "/api/auth/google/callback",
    params(
        ("code" = String, Query, description = "Authorization code from Google"),
        ("state" = String, Query, description = "CSRF token")
    ),
    responses(
        (status = 200, description = "OAuth successful", body = AuthResponse),
        (status = 500, description = "OAuth failed")
    ),
    tag = "auth"
)]
pub async fn google_callback(
    State(state): State<AppState>,
    Query(params): Query<GoogleCallback>,
) -> Result<Json<AuthResponse>> {
    let token_result = state
        .oauth_client
        .exchange_code(AuthorizationCode::new(params.code))
        .request_async(oauth2::reqwest::async_http_client)
        .await
        .map_err(|_| AppError::Authentication("Failed to exchange code".to_string()))?;

    let access_token = token_result.access_token().secret();

    let client = reqwest::Client::new();
    let user_info: GoogleUserInfo = client
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|_| AppError::Authentication("Failed to get user info".to_string()))?
        .json()
        .await
        .map_err(|_| AppError::Authentication("Failed to parse user info".to_string()))?;

    let user = query_as::<_, User>(
        "INSERT INTO users (username, email, google_id, avatar_url)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (google_id) DO UPDATE SET
            avatar_url = EXCLUDED.avatar_url,
            updated_at = NOW()
         RETURNING *"
    )
    .bind(&user_info.name)
    .bind(&user_info.email)
    .bind(&user_info.id)
    .bind(&user_info.picture)
    .fetch_one(&state.db)
    .await?;

    let token = create_jwt(
        user.id,
        &user.email,
        &state.config.jwt_secret,
        state.config.jwt_expiration_hours,
    )?;

    Ok(Json(AuthResponse {
        token,
        user: user.into(),
    }))
}
