use crate::error::Result;
use crate::message::message_repository::MessageRepository;
use crate::message::message_models::Message;
use crate::message::message_dto::SendMessageRequest;
use uuid::Uuid;

#[derive(Clone)]
pub struct MessageService {
    repo: MessageRepository,
}

impl MessageService {
    pub fn new(repo: MessageRepository) -> Self {
        Self { repo }
    }

    pub async fn send_message(
        &self,
        sender_id: Uuid,
        payload: SendMessageRequest,
    ) -> Result<Message> {
        self.repo
            .create(sender_id, payload.receiver_id, &payload.content)
            .await
    }

    pub async fn get_conversation(
        &self,
        user_id: Uuid,
        other_user_id: Uuid,
    ) -> Result<Vec<Message>> {
        self.repo.find_conversation(user_id, other_user_id).await
    }

    pub async fn get_conversations(&self, user_id: Uuid) -> Result<Vec<crate::message::message_dto::ConversationUser>> {
        self.repo.find_conversations(user_id).await
    }

    pub async fn mark_read(&self, user_id: Uuid, message_id: Uuid) -> Result<()> {
        self.repo.mark_as_read(message_id, user_id).await
    }
}
