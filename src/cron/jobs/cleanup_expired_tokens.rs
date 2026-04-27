use std::sync::Arc;

use crate::{context::AppContext, errors::AppResult};

/// Cleanup expired refresh tokens in database.
pub struct CleanupExpiredTokensJob;

impl CleanupExpiredTokensJob {
    /// Job name.
    pub fn name(&self) -> &'static str {
        "cleanup_expired_tokens"
    }

    /// Job schedule.
    pub fn schedule(&self) -> &'static str {
        "0 0 * * * *"
    }

    /// Execute cleanup routine.
    pub async fn run(&self, ctx: Arc<AppContext>) -> AppResult<u64> {
        let result = sqlx::query("DELETE FROM refresh_tokens WHERE expires_at < NOW()")
            .execute(ctx.db())
            .await?;
        Ok(result.rows_affected())
    }
}
