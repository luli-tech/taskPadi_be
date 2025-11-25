use crate::{models::Task, error::Result};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Clone)]
pub struct TaskRepository {
    pool: PgPool,
}

pub struct TaskFilters {
    pub status: Option<String>,
    pub priority: Option<String>,
    pub search: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

impl TaskRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_all(&self, user_id: Uuid, filters: TaskFilters) -> Result<(Vec<Task>, i64)> {
        let mut query = "SELECT * FROM tasks WHERE user_id = $1".to_string();
        let mut count_query = "SELECT COUNT(*) FROM tasks WHERE user_id = $1".to_string();
        let mut params_count = 1;

        if let Some(ref status) = filters.status {
            params_count += 1;
            let filter = format!(" AND status = ${}", params_count);
            query.push_str(&filter);
            count_query.push_str(&filter);
        }

        if let Some(ref priority) = filters.priority {
            params_count += 1;
            let filter = format!(" AND priority = ${}", params_count);
            query.push_str(&filter);
            count_query.push_str(&filter);
        }

        if let Some(ref search) = filters.search {
            params_count += 1;
            let filter = format!(" AND (title ILIKE ${} OR description ILIKE ${})", params_count, params_count);
            query.push_str(&filter);
            count_query.push_str(&filter);
        }

        // Calculate total count before pagination
        let mut count_db_query = sqlx::query_scalar::<_, i64>(&count_query).bind(user_id);

        if let Some(status) = &filters.status {
            count_db_query = count_db_query.bind(status);
        }
        if let Some(priority) = &filters.priority {
            count_db_query = count_db_query.bind(priority);
        }
        if let Some(search) = &filters.search {
            let search_pattern = format!("%{}%", search);
            count_db_query = count_db_query.bind(search_pattern);
        }

        let total_count = count_db_query.fetch_one(&self.pool).await?;

        // Add sorting
        let sort_column = match filters.sort_by.as_deref() {
            Some("priority") => "priority",
            Some("due_date") => "due_date",
            Some("created_at") => "created_at",
            _ => "created_at",
        };

        let sort_direction = match filters.sort_order.as_deref() {
            Some("asc") => "ASC",
            _ => "DESC",
        };

        query.push_str(&format!(" ORDER BY {} {}", sort_column, sort_direction));

        // Add pagination
        let page = filters.page.unwrap_or(1);
        let limit = filters.limit.unwrap_or(10);
        let offset = (page - 1) * limit;

        query.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));

        let mut db_query = sqlx::query_as::<_, Task>(&query).bind(user_id);

        if let Some(status) = filters.status {
            db_query = db_query.bind(status);
        }

        if let Some(priority) = filters.priority {
            db_query = db_query.bind(priority);
        }

        if let Some(search) = filters.search {
            let search_pattern = format!("%{}%", search);
            db_query = db_query.bind(search_pattern);
        }

        let tasks = db_query.fetch_all(&self.pool).await?;
        Ok((tasks, total_count))
    }

    pub async fn find_by_id(&self, id: Uuid, user_id: Uuid) -> Result<Option<Task>> {
        let task = sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(task)
    }

    pub async fn create(
        &self,
        user_id: Uuid,
        title: &str,
        description: Option<&str>,
        priority: &str,
        due_date: Option<DateTime<Utc>>,
        reminder_time: Option<DateTime<Utc>>,
    ) -> Result<Task> {
        let task = sqlx::query_as::<_, Task>(
            "INSERT INTO tasks (user_id, title, description, priority, due_date, reminder_time)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING *"
        )
        .bind(user_id)
        .bind(title)
        .bind(description)
        .bind(priority)
        .bind(due_date)
        .bind(reminder_time)
        .fetch_one(&self.pool)
        .await?;

        Ok(task)
    }

    pub async fn update(
        &self,
        id: Uuid,
        user_id: Uuid,
        title: Option<&str>,
        description: Option<&str>,
        status: Option<&str>,
        priority: Option<&str>,
        due_date: Option<DateTime<Utc>>,
        reminder_time: Option<DateTime<Utc>>,
    ) -> Result<Task> {
        let task = sqlx::query_as::<_, Task>(
            "UPDATE tasks SET
                title = COALESCE($1, title),
                description = COALESCE($2, description),
                status = COALESCE($3, status),
                priority = COALESCE($4, priority),
                due_date = COALESCE($5, due_date),
                reminder_time = COALESCE($6, reminder_time),
                notified = CASE WHEN $6 IS NOT NULL THEN false ELSE notified END,
                updated_at = NOW()
             WHERE id = $7 AND user_id = $8
             RETURNING *"
        )
        .bind(title)
        .bind(description)
        .bind(status)
        .bind(priority)
        .bind(due_date)
        .bind(reminder_time)
        .bind(id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(task)
    }

    pub async fn delete(&self, id: Uuid, user_id: Uuid) -> Result<u64> {
        let result = sqlx::query("DELETE FROM tasks WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        
        Ok(result.rows_affected())
    }

    pub async fn update_status(&self, id: Uuid, user_id: Uuid, status: &str) -> Result<Option<Task>> {
        let task = sqlx::query_as::<_, Task>(
            "UPDATE tasks SET status = $1, updated_at = NOW()
             WHERE id = $2 AND user_id = $3
             RETURNING *"
        )
        .bind(status)
        .bind(id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(task)
    }


    pub async fn find_due_reminders(&self) -> Result<Vec<Task>> {
        let now = Utc::now();
        let tasks = sqlx::query_as::<_, Task>(
            "SELECT * FROM tasks 
             WHERE reminder_time <= $1 
             AND notified = false 
             AND reminder_time IS NOT NULL"
        )
        .bind(now)
        .fetch_all(&self.pool)
        .await?;

        Ok(tasks)
    }

    pub async fn mark_as_notified(&self, id: Uuid) -> Result<()> {
        sqlx::query("UPDATE tasks SET notified = true WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
