use crate::error::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;
use super::auth_models::RefreshToken;

#[derive(Clone)]
pub struct RefreshTokenRepository {
    pool: PgPool,
}

impl RefreshTokenRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        user_id: Uuid,
        token: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<RefreshToken> {
        let refresh_token = sqlx::query_as::<_, RefreshToken>(
            "INSERT INTO refresh_tokens (user_id, token, expires_at)
             VALUES ($1, $2, $3)
             RETURNING *",
        )
        .bind(user_id)
        .bind(token)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(refresh_token)
    }

    pub async fn find_by_token(&self, token: &str) -> Result<Option<RefreshToken>> {
        let refresh_token = sqlx::query_as::<_, RefreshToken>(
            "SELECT * FROM refresh_tokens
             WHERE token = $1 AND expires_at > NOW()",
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await?;

        Ok(refresh_token)
    }

    pub async fn delete_by_token(&self, token: &str) -> Result<()> {
        sqlx::query("DELETE FROM refresh_tokens WHERE token = $1")
            .bind(token)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn delete_by_user(&self, user_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM refresh_tokens WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn delete_expired(&self) -> Result<u64> {
        let result = sqlx::query("DELETE FROM refresh_tokens WHERE expires_at <= NOW()")
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }
}
