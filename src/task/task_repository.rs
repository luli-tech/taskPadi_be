use crate::error::Result;
use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use super::task_models::Task;

#[derive(Clone)]
pub struct TaskRepository {
    pool: PgPool,
}

pub struct TaskFilters {
    pub status: Option<String>,
    pub statuses: Option<Vec<String>>,
    pub priority: Option<String>,
    pub priorities: Option<Vec<String>>,
    pub search: Option<String>,
    pub created_from: Option<DateTime<Utc>>,
    pub created_to: Option<DateTime<Utc>>,
    pub due_from: Option<DateTime<Utc>>,
    pub due_to: Option<DateTime<Utc>>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub user_id: Option<Uuid>,
}

impl TaskRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_all(&self, user_id: Uuid, filters: TaskFilters) -> Result<(Vec<Task>, i64)> {
        let mut query = "SELECT * FROM tasks WHERE user_id = $1".to_string();
        let mut count_query = "SELECT COUNT(*) FROM tasks WHERE user_id = $1".to_string();
        let mut params_count: usize = 1;

        // Status filters
        if let Some(ref statuses) = filters.statuses {
            if !statuses.is_empty() {
                let place_holders: Vec<String> = statuses.iter().enumerate().map(|(i, _)| format!("${}", params_count + i + 1)).collect();
                let filter = format!(" AND status IN ({})", place_holders.join(", "));
                query.push_str(&filter);
                count_query.push_str(&filter);
                params_count += statuses.len();
            }
        } else if let Some(ref _status) = filters.status {
            params_count += 1;
            let filter = format!(" AND status = ${}", params_count);
            query.push_str(&filter);
            count_query.push_str(&filter);
        }

        // Priority filters
        if let Some(ref priorities) = filters.priorities {
            if !priorities.is_empty() {
                let place_holders: Vec<String> = priorities.iter().enumerate().map(|(i, _)| format!("${}", params_count + i + 1)).collect();
                let filter = format!(" AND priority IN ({})", place_holders.join(", "));
                query.push_str(&filter);
                count_query.push_str(&filter);
                params_count += priorities.len();
            }
        } else if let Some(ref _priority) = filters.priority {
            params_count += 1;
            let filter = format!(" AND priority = ${}", params_count);
            query.push_str(&filter);
            count_query.push_str(&filter);
        }

        // Search filter
        if let Some(ref _search) = filters.search {
            params_count += 1;
            let filter = format!(" AND (title ILIKE ${} OR description ILIKE ${})", params_count, params_count);
            query.push_str(&filter);
            count_query.push_str(&filter);
        }

        // Date range filters
        if let Some(ref _from) = filters.created_from {
            params_count += 1;
            let filter = format!(" AND created_at >= ${}", params_count);
            query.push_str(&filter);
            count_query.push_str(&filter);
        }
        if let Some(ref _to) = filters.created_to {
            params_count += 1;
            let filter = format!(" AND created_at <= ${}", params_count);
            query.push_str(&filter);
            count_query.push_str(&filter);
        }
        if let Some(ref _from) = filters.due_from {
            params_count += 1;
            let filter = format!(" AND due_date >= ${}", params_count);
            query.push_str(&filter);
            count_query.push_str(&filter);
        }
        if let Some(ref _to) = filters.due_to {
            params_count += 1;
            let filter = format!(" AND due_date <= ${}", params_count);
            query.push_str(&filter);
            count_query.push_str(&filter);
        }

        // Create count query
        let mut count_db_query = sqlx::query_scalar::<_, i64>(&count_query).bind(user_id);
        
        // Bind parameters for count query
        if let Some(statuses) = &filters.statuses {
            for status in statuses { count_db_query = count_db_query.bind(status); }
        } else if let Some(status) = &filters.status {
            count_db_query = count_db_query.bind(status);
        }

        if let Some(priorities) = &filters.priorities {
            for priority in priorities { count_db_query = count_db_query.bind(priority); }
        } else if let Some(priority) = &filters.priority {
            count_db_query = count_db_query.bind(priority);
        }

        if let Some(search) = &filters.search {
            count_db_query = count_db_query.bind(format!("%{}%", search));
        }

        if let Some(from) = filters.created_from { count_db_query = count_db_query.bind(from); }
        if let Some(to) = filters.created_to { count_db_query = count_db_query.bind(to); }
        if let Some(from) = filters.due_from { count_db_query = count_db_query.bind(from); }
        if let Some(to) = filters.due_to { count_db_query = count_db_query.bind(to); }

        let total_count = count_db_query.fetch_one(&self.pool).await?;

        // Sorting
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

        // Pagination
        let page = filters.page.unwrap_or(1);
        let limit = filters.limit.unwrap_or(10);
        let offset = (page - 1) * limit;
        query.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));

        // Create main query
        let mut db_query = sqlx::query_as::<_, Task>(&query).bind(user_id);
        
        // Bind parameters for main query
        if let Some(statuses) = filters.statuses {
            for status in statuses { db_query = db_query.bind(status); }
        } else if let Some(status) = filters.status {
            db_query = db_query.bind(status);
        }

        if let Some(priorities) = filters.priorities {
            for priority in priorities { db_query = db_query.bind(priority); }
        } else if let Some(priority) = filters.priority {
            db_query = db_query.bind(priority);
        }

        if let Some(search) = filters.search {
            db_query = db_query.bind(format!("%{}%", search));
        }

        if let Some(from) = filters.created_from { db_query = db_query.bind(from); }
        if let Some(to) = filters.created_to { db_query = db_query.bind(to); }
        if let Some(from) = filters.due_from { db_query = db_query.bind(from); }
        if let Some(to) = filters.due_to { db_query = db_query.bind(to); }

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

    pub async fn get_user_stats(&self, user_id: Uuid) -> Result<(i64, i64, i64, i64, i64, i64, i64, i64, i64)> {
        let total_tasks: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tasks WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?;

        let pending_tasks: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM tasks WHERE user_id = $1 AND status = 'Pending'"
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        let in_progress_tasks: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM tasks WHERE user_id = $1 AND status = 'InProgress'"
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        let completed_tasks: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM tasks WHERE user_id = $1 AND status = 'Completed'"
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        let archived_tasks: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM tasks WHERE user_id = $1 AND status = 'Archived'"
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        let low_priority_tasks: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM tasks WHERE user_id = $1 AND priority = 'Low'"
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        let medium_priority_tasks: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM tasks WHERE user_id = $1 AND priority = 'Medium'"
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        let high_priority_tasks: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM tasks WHERE user_id = $1 AND priority = 'High'"
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        let urgent_priority_tasks: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM tasks WHERE user_id = $1 AND priority = 'Urgent'"
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok((
            total_tasks,
            pending_tasks,
            in_progress_tasks,
            completed_tasks,
            archived_tasks,
            low_priority_tasks,
            medium_priority_tasks,
            high_priority_tasks,
            urgent_priority_tasks,
        ))
    }

    // Collaborative task methods
    pub async fn add_task_member(
        &self,
        task_id: Uuid,
        user_id: Uuid,
        role: &str,
        added_by: Uuid,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO task_members (task_id, user_id, role, added_by)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (task_id, user_id) DO NOTHING"
        )
        .bind(task_id)
        .bind(user_id)
        .bind(role)
        .bind(added_by)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn remove_task_member(&self, task_id: Uuid, user_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM task_members WHERE task_id = $1 AND user_id = $2")
            .bind(task_id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_task_members(&self, task_id: Uuid) -> Result<Vec<super::task_models::TaskMemberInfo>> {
        let members = sqlx::query_as::<_, super::task_models::TaskMemberInfo>(
            "SELECT tm.user_id, u.username, u.avatar_url, tm.role, tm.added_at
             FROM task_members tm
             JOIN users u ON u.id = tm.user_id
             WHERE tm.task_id = $1
             ORDER BY tm.added_at ASC"
        )
        .bind(task_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(members)
    }

    pub async fn get_user_tasks_including_shared(&self, user_id: Uuid, filters: TaskFilters) -> Result<(Vec<Task>, i64)> {
        let mut query = "SELECT DISTINCT t.* FROM tasks t 
                         LEFT JOIN task_members tm ON t.id = tm.task_id
                         WHERE (t.user_id = $1 OR tm.user_id = $1)".to_string();
        
        let mut count_query = "SELECT COUNT(DISTINCT t.id) FROM tasks t
                               LEFT JOIN task_members tm ON t.id = tm.task_id
                               WHERE (t.user_id = $1 OR tm.user_id = $1)".to_string();
        
        let mut params_count: usize = 1;

        // Status filters
        if let Some(ref statuses) = filters.statuses {
            if !statuses.is_empty() {
                let place_holders: Vec<String> = statuses.iter().enumerate().map(|(i, _)| format!("${}", params_count + i + 1)).collect();
                let filter = format!(" AND t.status IN ({})", place_holders.join(", "));
                query.push_str(&filter);
                count_query.push_str(&filter);
                params_count += statuses.len();
            }
        } else if let Some(ref _status) = filters.status {
            params_count += 1;
            let filter = format!(" AND t.status = ${}", params_count);
            query.push_str(&filter);
            count_query.push_str(&filter);
        }

        // Priority filters
        if let Some(ref priorities) = filters.priorities {
            if !priorities.is_empty() {
                let place_holders: Vec<String> = priorities.iter().enumerate().map(|(i, _)| format!("${}", params_count + i + 1)).collect();
                let filter = format!(" AND t.priority IN ({})", place_holders.join(", "));
                query.push_str(&filter);
                count_query.push_str(&filter);
                params_count += priorities.len();
            }
        } else if let Some(ref _priority) = filters.priority {
            params_count += 1;
            let filter = format!(" AND t.priority = ${}", params_count);
            query.push_str(&filter);
            count_query.push_str(&filter);
        }

        // Search filter
        if let Some(ref _search) = filters.search {
            params_count += 1;
            let filter = format!(" AND (t.title ILIKE ${} OR t.description ILIKE ${})", params_count, params_count);
            query.push_str(&filter);
            count_query.push_str(&filter);
        }

        // Date range filters
        if let Some(ref _from) = filters.created_from {
            params_count += 1;
            let filter = format!(" AND t.created_at >= ${}", params_count);
            query.push_str(&filter);
            count_query.push_str(&filter);
        }
        if let Some(ref _to) = filters.created_to {
            params_count += 1;
            let filter = format!(" AND t.created_at <= ${}", params_count);
            query.push_str(&filter);
            count_query.push_str(&filter);
        }
        if let Some(ref _from) = filters.due_from {
            params_count += 1;
            let filter = format!(" AND t.due_date >= ${}", params_count);
            query.push_str(&filter);
            count_query.push_str(&filter);
        }
        if let Some(ref _to) = filters.due_to {
            params_count += 1;
            let filter = format!(" AND t.due_date <= ${}", params_count);
            query.push_str(&filter);
            count_query.push_str(&filter);
        }

        // Calculate total count
        let mut count_db_query = sqlx::query_scalar::<_, i64>(&count_query).bind(user_id);
        
        // Bind parameters for count query
        if let Some(statuses) = &filters.statuses {
            for status in statuses { count_db_query = count_db_query.bind(status); }
        } else if let Some(status) = &filters.status {
            count_db_query = count_db_query.bind(status);
        }

        if let Some(priorities) = &filters.priorities {
            for priority in priorities { count_db_query = count_db_query.bind(priority); }
        } else if let Some(priority) = &filters.priority {
            count_db_query = count_db_query.bind(priority);
        }

        if let Some(search) = &filters.search {
            count_db_query = count_db_query.bind(format!("%{}%", search));
        }

        if let Some(from) = filters.created_from { count_db_query = count_db_query.bind(from); }
        if let Some(to) = filters.created_to { count_db_query = count_db_query.bind(to); }
        if let Some(from) = filters.due_from { count_db_query = count_db_query.bind(from); }
        if let Some(to) = filters.due_to { count_db_query = count_db_query.bind(to); }

        let total_count = count_db_query.fetch_one(&self.pool).await?;

        // Sorting
        let sort_column = match filters.sort_by.as_deref() {
            Some("priority") => "t.priority",
            Some("due_date") => "t.due_date",
            Some("created_at") => "t.created_at",
            _ => "t.created_at",
        };
        let sort_direction = match filters.sort_order.as_deref() {
            Some("asc") => "ASC",
            _ => "DESC",
        };
        query.push_str(&format!(" ORDER BY {} {}", sort_column, sort_direction));

        // Pagination
        let page = filters.page.unwrap_or(1);
        let limit = filters.limit.unwrap_or(10);
        let offset = (page - 1) * limit;
        query.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));

        // Create main query
        let mut db_query = sqlx::query_as::<_, Task>(&query).bind(user_id);
        
        // Bind parameters for main query
        if let Some(statuses) = filters.statuses {
            for status in statuses { db_query = db_query.bind(status); }
        } else if let Some(status) = filters.status {
            db_query = db_query.bind(status);
        }

        if let Some(priorities) = filters.priorities {
            for priority in priorities { db_query = db_query.bind(priority); }
        } else if let Some(priority) = filters.priority {
            db_query = db_query.bind(priority);
        }

        if let Some(search) = filters.search {
            db_query = db_query.bind(format!("%{}%", search));
        }

        if let Some(from) = filters.created_from { db_query = db_query.bind(from); }
        if let Some(to) = filters.created_to { db_query = db_query.bind(to); }
        if let Some(from) = filters.due_from { db_query = db_query.bind(from); }
        if let Some(to) = filters.due_to { db_query = db_query.bind(to); }

        let tasks = db_query.fetch_all(&self.pool).await?;
        Ok((tasks, total_count))
    }

    pub async fn is_task_member(&self, task_id: Uuid, user_id: Uuid) -> Result<bool> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM task_members WHERE task_id = $1 AND user_id = $2"
        )
        .bind(task_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count > 0)
    }

    pub async fn is_task_owner(&self, task_id: Uuid, user_id: Uuid) -> Result<bool> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM tasks WHERE id = $1 AND user_id = $2"
        )
        .bind(task_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count > 0)
    }

    pub async fn has_task_access(&self, task_id: Uuid, user_id: Uuid) -> Result<bool> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM tasks t
             LEFT JOIN task_members tm ON t.id = tm.task_id
             WHERE t.id = $1 AND (t.user_id = $2 OR tm.user_id = $2)"
        )
        .bind(task_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count > 0)
    }

    pub async fn log_task_activity(
        &self,
        task_id: Uuid,
        user_id: Uuid,
        action: &str,
        details: Option<serde_json::Value>,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO task_activity (task_id, user_id, action, details)
             VALUES ($1, $2, $3, $4)"
        )
        .bind(task_id)
        .bind(user_id)
        .bind(action)
        .bind(details)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_task_activity(&self, task_id: Uuid) -> Result<Vec<super::task_dto::TaskActivityResponse>> {
        let activities = sqlx::query_as::<_, super::task_dto::TaskActivityResponse>(
            "SELECT ta.id, ta.user_id, u.username, ta.action, ta.details, ta.created_at
             FROM task_activity ta
             LEFT JOIN users u ON u.id = ta.user_id
             WHERE ta.task_id = $1
             ORDER BY ta.created_at DESC"
        )
        .bind(task_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(activities)
    }

    pub async fn find_by_id_with_access(&self, id: Uuid, user_id: Uuid) -> Result<Option<Task>> {
        let task = sqlx::query_as::<_, Task>(
            "SELECT DISTINCT t.* FROM tasks t
             LEFT JOIN task_members tm ON t.id = tm.task_id
             WHERE t.id = $1 AND (t.user_id = $2 OR tm.user_id = $2)"
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(task)
    }
}
