mod admin;
mod auth;
mod db;
mod error;
mod message;
mod middleware;
mod notification;
mod routes;
mod state;
mod task;
mod user;

use auth::create_oauth_client;
use db::{create_pool, run_migrations};
use notification::service::start_notification_service;
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

    // Create database connection pool
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    tracing::info!("Connecting to database...");
    let db = create_pool(&database_url).await?;

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
    
    // Create message broadcaster
    let (message_tx, _) = broadcast::channel(100);
    
    // Create task broadcaster
    let (task_tx, _) = broadcast::channel(100);

    // Create repositories
    let user_repository = crate::user::repository::UserRepository::new(db.clone());
    let task_repository = crate::task::repository::TaskRepository::new(db.clone());
    let notification_repository = crate::notification::repository::NotificationRepository::new(db.clone());
    let message_repository = crate::message::repository::MessageRepository::new(db.clone());
    let refresh_token_repository = crate::auth::repository::RefreshTokenRepository::new(db.clone());

    // Create application state
    let state = AppState {
        db: db.clone(),
        config: config.clone(),
        oauth_client,
        notification_tx: notification_tx.clone(),
        message_tx: message_tx.clone(),
        task_tx: task_tx.clone(),
        refresh_token_repository,
        user_repository,
        task_repository,
        notification_repository,
        message_repository,
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
