use crate::{
    auth::{
        dto::{AuthResponse, LoginRequest, RefreshTokenRequest, RefreshTokenResponse, RegisterRequest},
        handlers as auth_handlers,
    },
    message::{
        dto::{ConversationUser, SendMessageRequest},
        handlers as message_handlers,
        models::{Message, MessageResponse},
    },
    middleware::auth_middleware,
    notification::{
        dto::UpdateNotificationPreferencesRequest,
        handlers as notification_handlers,
        models::Notification,
    },
    state::AppState,
    task::{
        dto::{CreateTaskRequest, UpdateTaskRequest, UpdateTaskStatusRequest},
        handlers as task_handlers,
        models::{Task, TaskPriority, TaskStatus},
    },
    user::{
        dto::{UpdateProfileRequest, UserStatsResponse},
        handlers as user_handlers,
        models::{User, UserResponse},
    },
};
use axum::{
    middleware,
    routing::{delete, get, patch, post, put},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::auth::handlers::register,
        crate::auth::handlers::login,
        crate::auth::handlers::google_login,
        crate::auth::handlers::google_callback,
        crate::auth::handlers::refresh_token,
        crate::auth::handlers::logout,
        crate::task::handlers::get_tasks,
        crate::task::handlers::get_task,
        crate::task::handlers::create_task,
        crate::task::handlers::update_task,
        crate::task::handlers::delete_task,
        crate::task::handlers::update_task_status,
        crate::task::handlers::task_stream,
        crate::notification::handlers::get_notifications,
        crate::notification::handlers::notification_stream,
        crate::notification::handlers::mark_notification_read,
        crate::notification::handlers::delete_notification,
        crate::notification::handlers::update_notification_preferences,
        crate::user::handlers::get_current_user,
        crate::user::handlers::update_current_user,
        crate::user::handlers::get_user_stats,
        crate::message::handlers::send_message,
        crate::message::handlers::get_conversation,
        crate::message::handlers::get_conversations,
        crate::message::handlers::mark_message_read,
        crate::message::handlers::message_stream,
    ),
    components(
        schemas(
            RegisterRequest,
            LoginRequest,
            AuthResponse,
            RefreshTokenRequest,
            RefreshTokenResponse,
            CreateTaskRequest,
            UpdateTaskRequest,
            UpdateTaskStatusRequest,
            UpdateNotificationPreferencesRequest,
            UpdateProfileRequest,
            UserStatsResponse,
            SendMessageRequest,
            ConversationUser,
            User,
            UserResponse,
            Task,
            TaskStatus,
            TaskPriority,
            Notification,
            Message,
            MessageResponse,
        )
    ),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "tasks", description = "Task management endpoints"),
        (name = "notifications", description = "Notification endpoints"),
        (name = "users", description = "User profile endpoints"),
        (name = "messages", description = "User messaging endpoints")
    ),
    modifiers(&SecurityAddon)
)]
struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::Http::new(
                        utoipa::openapi::security::HttpAuthScheme::Bearer,
                    ),
                ),
            )
        }
    }
}

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Public routes (no auth required)
    let auth_routes = Router::new()
        .route("/register", post(auth_handlers::register))
        .route("/login", post(auth_handlers::login))
        .route("/refresh", post(auth_handlers::refresh_token))
        .route("/logout", post(auth_handlers::logout))
        .route("/google", get(auth_handlers::google_login))
        .route("/google/callback", get(auth_handlers::google_callback));

    // Protected routes (auth required)
    let task_routes = Router::new()
        .route("/", get(task_handlers::get_tasks).post(task_handlers::create_task))
        .route("/stream", get(task_handlers::task_stream))
        .route(
            "/:id",
            get(task_handlers::get_task)
                .put(task_handlers::update_task)
                .delete(task_handlers::delete_task),
        )
        .route("/:id/status", patch(task_handlers::update_task_status))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    let notification_routes = Router::new()
        .route("/", get(notification_handlers::get_notifications))
        .route("/stream", get(notification_handlers::notification_stream))
        .route("/:id/read", patch(notification_handlers::mark_notification_read))
        .route("/:id", delete(notification_handlers::delete_notification))
        .route(
            "/preferences",
            put(notification_handlers::update_notification_preferences),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    let user_routes = Router::new()
        .route("/me", get(user_handlers::get_current_user).put(user_handlers::update_current_user))
        .route("/me/stats", get(user_handlers::get_user_stats))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    let message_routes = Router::new()
        .route("/", post(message_handlers::send_message))
        .route("/conversations", get(message_handlers::get_conversations))
        .route("/stream", get(message_handlers::message_stream))
        .route("/:user_id", get(message_handlers::get_conversation))
        .route("/:id/read", patch(message_handlers::mark_message_read))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    let api_routes = Router::new()
        .nest("/auth", auth_routes)
        .nest("/tasks", task_routes)
        .nest("/notifications", notification_routes)
        .nest("/users", user_routes)
        .nest("/messages", message_routes);

    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .nest("/api", api_routes)
        .layer(cors)
        .with_state(state)
}
