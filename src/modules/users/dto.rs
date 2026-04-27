use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

/// Pagination query for users endpoint.
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct UsersQuery {
    #[validate(range(min = 1, max = 10000))]
    pub page: Option<u32>,
    #[validate(range(min = 1, max = 200))]
    pub per_page: Option<u32>,
}

/// User create request body.
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct CreateUserRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8, max = 128))]
    pub password: String,
    #[validate(length(min = 1, max = 100))]
    pub full_name: Option<String>,
    #[validate(length(min = 2, max = 10))]
    pub locale: Option<String>,
}

/// User update request body.
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct UpdateUserRequest {
    #[validate(length(min = 1, max = 100))]
    pub full_name: Option<String>,
    pub is_active: Option<bool>,
    #[validate(length(min = 2, max = 10))]
    pub locale: Option<String>,
}

/// Public API user object.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub full_name: Option<String>,
    pub locale: String,
    pub is_active: bool,
}
