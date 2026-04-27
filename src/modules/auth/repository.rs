use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{errors::AppResult, modules::auth::models::AuthUserRow};

/// Database access layer for auth module.
#[derive(Clone)]
pub struct AuthRepository {
    pool: PgPool,
}

impl AuthRepository {
    /// Build new auth repository.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Find user by email.
    pub async fn find_user_by_email(&self, email: &str) -> AppResult<Option<AuthUserRow>> {
        sqlx::query_as::<_, AuthUserRow>(
            r#"SELECT id, email, password_hash, full_name, locale, is_active, created_at
               FROM users WHERE email = $1 AND deleted_at IS NULL"#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(Into::into)
    }

    /// Create new user record.
    pub async fn create_user(
        &self,
        email: &str,
        password_hash: &str,
        full_name: Option<&str>,
        locale: &str,
    ) -> AppResult<AuthUserRow> {
        let id = Uuid::new_v4();
        sqlx::query_as::<_, AuthUserRow>(
            r#"INSERT INTO users (id, email, password_hash, full_name, locale, is_active, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, true, NOW(), NOW())
               RETURNING id, email, password_hash, full_name, locale, is_active, created_at"#,
        )
        .bind(id)
        .bind(email)
        .bind(password_hash)
        .bind(full_name)
        .bind(locale)
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }

    /// Persist refresh token hash.
    pub async fn store_refresh_token(&self, user_id: Uuid, token_hash: &str) -> AppResult<Uuid> {
        let jti = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::days(30);
        sqlx::query(
            r#"INSERT INTO refresh_tokens (jti, user_id, token_hash, expires_at, revoked, created_at)
               VALUES ($1, $2, $3, $4, false, NOW())"#,
        )
        .bind(jti)
        .bind(user_id)
        .bind(token_hash)
        .bind(expires_at)
        .execute(&self.pool)
        .await?;
        Ok(jti)
    }

    /// Revoke refresh token by jti.
    pub async fn revoke_refresh_token(&self, jti: Uuid) -> AppResult<()> {
        sqlx::query("UPDATE refresh_tokens SET revoked = true, revoked_at = NOW() WHERE jti = $1")
            .bind(jti)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
