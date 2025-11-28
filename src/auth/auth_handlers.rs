use uuid::Uuid;
use crate::{
    auth::{
        create_access_token, create_refresh_token, hash_password, verify_password, verify_jwt,
        oauth::GoogleUserInfo,
    },
    error::{AppError, Result},
    state::AppState,
};
use super::auth_dto::{AuthResponse, LoginRequest, RegisterRequest, RefreshTokenRequest, RefreshTokenResponse};
use axum::{extract::{State, Query}, http::StatusCode, response::{IntoResponse, Redirect}, Json};
use oauth2::{CsrfToken, PkceCodeChallenge, Scope, AuthorizationCode, TokenResponse};
use serde::Deserialize;
use validator::Validate;
// use chrono::Utc;

#[derive(Deserialize)]
pub struct GoogleCallback {
    code: String,
    #[allow(dead_code)]
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

           // Hash password using imported function
    let password_hash = hash_password(&payload.password)?;

    let (user, access_token, refresh_token) = state.auth_service
        .register(&payload.username, &payload.email, &password_hash)
        .await

        .map_err(|e| {
            if let AppError::Database(ref db_err) = e {
                if db_err.to_string().contains("duplicate key") {
                    return AppError::BadRequest("User already exists".to_string());
                }
            }
            e
        })?;

    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
          access_token: create_access_token(user.id, &user.email, &user.role, &state.config.jwt_secret)?,
    refresh_token: create_refresh_token(user.id, &user.email, &user.role, &state.config.jwt_secret)?,
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

    // let (user, access_token, refresh_token) = state.auth_service
    //     .login(&payload.email, &payload.password)
    //     .await?;
    let user = state.auth_service.find_by_email(&payload.email).await?
      .ok_or(AppError::Unauthorized("Invalid credentials".to_string()))?;

    // Verify password using imported function
    // verify_password(&payload.password, &user.password_hash)?;
    let password_hash = user.password_hash
    .as_ref()
    .ok_or(AppError::Unauthorized("No password set".to_string()))?;

      verify_password(&payload.password, password_hash)?;

    Ok(Json(AuthResponse {
    access_token: create_access_token(user.id, &user.email, &user.role, &state.config.jwt_secret)?,
    refresh_token: create_refresh_token(user.id, &user.email, &user.role, &state.config.jwt_secret)?,
        user: user.into(),
    }))
}

/// Refresh access token
#[utoipa::path(
    post,
    path = "/api/auth/refresh",
    request_body = RefreshTokenRequest,
    responses(
        (status = 200, description = "Token refreshed successfully", body = RefreshTokenResponse),
        (status = 401, description = "Invalid or expired refresh token")
    ),
    tag = "auth"
)]
pub async fn refresh_token(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<impl IntoResponse> {
    // let (access_token, _refresh_token) = state.auth_service
    //     .refresh_access_token(&payload.refresh_token)
    //     .await?;

    // Ok(Json(RefreshTokenResponse {
    //     access_token:create_access_token(&user)?,
    // }))
let claims = verify_jwt(&payload.refresh_token, &state.config.jwt_secret)?;
let user_id = Uuid::parse_str(&claims.sub)
    .map_err(|_| AppError::Unauthorized("Invalid refresh token".to_string()))?;

    let user = state.auth_service.find_by_id(user_id)
        .await?
        .ok_or(AppError::Unauthorized("Invalid refresh token".to_string()))?;


    Ok(Json(RefreshTokenResponse {
        access_token: create_access_token(user.id, &user.email, &user.role, &state.config.jwt_secret)?,

    }))
}

/// Logout (revoke refresh token)
#[utoipa::path(
    post,
    path = "/api/auth/logout",
    request_body = RefreshTokenRequest,
    responses(
        (status = 200, description = "Logged out successfully"),
        (status = 400, description = "Invalid input")
    ),
    tag = "auth"
)]
pub async fn logout(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<impl IntoResponse> {
    state.auth_service.logout(&payload.refresh_token).await?;
    Ok(StatusCode::OK)
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

    let access_token_google = token_result.access_token().secret();

    let client = reqwest::Client::new();
    let user_info: GoogleUserInfo = client
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .bearer_auth(access_token_google)
        .send()
        .await
        .map_err(|_| AppError::Authentication("Failed to get user info".to_string()))?
        .json()
        .await
        .map_err(|_| AppError::Authentication("Failed to parse user info".to_string()))?;

    let (user, access_token, refresh_token) = state.auth_service
        .google_login_or_register(
            &user_info.name,
            &user_info.email,
            &user_info.id,
            user_info.picture.as_deref().unwrap_or(""),
        )
        .await?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        user: user.into(),
    }))
}
