use crate::error::{AppError, Result};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub email: String,
    pub role: String,
    pub exp: i64,
}

/// Create access token (short-lived, 15 minutes)
pub fn create_access_token(user_id: Uuid, email: &str, role: &str, secret: &str) -> Result<String> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::minutes(15))
        .ok_or(AppError::InternalError)?
        .timestamp();

    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        role: role.to_string(),
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| AppError::Authentication("Failed to create access token".to_string()))
}

/// Create refresh token (long-lived, 7 days)
pub fn create_refresh_token(user_id: Uuid, email: &str, role: &str, secret: &str) -> Result<String> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::days(7))
        .ok_or(AppError::InternalError)?
        .timestamp();

    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        role: role.to_string(),
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| AppError::Authentication("Failed to create refresh token".to_string()))
}

/// Verify JWT token and extract claims
pub fn verify_jwt(token: &str, secret: &str) -> Result<Claims> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
  .map(|data| data.claims)
    .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))
}

/// Legacy function for backward compatibility
pub fn create_jwt(user_id: Uuid, email: &str, secret: &str, expiration_hours: i64) -> Result<String> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(expiration_hours))
        .ok_or(AppError::InternalError)?
        .timestamp();

    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        role: "user".to_string(),
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| AppError::Authentication("Failed to create token".to_string()))
}
