use crate::error::Result;
use crate::message::message_repository::MessageRepository;
use crate::message::message_models::Message;
use crate::message::message_dto::SendMessageRequest;
use crate::websocket::ConnectionManager;
use crate::notification::notification_repository::NotificationRepository;
use crate::websocket::types::{WsMessage, ChatMessagePayload};
use uuid::Uuid;

#[derive(Clone)]
pub struct MessageService {
    repo: MessageRepository,
    ws_manager: ConnectionManager,
    notification_repo: NotificationRepository,
    group_repo: GroupRepository,
}

impl MessageService {
    pub fn new(
        repo: MessageRepository,
        ws_manager: ConnectionManager,
        notification_repo: NotificationRepository,
        group_repo: GroupRepository,
    ) -> Self {
        Self {
            repo,
            ws_manager,
            notification_repo,
            group_repo,
        }
    }

    pub async fn send_message(
        &self,
        sender_id: Uuid,
        payload: SendMessageRequest,
    ) -> Result<Message> {
        // Validate that either receiver_id or group_id is provided, but not both
        if (payload.receiver_id.is_some() && payload.group_id.is_some()) 
            || (payload.receiver_id.is_none() && payload.group_id.is_none()) {
            return Err(crate::error::AppError::BadRequest(
                "Either receiver_id (for 1-on-1) or group_id (for group message) must be provided".to_string()
            ));
        }

        let message = self.repo
            .create(sender_id, payload.receiver_id, payload.group_id, &payload.content, payload.image_url.as_deref())
            .await?;

        if let Some(receiver_id) = payload.receiver_id {
            // 1-on-1 message
            let ws_message = WsMessage::ChatMessage(ChatMessagePayload {
                id: message.id,
                sender_id,
                receiver_id,
                content: message.content.clone(),
                image_url: message.image_url.clone(),
                created_at: message.created_at.to_rfc3339(),
            });

            self.ws_manager.send_to_user(&receiver_id, ws_message.clone());
            self.ws_manager.send_to_user(&sender_id, ws_message);

            // Create notification for receiver
            let notification_text = if message.image_url.is_some() {
                "New message with image".to_string()
            } else {
                format!("New message: {}", &message.content)
            };

            let _ = self.notification_repo
                .create(receiver_id, None, &notification_text)
                .await;
        } else if let Some(group_id) = payload.group_id {
            // Group message - send to all group members
            let members_data: Vec<(crate::group::group_models::GroupMember, String, Option<String>)> = 
                self.group_repo.get_group_members(group_id).await?;
            let member_ids: Vec<Uuid> = members_data.iter().map(|(member, _, _)| member.user_id).collect();
            
            // For group messages, use group_id as receiver_id for WebSocket compatibility
            let ws_message = WsMessage::ChatMessage(ChatMessagePayload {
                id: message.id,
                sender_id,
                receiver_id: group_id, // Use group_id as receiver_id for WebSocket
                content: message.content.clone(),
                image_url: message.image_url.clone(),
                created_at: message.created_at.to_rfc3339(),
            });
            
            self.ws_manager.send_to_users(&member_ids, ws_message.clone());
            // Also send to sender for confirmation
            self.ws_manager.send_to_user(&sender_id, ws_message);

            // Create notifications for all members except sender
            let notification_text = if message.image_url.is_some() {
                "New group message with image".to_string()
            } else {
                format!("New group message: {}", &message.content)
            };

            for member_id in member_ids {
                if member_id != sender_id {
                    let _ = self.notification_repo
                        .create(member_id, None, &notification_text)
                        .await;
                }
            }
        }

        Ok(message)
    }

    pub async fn get_group_messages_with_count(
        &self,
        group_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Message>, i64)> {
        let messages = self.repo.find_group_messages(group_id, limit, offset).await?;
        let total = self.repo.count_group_messages(group_id).await?;
        Ok((messages, total))
    }

    pub async fn get_group_messages(
        &self,
        group_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Message>> {
        self.repo.find_group_messages(group_id, limit, offset).await
    }

    pub async fn mark_group_messages_as_read(
        &self,
        user_id: Uuid,
        group_id: Uuid,
    ) -> Result<()> {
        self.repo.mark_group_messages_as_read(user_id, group_id).await
    }

    pub async fn get_conversation_with_count(
        &self,
        user_id: Uuid,
        other_user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Message>, i64)> {
        let messages = self.repo.find_conversation(user_id, other_user_id, limit, offset).await?;
        let total = self.repo.count_conversation(user_id, other_user_id).await?;
        Ok((messages, total))
    }

    pub async fn get_conversation(
        &self,
        user_id: Uuid,
        other_user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Message>> {
        self.repo.find_conversation(user_id, other_user_id, limit, offset).await
    }

    pub async fn get_conversations(&self, user_id: Uuid) -> Result<Vec<crate::message::message_dto::ConversationUser>> {
        self.repo.find_user_conversations(user_id).await
    }

    pub async fn mark_read(&self, user_id: Uuid, message_id: Uuid) -> Result<()> {
        self.repo.mark_as_read(message_id, user_id).await
    }

    pub async fn mark_conversation_as_read(&self, user_id: Uuid, other_user_id: Uuid) -> Result<()> {
        self.repo.mark_conversation_as_read(user_id, other_user_id).await
    }
}
