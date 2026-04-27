use thiserror::Error;

use crate::errors::AppError;

/// Users module errors.
#[derive(Debug, Error)]
pub enum UsersError {
    #[error("user not found")]
    NotFound,
    #[error("email already exists")]
    EmailConflict,
}

impl From<UsersError> for AppError {
    fn from(value: UsersError) -> Self {
        match value {
            UsersError::NotFound => AppError::not_found("user"),
            UsersError::EmailConflict => AppError::conflict("Email already exists"),
        }
    }
}
