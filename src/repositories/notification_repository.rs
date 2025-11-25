use crate::{models::Notification, error::Result};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct NotificationRepository {
    pool: PgPool,
}

impl NotificationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_all_by_user(&self, user_id: Uuid) -> Result<Vec<Notification>> {
        let notifications = sqlx::query_as::<_, Notification>(
            "SELECT * FROM notifications WHERE user_id = $1 ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(notifications)
    }

    pub async fn mark_as_read(&self, id: Uuid, user_id: Uuid) -> Result<Option<Notification>> {
        let notification = sqlx::query_as::<_, Notification>(
            "UPDATE notifications SET is_read = true WHERE id = $1 AND user_id = $2 RETURNING *"
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(notification)
    }

    pub async fn delete(&self, id: Uuid, user_id: Uuid) -> Result<u64> {
        let result = sqlx::query("DELETE FROM notifications WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }

    pub async fn create(
        &self,
        user_id: Uuid,
        task_id: Option<Uuid>,
        message: &str,
    ) -> Result<Notification> {
        let notification = sqlx::query_as::<_, Notification>(
            "INSERT INTO notifications (user_id, task_id, message)
             VALUES ($1, $2, $3)
             RETURNING *"
        )
        .bind(user_id)
        .bind(task_id)
        .bind(message)
        .fetch_one(&self.pool)
        .await?;

        Ok(notification)
    }
}
