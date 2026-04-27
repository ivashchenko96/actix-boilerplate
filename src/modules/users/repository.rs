use sqlx::PgPool;
use uuid::Uuid;

use crate::{errors::AppResult, modules::users::models::UserRow};

/// Users data access layer.
#[derive(Clone)]
pub struct UsersRepository {
    pool: PgPool,
}

impl UsersRepository {
    /// Build repository.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// List users with simple pagination.
    pub async fn list(&self, page: u32, per_page: u32) -> AppResult<Vec<UserRow>> {
        let offset = (page.saturating_sub(1) * per_page) as i64;
        let limit = per_page as i64;
        sqlx::query_as::<_, UserRow>(
            r#"SELECT id, email, full_name, locale, is_active, created_at, updated_at
               FROM users WHERE deleted_at IS NULL
               ORDER BY created_at DESC OFFSET $1 LIMIT $2"#,
        )
        .bind(offset)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(Into::into)
    }

    /// Get user by id.
    pub async fn get(&self, id: Uuid) -> AppResult<Option<UserRow>> {
        sqlx::query_as::<_, UserRow>(
            r#"SELECT id, email, full_name, locale, is_active, created_at, updated_at
               FROM users WHERE id = $1 AND deleted_at IS NULL"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Into::into)
    }

    /// Soft delete user.
    pub async fn soft_delete(&self, id: Uuid) -> AppResult<()> {
        sqlx::query("UPDATE users SET deleted_at = NOW(), updated_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
