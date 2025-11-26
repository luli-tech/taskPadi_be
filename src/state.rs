use crate::db::DbPool;
use oauth2::basic::BasicClient;
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::{
    user::user_repository::UserRepository,
    task::task_repository::TaskRepository,
    notification::notification_repository::NotificationRepository,
    message::message_repository::MessageRepository,
    auth::auth_repository::RefreshTokenRepository,
    user::user_service::UserService,
    task::task_service::TaskService,
    auth::auth_service::AuthService,
    message::message_service::MessageService,
};

// Wait, I didn't move RefreshTokenRepository yet. It was created in src/repositories/refresh_token_repository.rs in Step 372.
// I should probably move it to src/auth/repository.rs or src/user/repository.rs?
// Or maybe src/auth/repository.rs is better.
// Let's assume I'll move it to src/auth/repository.rs.

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub config: Arc<Config>,
    pub oauth_client: BasicClient,
    pub notification_tx: broadcast::Sender<String>,
    pub message_tx: broadcast::Sender<(uuid::Uuid, crate::message::message_models::Message)>,
    pub task_tx: broadcast::Sender<(uuid::Uuid, crate::task::task_models::Task)>,
    pub user_repository: UserRepository,
    pub task_repository: TaskRepository,
    pub notification_repository: NotificationRepository,
    pub message_repository: MessageRepository,
    pub refresh_token_repository: RefreshTokenRepository,
    pub user_service: UserService,
    pub task_service: TaskService,
    pub auth_service: AuthService,
    pub message_service: MessageService,
}

#[derive(Clone)]
pub struct Config {
    pub jwt_secret: String,
    pub jwt_expiration_hours: i64,
    pub google_client_id: String,
    pub google_client_secret: String,
    pub google_redirect_uri: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            jwt_secret: std::env::var("JWT_SECRET")
                .expect("JWT_SECRET must be set"),
            jwt_expiration_hours: std::env::var("JWT_EXPIRATION_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .expect("JWT_EXPIRATION_HOURS must be a number"),
            google_client_id: std::env::var("GOOGLE_CLIENT_ID")
                .expect("GOOGLE_CLIENT_ID must be set"),
            google_client_secret: std::env::var("GOOGLE_CLIENT_SECRET")
                .expect("GOOGLE_CLIENT_SECRET must be set"),
            google_redirect_uri: std::env::var("GOOGLE_REDIRECT_URI")
                .expect("GOOGLE_REDIRECT_URI must be set"),
        }
    }
}
