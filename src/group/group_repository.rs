use crate::error::Result;
use sqlx::PgPool;
use uuid::Uuid;
use super::group_models::{Group, GroupMember};

#[derive(Clone)]
pub struct GroupRepository {
    pool: PgPool,
}

impl GroupRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        name: &str,
        description: Option<&str>,
        creator_id: Uuid,
        avatar_url: Option<&str>,
    ) -> Result<Group> {
        let group = sqlx::query_as::<_, Group>(
            "INSERT INTO groups (name, description, creator_id, avatar_url) 
             VALUES ($1, $2, $3, $4) 
             RETURNING *"
        )
        .bind(name)
        .bind(description)
        .bind(creator_id)
        .bind(avatar_url)
        .fetch_one(&self.pool)
        .await?;

        Ok(group)
    }

    pub async fn add_creator_as_member(&self, group_id: Uuid, creator_id: Uuid) -> Result<GroupMember> {
        let member = sqlx::query_as::<_, GroupMember>(
            "INSERT INTO group_members (group_id, user_id, role) 
             VALUES ($1, $2, 'creator') 
             RETURNING *"
        )
        .bind(group_id)
        .bind(creator_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(member)
    }

    pub async fn find_by_id(&self, group_id: Uuid) -> Result<Option<Group>> {
        let group = sqlx::query_as::<_, Group>(
            "SELECT * FROM groups WHERE id = $1"
        )
        .bind(group_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(group)
    }

    pub async fn find_user_groups(&self, user_id: Uuid) -> Result<Vec<Group>> {
        let groups = sqlx::query_as::<_, Group>(
            "SELECT g.* FROM groups g
             INNER JOIN group_members gm ON g.id = gm.group_id
             WHERE gm.user_id = $1
             ORDER BY g.updated_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(groups)
    }

    pub async fn is_member(&self, group_id: Uuid, user_id: Uuid) -> Result<bool> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM group_members 
             WHERE group_id = $1 AND user_id = $2"
        )
        .bind(group_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count > 0)
    }

    pub async fn is_creator(&self, group_id: Uuid, user_id: Uuid) -> Result<bool> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM group_members 
             WHERE group_id = $1 AND user_id = $2 AND role = 'creator'"
        )
        .bind(group_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count > 0)
    }

    pub async fn add_member(&self, group_id: Uuid, user_id: Uuid) -> Result<GroupMember> {
        let member = sqlx::query_as::<_, GroupMember>(
            "INSERT INTO group_members (group_id, user_id, role) 
             VALUES ($1, $2, 'member') 
             RETURNING *"
        )
        .bind(group_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(member)
    }

    pub async fn remove_member(&self, group_id: Uuid, user_id: Uuid) -> Result<()> {
        sqlx::query(
            "DELETE FROM group_members 
             WHERE group_id = $1 AND user_id = $2 AND role != 'creator'"
        )
        .bind(group_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_group_members(&self, group_id: Uuid) -> Result<Vec<(GroupMember, String, Option<String>)>> {
        // Query group members and user info separately, then combine
        let members: Vec<GroupMember> = sqlx::query_as::<_, GroupMember>(
            "SELECT * FROM group_members 
             WHERE group_id = $1
             ORDER BY joined_at ASC"
        )
        .bind(group_id)
        .fetch_all(&self.pool)
        .await?;

        let mut result = Vec::new();
        for member in members {
            // Get user info for each member
            let (username, avatar_url): (String, Option<String>) = sqlx::query_as(
                "SELECT username, avatar_url FROM users WHERE id = $1"
            )
            .bind(member.user_id)
            .fetch_one(&self.pool)
            .await?;

            result.push((member, username, avatar_url));
        }

        Ok(result)
    }

    pub async fn get_member_count(&self, group_id: Uuid) -> Result<i64> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM group_members WHERE group_id = $1"
        )
        .bind(group_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    pub async fn update(
        &self,
        group_id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        avatar_url: Option<&str>,
    ) -> Result<Group> {
        let group = sqlx::query_as::<_, Group>(
            "UPDATE groups 
             SET name = COALESCE($1, name),
                 description = COALESCE($2, description),
                 avatar_url = COALESCE($3, avatar_url),
                 updated_at = NOW()
             WHERE id = $4 
             RETURNING *"
        )
        .bind(name)
        .bind(description)
        .bind(avatar_url)
        .bind(group_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(group)
    }

    pub async fn delete(&self, group_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM groups WHERE id = $1")
            .bind(group_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
