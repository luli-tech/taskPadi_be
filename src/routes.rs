use crate::{
    admin::{
        handlers as admin_handlers,
        dto as admin_dto,
    },
    auth::{
        auth_dto::{AuthResponse, LoginRequest, RefreshTokenRequest, RefreshTokenResponse, RegisterRequest},
        auth_handlers,
    },
    message::{
        message_dto::{ConversationUser, SendMessageRequest},
        message_handlers,
        message_models::{Message, MessageResponse},
    },
    middleware::auth_middleware,
    notification::{
        notification_dto::UpdateNotificationPreferencesRequest,
        notification_handlers,
        notification_models::Notification,
    },
    state::AppState,
    task::{
        task_dto::{CreateTaskRequest, UpdateTaskRequest, UpdateTaskStatusRequest},
        task_handlers,
        task_models::{Task, TaskPriority, TaskStatus},
    },
    user::{
        user_dto::{UpdateProfileRequest, UserStatsResponse},
        user_handlers,
        user_models::{User, UserResponse},
    },
};
use axum::{
    middleware,
    routing::{delete, get, patch, post, put},
    Router,
};
use axum::http::{header::{AUTHORIZATION, CONTENT_TYPE}, Method};
use tower_http::cors::{AllowOrigin, CorsLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::auth::auth_handlers::register,
        crate::auth::auth_handlers::login,
        crate::auth::auth_handlers::google_login,
        crate::auth::auth_handlers::google_callback,
        crate::auth::auth_handlers::refresh_token,
        crate::auth::auth_handlers::logout,
        crate::task::task_handlers::get_tasks,
        crate::task::task_handlers::get_task,
        crate::task::task_handlers::create_task,
        crate::task::task_handlers::update_task,
        crate::task::task_handlers::delete_task,
        crate::task::task_handlers::update_task_status,
        crate::task::task_handlers::task_stream,
        crate::task::task_handlers::share_task,
        crate::task::task_handlers::remove_task_member,
        crate::task::task_handlers::get_task_members,
        crate::task::task_handlers::get_task_activity,
        crate::notification::notification_handlers::get_notifications,
        crate::notification::notification_handlers::notification_stream,
        crate::notification::notification_handlers::mark_notification_read,
        crate::notification::notification_handlers::delete_notification,
        crate::notification::notification_handlers::update_notification_preferences,
        crate::user::user_handlers::get_current_user,
        crate::user::user_handlers::update_current_user,
        crate::user::user_handlers::get_user_stats,
        crate::admin::handlers::get_all_users,
        crate::admin::handlers::get_user_by_id,
        crate::admin::handlers::admin_update_user,
        crate::admin::handlers::delete_user,
        crate::admin::handlers::update_user_status,
        crate::admin::handlers::update_admin_status,
        crate::admin::handlers::get_all_tasks,
        crate::admin::handlers::get_user_tasks,
        crate::admin::handlers::delete_task,
        crate::message::message_handlers::send_message,
        crate::message::message_handlers::get_conversation,
        crate::message::message_handlers::get_conversations,
        crate::message::message_handlers::mark_message_read,
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
            admin_dto::AdminUpdateUserRequest,
            admin_dto::UpdateUserStatusRequest,
            admin_dto::UpdateAdminStatusRequest,
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
        (name = "admin", description = "Admin user management endpoints"),
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
        .allow_origin(AllowOrigin::list([
            "http://localhost:3000".parse().unwrap(),
            "http://127.0.0.1:3000".parse().unwrap(),
            "https://preview-task-manager-web-app-kzmr08fjkyg1tq51kj1l.vusercontent.net"
            .parse()
            .unwrap(),
            "https://id-preview--b130d367-8904-4b37-9f41-ae51af942bec.lovable.app".parse().unwrap(),
            "https://taskpadi.vercel.app".parse().unwrap(),
            "http://localhost:8080".parse().unwrap()
        ]))
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::OPTIONS,
        ])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE])
        .allow_credentials(true);

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
        .route("/:id/share", post(task_handlers::share_task))
        .route("/:id/members", get(task_handlers::get_task_members))
        .route("/:id/members/:user_id", delete(task_handlers::remove_task_member))
        .route("/:id/activity", get(task_handlers::get_task_activity))
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

    // Admin routes
    let admin_routes = Router::new()
        .route("/users", get(admin_handlers::get_all_users))
        .route("/users/:user_id", get(admin_handlers::get_user_by_id)
            .put(admin_handlers::admin_update_user)
            .delete(admin_handlers::delete_user))
        .route("/users/:user_id/status", patch(admin_handlers::update_user_status))
        .route("/users/:user_id/admin", patch(admin_handlers::update_admin_status))
        .route("/users/:user_id/tasks", get(admin_handlers::get_user_tasks))
        .route("/tasks", get(admin_handlers::get_all_tasks))
        .route("/tasks/:task_id", delete(admin_handlers::delete_task))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            crate::admin::admin_middleware::admin_middleware,
        ))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    let message_routes = Router::new()
        .route("/", post(message_handlers::send_message))
        .route("/conversations", get(message_handlers::get_conversations))
        .route("/:user_id", get(message_handlers::get_conversation))
        .route("/:id/read", patch(message_handlers::mark_message_read))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    // WebSocket route
    let ws_routes = Router::new()
        .route("/ws", get(crate::websocket::ws_handler))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    let api_routes = Router::new()
        .nest("/auth", auth_routes)
        .nest("/tasks", task_routes)
        .nest("/notifications", notification_routes)
        .nest("/users", user_routes)
        .nest("/admin", admin_routes)
        .nest("/messages", message_routes)
        .merge(ws_routes);

    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .nest("/api", api_routes)
        .layer(cors)
        .with_state(state)
}
