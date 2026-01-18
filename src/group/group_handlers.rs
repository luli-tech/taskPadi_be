use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::Result,
    middleware::AuthUser,
    state::AppState,
    group::group_dto::{CreateGroupRequest, UpdateGroupRequest, AddGroupMemberRequest, RemoveGroupMemberRequest},
};

/// Create a new group
#[utoipa::path(
    post,
    path = "/api/groups",
    tag = "groups",
    request_body = CreateGroupRequest,
    responses(
        (status = 201, description = "Group created successfully", body = GroupResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_group(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(payload): Json<CreateGroupRequest>,
) -> Result<impl IntoResponse> {
    payload.validate()?;

    let group = state.group_service
        .create_group(
            user_id,
            payload.name,
            payload.description,
            payload.avatar_url,
        )
        .await?;

    Ok((StatusCode::CREATED, Json(group)))
}

/// Get all groups for the authenticated user
#[utoipa::path(
    get,
    path = "/api/groups",
    tag = "groups",
    responses(
        (status = 200, description = "Groups retrieved successfully", body = Vec<GroupResponse>),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_groups(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Result<impl IntoResponse> {
    let groups = state.group_service.list_user_groups(user_id).await?;

    Ok((StatusCode::OK, Json(groups)))
}

/// Get a specific group by ID
#[utoipa::path(
    get,
    path = "/api/groups/{group_id}",
    tag = "groups",
    params(
        ("group_id" = Uuid, Path, description = "Group ID")
    ),
    responses(
        (status = 200, description = "Group retrieved successfully", body = GroupResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Not a member"),
        (status = 404, description = "Group not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_group(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(group_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let group = state.group_service.get_group(group_id, user_id).await?;

    Ok((StatusCode::OK, Json(group)))
}

/// Update group (creator only)
#[utoipa::path(
    put,
    path = "/api/groups/{group_id}",
    tag = "groups",
    params(
        ("group_id" = Uuid, Path, description = "Group ID")
    ),
    request_body = UpdateGroupRequest,
    responses(
        (status = 200, description = "Group updated successfully", body = GroupResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Only creator can update"),
        (status = 404, description = "Group not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_group(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(group_id): Path<Uuid>,
    Json(payload): Json<UpdateGroupRequest>,
) -> Result<impl IntoResponse> {
    payload.validate()?;

    let group = state.group_service
        .update_group(
            group_id,
            user_id,
            payload.name,
            payload.description,
            payload.avatar_url,
        )
        .await?;

    Ok((StatusCode::OK, Json(group)))
}

/// Delete group (creator only)
#[utoipa::path(
    delete,
    path = "/api/groups/{group_id}",
    tag = "groups",
    params(
        ("group_id" = Uuid, Path, description = "Group ID")
    ),
    responses(
        (status = 204, description = "Group deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Only creator can delete"),
        (status = 404, description = "Group not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_group(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(group_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    state.group_service.delete_group(group_id, user_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Add a member to the group (creator only)
#[utoipa::path(
    post,
    path = "/api/groups/{group_id}/members",
    tag = "groups",
    params(
        ("group_id" = Uuid, Path, description = "Group ID")
    ),
    request_body = AddGroupMemberRequest,
    responses(
        (status = 201, description = "Member added successfully", body = GroupMemberResponse),
        (status = 400, description = "User already a member"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Only creator can add members"),
        (status = 404, description = "Group not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn add_group_member(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(group_id): Path<Uuid>,
    Json(payload): Json<AddGroupMemberRequest>,
) -> Result<impl IntoResponse> {
    let member = state.group_service
        .add_member(group_id, user_id, payload.user_id)
        .await?;

    // Get member details
    let members = state.group_service.list_group_members(group_id, user_id).await?;
    let member_response = members
        .into_iter()
        .find(|m| m.user_id == payload.user_id)
        .ok_or(crate::error::AppError::NotFound("Member not found".to_string()))?;

    Ok((StatusCode::CREATED, Json(member_response)))
}

/// Remove a member from the group (creator only)
#[utoipa::path(
    delete,
    path = "/api/groups/{group_id}/members/{user_id}",
    tag = "groups",
    params(
        ("group_id" = Uuid, Path, description = "Group ID"),
        ("user_id" = Uuid, Path, description = "User ID to remove")
    ),
    responses(
        (status = 204, description = "Member removed successfully"),
        (status = 400, description = "Cannot remove creator"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Only creator can remove members"),
        (status = 404, description = "Group or member not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn remove_group_member(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path((group_id, member_user_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse> {
    state.group_service
        .remove_member(group_id, user_id, member_user_id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Get all members of a group
#[utoipa::path(
    get,
    path = "/api/groups/{group_id}/members",
    tag = "groups",
    params(
        ("group_id" = Uuid, Path, description = "Group ID")
    ),
    responses(
        (status = 200, description = "Members retrieved successfully", body = Vec<GroupMemberResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Not a member"),
        (status = 404, description = "Group not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_group_members(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(group_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let members = state.group_service.list_group_members(group_id, user_id).await?;

    Ok((StatusCode::OK, Json(members)))
}
