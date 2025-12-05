use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;
use uuid::Uuid;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateProfileRequest {
    #[validate(length(min = 3, max = 255))]
    pub username: Option<String>,
    pub bio: Option<String>,
    #[validate(length(min = 1, max = 50))]
    pub theme: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserStatsResponse {
    pub total_tasks: i64,
    pub pending_tasks: i64,
    pub in_progress_tasks: i64,
    pub completed_tasks: i64,
    pub archived_tasks: i64,
    pub completion_rate: f64,
    pub low_priority_tasks: i64,
    pub medium_priority_tasks: i64,
    pub high_priority_tasks: i64,
    pub urgent_priority_tasks: i64,
}

// Admin DTOs
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct AdminUpdateUserRequest {
    #[validate(length(min = 3, max = 255))]
    pub username: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    pub bio: Option<String>,
    pub theme: Option<String>,
    pub avatar_url: Option<String>,
    pub is_admin: Option<bool>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateUserStatusRequest {
    pub is_active: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateAdminStatusRequest {
    pub is_admin: bool,
}
