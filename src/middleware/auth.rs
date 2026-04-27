use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};
use std::rc::Rc;
use std::sync::Arc;
use uuid::Uuid;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};

use crate::{
    context::AppContext,
    errors::{AppError, AppResult},
};

/// JWT Claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,      // Subject (user ID)
    pub role: String,   // User role
    pub locale: String, // User locale
    pub jti: Uuid,      // JWT ID
    pub exp: i64,       // Expiration time
    pub iat: i64,       // Issued at time
}

/// User information extracted from JWT
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
    pub role: String,
    pub locale: String,
    pub jti: Uuid,
}

/// Authentication middleware
pub struct AuthMiddleware {
    context: Arc<AppContext>,
    required: bool,
}

impl AuthMiddleware {
    pub fn new(context: Arc<AppContext>) -> Self {
        Self {
            context,
            required: true,
        }
    }

    pub fn optional(context: Arc<AppContext>) -> Self {
        Self {
            context,
            required: false,
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService {
            service: Rc::new(service),
            context: Arc::clone(&self.context),
            required: self.required,
        }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
    context: Arc<AppContext>,
    required: bool,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);
        let context = Arc::clone(&self.context);
        let required = self.required;

        Box::pin(async move {
            // Try to extract token from Authorization header
            let auth_result = match extract_token_from_request(&req) {
                Ok(token) => validate_token(&token, &context).await,
                Err(err) => Err(err),
            };

            match auth_result {
                Ok(user) => {
                    // Add user to request extensions
                    req.extensions_mut().insert(user);
                }
                Err(e) if required => {
                    // If authentication is required and failed, return error
                    return Err(actix_web::error::ErrorUnauthorized(e));
                }
                Err(_) => {
                    // If authentication is optional and failed, continue without user
                }
            }

            service.call(req).await
        })
    }
}

/// Extract JWT token from Authorization header
fn extract_token_from_request(req: &ServiceRequest) -> AppResult<String> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .ok_or_else(|| AppError::authentication("Missing Authorization header"))?;

    let auth_str = auth_header
        .to_str()
        .map_err(|_| AppError::authentication("Invalid Authorization header format"))?;

    if !auth_str.starts_with("Bearer ") {
        return Err(AppError::authentication("Invalid Authorization header format"));
    }

    let token = auth_str
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::authentication("Invalid Authorization header format"))?;
    if token.is_empty() {
        return Err(AppError::authentication("Empty token"));
    }

    Ok(token.to_string())
}

/// Validate JWT token and extract user information
async fn validate_token(token: &str, context: &AppContext) -> AppResult<AuthUser> {
    // Get JWT secret from environment
    let jwt_secret = std::env::var("JWT_SECRET")
        .map_err(|_| AppError::internal("JWT_SECRET not configured"))?;

    // Decode and validate token
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )?;

    let claims = token_data.claims;

    // Check if token is blacklisted
    let is_blacklisted = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM blacklisted_tokens WHERE jti = $1)"
    )
    .bind(claims.jti)
    .fetch_one(context.db())
    .await
    .map_err(AppError::Database)?;

    if is_blacklisted {
        return Err(AppError::authentication("Token has been revoked"));
    }

    // Verify user still exists and is active
    let user_exists: bool = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM users WHERE id = $1 AND is_active = true)"
    )
    .bind(claims.sub)
    .fetch_one(context.db())
    .await
    .map_err(AppError::Database)?;

    if !user_exists {
        return Err(AppError::authentication("User not found or inactive"));
    }

    Ok(AuthUser {
        id: claims.sub,
        role: claims.role,
        locale: claims.locale,
        jti: claims.jti,
    })
}

/// Helper function to get authenticated user from request
pub fn get_auth_user(req: &ServiceRequest) -> Option<AuthUser> {
    req.extensions().get::<AuthUser>().cloned()
}

/// Helper function to require authenticated user from request
pub fn require_auth_user(req: &ServiceRequest) -> AppResult<AuthUser> {
    get_auth_user(req).ok_or_else(|| AppError::authentication("Authentication required"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_token_from_bearer_header() {
        // This would require setting up a mock ServiceRequest
        // For now, just test the Claims structure
        let claims = Claims {
            sub: Uuid::new_v4(),
            role: "user".to_string(),
            locale: "en".to_string(),
            jti: Uuid::new_v4(),
            exp: chrono::Utc::now().timestamp() + 3600,
            iat: chrono::Utc::now().timestamp(),
        };

        assert_eq!(claims.role, "user");
        assert_eq!(claims.locale, "en");
    }

    #[test]
    fn test_auth_user_creation() {
        let user = AuthUser {
            id: Uuid::new_v4(),
            role: "admin".to_string(),
            locale: "de".to_string(),
            jti: Uuid::new_v4(),
        };

        assert_eq!(user.role, "admin");
        assert_eq!(user.locale, "de");
    }
}
