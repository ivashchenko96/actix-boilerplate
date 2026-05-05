use chrono::Utc;
use sha2::{Digest, Sha256};

use crate::{
    errors::AppResult,
    modules::auth::{
        dto::{AuthResponse, AuthUserView, LoginRequest, RegisterRequest},
        errors::AuthError,
        repository::AuthRepository,
    },
    utils::password,
};

/// Business logic for auth workflows.
#[derive(Clone)]
pub struct AuthService {
    repository: AuthRepository,
}

impl AuthService {
    /// Build auth service.
    pub fn new(repository: AuthRepository) -> Self {
        Self { repository }
    }

    /// Register a new user.
    pub async fn register(&self, request: RegisterRequest) -> AppResult<AuthResponse> {
        if self
            .repository
            .find_user_by_email(&request.email)
            .await?
            .is_some()
        {
            return Err(AuthError::UserAlreadyExists.into());
        }

        let hash = password::hash_password(&request.password)
            .map_err(|e| crate::errors::AppError::internal(&e.to_string()))?;
        let locale = request.locale.unwrap_or_else(|| "en".to_string());
        let user = self
            .repository
            .create_user(&request.email, &hash, request.full_name.as_deref(), &locale)
            .await?;

        self.issue_tokens(user.id, user.email, user.full_name, user.locale)
            .await
    }

    /// Validate login and issue token pair.
    pub async fn login(&self, request: LoginRequest) -> AppResult<AuthResponse> {
        let user = self
            .repository
            .find_user_by_email(&request.email)
            .await?
            .ok_or(AuthError::InvalidCredentials)?;

        if !password::verify_password(&request.password, &user.password_hash)
            .map_err(|e| crate::errors::AppError::internal(&e.to_string()))?
        {
            return Err(AuthError::InvalidCredentials.into());
        }

        self.issue_tokens(user.id, user.email, user.full_name, user.locale)
            .await
    }

    async fn issue_tokens(
        &self,
        user_id: uuid::Uuid,
        email: String,
        full_name: Option<String>,
        locale: String,
    ) -> AppResult<AuthResponse> {
        let access_token = format!("access.{}.{}", user_id, Utc::now().timestamp());
        let refresh_raw = format!("refresh.{}.{}", user_id, Utc::now().timestamp_millis());

        let mut hasher = Sha256::new();
        hasher.update(refresh_raw.as_bytes());
        let refresh_hash = hex::encode(hasher.finalize());
        let _ = self
            .repository
            .store_refresh_token(user_id, &refresh_hash)
            .await?;

        Ok(AuthResponse {
            access_token,
            refresh_token: refresh_raw,
            token_type: "Bearer".to_string(),
            expires_in: 900,
            user: AuthUserView {
                id: user_id.to_string(),
                email,
                full_name,
                locale,
            },
        })
    }
}
