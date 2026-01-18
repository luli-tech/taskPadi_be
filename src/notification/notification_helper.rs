use crate::error::Result;
use crate::notification::notification_repository::NotificationRepository;
use uuid::Uuid;

/// Helper module for creating notifications for various events
#[derive(Clone)]
pub struct NotificationHelper {
    repo: NotificationRepository,
}

impl NotificationHelper {
    pub fn new(repo: NotificationRepository) -> Self {
        Self { repo }
    }

    /// Send notification when user receives a message
    pub async fn notify_message_received(
        &self,
        receiver_id: Uuid,
        sender_username: &str,
        message_preview: &str,
    ) -> Result<()> {
        let message = if message_preview.len() > 50 {
            format!("New message from {}: {}...", sender_username, &message_preview[..50])
        } else {
            format!("New message from {}: {}", sender_username, message_preview)
        };

        let _ = self.repo.create(receiver_id, None, &message).await;
        Ok(())
    }

    /// Send notification when user receives a group message
    pub async fn notify_group_message_received(
        &self,
        receiver_id: Uuid,
        group_name: &str,
        sender_username: &str,
        message_preview: &str,
    ) -> Result<()> {
        let message = if message_preview.len() > 50 {
            format!("{} in {}: {}...", sender_username, group_name, &message_preview[..50])
        } else {
            format!("{} in {}: {}", sender_username, group_name, message_preview)
        };

        let _ = self.repo.create(receiver_id, None, &message).await;
        Ok(())
    }

    /// Send notification when a task is created
    pub async fn notify_task_created(
        &self,
        user_id: Uuid,
        task_title: &str,
        task_id: Uuid,
    ) -> Result<()> {
        let message = format!("Task created: {}", task_title);
        let _ = self.repo.create(user_id, Some(task_id), &message).await;
        Ok(())
    }

    /// Send notification when a task is updated
    pub async fn notify_task_updated(
        &self,
        user_id: Uuid,
        task_title: &str,
        task_id: Uuid,
        changes: &str,
    ) -> Result<()> {
        let message = format!("Task '{}' updated: {}", task_title, changes);
        let _ = self.repo.create(user_id, Some(task_id), &message).await;
        Ok(())
    }

    /// Send notification when a task is completed
    pub async fn notify_task_completed(
        &self,
        user_id: Uuid,
        task_title: &str,
        task_id: Uuid,
    ) -> Result<()> {
        let message = format!("Task completed: {}", task_title);
        let _ = self.repo.create(user_id, Some(task_id), &message).await;
        Ok(())
    }

    /// Send notification when a task is shared with user
    pub async fn notify_task_shared(
        &self,
        receiver_id: Uuid,
        task_title: &str,
        sharer_username: &str,
        task_id: Uuid,
    ) -> Result<()> {
        let message = format!("{} shared task: {}", sharer_username, task_title);
        let _ = self.repo.create(receiver_id, Some(task_id), &message).await;
        Ok(())
    }

    /// Send notification when user is removed from a task
    pub async fn notify_task_removed(
        &self,
        receiver_id: Uuid,
        task_title: &str,
        remover_username: &str,
    ) -> Result<()> {
        let message = format!("{} removed you from task: {}", remover_username, task_title);
        let _ = self.repo.create(receiver_id, None, &message).await;
        Ok(())
    }

    /// Send notification when user is added to a group
    pub async fn notify_group_member_added(
        &self,
        receiver_id: Uuid,
        group_name: &str,
        adder_username: &str,
    ) -> Result<()> {
        let message = format!("{} added you to group: {}", adder_username, group_name);
        let _ = self.repo.create(receiver_id, None, &message).await;
        Ok(())
    }

    /// Send notification when user is removed from a group
    pub async fn notify_group_member_removed(
        &self,
        receiver_id: Uuid,
        group_name: &str,
        remover_username: &str,
    ) -> Result<()> {
        let message = format!("{} removed you from group: {}", remover_username, group_name);
        let _ = self.repo.create(receiver_id, None, &message).await;
        Ok(())
    }

    /// Send notification when a task reminder is due
    pub async fn notify_task_reminder(
        &self,
        user_id: Uuid,
        task_title: &str,
        task_id: Uuid,
    ) -> Result<()> {
        let message = format!("Reminder: {} is due soon!", task_title);
        let _ = self.repo.create(user_id, Some(task_id), &message).await;
        Ok(())
    }

    /// Generic notification creator
    pub async fn create_notification(
        &self,
        user_id: Uuid,
        message: &str,
        task_id: Option<Uuid>,
    ) -> Result<()> {
        let _ = self.repo.create(user_id, task_id, message).await;
        Ok(())
    }
}
