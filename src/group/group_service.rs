use crate::error::{Result, AppError};
use uuid::Uuid;
use super::group_repository::GroupRepository;
use super::group_models::{GroupMember, GroupResponse, GroupMemberResponse};

#[derive(Clone)]
pub struct GroupService {
    repo: GroupRepository,
}

impl GroupService {
    pub fn new(repo: GroupRepository) -> Self {
        Self { repo }
    }

    pub async fn create_group(
        &self,
        creator_id: Uuid,
        name: String,
        description: Option<String>,
        avatar_url: Option<String>,
    ) -> Result<GroupResponse> {
        let group = self.repo
            .create(&name, description.as_deref(), creator_id, avatar_url.as_deref())
            .await?;

        // Add creator as a member with 'creator' role
        self.repo.add_creator_as_member(group.id, creator_id).await?;

        let member_count = self.repo.get_member_count(group.id).await?;

        let mut response: GroupResponse = group.into();
        response.member_count = Some(member_count);

        Ok(response)
    }

    pub async fn get_group(&self, group_id: Uuid, user_id: Uuid) -> Result<GroupResponse> {
        // Verify user is a member
        if !self.repo.is_member(group_id, user_id).await? {
            return Err(AppError::Forbidden("You are not a member of this group".to_string()));
        }

        let group = self.repo
            .find_by_id(group_id)
            .await?
            .ok_or(AppError::NotFound("Group not found".to_string()))?;

        let member_count = self.repo.get_member_count(group_id).await?;

        let mut response: GroupResponse = group.into();
        response.member_count = Some(member_count);

        Ok(response)
    }

    pub async fn list_user_groups(&self, user_id: Uuid) -> Result<Vec<GroupResponse>> {
        let groups = self.repo.find_user_groups(user_id).await?;

        let mut responses = Vec::new();
        for group in groups {
            let member_count = self.repo.get_member_count(group.id).await?;
            let mut response: GroupResponse = group.into();
            response.member_count = Some(member_count);
            responses.push(response);
        }

        Ok(responses)
    }

    pub async fn update_group(
        &self,
        group_id: Uuid,
        user_id: Uuid,
        name: Option<String>,
        description: Option<String>,
        avatar_url: Option<String>,
    ) -> Result<GroupResponse> {
        // Only creator can update group
        if !self.repo.is_creator(group_id, user_id).await? {
            return Err(AppError::Forbidden("Only the group creator can update the group".to_string()));
        }

        let group = self.repo
            .update(group_id, name.as_deref(), description.as_deref(), avatar_url.as_deref())
            .await?;

        let member_count = self.repo.get_member_count(group_id).await?;

        let mut response: GroupResponse = group.into();
        response.member_count = Some(member_count);

        Ok(response)
    }

    pub async fn delete_group(&self, group_id: Uuid, user_id: Uuid) -> Result<()> {
        // Only creator can delete group
        if !self.repo.is_creator(group_id, user_id).await? {
            return Err(AppError::Forbidden("Only the group creator can delete the group".to_string()));
        }

        self.repo.delete(group_id).await?;

        Ok(())
    }

    pub async fn add_member(
        &self,
        group_id: Uuid,
        creator_id: Uuid,
        user_id: Uuid,
    ) -> Result<GroupMember> {
        // Only creator can add members
        if !self.repo.is_creator(group_id, creator_id).await? {
            return Err(AppError::Forbidden("Only the group creator can add members".to_string()));
        }

        // Check if user is already a member
        if self.repo.is_member(group_id, user_id).await? {
            return Err(AppError::BadRequest("User is already a member of this group".to_string()));
        }

        self.repo.add_member(group_id, user_id).await
    }

    pub async fn remove_member(
        &self,
        group_id: Uuid,
        creator_id: Uuid,
        user_id: Uuid,
    ) -> Result<()> {
        // Only creator can remove members
        if !self.repo.is_creator(group_id, creator_id).await? {
            return Err(AppError::Forbidden("Only the group creator can remove members".to_string()));
        }

        // Cannot remove the creator
        if self.repo.is_creator(group_id, user_id).await? {
            return Err(AppError::BadRequest("Cannot remove the group creator".to_string()));
        }

        self.repo.remove_member(group_id, user_id).await
    }

    pub async fn list_group_members(&self, group_id: Uuid, user_id: Uuid) -> Result<Vec<GroupMemberResponse>> {
        // Verify user is a member
        if !self.repo.is_member(group_id, user_id).await? {
            return Err(AppError::Forbidden("You are not a member of this group".to_string()));
        }

        let members_data = self.repo.get_group_members(group_id).await?;

        let members: Vec<GroupMemberResponse> = members_data
            .into_iter()
            .map(|(member, username, avatar_url)| GroupMemberResponse {
                id: member.id,
                group_id: member.group_id,
                user_id: member.user_id,
                username,
                avatar_url,
                role: member.role,
                joined_at: member.joined_at,
            })
            .collect();

        Ok(members)
    }

    pub async fn verify_membership(&self, group_id: Uuid, user_id: Uuid) -> Result<()> {
        if !self.repo.is_member(group_id, user_id).await? {
            return Err(AppError::Forbidden("You are not a member of this group".to_string()));
        }
        Ok(())
    }
}
