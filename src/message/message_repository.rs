use crate::{
    error::Result,
    message::{message_dto::ConversationUser, message_models::Message},
};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct MessageRepository {
    pool: PgPool,
}

impl MessageRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        sender_id: Uuid,
        receiver_id: Option<Uuid>,
        group_id: Option<Uuid>,
        content: &str,
        image_url: Option<&str>,
    ) -> Result<Message> {
        let message = sqlx::query_as::<_, Message>(
            "INSERT INTO messages (sender_id, receiver_id, group_id, content, image_url)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING *",
        )
        .bind(sender_id)
        .bind(receiver_id)
        .bind(group_id)
        .bind(content)
        .bind(image_url)
        .fetch_one(&self.pool)
        .await?;

        Ok(message)
    }

    pub async fn find_conversation(
        &self,
        user_id: Uuid,
        other_user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Message>> {
        let messages = sqlx::query_as::<_, Message>(
            "SELECT * FROM messages
             WHERE ((sender_id = $1 AND receiver_id = $2)
                OR (sender_id = $2 AND receiver_id = $1))
             AND group_id IS NULL
             ORDER BY created_at DESC
             LIMIT $3 OFFSET $4",
        )
        .bind(user_id)
        .bind(other_user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(messages)
    }

    pub async fn find_group_messages(
        &self,
        group_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Message>> {
        let messages = sqlx::query_as::<_, Message>(
            "SELECT * FROM messages
             WHERE group_id = $1
             ORDER BY created_at DESC
             LIMIT $2 OFFSET $3",
        )
        .bind(group_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(messages)
    }

    pub async fn count_group_messages(&self, group_id: Uuid) -> Result<i64> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM messages WHERE group_id = $1",
        )
        .bind(group_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    pub async fn count_conversation(&self, user_id: Uuid, other_user_id: Uuid) -> Result<i64> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM messages
             WHERE ((sender_id = $1 AND receiver_id = $2)
                OR (sender_id = $2 AND receiver_id = $1))
             AND group_id IS NULL",
        )
        .bind(user_id)
        .bind(other_user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    pub async fn find_user_conversations(&self, user_id: Uuid) -> Result<Vec<ConversationUser>> {
        let conversations = sqlx::query_as::<_, ConversationUser>(
            "WITH latest_messages AS (
                SELECT DISTINCT ON (
                    CASE
                        WHEN sender_id = $1 THEN receiver_id
                        ELSE sender_id
                    END
                )
                CASE
                    WHEN sender_id = $1 THEN receiver_id
                    ELSE sender_id
                END AS user_id,
                content AS last_message,
                created_at AS last_message_time
                FROM messages
                WHERE (sender_id = $1 OR receiver_id = $1) AND group_id IS NULL
                ORDER BY
                    CASE
                        WHEN sender_id = $1 THEN receiver_id
                        ELSE sender_id
                    END,
                    created_at DESC
            ),
            unread_counts AS (
                SELECT sender_id AS user_id, COUNT(*) AS unread_count
                FROM messages
                WHERE receiver_id = $1 AND is_read = false AND group_id IS NULL
                GROUP BY sender_id
            )
            SELECT
                lm.user_id,
                u.username,
                u.avatar_url,
                lm.last_message,
                lm.last_message_time,
                COALESCE(uc.unread_count, 0) AS unread_count
            FROM latest_messages lm
            JOIN users u ON u.id = lm.user_id
            LEFT JOIN unread_counts uc ON uc.user_id = lm.user_id
            ORDER BY lm.last_message_time DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(conversations)
    }

    pub async fn mark_as_read(&self, message_id: Uuid, user_id: Uuid) -> Result<()> {
        sqlx::query(
            "UPDATE messages
             SET is_read = true
             WHERE id = $1 AND receiver_id = $2 AND group_id IS NULL",
        )
        .bind(message_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn mark_conversation_as_read(
        &self,
        user_id: Uuid,
        other_user_id: Uuid,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE messages
             SET is_read = true
             WHERE receiver_id = $1 AND sender_id = $2 AND is_read = false AND group_id IS NULL",
        )
        .bind(user_id)
        .bind(other_user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn mark_group_messages_as_read(
        &self,
        user_id: Uuid,
        group_id: Uuid,
    ) -> Result<()> {
        // Mark messages in group as read for this user
        // Note: Group messages don't have individual receiver_id, so we track read status differently
        // For now, we'll mark messages where the user is not the sender
        sqlx::query(
            "UPDATE messages
             SET is_read = true
             WHERE group_id = $1 AND sender_id != $2 AND is_read = false",
        )
        .bind(group_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }


    #[allow(dead_code)]
    pub async fn count_unread(&self, user_id: Uuid) -> Result<i64> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM messages
             WHERE receiver_id = $1 AND is_read = false",
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }



    pub async fn update(
        &self,
        message_id: Uuid,
        sender_id: Uuid,
        content: &str,
        image_url: Option<&str>,
    ) -> Result<Message> {
        let message = sqlx::query_as::<_, Message>(
            "UPDATE messages 
             SET content = $1, image_url = $2, updated_at = NOW()
             WHERE id = $3 AND sender_id = $4
             RETURNING *"
        )
        .bind(content)
        .bind(image_url)
        .bind(message_id)
        .bind(sender_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(crate::error::AppError::NotFound("Message not found or you are not the sender".to_string()))?;

        Ok(message)
    }

    pub async fn delete(&self, message_id: Uuid, sender_id: Uuid) -> Result<()> {
        let result = sqlx::query(
            "DELETE FROM messages 
             WHERE id = $1 AND sender_id = $2"
        )
        .bind(message_id)
        .bind(sender_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(crate::error::AppError::NotFound("Message not found or you are not the sender".to_string()));
        }

        Ok(())
    }
}
