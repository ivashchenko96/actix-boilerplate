use actix_web::{HttpResponse, ResponseError};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::ValidationErrors;

/// Main application error type
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Redis error: {0}")]
    Redis(#[from] fred::error::RedisError),
    
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationErrors),
    
    #[error("Authentication error: {message}")]
    Authentication { message: String },
    
    #[error("Authorization error: {message}")]
    Authorization { message: String },
    
    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    
    #[error("Password hashing error: {0}")]
    PasswordHash(String),
    
    #[error("Email error: {0}")]
    Email(#[from] lettre::error::Error),
    
    #[error("Storage error: {0}")]
    Storage(String),
    
    #[error("External service error: {service}: {message}")]
    ExternalService { service: String, message: String },
    
    #[error("Rate limit exceeded")]
    RateLimit,
    
    #[error("Not found: {resource}")]
    NotFound { resource: String },
    
    #[error("Conflict: {message}")]
    Conflict { message: String },
    
    #[error("Bad request: {message}")]
    BadRequest { message: String },
    
    #[error("Internal server error: {message}")]
    Internal { message: String },
    
    #[error("Feature not enabled: {feature}")]
    FeatureNotEnabled { feature: String },
    
    #[error("Service unavailable: {service}")]
    ServiceUnavailable { service: String },
}

/// Standard API response structure
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
    pub errors: Option<Vec<ApiError>>,
    pub locale: String,
    pub request_id: String,
}

/// Individual error in API response
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub field: Option<String>,
}

impl<T> ApiResponse<T> {
    /// Create a successful response
    pub fn success(data: T, message: String, locale: String, request_id: String) -> Self {
        Self {
            success: true,
            message,
            data: Some(data),
            errors: None,
            locale,
            request_id,
        }
    }

    /// Create an error response
    pub fn error(
        errors: Vec<ApiError>,
        message: String,
        locale: String,
        request_id: String,
    ) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            message,
            data: None,
            errors: Some(errors),
            locale,
            request_id,
        }
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let request_id = Uuid::new_v4().to_string();
        let locale = "en".to_string(); // Default locale, should be extracted from request

        let (status_code, error_code, message) = match self {
            AppError::Database(_) => (
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                "DATABASE_ERROR",
                "A database error occurred".to_string(),
            ),
            AppError::Redis(_) => (
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                "REDIS_ERROR", 
                "A cache error occurred".to_string(),
            ),
            AppError::Validation(errors) => {
                let validation_errors: Vec<ApiError> = errors
                    .field_errors()
                    .iter()
                    .flat_map(|(field, field_errors)| {
                        field_errors.iter().map(move |error| ApiError {
                            code: error.code.to_string(),
                            message: error.message.as_deref().unwrap_or("Validation error").to_string(),
                            field: Some(field.to_string()),
                        })
                    })
                    .collect();

                return HttpResponse::BadRequest().json(ApiResponse::<()>::error(
                    validation_errors,
                    "Validation failed".to_string(),
                    locale,
                    request_id,
                ));
            }
            AppError::Authentication { message } => (
                actix_web::http::StatusCode::UNAUTHORIZED,
                "AUTHENTICATION_ERROR",
                message.clone(),
            ),
            AppError::Authorization { message } => (
                actix_web::http::StatusCode::FORBIDDEN,
                "AUTHORIZATION_ERROR",
                message.clone(),
            ),
            AppError::Jwt(_) => (
                actix_web::http::StatusCode::UNAUTHORIZED,
                "JWT_ERROR",
                "Invalid or expired token".to_string(),
            ),
            AppError::NotFound { resource } => (
                actix_web::http::StatusCode::NOT_FOUND,
                "NOT_FOUND",
                format!("{} not found", resource),
            ),
            AppError::Conflict { message } => (
                actix_web::http::StatusCode::CONFLICT,
                "CONFLICT",
                message.clone(),
            ),
            AppError::BadRequest { message } => (
                actix_web::http::StatusCode::BAD_REQUEST,
                "BAD_REQUEST",
                message.clone(),
            ),
            AppError::RateLimit => (
                actix_web::http::StatusCode::TOO_MANY_REQUESTS,
                "RATE_LIMIT_EXCEEDED",
                "Rate limit exceeded".to_string(),
            ),
            AppError::FeatureNotEnabled { feature } => (
                actix_web::http::StatusCode::NOT_IMPLEMENTED,
                "FEATURE_NOT_ENABLED",
                format!("Feature '{}' is not enabled", feature),
            ),
            AppError::ServiceUnavailable { service } => (
                actix_web::http::StatusCode::SERVICE_UNAVAILABLE,
                "SERVICE_UNAVAILABLE",
                format!("Service '{}' is unavailable", service),
            ),
            _ => (
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "An internal error occurred".to_string(),
            ),
        };

        let error = ApiError {
            code: error_code.to_string(),
            message: message.clone(),
            field: None,
        };

        HttpResponse::build(status_code).json(ApiResponse::<()>::error(
            vec![error],
            message,
            locale,
            request_id,
        ))
    }

    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            AppError::Validation(_) | AppError::BadRequest { .. } => {
                actix_web::http::StatusCode::BAD_REQUEST
            }
            AppError::Authentication { .. } | AppError::Jwt(_) => {
                actix_web::http::StatusCode::UNAUTHORIZED
            }
            AppError::Authorization { .. } => actix_web::http::StatusCode::FORBIDDEN,
            AppError::NotFound { .. } => actix_web::http::StatusCode::NOT_FOUND,
            AppError::Conflict { .. } => actix_web::http::StatusCode::CONFLICT,
            AppError::RateLimit => actix_web::http::StatusCode::TOO_MANY_REQUESTS,
            AppError::FeatureNotEnabled { .. } => actix_web::http::StatusCode::NOT_IMPLEMENTED,
            AppError::ServiceUnavailable { .. } => actix_web::http::StatusCode::SERVICE_UNAVAILABLE,
            _ => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// Result type alias for application operations
pub type AppResult<T> = Result<T, AppError>;

/// Helper functions for creating common errors
impl AppError {
    pub fn not_found(resource: &str) -> Self {
        Self::NotFound {
            resource: resource.to_string(),
        }
    }

    pub fn conflict(message: &str) -> Self {
        Self::Conflict {
            message: message.to_string(),
        }
    }

    pub fn bad_request(message: &str) -> Self {
        Self::BadRequest {
            message: message.to_string(),
        }
    }

    pub fn internal(message: &str) -> Self {
        Self::Internal {
            message: message.to_string(),
        }
    }

    pub fn authentication(message: &str) -> Self {
        Self::Authentication {
            message: message.to_string(),
        }
    }

    pub fn authorization(message: &str) -> Self {
        Self::Authorization {
            message: message.to_string(),
        }
    }

    pub fn feature_not_enabled(feature: &str) -> Self {
        Self::FeatureNotEnabled {
            feature: feature.to_string(),
        }
    }

    pub fn service_unavailable(service: &str) -> Self {
        Self::ServiceUnavailable {
            service: service.to_string(),
        }
    }
}

impl From<argon2::password_hash::Error> for AppError {
    fn from(value: argon2::password_hash::Error) -> Self {
        Self::PasswordHash(value.to_string())
    }
}
