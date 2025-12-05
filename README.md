# Task Manager API

A robust task management REST API built with Rust, Axum, PostgreSQL, featuring **WebSocket real-time chat**, **collaborative tasks**, **admin user management**, JWT authentication, Google OAuth, push notifications, and comprehensive API documentation.

## Features

- **User Authentication**
  - Manual registration and login with JWT
  - Short-lived access tokens (15 min) + long-lived refresh tokens (7 days)
  - Token refresh endpoint (rotating refresh tokens)
  - Secure token revocation on logout
  - Google OAuth 2.0 integration
  - Secure password hashing with bcrypt
  - Role‑based authorization (user/admin)
  - Account status management (active/inactive)

- **Admin User Management** 🆕
  - List all users (paginated)
  - View user details
  - Update user information
  - Delete users
  - Activate/deactivate user accounts
  - Promote/demote admin privileges
  - Admin-only protected endpoints

- **Task Management**
  - Full CRUD operations
  - Filtering by status, priority, due date, etc.
  - Due dates and reminder times
  - Status tracking (Pending, InProgress, Completed, Archived)
  - Priority levels (Low, Medium, High, Urgent)

- **Collaborative Tasks** 🆕
  - Share tasks with multiple users
  - Real-time task updates via WebSocket
  - Task member management (add/remove collaborators)
  - Activity audit logging
  - Access control (owner vs collaborator permissions)
  - View shared tasks in task list

- **Real-time Chat** 🆕
  - WebSocket-based bidirectional communication
  - Real-time message delivery
  - Typing indicators
  - Online/offline status tracking
  - Message delivery confirmations
  - Thread-safe connection management

- **Push Notifications**
  - Real‑time notifications via Server‑Sent Events (SSE)
  - Automated cron job checking for due tasks
  - Per‑user notification preferences
  - Mark notifications as read / delete

- **API Documentation**
  - Interactive Swagger UI at `/swagger-ui`
  - OpenAPI 3.0 specification generated with `utoipa`
  - Complete endpoint documentation with examples

## Tech Stack

- **Framework**: Axum 0.7 (with WebSocket support)
- **Database**: PostgreSQL with SQLx
- **Authentication**: JWT + OAuth2 (Google)
- **Real-time**: WebSocket + Server-Sent Events (SSE)
- **Concurrency**: DashMap for thread-safe connection management
- **Scheduling**: tokio‑cron‑scheduler
- **Documentation**: utoipa + Swagger UI
- **Validation**: validator

## Prerequisites

- Rust 1.70+
- PostgreSQL 14+
- Google Cloud OAuth credentials (for Google login)

## Setup

### 1. Clone the repository

```bash
git clone <repository-url>
cd task-manager
```

### 2. Set up the PostgreSQL database

```bash
# Create database
createdb task_manager
# Or using psql
psql -U postgres -c "CREATE DATABASE task_manager;"
```

### 3. Configure environment variables

```bash
cp .env.example .env
```

Edit `.env` and set the required values:

```env
DATABASE_URL=postgresql://username:password@localhost:5432/task_manager
JWT_SECRET=your-super-secret-jwt-key-change-this-in-production
JWT_EXPIRATION_HOURS=24
GOOGLE_CLIENT_ID=your-google-client-id.apps.googleusercontent.com
GOOGLE_CLIENT_SECRET=your-google-client-secret
GOOGLE_REDIRECT_URI=http://localhost:3000/api/auth/google/callback
HOST=127.0.0.1
PORT=3000
RUST_LOG=info,task_manager=debug
```

### 4. Google OAuth setup

1. Go to the [Google Cloud Console](https://console.cloud.google.com/)
2. Create/select a project
3. Enable the Google+ API
4. Create **OAuth 2.0 Client ID** credentials
5. Add the authorized redirect URI `http://localhost:3000/api/auth/google/callback`
6. Copy the client ID and secret into `.env`

### 5. Run database migrations

The application runs migrations automatically on startup, or you can run them manually:

```bash
cargo install sqlx-cli --no-default-features --features postgres
sqlx migrate run
```

**Migrations include:**
- User tables with admin and active status fields
- Task tables with collaborative features
- Task members and activity logging tables
- Message and notification tables

### 6. Build and run the application

```bash
# Development
cargo run

# Production build
cargo build --release
./target/release/task-manager
```

The server will start on `http://localhost:3000`.

## API Documentation

Access the interactive Swagger UI at:

```
http://localhost:3000/swagger-ui
```

## API Endpoints

### Authentication

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/auth/register` | Register a new user |
| POST | `/api/auth/login` | Login with email/password |
| POST | `/api/auth/refresh` | Refresh access token (returns new access & refresh tokens) |
| POST | `/api/auth/logout` | Logout and revoke refresh token |
| GET | `/api/auth/google` | Initiate Google OAuth |
| GET | `/api/auth/google/callback` | Google OAuth callback |

### Tasks (requires authentication)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/tasks` | List all tasks (includes shared tasks, supports filters) |
| GET | `/api/tasks/:id` | Retrieve a single task |
| POST | `/api/tasks` | Create a new task |
| PUT | `/api/tasks/:id` | Update an existing task |
| DELETE | `/api/tasks/:id` | Delete a task (owner only) |
| PATCH | `/api/tasks/:id/status` | Update task status |
| POST | `/api/tasks/:id/share` | Share task with users 🆕 |
| GET | `/api/tasks/:id/members` | Get task members 🆕 |
| DELETE | `/api/tasks/:id/members/:user_id` | Remove collaborator 🆕 |
| GET | `/api/tasks/:id/activity` | Get task activity log 🆕 |

### Admin (requires admin role) 🆕

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/admin/users` | List all users (paginated) |
| GET | `/api/admin/users/:id` | Get specific user details |
| PUT | `/api/admin/users/:id` | Update user information |
| DELETE | `/api/admin/users/:id` | Delete user |
| PATCH | `/api/admin/users/:id/status` | Activate/deactivate user |
| PATCH | `/api/admin/users/:id/admin` | Promote/demote admin |

### WebSocket 🆕

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/ws` | WebSocket upgrade endpoint for real-time chat |

### Messages (requires authentication)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/messages/conversations` | List conversations |
| GET | `/api/messages/:user_id` | Get messages in a conversation |
| POST | `/api/messages` | Send a new message (deprecated - use WebSocket) |
| PATCH | `/api/messages/:id/read` | Mark a message as read |

### Notifications (requires authentication)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/notifications` | List notifications |
| GET | `/api/notifications/stream` | SSE stream for real-time notifications |
| PATCH | `/api/notifications/:id/read` | Mark as read |
| DELETE | `/api/notifications/:id` | Delete notification |
| PUT | `/api/notifications/preferences` | Update preferences |

### Users (requires authentication)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/users/me` | Get current user profile |
| PUT | `/api/users/me` | Update current user profile |
| GET | `/api/users/me/stats` | Get user statistics |

## Endpoint Use Cases

### Authentication
- **Register** – Create a new account with username, email, and password.
- **Login** – Obtain short‑lived access token and long‑lived refresh token.
- **Refresh** – Exchange a valid refresh token for a new access token without re‑entering credentials.
- **Logout** – Invalidate the refresh token, effectively signing the user out.
- **Google OAuth** – Sign‑in using a Google account, simplifying registration and login.

### Tasks
- **List Tasks** – Retrieve a paginated list including both owned and shared tasks; supports filtering by status, priority, due date, etc.
- **Get Task** – Fetch detailed information for a task (requires access permission).
- **Create Task** – Authenticated users can create tasks with title, description, priority, due date, and optional reminder.
- **Update Task** – Modify mutable fields such as title, description, priority, or due date (requires access permission).
- **Delete Task** – Permanently remove a task (owner only).
- **Update Task Status** – Change the status (e.g., from `Pending` to `InProgress`). Broadcasts real-time updates to all task members via WebSocket.
- **Share Task** – Share a task with multiple users, granting them collaborator access. Sends real-time notifications via WebSocket.
- **Get Task Members** – View all collaborators on a task with their roles and details.
- **Remove Collaborator** – Remove a user from a task (owner only). Sends real-time notification via WebSocket.
- **Get Task Activity** – View complete audit log of all actions performed on a task.

### Admin Operations
- **List Users** – View all registered users with pagination (admin only).
- **Get User** – View detailed information about any user (admin only).
- **Update User** – Modify user information including email, username, profile, admin status, and active status (admin only).
- **Delete User** – Permanently remove a user account (admin only).
- **Activate/Deactivate** – Enable or disable user accounts. Deactivated users cannot log in (admin only).
- **Promote/Demote Admin** – Grant or revoke admin privileges (admin only).

### WebSocket Real-time Chat
- **Connect** – Establish WebSocket connection at `/api/ws` with JWT authentication.
- **Send Message** – Send real-time chat messages to other users.
- **Typing Indicators** – Broadcast typing status to conversation participants.
- **Online Status** – Automatic online/offline status tracking and broadcasting.
- **Message Delivery** – Real-time message delivery confirmations.
- **Task Notifications** – Receive real-time notifications when tasks are shared, updated, or when members are added/removed.

### Notifications
- **List Notifications** – Return all notifications for the authenticated user, optionally filtered by read/unread state.
- **Notification Stream (SSE)** – Open a Server‑Sent Events connection to receive real‑time push notifications when tasks reach their reminder time or other events occur.
- **Mark as Read** – Mark a specific notification as read, allowing UI state updates.
- **Delete Notification** – Remove a notification from the user's inbox.
- **Update Preferences** – Configure notification settings such as enabling/disabling email or push notifications.

## Usage Examples

### Register a user

```bash
curl -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"john_doe","email":"john@example.com","password":"securepassword123"}'
```

### Login

```bash
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"john@example.com","password":"securepassword123"}'
```

### Create a task with a reminder

```bash
curl -X POST http://localhost:3000/api/tasks \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{"title":"Team Meeting","description":"Quarterly review meeting","priority":"High","due_date":"2025-11-25T14:00:00Z","reminder_time":"2025-11-25T13:45:00Z"}'
```

### Share a task with collaborators

```bash
curl -X POST http://localhost:3000/api/tasks/TASK_ID/share \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{"user_ids":["user-uuid-1","user-uuid-2"]}'
```

### Connect to WebSocket for real-time chat

```javascript
const ws = new WebSocket('ws://localhost:3000/api/ws');

ws.onopen = () => {
  console.log('Connected to WebSocket');
};

ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  console.log('Received:', message);
};

// Send a chat message
ws.send(JSON.stringify({
  type: 'send_message',
  receiver_id: 'user-uuid',
  content: 'Hello!',
  image_url: null
}));

// Send typing indicator
ws.send(JSON.stringify({
  type: 'typing_indicator',
  conversation_with: 'user-uuid',
  is_typing: true
}));
```

### Admin: List all users

```bash
curl -H "Authorization: Bearer YOUR_ADMIN_TOKEN" \
  "http://localhost:3000/api/admin/users?page=1&limit=10"
```

### Admin: Deactivate a user

```bash
curl -X PATCH \
  -H "Authorization: Bearer YOUR_ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"is_active":false}' \
  http://localhost:3000/api/admin/users/USER_ID/status
```

### Subscribe to notifications (SSE)

```bash
curl -N http://localhost:3000/api/notifications/stream \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

### Filter tasks

```bash
# High‑priority tasks
curl http://localhost:3000/api/tasks?priority=High \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"

# Completed tasks
curl http://localhost:3000/api/tasks?status=Completed \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

## Project Structure

```text
task-manager/
├── migrations/                    # Database migrations
│   ├── 20251124_001_init.sql
│   ├── 20251126_001_add_features.sql
│   ├── 20251205_001_add_user_status.sql
│   └── 20251205_002_add_task_collaboration.sql
│
├── src/
│   ├── auth/                      # Authentication module
│   │   ├── auth_dto.rs            # DTOs (RegisterRequest, LoginRequest, etc.)
│   │   ├── auth_handlers.rs       # Handlers (register, login, OAuth)
│   │   ├── auth_models.rs         # RefreshToken model
│   │   ├── auth_repository.rs     # RefreshToken repository
│   │   ├── auth_service.rs        # Business logic
│   │   ├── jwt.rs                 # JWT generation/validation
│   │   ├── oauth.rs               # Google OAuth client
│   │   ├── password.rs            # Password hashing/verification
│   │   └── mod.rs                 # Module exports
│   │
│   ├── message/                   # Messaging module
│   │   ├── message_dto.rs         # DTOs
│   │   ├── message_handlers.rs    # Handlers
│   │   ├── message_models.rs      # Models
│   │   ├── message_repository.rs  # Repository
│   │   ├── message_service.rs     # Service layer
│   │   └── mod.rs                 # Module exports
│   │
│   ├── notification/              # Notification module
│   │   ├── notification_dto.rs    # DTOs
│   │   ├── notification_handlers.rs # Handlers
│   │   ├── notification_models.rs # Models
│   │   ├── notification_repository.rs # Repository
│   │   ├── notification_service.rs # Service (background job)
│   │   └── mod.rs                 # Module exports
│   │
│   ├── task/                      # Task module
│   │   ├── task_dto.rs            # DTOs (CreateTaskRequest, ShareTaskRequest, etc.)
│   │   ├── task_handlers.rs       # Handlers (includes collaboration endpoints)
│   │   ├── task_models.rs         # Models (Task, TaskMember, TaskActivity, etc.)
│   │   ├── task_repository.rs     # Repository (includes collaboration methods)
│   │   ├── task_service.rs        # Service layer (includes collaboration logic)
│   │   └── mod.rs                 # Module exports
│   │
│   ├── user/                      # User module
│   │   ├── user_dto.rs            # DTOs (UpdateProfileRequest, AdminUpdateUserRequest, etc.)
│   │   ├── user_handlers.rs       # Handlers (includes admin endpoints)
│   │   ├── user_models.rs         # Models (User, UserResponse)
│   │   ├── user_repository.rs     # Repository (includes admin methods)
│   │   ├── user_service.rs        # Service layer
│   │   └── mod.rs                 # Module exports
│   │
│   ├── websocket/                 # WebSocket module 🆕
│   │   ├── connection.rs          # Connection manager (DashMap-based)
│   │   ├── handler.rs             # WebSocket upgrade and message handlers
│   │   ├── types.rs               # WebSocket message types and protocols
│   │   └── mod.rs                 # Module exports
│   │
│   ├── middleware/                # Middleware
│   │   ├── auth.rs                # JWT authentication middleware
│   │   ├── admin_middleware.rs    # Admin authorization middleware 🆕
│   │   └── mod.rs                 # Module exports
│   │
│   ├── db.rs                      # Database connection & migrations
│   ├── error.rs                   # Error handling & AppError type
│   ├── routes.rs                  # API route configuration
│   ├── state.rs                   # AppState & Config
│   └── main.rs                    # Application entry point
│
├── .github/
│   └── workflows/
│       └── ci.yml                 # GitHub Actions CI/CD
│
├── Cargo.toml                     # Rust dependencies
├── .env.example                   # Environment variables template
└── README.md
```

### Architecture Layers

- **Repository Layer** (`*.repository.rs`): Direct DB interactions using SQLx.
- **Service Layer** (`*.service.rs`): Business logic, orchestrates repositories.
- **Handler Layer** (`*.handlers.rs`): HTTP request/response handling, validation, calls services.
- **Models** (`*.models.rs`): Database models and response DTOs.
- **DTOs** (`*.dto.rs`): Request/response data structures with validation rules.

## Development

### Running tests

```bash
cargo test
```

### Code formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

## How Features Work

### Notifications
1. When creating/updating a task, set a `reminder_time`.
2. A background cron job runs every minute.
3. Tasks with `reminder_time <= now` and `notified = false` trigger notifications.
4. Notifications are saved to the DB, broadcast via SSE, and the task is marked `notified = true`.

### WebSocket Real-time Chat
1. Client connects to `/api/ws` with JWT token in Authorization header.
2. Connection is upgraded to WebSocket protocol.
3. User is added to the connection manager (DashMap).
4. Online status is broadcast to all connected users.
5. Messages are sent/received in real-time using JSON-formatted WebSocket messages.
6. Typing indicators and delivery confirmations work similarly.
7. On disconnect, user is removed and offline status is broadcast.

### Collaborative Tasks
1. Task owner shares a task with collaborators using `/api/tasks/:id/share`.
2. Collaborators are added to `task_members` table with role "collaborator".
3. All actions (create, update, share, status change) are logged to `task_activity` table.
4. When task is updated, WebSocket notifications are sent to all members in real-time.
5. Both owners and collaborators can view and update the task (owners have additional permissions).
6. Task list includes both owned tasks and tasks shared with the user.

### Admin User Management
1. First user registered is automatically set as admin (via migration).
2. Admin middleware checks `is_admin` and `is_active` status before allowing access.
3. Admins can view all users, update any user information, and manage account status.
4. Deactivated users (`is_active = false`) cannot log in (checked in auth middleware).
5. Admin status can be granted/revoked by other admins.

## Security Notes

- Use strong JWT secrets in production.
- Enable HTTPS.
- Rotate JWT tokens regularly.
- Keep Google OAuth credentials secure.
- Store sensitive data in environment variables.
- WebSocket connections are authenticated via JWT.
- Admin endpoints are protected by admin middleware.
- Task access is controlled by ownership and membership checks.

## License

MIT

## Contributing

Pull requests are welcome! For major changes, please open an issue first.
