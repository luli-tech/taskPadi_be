use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

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
