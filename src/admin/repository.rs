use crate::error::Result;
use sqlx::PgPool;
use uuid::Uuid;
use crate::user::user_models::User;
use crate::task::task_models::Task;
use crate::task::task_repository::TaskFilters;

#[derive(Clone)]
pub struct AdminRepository {
    pool: PgPool,
}

impl AdminRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // User management
    pub async fn find_all_users(&self, limit: i64, offset: i64) -> Result<Vec<User>> {
        let users = sqlx::query_as::<_, User>(
            "SELECT * FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(users)
    }

    pub async fn count_all_users(&self) -> Result<i64> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool)
            .await?;

        Ok(count)
    }

    pub async fn update_admin_status(&self, user_id: Uuid, is_admin: bool) -> Result<User> {
        let role = if is_admin { "admin" } else { "user" };
        let user = sqlx::query_as::<_, User>(
            "UPDATE users SET is_admin = $1, role = $2, updated_at = NOW() WHERE id = $3 RETURNING *"
        )
        .bind(is_admin)
        .bind(role)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn update_active_status(&self, user_id: Uuid, is_active: bool) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            "UPDATE users SET is_active = $1, updated_at = NOW() WHERE id = $2 RETURNING *"
        )
        .bind(is_active)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn delete_user(&self, user_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn admin_update_user(
        &self,
        user_id: Uuid,
        username: Option<String>,
        email: Option<String>,
        bio: Option<String>,
        theme: Option<String>,
        avatar_url: Option<String>,
        is_admin: Option<bool>,
        is_active: Option<bool>,
    ) -> Result<User> {
        let mut query = String::from("UPDATE users SET updated_at = NOW()");
        let mut param_count = 1;
        let mut bindings: Vec<String> = vec![];

        if username.is_some() {
            param_count += 1;
            query.push_str(&format!(", username = ${}", param_count));
            bindings.push("username".to_string());
        }
        if email.is_some() {
            param_count += 1;
            query.push_str(&format!(", email = ${}", param_count));
            bindings.push("email".to_string());
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
        if let Some(_is_adm) = is_admin {
            param_count += 1;
            query.push_str(&format!(", is_admin = ${}", param_count));
            bindings.push("is_admin".to_string());
            
            param_count += 1;
            query.push_str(&format!(", role = ${}", param_count));
            bindings.push("role".to_string());
        }
        if is_active.is_some() {
            param_count += 1;
            query.push_str(&format!(", is_active = ${}", param_count));
            bindings.push("is_active".to_string());
        }

        query.push_str(&format!(" WHERE id = $1 RETURNING *"));

        let mut q = sqlx::query_as::<_, User>(&query).bind(user_id);

        for binding in bindings {
            match binding.as_str() {
                "username" => q = q.bind(username.clone().unwrap()),
                "email" => q = q.bind(email.clone().unwrap()),
                "bio" => q = q.bind(bio.clone()),
                "theme" => q = q.bind(theme.clone().unwrap()),
                "avatar_url" => q = q.bind(avatar_url.clone()),
                "is_admin" => q = q.bind(is_admin.unwrap()),
                "role" => q = q.bind(if is_admin.unwrap() { "admin" } else { "user" }),
                "is_active" => q = q.bind(is_active.unwrap()),
                _ => {}
            }
        }

        let user = q.fetch_one(&self.pool).await?;

        Ok(user)
    }

    // Task management
    pub async fn find_all_tasks(&self, filters: TaskFilters) -> Result<(Vec<Task>, i64)> {
        let mut query = "SELECT t.* FROM tasks t WHERE 1=1".to_string();
        let mut count_query = "SELECT COUNT(*) FROM tasks t WHERE 1=1".to_string();
        let mut params_count: usize = 0;

        if let Some(ref user_id) = filters.user_id {
            params_count += 1;
            let filter = format!(" AND t.user_id = ${}", params_count);
            query.push_str(&filter);
            count_query.push_str(&filter);
        }

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

        // Calculate total count before pagination
        let mut count_db_query = sqlx::query_scalar::<_, i64>(&count_query);

        if let Some(user_id) = filters.user_id {
            count_db_query = count_db_query.bind(user_id);
        }
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

        // Add sorting
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

        // Add pagination
        let page = filters.page.unwrap_or(1);
        let limit = filters.limit.unwrap_or(10);
        let offset = (page - 1) * limit;
        query.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));

        let mut db_query = sqlx::query_as::<_, Task>(&query);

        if let Some(user_id) = filters.user_id {
            db_query = db_query.bind(user_id);
        }
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

    pub async fn delete_task_admin(&self, id: Uuid) -> Result<u64> {
        let result = sqlx::query("DELETE FROM tasks WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        
        Ok(result.rows_affected())
    }
}
