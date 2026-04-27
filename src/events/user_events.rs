use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCreatedEvent {
    pub user_id: Uuid,
    pub email: String,
    pub username: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserUpdatedEvent {
    pub user_id: Uuid,
    pub changes: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDeletedEvent {
    pub user_id: Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}