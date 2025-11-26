use crate::{
    error::Result,
    task::task_repository::TaskRepository,
    user::{
        user_dto::{UpdateProfileRequest, UserStatsResponse},
        user_models::{User, UserResponse},
        user_repository::UserRepository,
    },
};
use uuid::Uuid;

#[derive(Clone)]
pub struct UserService {
    user_repository: UserRepository,
    task_repository: TaskRepository,
}

impl UserService {
    pub fn new(user_repository: UserRepository, task_repository: TaskRepository) -> Self {
        Self {
            user_repository,
            task_repository,
        }
    }

    pub async fn get_current_user(&self, user_id: Uuid) -> Result<UserResponse> {
        let user = self
            .user_repository
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| crate::error::AppError::NotFound("User not found".to_string()))?;

        Ok(user.into())
    }

    pub async fn update_current_user(
        &self,
        user_id: Uuid,
        payload: UpdateProfileRequest,
    ) -> Result<UserResponse> {
        let user = self
            .user_repository
            .update_profile(user_id, payload.bio, payload.theme, payload.avatar_url)
            .await?
            .ok_or_else(|| crate::error::AppError::NotFound("User not found".to_string()))?;

        Ok(user.into())
    }

    pub async fn get_user_stats(&self, user_id: Uuid) -> Result<UserStatsResponse> {
        let (
            total_tasks,
            pending_tasks,
            in_progress_tasks,
            completed_tasks,
            archived_tasks,
            low_priority_tasks,
            medium_priority_tasks,
            high_priority_tasks,
            urgent_priority_tasks,
        ) = self.task_repository.get_user_stats(user_id).await?;

        let completion_rate = if total_tasks > 0 {
            (completed_tasks as f64 / total_tasks as f64) * 100.0
        } else {
            0.0
        };

        Ok(UserStatsResponse {
            total_tasks,
            pending_tasks,
            in_progress_tasks,
            completed_tasks,
            archived_tasks,
            completion_rate,
            low_priority_tasks,
            medium_priority_tasks,
            high_priority_tasks,
            urgent_priority_tasks,
        })
    }
}
