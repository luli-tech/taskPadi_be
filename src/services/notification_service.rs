use crate::state::AppState;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info};

pub async fn start_notification_service(
    state: AppState,
) -> Result<(), Box<dyn std::error::Error>> {
    let scheduler = JobScheduler::new().await?;

    // Run every minute to check for tasks with upcoming reminders
    let job = Job::new_async("0 * * * * *", move |_uuid, _l| {
        let state = state.clone();

        Box::pin(async move {
            if let Err(e) = check_and_send_notifications(state).await {
                error!("Error checking notifications: {:?}", e);
            }
        })
    })?;

    scheduler.add(job).await?;
    scheduler.start().await?;

    info!("Notification service started");
    Ok(())
}

async fn check_and_send_notifications(
    state: AppState,
) -> Result<(), Box<dyn std::error::Error>> {
    // Find tasks with reminders that are due and haven't been notified yet
    let tasks = state.task_repository.find_due_reminders().await?;

    for task in tasks {
        // Create notification in database
        let notification_message = format!(
            "Reminder: {} is due soon!",
            task.title
        );

        state.notification_repository.create(
            task.user_id,
            Some(task.id),
            &notification_message,
        ).await?;

        // Mark task as notified
        state.task_repository.mark_as_notified(task.id).await?;

        // Broadcast to SSE clients
        let broadcast_message = format!(
            "{}:{}",
            task.user_id,
            notification_message
        );
        
        let _ = state.notification_tx.send(broadcast_message);
        
        info!("Sent notification for task: {}", task.title);
    }

    Ok(())
}
