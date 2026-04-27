use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Auth module database user projection.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuthUserRow {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub full_name: Option<String>,
    pub locale: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

/// Refresh token persistence model.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RefreshTokenRow {
    pub jti: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub revoked: bool,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}
