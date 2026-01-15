use crate::error::Result;
use crate::message::message_repository::MessageRepository;
use crate::message::message_models::Message;
use crate::message::message_dto::{SendMessageRequest, ConversationUser};
use crate::websocket::ConnectionManager;
use crate::notification::notification_repository::NotificationRepository;
use crate::websocket::types::{WsMessage, ChatMessagePayload};
use uuid::Uuid;

#[derive(Clone)]
pub struct MessageService {
    repo: MessageRepository,
    ws_manager: ConnectionManager,
    notification_repo: NotificationRepository,
}

impl MessageService {
    pub fn new(
        repo: MessageRepository,
        ws_manager: ConnectionManager,
        notification_repo: NotificationRepository,
    ) -> Self {
        Self {
            repo,
            ws_manager,
            notification_repo,
        }
    }

    pub async fn send_message(
        &self,
        sender_id: Uuid,
        payload: SendMessageRequest,
    ) -> Result<Message> {
        let message = self.repo
            .create(sender_id, payload.receiver_id, &payload.content, payload.image_url.as_deref())
            .await?;

        // Broadcast via WebSocket
        let ws_message = WsMessage::ChatMessage(ChatMessagePayload {
            id: message.id,
            sender_id,
            receiver_id: payload.receiver_id,
            content: message.content.clone(),
            image_url: message.image_url.clone(),
            created_at: message.created_at.to_rfc3339(),
        });

        // Send to receiver
        self.ws_manager.send_to_user(&payload.receiver_id, ws_message.clone());
        // Send back to sender for confirmation
        self.ws_manager.send_to_user(&sender_id, ws_message);

        // Create notification for receiver
        let notification_text = if message.image_url.is_some() {
            "New message with image".to_string()
        } else {
            format!("New message: {}", &message.content)
        };

        let _ = self.notification_repo
            .create(payload.receiver_id, None, &notification_text)
            .await;

        Ok(message)
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
