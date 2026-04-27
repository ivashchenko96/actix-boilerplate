use thiserror::Error;

use crate::errors::AppError;

/// Auth module specific errors.
#[derive(Debug, Error)]
pub enum AuthError {
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("user already exists")]
    UserAlreadyExists,
    #[error("refresh token is invalid or expired")]
    InvalidRefreshToken,
}

impl From<AuthError> for AppError {
    fn from(value: AuthError) -> Self {
        match value {
            AuthError::InvalidCredentials => AppError::authentication("Invalid credentials"),
            AuthError::UserAlreadyExists => AppError::conflict("User already exists"),
            AuthError::InvalidRefreshToken => AppError::authentication("Invalid refresh token"),
        }
    }
}
