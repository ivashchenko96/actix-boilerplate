use uuid::Uuid;

use crate::{
    errors::AppResult,
    modules::users::{
        dto::{UserResponse, UsersQuery},
        errors::UsersError,
        repository::UsersRepository,
    },
};

/// Users business logic.
#[derive(Clone)]
pub struct UsersService {
    repository: UsersRepository,
}

impl UsersService {
    /// Build users service.
    pub fn new(repository: UsersRepository) -> Self {
        Self { repository }
    }

    /// List users.
    pub async fn list(&self, query: UsersQuery) -> AppResult<Vec<UserResponse>> {
        let page = query.page.unwrap_or(1);
        let per_page = query.per_page.unwrap_or(20);
        let rows = self.repository.list(page, per_page).await?;
        Ok(rows
            .into_iter()
            .map(|row| UserResponse {
                id: row.id.to_string(),
                email: row.email,
                full_name: row.full_name,
                locale: row.locale,
                is_active: row.is_active,
            })
            .collect())
    }

    /// Get user.
    pub async fn get(&self, id: Uuid) -> AppResult<UserResponse> {
        let row = self.repository.get(id).await?.ok_or(UsersError::NotFound)?;
        Ok(UserResponse {
            id: row.id.to_string(),
            email: row.email,
            full_name: row.full_name,
            locale: row.locale,
            is_active: row.is_active,
        })
    }

    /// Soft delete user.
    pub async fn delete(&self, id: Uuid) -> AppResult<()> {
        self.repository.soft_delete(id).await
    }
}
