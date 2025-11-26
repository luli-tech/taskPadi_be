use crate::error::Result;
use crate::auth::auth_repository::RefreshTokenRepository;
use crate::auth::auth_models::RefreshToken;
use crate::auth::{create_access_token, create_refresh_token, verify_jwt, hash_password, verify_password};
use crate::user::user_repository::UserRepository;
use crate::user::user_models::User;
use uuid::Uuid;

#[derive(Clone)]
pub struct AuthService {
    user_repo: UserRepository,
    refresh_token_repo: RefreshTokenRepository,
    jwt_secret: String,
}

impl AuthService {
    pub fn new(
        user_repo: UserRepository,
        refresh_token_repo: RefreshTokenRepository,
        jwt_secret: String,
    ) -> Self {
        Self {
            user_repo,
            refresh_token_repo,
            jwt_secret,
        }
    }

    pub async fn register(
        &self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<(User, String, String)> {
        let password_hash = hash_password(password)?;
        let user = self.user_repo.create(username, email, &password_hash).await?;
        
        let access_token = create_access_token(user.id, &user.role, &self.jwt_secret)?;
        let refresh_token = create_refresh_token(user.id, &user.role, &self.jwt_secret)?;
        
        self.refresh_token_repo
            .create(user.id, &refresh_token)
            .await?;

        Ok((user, access_token, refresh_token))
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<(User, String, String)> {
        let user = self
            .user_repo
            .find_by_email(email)
            .await?
            .ok_or_else(|| crate::error::AppError::Unauthorized("Invalid credentials".into()))?;

        if let Some(ref password_hash) = user.password_hash {
            verify_password(password, password_hash)?;
        } else {
            return Err(crate::error::AppError::Unauthorized(
                "Invalid credentials".into(),
            ));
        }

        let access_token = create_access_token(user.id, &user.role, &self.jwt_secret)?;
        let refresh_token = create_refresh_token(user.id, &user.role, &self.jwt_secret)?;

        self.refresh_token_repo
            .create(user.id, &refresh_token)
            .await?;

        Ok((user, access_token, refresh_token))
    }

    pub async fn refresh_access_token(&self, refresh_token: &str) -> Result<(String, String)> {
        let claims = verify_jwt(refresh_token, &self.jwt_secret)?;
        
        let stored_token = self
            .refresh_token_repo
            .find_by_token(refresh_token)
            .await?
            .ok_or_else(|| crate::error::AppError::Unauthorized("Invalid refresh token".into()))?;

        if stored_token.revoked {
            return Err(crate::error::AppError::Unauthorized(
                "Token has been revoked".into(),
            ));
        }

        let user = self
            .user_repo
            .find_by_id(claims.sub)
            .await?
            .ok_or_else(|| crate::error::AppError::NotFound("User not found".into()))?;

        let new_access_token = create_access_token(user.id, &user.role, &self.jwt_secret)?;
        let new_refresh_token = create_refresh_token(user.id, &user.role, &self.jwt_secret)?;

        self.refresh_token_repo
            .revoke_by_token(refresh_token)
            .await?;
        self.refresh_token_repo
            .create(user.id, &new_refresh_token)
            .await?;

        Ok((new_access_token, new_refresh_token))
    }

    pub async fn logout(&self, refresh_token: &str) -> Result<()> {
        self.refresh_token_repo
            .revoke_by_token(refresh_token)
            .await
    }

    pub async fn google_login_or_register(
        &self,
        username: &str,
        email: &str,
        google_id: &str,
        avatar_url: &str,
    ) -> Result<(User, String, String)> {
        let user = self
            .user_repo
            .upsert_google_user(username, email, google_id, avatar_url)
            .await?;

        let access_token = create_access_token(user.id, &user.role, &self.jwt_secret)?;
        let refresh_token = create_refresh_token(user.id, &user.role, &self.jwt_secret)?;

        self.refresh_token_repo
            .create(user.id, &refresh_token)
            .await?;

        Ok((user, access_token, refresh_token))
    }
}
