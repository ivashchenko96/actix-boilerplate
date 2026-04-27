use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Users table projection.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub email: String,
    pub full_name: Option<String>,
    pub locale: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
