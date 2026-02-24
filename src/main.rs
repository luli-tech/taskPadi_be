mod admin;
mod auth;
mod db;
mod error;
mod group;
mod message;
mod middleware;
mod notification;
mod routes;
mod state;
mod task;
mod user;
mod video_call;
mod websocket;

use auth::create_oauth_client;
use db::{create_pool, run_migrations};
use notification::start_notification_service;
use routes::create_router;
use state::{AppState, Config};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,task_manager=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Arc::new(Config::from_env());

    // Create database connection pools
    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| {
            let error = "DATABASE_URL environment variable is not set. Please set it in your .env file or environment.";
            eprintln!("❌ Error: {}", error);
            eprintln!("💡 Example: DATABASE_URL=postgresql://username:password@localhost:5432/task_manager");
            std::io::Error::new(std::io::ErrorKind::InvalidInput, error)
        })?;
    
    // Sanitize URL for logging (hide password)
    let url_for_logging = database_url
        .split('@')
        .next()
        .map(|part| format!("{}@<hidden>", part))
        .unwrap_or_else(|| "<invalid format>".to_string());
    
    tracing::info!("Connecting to database at {}...", url_for_logging);
    let db = create_pool(&database_url).await.map_err(|e| {
        let error_msg = format!(
            "Failed to connect to database: {}. Please check that:\n  - PostgreSQL is running\n  - DATABASE_URL is correct\n  - The hostname is resolvable\n  - Network connectivity is available",
            e
        );
        eprintln!("❌ {}", error_msg);
        eprintln!("💡 Current DATABASE_URL format: {}", url_for_logging);
        e
    })?;

    // Run migrations
    tracing::info!("Running migrations...");
    run_migrations(&db).await?;

    // Create OAuth client
    let oauth_client = create_oauth_client(
        config.google_client_id.clone(),
        config.google_client_secret.clone(),
        config.google_redirect_uri.clone(),
    )?;

    // Create notification broadcaster
    let (notification_tx, _) = broadcast::channel(100);
    
    // Create task broadcaster
    let (task_tx, _) = broadcast::channel(100);

    // Create WebSocket connection manager
    let ws_connections = crate::websocket::ConnectionManager::new();

    // Connect to NATS for media relay (videocall-rs architecture)
    // App boots normally without NATS — media relay endpoint returns 503 when absent.
    let nats_client = match std::env::var("NATS_URL") {
        Ok(url) => {
            tracing::info!("Connecting to NATS at {}...", url);
            
            let options = if let Ok(creds) = std::env::var("NATS_CREDS") {
                tracing::info!("Using NATS credentials from environment variable");
                
                // Write credentials to a temporary file if it's the raw contents
                let creds_path = if creds.contains("BEGIN NATS USER JWT") {
                    let path = std::env::temp_dir().join("nats.creds");
                    if let Err(e) = std::fs::write(&path, &creds) {
                        tracing::error!("Failed to write temp credentials file: {}", e);
                    }
                    path.to_string_lossy().to_string()
                } else {
                    creds
                };

                match async_nats::ConnectOptions::new().credentials(&creds_path) {
                    Ok(opts) => opts,
                    Err(e) => {
                        tracing::error!("Failed to parse NATS credentials: {}", e);
                        async_nats::ConnectOptions::new()
                    }
                }
            } else {
                async_nats::ConnectOptions::new()
            };

            match async_nats::connect_with_options(&url, options).await {
                Ok(client) => {
                    tracing::info!("✓ Connected to NATS — media relay enabled");
                    Some(client)
                }
                Err(e) => {
                    tracing::warn!("Failed to connect to NATS: {} — media relay disabled", e);
                    None
                }
            }
        }
        Err(_) => {
            tracing::warn!("NATS_URL not set — media relay disabled (set NATS_URL to enable)");
            None
        }
    };

    // Create repositories
    let user_repository = crate::user::user_repository::UserRepository::new(db.clone());
    let task_repository = crate::task::task_repository::TaskRepository::new(db.clone());
    let notification_repository = crate::notification::notification_repository::NotificationRepository::new(db.clone());
    let message_repository = crate::message::message_repository::MessageRepository::new(db.clone());
    let refresh_token_repository = crate::auth::auth_repository::RefreshTokenRepository::new(db.clone());
    let admin_repository = crate::admin::repository::AdminRepository::new(db.clone());
    let group_repository = crate::group::group_repository::GroupRepository::new(db.clone());
    let video_call_repository = crate::video_call::video_call_repository::VideoCallRepository::new(db.clone());

    // Create services
    let user_service = crate::user::user_service::UserService::new(
        user_repository.clone(),
        task_repository.clone(),
    );
    let notification_helper = crate::notification::notification_helper::NotificationHelper::new(notification_repository.clone());
    let task_service = crate::task::task_service::TaskService::new(task_repository.clone(), notification_helper.clone());
    let auth_service = crate::auth::auth_service::AuthService::new(
        db.clone(),
        user_repository.clone(),
        refresh_token_repository.clone(),
        config.jwt_secret.clone(),
    );
    let group_service = crate::group::group_service::GroupService::new(group_repository.clone());
    let message_service = crate::message::message_service::MessageService::new(
        message_repository.clone(),
        ws_connections.clone(),
        notification_repository.clone(),
        group_service.clone(),
        notification_helper.clone(),
        user_repository.clone(),
    );
    let video_call_service = crate::video_call::video_call_service::VideoCallService::new(
        video_call_repository.clone(),
        ws_connections.clone(),
    );
    let admin_service = crate::admin::service::AdminService::new(admin_repository.clone());

    // Create application state
    let state = AppState {
        db: db.clone(),
        config: config.clone(),
        oauth_client,
        notification_tx: notification_tx.clone(),
        task_tx: task_tx.clone(),
        ws_connections,
        nats_client,
        refresh_token_repository,
        user_repository,
        task_repository,
        notification_repository,
        message_repository,
        user_service,
        task_service,
        auth_service,
        message_service,
        admin_repository,
        admin_service,
        group_repository,
        group_service,
        video_call_repository,
        video_call_service,
        notification_helper,
    };

    // Start notification service
    let notification_state = state.clone();
    tokio::spawn(async move {
        if let Err(e) = start_notification_service(notification_state).await {
            tracing::error!("Notification service error: {:?}", e);
        }
    });

    // Create router
    let app = create_router(state);

    // Start server
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("{}:{}", host, port);

    tracing::info!("Server starting on http://{}", addr);
    tracing::info!("Swagger UI available at http://{}/swagger-ui", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
