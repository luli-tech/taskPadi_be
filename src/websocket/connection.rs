use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

use super::types::WsMessage;

pub type WsSender = mpsc::UnboundedSender<WsMessage>;

#[derive(Clone)]
pub struct ConnectionManager {
    connections: Arc<DashMap<Uuid, WsSender>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(DashMap::new()),
        }
    }

    /// Add a new user connection
    pub fn add_connection(&self, user_id: Uuid, sender: WsSender) {
        self.connections.insert(user_id, sender);
        tracing::info!("User {} connected via WebSocket", user_id);
    }

    /// Remove a user connection
    pub fn remove_connection(&self, user_id: &Uuid) {
        self.connections.remove(user_id);
        tracing::info!("User {} disconnected from WebSocket", user_id);
    }

    /// Send a message to a specific user
    pub fn send_to_user(&self, user_id: &Uuid, message: WsMessage) -> bool {
        if let Some(sender) = self.connections.get(user_id) {
            sender.send(message).is_ok()
        } else {
            false
        }
    }

    /// Send a message to multiple users
    pub fn send_to_users(&self, user_ids: &[Uuid], message: WsMessage) {
        for user_id in user_ids {
            self.send_to_user(user_id, message.clone());
        }
    }

    /// Broadcast a message to all connected users
    pub fn broadcast(&self, message: WsMessage) {
        for entry in self.connections.iter() {
            let _ = entry.value().send(message.clone());
        }
    }

    /// Get list of online users
    #[allow(dead_code)]
    pub fn get_online_users(&self) -> Vec<Uuid> {
        self.connections.iter().map(|entry| *entry.key()).collect()
    }

    /// Check if a user is online
    #[allow(dead_code)]
    pub fn is_user_online(&self, user_id: &Uuid) -> bool {
        self.connections.contains_key(user_id)
    }

    /// Get count of online users
    #[allow(dead_code)]
    pub fn online_count(&self) -> usize {
        self.connections.len()
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}
