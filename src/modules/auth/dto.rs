use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

/// Register request payload.
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct RegisterRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8, max = 128))]
    pub password: String,
    #[validate(length(min = 1, max = 100))]
    pub full_name: Option<String>,
    #[validate(length(min = 2, max = 10))]
    pub locale: Option<String>,
}

/// Login request payload.
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1, max = 128))]
    pub password: String,
}

/// Refresh token request payload.
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct RefreshRequest {
    #[validate(length(min = 10))]
    pub refresh_token: String,
}

/// User view in auth responses.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AuthUserView {
    pub id: String,
    pub email: String,
    pub full_name: Option<String>,
    pub locale: String,
}

/// Auth response payload.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: AuthUserView,
}
