# Task Manager API

A robust task management REST API built with Rust, Axum, PostgreSQL, featuring JWT authentication, Google OAuth, real-time push notifications, and comprehensive API documentation.

## Features

- ✅ **User Authentication**
  - Manual registration and login with JWT
  - Google OAuth 2.0 integration
  - Secure password hashing with bcrypt

- ✅ **Task Management**
  - Full CRUD operations
  - Task filtering by status and priority
  - Due dates and reminder times
  - Task status tracking (Pending, InProgress, Completed, Archived)
  - Priority levels (Low, Medium, High, Urgent)

- ✅ **Push Notifications**
  - Real-time notifications via Server-Sent Events (SSE)
  - Automated cron job checking for due tasks
  - Notification preferences per user
  - Mark notifications as read/delete

- ✅ **API Documentation**
  - Interactive Swagger UI at `/swagger-ui`
  - OpenAPI 3.0 specification
  - Complete endpoint documentation with examples

## Tech Stack

- **Framework**: Axum 0.7
- **Database**: PostgreSQL with SQLx
- **Authentication**: JWT + OAuth2
- **Scheduling**: tokio-cron-scheduler
- **Documentation**: utoipa + Swagger UI
- **Validation**: validator

## Prerequisites

- Rust 1.70+ 
- PostgreSQL 14+
- Google Cloud OAuth credentials (for Google login)

## Setup

### 1. Clone and Install Dependencies

```bash
git clone <repository-url>
cd task-manager
```

### 2. Set Up PostgreSQL Database

```bash
# Create database
createdb task_manager

# Or using psql
psql -U postgres
CREATE DATABASE task_manager;
```

### 3. Configure Environment Variables

Copy `.env.example` to `.env` and update the values:

```bash
cp .env.example .env
```

Edit `.env`:
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

### 4. Google OAuth Setup

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project or select existing
3. Enable Google+ API
4. Go to "Credentials" → "Create Credentials" → "OAuth 2.0 Client ID"
5. Add authorized redirect URI: `http://localhost:3000/api/auth/google/callback`
6. Copy Client ID and Client Secret to `.env`

### 5. Run Migrations

The application automatically runs migrations on startup, or you can run them manually:

```bash
# Install SQLx CLI
cargo install sqlx-cli --no-default-features --features postgres

# Run migrations
sqlx migrate run
```

### 6. Build and Run

```bash
# Development
cargo run

# Production build
cargo build --release
./target/release/task-manager
```

The server will start on `http://localhost:3000`

## API Documentation

Once the server is running, access the interactive API documentation:

**Swagger UI**: http://localhost:3000/swagger-ui

## API Endpoints

### Authentication

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/auth/register` | Register new user |
| POST | `/api/auth/login` | Login with email/password |
| GET | `/api/auth/google` | Initiate Google OAuth |
| GET | `/api/auth/google/callback` | Google OAuth callback |

### Tasks (Requires Authentication)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/tasks` | List all tasks (with filters) |
| GET | `/api/tasks/:id` | Get single task |
| POST | `/api/tasks` | Create new task |
| PUT | `/api/tasks/:id` | Update task |
| DELETE | `/api/tasks/:id` | Delete task |
| PATCH | `/api/tasks/:id/status` | Update task status |

### Notifications (Requires Authentication)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/notifications` | List notifications |
| GET | `/api/notifications/stream` | SSE stream for real-time notifications |
| PATCH | `/api/notifications/:id/read` | Mark as read |
| DELETE | `/api/notifications/:id` | Delete notification |
| PUT | `/api/notifications/preferences` | Update preferences |

## Usage Examples

### 1. Register User

```bash
curl -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "john_doe",
    "email": "john@example.com",
    "password": "securepassword123"
  }'
```

### 2. Login

```bash
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "john@example.com",
    "password": "securepassword123"
  }'
```

Response:
```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGc...",
  "user": {
    "id": "...",
    "username": "john_doe",
    "email": "john@example.com"
  }
}
```

### 3. Create Task with Reminder

```bash
curl -X POST http://localhost:3000/api/tasks \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{
    "title": "Team Meeting",
    "description": "Quarterly review meeting",
    "priority": "High",
    "due_date": "2025-11-25T14:00:00Z",
    "reminder_time": "2025-11-25T13:45:00Z"
  }'
```

### 4. Subscribe to Notifications (SSE)

```bash
curl -N http://localhost:3000/api/notifications/stream \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

Keep this connection open to receive real-time notifications!

### 5. Filter Tasks

```bash
# Get all high priority tasks
curl http://localhost:3000/api/tasks?priority=High \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"

# Get completed tasks
curl http://localhost:3000/api/tasks?status=Completed \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

## Project Structure

```
task-manager/
├── migrations/          # Database migrations
├── src/
│   ├── auth/           # Authentication (JWT, OAuth, passwords)
│   ├── handlers/       # API request handlers
│   ├── middleware/     # Auth middleware
│   ├── models/         # Database models
│   ├── services/       # Background services (notifications)
│   ├── db.rs           # Database connection
│   ├── dto.rs          # Data transfer objects
│   ├── error.rs        # Error handling
│   ├── routes.rs       # Route configuration
│   ├── state.rs        # Application state
│   └── main.rs         # Entry point
├── Cargo.toml
└── README.md
```

## Development

### Running Tests

```bash
cargo test
```

### Code Formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

## How Notifications Work

1. When creating/updating a task, set a `reminder_time`
2. A background cron job runs every minute
3. Tasks with `reminder_time <= now` and `notified = false` trigger notifications
4. Notifications are:
   - Saved to the database
   - Broadcast to connected SSE clients in real-time
   - Task is marked as `notified = true`

## Security Notes

- Always use strong JWT secrets in production
- Enable HTTPS in production
- Rotate JWT tokens regularly
- Keep Google OAuth credentials secure
- Use environment variables for sensitive data

## License

MIT

## Contributing

Pull requests are welcome! For major changes, please open an issue first.
