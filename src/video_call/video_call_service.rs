use crate::error::{AppError, Result};
use crate::video_call::video_call_models::VideoCallResponse;
use crate::video_call::video_call_repository::VideoCallRepository;
use crate::websocket::ConnectionManager;
use crate::websocket::types::WsMessage;
use uuid::Uuid;

#[derive(Clone)]
pub struct VideoCallService {
    repo: VideoCallRepository,
    ws_manager: ConnectionManager,
}

impl VideoCallService {
    pub fn new(repo: VideoCallRepository, ws_manager: ConnectionManager) -> Self {
        Self { repo, ws_manager }
    }

    pub async fn initiate_call(
        &self,
        caller_id: Uuid,
        receiver_id: Option<Uuid>,
        group_id: Option<Uuid>,
        call_type: String,
    ) -> Result<VideoCallResponse> {
        // Validate inputs
        if receiver_id.is_none() && group_id.is_none() {
            return Err(AppError::BadRequest(
                "Either receiver_id or group_id must be provided".to_string(),
            ));
        }

        if let Some(r_id) = receiver_id {
            if r_id == caller_id {
                return Err(AppError::BadRequest(
                    "Cannot call yourself".to_string(),
                ));
            }
        }

        // Validate call_type
        if call_type != "video" && call_type != "voice" {
            return Err(AppError::BadRequest(
                "Invalid call_type. Must be 'video' or 'voice'".to_string(),
            ));
        }

        // Check for an existing active direct call (only for 1-on-1)
        if let Some(r_id) = receiver_id {
            if let Ok(Some(active_call)) = self.repo.find_active_call(caller_id, r_id).await {
                return Err(AppError::BadRequest(format!(
                    "There is already an active call in progress (call ID: {})",
                    active_call.id
                )));
            }
        }

        // Create new call
        let call = self
            .repo
            .create(caller_id, receiver_id, group_id, &call_type, "initiating")
            .await?;

        // For direct calls: notify the receiver
        if let Some(r_id) = receiver_id {
            let ws_message = WsMessage::CallInitiated(crate::websocket::types::CallInitiatedPayload {
                call_id: call.id,
                caller_id,
                receiver_id: r_id,
                call_type: call_type.clone(),
            });
            self.ws_manager.send_to_user(&r_id, ws_message);

            // Pre-add receiver as participant (status ringing)
            let _ = self.repo.add_participant(call.id, r_id, "participant").await;
        }

        // Update status to ringing
        let call_clone = self.repo.clone();
        let call_id_clone = call.id;
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            let _ = call_clone.update_status(call_id_clone, "ringing").await;
        });

        let mut response: VideoCallResponse = call.into();
        response.participants = self.repo.get_participants(response.id).await.unwrap_or_default();

        Ok(response)
    }

    pub async fn accept_call(
        &self,
        call_id: Uuid,
        user_id: Uuid,
    ) -> Result<VideoCallResponse> {
        let call = self
            .repo
            .find_by_id(call_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Call not found".to_string()))?;

        // Verify user is the receiver
        if let Some(r_id) = call.receiver_id {
            if r_id != user_id {
                return Err(AppError::Forbidden(
                    "Only the receiver can accept the call".to_string(),
                ));
            }
        }

        // Verify call is in a valid state
        if call.status != "ringing" && call.status != "initiating" {
            return Err(AppError::BadRequest(format!(
                "Cannot accept call with status: {}",
                call.status
            )));
        }

        // Start the call
        let call = self.repo.start_call(call_id).await?;

        // Notify caller that call was accepted
        let ws_message = WsMessage::CallAccepted(crate::websocket::types::CallAcceptedPayload {
            call_id: call.id,
            caller_id: call.caller_id,
            receiver_id: call.receiver_id.unwrap_or(user_id), // Simplified for now
            call_type: call.call_type.clone(),
        });
        self.ws_manager.send_to_user(&call.caller_id, ws_message);

        self.repo.update_participant_status(call_id, user_id, "joined").await?;

        let mut response: VideoCallResponse = call.into();
        response.participants = self.repo.get_participants(call_id).await?;

        Ok(response)
    }

    pub async fn add_participant(
        &self,
        call_id: Uuid,
        inviter_id: Uuid,
        new_participant_id: Uuid,
    ) -> Result<VideoCallResponse> {
        let call = self.repo
            .find_by_id(call_id)
            .await?
            .ok_or(AppError::NotFound("Call not found".to_string()))?;

        // Verify inviter is in the call
        let participants = self.repo.get_participants(call_id).await?;
        if !participants.iter().any(|p| p.user_id == inviter_id) {
            return Err(AppError::Forbidden("Only active participants can add others".to_string()));
        }

        // Add to DB
        self.repo.add_participant(call_id, new_participant_id, "participant").await?;

        // Notify the new participant
        let ws_message = WsMessage::CallInitiated(crate::websocket::types::CallInitiatedPayload {
            call_id,
            caller_id: inviter_id,
            receiver_id: new_participant_id,
            call_type: call.call_type.clone(),
        });
        self.ws_manager.send_to_user(&new_participant_id, ws_message);

        // Notify other participants
        // (For mesh, the new guy needs to know WHO to connect to)
        // I should probably add a ParticipantJoined message

        let mut response: VideoCallResponse = call.into();
        response.participants = self.repo.get_participants(call_id).await?;
        Ok(response)
    }

    pub async fn reject_call(
        &self,
        call_id: Uuid,
        user_id: Uuid,
    ) -> Result<VideoCallResponse> {
        let call = self
            .repo
            .find_by_id(call_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Call not found".to_string()))?;

        // Verify user is the receiver
        if let Some(r_id) = call.receiver_id {
            if r_id != user_id {
                return Err(AppError::Forbidden(
                    "Only the receiver can reject the call".to_string(),
                ));
            }
        }

        // Update call status to rejected
        let call = self.repo.update_status(call_id, "rejected").await?;

        // Notify caller that call was rejected
        let ws_message = WsMessage::CallRejected(crate::websocket::types::CallRejectedPayload {
            call_id: call.id,
            caller_id: call.caller_id,
            receiver_id: call.receiver_id.unwrap_or(user_id),
        });
        self.ws_manager.send_to_user(&call.caller_id, ws_message);

        Ok(call.into())
    }

    pub async fn end_call(
        &self,
        call_id: Uuid,
        user_id: Uuid,
    ) -> Result<VideoCallResponse> {
        let call = self
            .repo
            .find_by_id(call_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Call not found".to_string()))?;

        // Verify user is part of the call
        if call.caller_id != user_id && call.receiver_id != Some(user_id) {
            return Err(AppError::Forbidden(
                "You are not part of this call".to_string(),
            ));
        }

        // Calculate duration if call was active
        let duration_seconds = if let Some(started_at) = call.started_at {
            Some((chrono::Utc::now() - started_at).num_seconds() as i32)
        } else {
            None
        };

        // End the call
        let call = self.repo.end_call(call_id, duration_seconds).await?;

        // Notify the other participant
        let other_user_id = if call.caller_id == user_id {
            call.receiver_id
        } else {
            Some(call.caller_id)
        };

        let ws_message = WsMessage::CallEnded(crate::websocket::types::CallEndedPayload {
            call_id: call.id,
            ended_by: user_id,
        });
        if let Some(other_id) = other_user_id {
            self.ws_manager.send_to_user(&other_id, ws_message);
        }

        Ok(call.into())
    }

    pub async fn get_call_history(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<VideoCallResponse>, i64)> {
        let (calls, total) = self
            .repo
            .find_user_call_history(user_id, limit, offset)
            .await?;

        let responses: Vec<VideoCallResponse> = calls.into_iter().map(|c| c.into()).collect();
        Ok((responses, total))
    }

    pub async fn get_active_calls(&self, user_id: Uuid) -> Result<Vec<VideoCallResponse>> {
        let calls = self.repo.get_user_active_calls(user_id).await?;
        let responses: Vec<VideoCallResponse> = calls.into_iter().map(|c| c.into()).collect();
        Ok(responses)
    }

    pub async fn get_call(&self, call_id: Uuid, user_id: Uuid) -> Result<VideoCallResponse> {
        let call = self
            .repo
            .find_by_id(call_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Call not found".to_string()))?;

        // Verify user is part of the call
        if call.caller_id != user_id && call.receiver_id != Some(user_id) {
            return Err(AppError::Forbidden(
                "You are not part of this call".to_string(),
            ));
        }

        Ok(call.into())
    }
}
