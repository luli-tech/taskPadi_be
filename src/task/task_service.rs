// src/task/task.service.rs
use crate::error::Result;
use crate::task::task_repository::TaskRepository;
use crate::task::task_models::Task;
use crate::task::task_dto::{CreateTaskRequest, UpdateTaskRequest, UpdateTaskStatusRequest};
use uuid::Uuid;
use std::sync::Arc;

/// Service layer for task‑related business logic.
#[derive(Clone)]
pub struct TaskService {
    repo: TaskRepository,
}

impl TaskService {
    pub fn new(repo: TaskRepository) -> Self {
        Self { repo }
    }

    pub async fn list_tasks(
        &self,
        user_id: Uuid,
        filters: crate::task::task_repository::TaskFilters,
    ) -> Result<(Vec<Task>, i64)> {
        // Use the method that includes shared tasks
        self.repo.get_user_tasks_including_shared(user_id, filters).await
    }

    pub async fn get_task(&self, user_id: Uuid, task_id: Uuid) -> Result<Task> {
        self.repo
            .find_by_id_with_access(task_id, user_id)
            .await?
            .ok_or_else(|| crate::error::AppError::NotFound("Task not found".into()))
    }

    pub async fn create_task(
        &self,
        user_id: Uuid,
        payload: CreateTaskRequest,
    ) -> Result<Task> {
        let priority = payload.priority.unwrap_or_else(|| "Medium".to_string());
        let task = self.repo
            .create(
                user_id,
                &payload.title,
                payload.description.as_deref(),
                &priority,
                payload.due_date,
                payload.reminder_time,
            )
            .await?;

        // Log activity
        let _ = self.repo.log_task_activity(
            task.id,
            user_id,
            "created",
            Some(serde_json::json!({"title": task.title})),
        ).await;

        // Add creator as owner
        let _ = self.repo.add_task_member(task.id, user_id, "owner", user_id).await;

        Ok(task)
    }

    pub async fn update_task(
        &self,
        user_id: Uuid,
        task_id: Uuid,
        payload: UpdateTaskRequest,
    ) -> Result<Task> {
        // Check access
        if !self.repo.has_task_access(task_id, user_id).await? {
            return Err(crate::error::AppError::Forbidden("Access denied".to_string()));
        }

        let task = self.repo
            .update(
                task_id,
                user_id,
                payload.title.as_deref(),
                payload.description.as_deref(),
                payload.status.as_deref(),
                payload.priority.as_deref(),
                payload.due_date,
                payload.reminder_time,
            )
            .await?;

        // Log activity
        let _ = self.repo.log_task_activity(
            task_id,
            user_id,
            "updated",
            Some(serde_json::json!(payload)),
        ).await;

        Ok(task)
    }

    pub async fn delete_task(&self, user_id: Uuid, task_id: Uuid) -> Result<u64> {
        // Only owner can delete
        if !self.repo.is_task_owner(task_id, user_id).await? {
            return Err(crate::error::AppError::Forbidden("Only task owner can delete".to_string()));
        }

        self.repo.delete(task_id, user_id).await
    }

    pub async fn update_status(
        &self,
        user_id: Uuid,
        task_id: Uuid,
        payload: UpdateTaskStatusRequest,
    ) -> Result<Task> {
        // Check access
        if !self.repo.has_task_access(task_id, user_id).await? {
            return Err(crate::error::AppError::Forbidden("Access denied".to_string()));
        }

        let task = self.repo
            .update_status(task_id, user_id, &payload.status)
            .await?
            .ok_or_else(|| crate::error::AppError::NotFound("Task not found".into()))?;

        // Log activity
        let _ = self.repo.log_task_activity(
            task_id,
            user_id,
            "status_updated",
            Some(serde_json::json!({"new_status": payload.status})),
        ).await;

        Ok(task)
    }

    // Collaboration methods
    pub async fn share_task(
        &self,
        task_id: Uuid,
        user_ids: Vec<Uuid>,
        shared_by: Uuid,
    ) -> Result<()> {
        // Only owner can share
        if !self.repo.is_task_owner(task_id, shared_by).await? {
            return Err(crate::error::AppError::Forbidden("Only task owner can share".to_string()));
        }

        for user_id in user_ids {
            self.repo.add_task_member(task_id, user_id, "collaborator", shared_by).await?;
            
            // Log activity
            let _ = self.repo.log_task_activity(
                task_id,
                shared_by,
                "member_added",
                Some(serde_json::json!({"added_user_id": user_id})),
            ).await;
        }

        Ok(())
    }

    pub async fn remove_collaborator(
        &self,
        task_id: Uuid,
        user_id: Uuid,
        removed_by: Uuid,
    ) -> Result<()> {
        // Only owner can remove collaborators
        if !self.repo.is_task_owner(task_id, removed_by).await? {
            return Err(crate::error::AppError::Forbidden("Only task owner can remove collaborators".to_string()));
        }

        // Cannot remove owner
        if self.repo.is_task_owner(task_id, user_id).await? {
            return Err(crate::error::AppError::BadRequest("Cannot remove task owner".to_string()));
        }

        self.repo.remove_task_member(task_id, user_id).await?;

        // Log activity
        let _ = self.repo.log_task_activity(
            task_id,
            removed_by,
            "member_removed",
            Some(serde_json::json!({"removed_user_id": user_id})),
        ).await;

        Ok(())
    }

    pub async fn get_task_with_members(
        &self,
        task_id: Uuid,
        requesting_user: Uuid,
    ) -> Result<crate::task::task_models::TaskWithMembers> {
        // Check access
        if !self.repo.has_task_access(task_id, requesting_user).await? {
            return Err(crate::error::AppError::Forbidden("Access denied".to_string()));
        }

        let task = self.get_task(requesting_user, task_id).await?;
        let members = self.repo.get_task_members(task_id).await?;
        let is_owner = self.repo.is_task_owner(task_id, requesting_user).await?;

        Ok(crate::task::task_models::TaskWithMembers {
            task,
            members,
            is_owner,
        })
    }

    pub async fn get_task_members(&self, task_id: Uuid, requesting_user: Uuid) -> Result<Vec<crate::task::task_models::TaskMemberInfo>> {
        // Check access
        if !self.repo.has_task_access(task_id, requesting_user).await? {
            return Err(crate::error::AppError::Forbidden("Access denied".to_string()));
        }

        self.repo.get_task_members(task_id).await
    }

    pub async fn get_task_activity(&self, task_id: Uuid, requesting_user: Uuid) -> Result<Vec<crate::task::task_dto::TaskActivityResponse>> {
        // Check access
        if !self.repo.has_task_access(task_id, requesting_user).await? {
            return Err(crate::error::AppError::Forbidden("Access denied".to_string()));
        }

        self.repo.get_task_activity(task_id).await
    }
}
