// src/task/task.service.rs
use crate::error::Result;
use crate::task::task_repository::TaskRepository;
use crate::task::task_models::Task;
use crate::task::task_dto::{CreateTaskRequest, UpdateTaskRequest, UpdateTaskStatusRequest};
use uuid::Uuid;

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
        self.repo.find_all(user_id, filters).await
    }

    pub async fn get_task(&self, user_id: Uuid, task_id: Uuid) -> Result<Task> {
        self.repo
            .find_by_id(task_id, user_id)
            .await?
            .ok_or_else(|| crate::error::AppError::NotFound("Task not found".into()))
    }

    pub async fn create_task(
        &self,
        user_id: Uuid,
        payload: CreateTaskRequest,
    ) -> Result<Task> {
        let priority = payload.priority.unwrap_or_else(|| "Medium".to_string());
        self.repo
            .create(
                user_id,
                &payload.title,
                payload.description.as_deref(),
                &priority,
                payload.due_date,
                payload.reminder_time,
            )
            .await
    }

    pub async fn update_task(
        &self,
        user_id: Uuid,
        task_id: Uuid,
        payload: UpdateTaskRequest,
    ) -> Result<Task> {
        self.repo
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
            .await
    }

    pub async fn delete_task(&self, user_id: Uuid, task_id: Uuid) -> Result<u64> {
        self.repo.delete(task_id, user_id).await
    }

    pub async fn update_status(
        &self,
        user_id: Uuid,
        task_id: Uuid,
        payload: UpdateTaskStatusRequest,
    ) -> Result<Task> {
        self.repo
            .update_status(task_id, user_id, &payload.status)
            .await?
            .ok_or_else(|| crate::error::AppError::NotFound("Task not found".into()))
    }
}
