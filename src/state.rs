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
    websocket::ConnectionManager,
    admin::{
        repository::AdminRepository,
        service::AdminService,
    },
    group::{
        group_repository::GroupRepository,
        group_service::GroupService,
    },
};



#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub config: Arc<Config>,
    pub oauth_client: BasicClient,
    pub notification_tx: broadcast::Sender<String>,
    pub task_tx: broadcast::Sender<(uuid::Uuid, crate::task::task_models::Task)>,
    pub ws_connections: ConnectionManager,
    pub user_repository: UserRepository,
    pub task_repository: TaskRepository,
    pub notification_repository: NotificationRepository,
    pub message_repository: MessageRepository,
    pub refresh_token_repository: RefreshTokenRepository,
    pub user_service: UserService,
    pub task_service: TaskService,
    pub auth_service: AuthService,
    pub message_service: MessageService,
    pub admin_repository: AdminRepository,
    pub admin_service: AdminService,
    pub group_repository: GroupRepository,
    pub group_service: GroupService,
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
