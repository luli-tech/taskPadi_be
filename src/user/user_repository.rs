use crate::error::Result;
use sqlx::PgPool;
use uuid::Uuid;
use super::user_models::User;

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

    pub async fn create_with_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3) RETURNING *"
        )
        .bind(username)
        .bind(email)
        .bind(password_hash)
        .fetch_one(&mut **tx)
        .await?;

        Ok(user)
    }

    pub async fn create_admin_with_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<User> {
        // Create admin user with is_admin=true, role='admin', and is_active=true
        // This ensures immediate access to all admin endpoints without approval
        let user = sqlx::query_as::<_, User>(
            "INSERT INTO users (username, email, password_hash, role, is_admin, is_active) VALUES ($1, $2, $3, 'admin', true, true) RETURNING *"
        )
        .bind(username)
        .bind(email)
        .bind(password_hash)
        .fetch_one(&mut **tx)
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



    pub async fn upsert_google_user_with_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
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
        .fetch_one(&mut **tx)
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

    pub async fn find_by_id(&self, user_id: Uuid) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }

    pub async fn list_all_users(
        &self,
        exclude_user_id: Uuid,
        limit: i64,
        offset: i64,
        search: Option<&str>,
    ) -> Result<(Vec<User>, i64)> {
        let search_term = search.map(|s| format!("%{}%", s));
        
        let users = if let Some(ref search_str) = search_term {
            sqlx::query_as::<_, User>(
                "SELECT * FROM users 
                 WHERE id != $1 
                 AND is_active = true 
                 AND (username ILIKE $4 OR email ILIKE $4)
                 ORDER BY username ASC 
                 LIMIT $2 OFFSET $3"
            )
            .bind(exclude_user_id)
            .bind(limit)
            .bind(offset)
            .bind(search_str)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, User>(
                "SELECT * FROM users 
                 WHERE id != $1 
                 AND is_active = true 
                 ORDER BY username ASC 
                 LIMIT $2 OFFSET $3"
            )
            .bind(exclude_user_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?
        };

        let total = if let Some(ref search_str) = search_term {
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM users 
                 WHERE id != $1 
                 AND is_active = true 
                 AND (username ILIKE $2 OR email ILIKE $2)"
            )
            .bind(exclude_user_id)
            .bind(search_str)
            .fetch_one(&self.pool)
            .await?
        } else {
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM users 
                 WHERE id != $1 
                 AND is_active = true"
            )
            .bind(exclude_user_id)
            .fetch_one(&self.pool)
            .await?
        };

        Ok((users, total))
    }

    pub async fn update_profile(
        &self,
        user_id: Uuid,
        username: Option<String>,
        bio: Option<String>,
        theme: Option<String>,
        avatar_url: Option<String>,
    ) -> Result<User> {
        let mut query = String::from("UPDATE users SET updated_at = NOW()");
        let mut param_count = 1;
        let mut bindings: Vec<String> = vec![];

        if username.is_some() {
            param_count += 1;
            query.push_str(&format!(", username = ${}", param_count));
            bindings.push("username".to_string());
        }
        if bio.is_some() {
            param_count += 1;
            query.push_str(&format!(", bio = ${}", param_count));
            bindings.push("bio".to_string());
        }
        if theme.is_some() {
            param_count += 1;
            query.push_str(&format!(", theme = ${}", param_count));
            bindings.push("theme".to_string());
        }
        if avatar_url.is_some() {
            param_count += 1;
            query.push_str(&format!(", avatar_url = ${}", param_count));
            bindings.push("avatar_url".to_string());
        }

        query.push_str(&format!(" WHERE id = $1 RETURNING *"));

        let mut q = sqlx::query_as::<_, User>(&query).bind(user_id);

        for binding in bindings {
            match binding.as_str() {
                "username" => q = q.bind(username.clone().unwrap()),
                "bio" => q = q.bind(bio.clone()),
                "theme" => q = q.bind(theme.clone().unwrap()),
                "avatar_url" => q = q.bind(avatar_url.clone()),
                _ => {}
            }
        }

        let user = q.fetch_one(&self.pool).await?;

        Ok(user)
    }

}
