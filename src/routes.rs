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
        message_dto::{ConversationUser, SendMessageRequest, UpdateMessageRequest},
        message_handlers,
        message_models::{Message, MessageResponse},
    },
    group::{
        group_handlers,
        group_models::{Group, GroupResponse, GroupMemberResponse},
        group_dto::{CreateGroupRequest, UpdateGroupRequest, AddGroupMemberRequest},
    },
    video_call::{
        video_call_handlers,
        video_call_models::{VideoCall, VideoCallResponse, CallStatus, CallParticipant, CallParticipantResponse},
        video_call_dto::{InitiateCallRequest, AddParticipantRequest, EndCallRequest, CallHistoryParams},
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
        crate::user::user_handlers::list_users,
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
        crate::auth::auth_handlers::register_admin,
        crate::message::message_handlers::send_message,
        crate::message::message_handlers::get_conversation,
        crate::message::message_handlers::get_conversations,
        crate::message::message_handlers::get_group_messages,
        crate::message::message_handlers::mark_message_read,
        crate::message::message_handlers::update_message,
        crate::message::message_handlers::delete_message,
        crate::group::group_handlers::create_group,
        crate::group::group_handlers::list_groups,
        crate::group::group_handlers::get_group,
        crate::group::group_handlers::update_group,
        crate::group::group_handlers::delete_group,
        crate::group::group_handlers::add_group_member,
        crate::group::group_handlers::remove_group_member,
        crate::group::group_handlers::list_group_members,
        crate::video_call::video_call_handlers::initiate_call,
        crate::video_call::video_call_handlers::accept_call,
        crate::video_call::video_call_handlers::reject_call,
        crate::video_call::video_call_handlers::end_call,
        crate::video_call::video_call_handlers::get_call,
        crate::video_call::video_call_handlers::get_call_history,
        crate::video_call::video_call_handlers::get_active_calls,
        crate::video_call::video_call_handlers::add_participant,
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
            UpdateMessageRequest,
            ConversationUser,
            CreateGroupRequest,
            UpdateGroupRequest,
            AddGroupMemberRequest,
            Group,
            GroupResponse,
            GroupMemberResponse,
            InitiateCallRequest,
            AddParticipantRequest,
            EndCallRequest,
            CallHistoryParams,
            VideoCall,
            CallParticipant,
            CallParticipantResponse,
            VideoCallResponse,
            CallStatus,
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
        (name = "messages", description = "User messaging endpoints"),
        (name = "groups", description = "Group chat endpoints"),
        (name = "video-calls", description = "Video call endpoints")
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
            "https://taskpadi-fe-1.onrender.com".parse().unwrap(),
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
        .route("/", get(user_handlers::list_users))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    // Admin routes - all require admin auth (register is separate, see below)
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
    
    // Separate public admin registration route (no auth required)
    // This allows anyone to register as admin, and they immediately have full admin access
    let admin_register_route = Router::new()
        .route("/register", post(auth_handlers::register_admin));

    let message_routes = Router::new()
        .route("/", post(message_handlers::send_message))
        .route("/conversations", get(message_handlers::get_conversations))
        .route("/conversation/:user_id", get(message_handlers::get_conversation))
        .route("/groups/:group_id", get(message_handlers::get_group_messages))
        .route("/:id", put(message_handlers::update_message).delete(message_handlers::delete_message))
        .route("/:id/read", patch(message_handlers::mark_message_read))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    // Group routes
    let group_routes = Router::new()
        .route("/", post(group_handlers::create_group).get(group_handlers::list_groups))
        .route("/:group_id", get(group_handlers::get_group).put(group_handlers::update_group).delete(group_handlers::delete_group))
        .route("/:group_id/members", get(group_handlers::list_group_members).post(group_handlers::add_group_member))
        .route("/:group_id/members/:user_id", delete(group_handlers::remove_group_member))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    // Video call routes
    let video_call_routes = Router::new()
        .route("/", post(video_call_handlers::initiate_call).get(video_call_handlers::get_call_history))
        .route("/active", get(video_call_handlers::get_active_calls))
        .route("/:call_id", get(video_call_handlers::get_call))
        .route("/:call_id/accept", post(video_call_handlers::accept_call))
        .route("/:call_id/reject", post(video_call_handlers::reject_call))
        .route("/:call_id/end", post(video_call_handlers::end_call))
        .route("/:call_id/participants", post(video_call_handlers::add_participant))
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
        .nest("/admin", admin_routes.merge(admin_register_route))
        .nest("/messages", message_routes)
        .nest("/groups", group_routes)
        .nest("/video-calls", video_call_routes)
        .merge(ws_routes);

    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .nest("/api", api_routes)
        .layer(cors)
        .with_state(state)
}
