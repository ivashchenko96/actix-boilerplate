use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};
use std::rc::Rc;
use std::sync::Arc;
use sqlx::Row;

use crate::{
    context::AppContext,
    middleware::auth::{get_auth_user, AuthUser},
    errors::{AppError, AppResult},
};

/// Role-based access control middleware
pub struct RoleMiddleware {
    required_permissions: Vec<String>,
    context: Arc<AppContext>,
}

impl RoleMiddleware {
    /// Create middleware that requires specific permissions
    pub fn require_permissions(permissions: Vec<String>, context: Arc<AppContext>) -> Self {
        Self {
            required_permissions: permissions,
            context,
        }
    }

    /// Create middleware that requires any of the specified roles
    pub fn require_any_role(roles: Vec<String>, context: Arc<AppContext>) -> Self {
        // Convert roles to permissions (assuming role-based permissions)
        let permissions = roles.into_iter().map(|role| format!("role:{}", role)).collect();
        Self {
            required_permissions: permissions,
            context,
        }
    }

    /// Create middleware for admin-only access
    pub fn admin_only(context: Arc<AppContext>) -> Self {
        Self {
            required_permissions: vec!["role:admin".to_string()],
            context,
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RoleMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RoleMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RoleMiddlewareService {
            service: Rc::new(service),
            required_permissions: self.required_permissions.clone(),
            context: Arc::clone(&self.context),
        }))
    }
}

pub struct RoleMiddlewareService<S> {
    service: Rc<S>,
    required_permissions: Vec<String>,
    context: Arc<AppContext>,
}

impl<S, B> Service<ServiceRequest> for RoleMiddlewareService<S>
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
        let required_permissions = self.required_permissions.clone();
        let context = Arc::clone(&self.context);

        Box::pin(async move {
            // Get authenticated user
            let auth_user = get_auth_user(&req)
                .ok_or_else(|| AppError::authentication("Authentication required for role check"))?;

            // Check permissions
            let has_permission = check_user_permissions(&auth_user, &required_permissions, &context).await?;

            if !has_permission {
                return Err(actix_web::error::ErrorForbidden(
                    AppError::authorization("Insufficient permissions")
                ));
            }

            service.call(req).await
        })
    }
}

/// Check if user has required permissions
async fn check_user_permissions(
    user: &AuthUser,
    required_permissions: &[String],
    context: &AppContext,
) -> AppResult<bool> {
    // Special case: admin role has all permissions
    if user.role == "admin" {
        return Ok(true);
    }

    // Get user's permissions from database
    let user_permissions = get_user_permissions(user, context).await?;

    // Check if user has any of the required permissions
    for required_permission in required_permissions {
        if has_permission(&user_permissions, required_permission) {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Get user's permissions from database
async fn get_user_permissions(user: &AuthUser, context: &AppContext) -> AppResult<Vec<String>> {
    let mut permissions = Vec::new();

    // Get permissions from user roles
    let role_permissions = sqlx::query(
        r#"
        SELECT r.permissions
        FROM user_roles ur
        JOIN roles r ON ur.role_id = r.id
        WHERE ur.user_id = $1 
        AND (ur.expires_at IS NULL OR ur.expires_at > NOW())
        "#
    )
    .bind(user.id)
    .fetch_all(context.db())
    .await?;

    // Flatten all permissions from all roles
    for row in role_permissions {
        if let Ok(role_perms) = row.try_get::<Vec<String>, _>("permissions") {
            permissions.extend(role_perms);
        }
    }

    // Add implicit role permission
    permissions.push(format!("role:{}", user.role));

    Ok(permissions)
}

/// Check if a specific permission exists in the user's permission list
fn has_permission(user_permissions: &[String], required_permission: &str) -> bool {
    // Check for exact match
    if user_permissions.contains(&required_permission.to_string()) {
        return true;
    }

    // Check for wildcard permissions
    if user_permissions.contains(&"*".to_string()) {
        return true;
    }

    // Check for namespace wildcards (e.g., "users:*" allows "users:read", "users:write")
    if let Some(namespace) = required_permission.split(':').next() {
        let wildcard = format!("{}:*", namespace);
        if user_permissions.contains(&wildcard) {
            return true;
        }
    }

    false
}

/// Permission constants for common operations
pub mod permissions {
    // User permissions
    pub const USER_READ: &str = "users:read";
    pub const USER_WRITE: &str = "users:write";
    pub const USER_DELETE: &str = "users:delete";
    pub const USER_MODERATE: &str = "users:moderate";

    // Profile permissions
    pub const PROFILE_READ: &str = "profile:read";
    pub const PROFILE_WRITE: &str = "profile:write";

    // Admin permissions
    pub const ADMIN_ALL: &str = "*";
    
    // System permissions
    pub const SYSTEM_HEALTH: &str = "system:health";
    pub const SYSTEM_METRICS: &str = "system:metrics";
}

/// Role constants
pub mod roles {
    pub const ADMIN: &str = "admin";
    pub const USER: &str = "user";
    pub const MODERATOR: &str = "moderator";
}

/// Helper functions for creating role middleware
impl RoleMiddleware {
    /// Require user to read users
    pub fn require_user_read(context: Arc<AppContext>) -> Self {
        Self::require_permissions(vec![permissions::USER_READ.to_string()], context)
    }

    /// Require user to write users
    pub fn require_user_write(context: Arc<AppContext>) -> Self {
        Self::require_permissions(vec![permissions::USER_WRITE.to_string()], context)
    }

    /// Require user to delete users
    pub fn require_user_delete(context: Arc<AppContext>) -> Self {
        Self::require_permissions(vec![permissions::USER_DELETE.to_string()], context)
    }

    /// Require moderator or admin role
    pub fn require_moderator(context: Arc<AppContext>) -> Self {
        Self::require_any_role(vec![roles::MODERATOR.to_string(), roles::ADMIN.to_string()], context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_has_permission() {
        let permissions = vec![
            "users:read".to_string(),
            "users:write".to_string(),
            "profile:*".to_string(),
        ];

        // Test exact match
        assert!(has_permission(&permissions, "users:read"));
        assert!(has_permission(&permissions, "users:write"));

        // Test wildcard match
        assert!(has_permission(&permissions, "profile:read"));
        assert!(has_permission(&permissions, "profile:write"));
        assert!(has_permission(&permissions, "profile:delete"));

        // Test no match
        assert!(!has_permission(&permissions, "users:delete"));
        assert!(!has_permission(&permissions, "admin:read"));
    }

    #[test]
    fn test_wildcard_permission() {
        let permissions = vec!["*".to_string()];

        assert!(has_permission(&permissions, "users:read"));
        assert!(has_permission(&permissions, "admin:write"));
        assert!(has_permission(&permissions, "anything:anything"));
    }

    #[test]
    fn test_auth_user() {
        let user = AuthUser {
            id: Uuid::new_v4(),
            role: "user".to_string(),
            locale: "en".to_string(),
            jti: Uuid::new_v4(),
        };

        assert_eq!(user.role, "user");
        assert_eq!(user.locale, "en");
    }

    #[test]
    fn test_permission_constants() {
        assert_eq!(permissions::USER_READ, "users:read");
        assert_eq!(permissions::PROFILE_WRITE, "profile:write");
        assert_eq!(permissions::ADMIN_ALL, "*");
        assert_eq!(roles::ADMIN, "admin");
    }
}