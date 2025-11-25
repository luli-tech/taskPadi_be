use crate::{models::User, error::Result};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, username: &str, email: &str, password_hash: &str) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3) RETURNING *"
        )
        .bind(username)
        .bind(email)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }

    pub async fn upsert_google_user(
        &self,
        username: &str,
        email: &str,
        google_id: &str,
        avatar_url: &str,
    ) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            "INSERT INTO users (username, email, google_id, avatar_url)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (google_id) DO UPDATE SET
                avatar_url = EXCLUDED.avatar_url,
                updated_at = NOW()
             RETURNING *"
        )
        .bind(username)
        .bind(email)
        .bind(google_id)
        .bind(avatar_url)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn update_notification_preferences(&self, user_id: Uuid, enabled: bool) -> Result<()> {
        sqlx::query("UPDATE users SET notification_enabled = $1 WHERE id = $2")
            .bind(enabled)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
