use crate::error::Result;
use crate::video_call::video_call_models::{VideoCall, CallParticipantResponse};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct VideoCallRepository {
    pool: PgPool,
}

impl VideoCallRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        caller_id: Uuid,
        receiver_id: Option<Uuid>,
        group_id: Option<Uuid>,
        call_type: &str,
        status: &str,
    ) -> Result<VideoCall> {
        let call = sqlx::query_as::<_, VideoCall>(
            "INSERT INTO video_calls (caller_id, receiver_id, group_id, call_type, status)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING *",
        )
        .bind(caller_id)
        .bind(receiver_id)
        .bind(group_id)
        .bind(call_type)
        .bind(status)
        .fetch_one(&self.pool)
        .await?;

        // Add caller as first participant
        self.add_participant(call.id, caller_id, "caller").await?;

        Ok(call)
    }

    pub async fn add_participant(&self, call_id: Uuid, user_id: Uuid, role: &str) -> Result<()> {
        sqlx::query(
            "INSERT INTO call_participants (call_id, user_id, role, status)
             VALUES ($1, $2, $3, 'joined')
             ON CONFLICT (call_id, user_id) DO UPDATE SET status = 'joined', left_at = NULL",
        )
        .bind(call_id)
        .bind(user_id)
        .bind(role)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_participants(&self, call_id: Uuid) -> Result<Vec<CallParticipantResponse>> {
        let participants = sqlx::query_as::<_, CallParticipantResponse>(
            r#"
            SELECT 
                p.user_id,
                u.username,
                u.avatar_url,
                p.role,
                p.status,
                p.joined_at
            FROM call_participants p
            JOIN users u ON p.user_id = u.id
            WHERE p.call_id = $1 AND p.status = 'joined'
            "#,
        )
        .bind(call_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(participants)
    }

    pub async fn update_participant_status(&self, call_id: Uuid, user_id: Uuid, status: &str) -> Result<()> {
        let left_at = if status == "left" { Some(chrono::Utc::now()) } else { None };
        sqlx::query(
            "UPDATE call_participants SET status = $3, left_at = $4 WHERE call_id = $1 AND user_id = $2",
        )
        .bind(call_id)
        .bind(user_id)
        .bind(status)
        .bind(left_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn find_by_id(&self, call_id: Uuid) -> Result<Option<VideoCall>> {
        let call = sqlx::query_as::<_, VideoCall>(
            "SELECT * FROM video_calls WHERE id = $1",
        )
        .bind(call_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(call)
    }

    pub async fn find_active_call(
        &self,
        user_id: Uuid,
        other_user_id: Uuid,
    ) -> Result<Option<VideoCall>> {
        let call = sqlx::query_as::<_, VideoCall>(
            "SELECT * FROM video_calls
             WHERE ((caller_id = $1 AND receiver_id = $2) OR (caller_id = $2 AND receiver_id = $1))
             AND status IN ('initiating', 'ringing', 'active')
             ORDER BY created_at DESC
             LIMIT 1",
        )
        .bind(user_id)
        .bind(other_user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(call)
    }

    pub async fn update_status(
        &self,
        call_id: Uuid,
        status: &str,
    ) -> Result<VideoCall> {
        let call = sqlx::query_as::<_, VideoCall>(
            "UPDATE video_calls
             SET status = $1
             WHERE id = $2
             RETURNING *",
        )
        .bind(status)
        .bind(call_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(call)
    }

    pub async fn start_call(&self, call_id: Uuid) -> Result<VideoCall> {
        let call = sqlx::query_as::<_, VideoCall>(
            "UPDATE video_calls
             SET status = 'active', started_at = NOW()
             WHERE id = $1
             RETURNING *",
        )
        .bind(call_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(call)
    }

    pub async fn end_call(&self, call_id: Uuid, duration_seconds: Option<i32>) -> Result<VideoCall> {
        let call = sqlx::query_as::<_, VideoCall>(
            "UPDATE video_calls
             SET status = 'ended', ended_at = NOW(), duration_seconds = $1
             WHERE id = $2
             RETURNING *",
        )
        .bind(duration_seconds)
        .bind(call_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(call)
    }

    pub async fn find_user_call_history(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<VideoCall>, i64)> {
        // Get total count
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM video_calls
             WHERE caller_id = $1 OR receiver_id = $1",
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        // Get calls
        let calls = sqlx::query_as::<_, VideoCall>(
            "SELECT * FROM video_calls
             WHERE caller_id = $1 OR receiver_id = $1
             ORDER BY created_at DESC
             LIMIT $2 OFFSET $3",
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok((calls, total))
    }

    pub async fn get_user_active_calls(&self, user_id: Uuid) -> Result<Vec<VideoCall>> {
        let calls = sqlx::query_as::<_, VideoCall>(
            "SELECT * FROM video_calls
             WHERE (caller_id = $1 OR receiver_id = $1)
             AND status IN ('initiating', 'ringing', 'active')
             ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(calls)
    }
}
