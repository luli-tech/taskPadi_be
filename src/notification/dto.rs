use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateNotificationPreferencesRequest {
    pub notification_enabled: bool,
}
