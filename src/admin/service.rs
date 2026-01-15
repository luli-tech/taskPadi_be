use crate::error::Result;
use crate::admin::repository::AdminRepository;
use crate::task::task_repository::TaskFilters;
use crate::task::task_models::Task;
use crate::user::user_models::User;
use uuid::Uuid;

#[derive(Clone)]
pub struct AdminService {
    repository: AdminRepository,
}

impl AdminService {
    pub fn new(repository: AdminRepository) -> Self {
        Self { repository }
    }

    pub async fn list_tasks(&self, filters: TaskFilters) -> Result<(Vec<Task>, i64)> {
        self.repository.find_all_tasks(filters).await
    }

    pub async fn delete_task(&self, task_id: Uuid) -> Result<()> {
        let rows = self.repository.delete_task_admin(task_id).await?;
        if rows == 0 {
            return Err(crate::error::AppError::NotFound("Task not found".to_string()));
        }
        Ok(())
    }

    pub async fn list_users(&self, limit: i64, offset: i64) -> Result<Vec<User>> {
        self.repository.find_all_users(limit, offset).await
    }

    pub async fn count_users(&self) -> Result<i64> {
        self.repository.count_all_users().await
    }

    pub async fn update_user_admin_status(&self, user_id: Uuid, is_admin: bool) -> Result<User> {
        self.repository.update_admin_status(user_id, is_admin).await
    }

    pub async fn update_user_active_status(&self, user_id: Uuid, is_active: bool) -> Result<User> {
        self.repository.update_active_status(user_id, is_active).await
    }

    pub async fn delete_user(&self, user_id: Uuid) -> Result<()> {
        // Here you could add logic to delete user's tasks first if needed, 
        // though many DBs use ON DELETE CASCADE.
        self.repository.delete_user(user_id).await
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
        self.repository.admin_update_user(
            user_id,
            username,
            email,
            bio,
            theme,
            avatar_url,
            is_admin,
            is_active
        ).await
    }
}
