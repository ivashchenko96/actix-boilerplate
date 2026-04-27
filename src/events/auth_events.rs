use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginEvent {
    pub user_id: Uuid,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogoutEvent {
    pub user_id: Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginFailedEvent {
    pub email: String,
    pub ip_address: Option<String>,
    pub reason: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}