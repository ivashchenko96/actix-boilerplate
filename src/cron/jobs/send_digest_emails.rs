use std::sync::Arc;

use crate::{context::AppContext, errors::AppResult};

/// Send daily digest emails.
pub struct SendDigestEmailsJob;

impl SendDigestEmailsJob {
    /// Job name.
    pub fn name(&self) -> &'static str {
        "send_digest_emails"
    }

    /// Job schedule.
    pub fn schedule(&self) -> &'static str {
        "0 0 8 * * *"
    }

    /// Execute digest routine.
    pub async fn run(&self, _ctx: Arc<AppContext>) -> AppResult<()> {
        Ok(())
    }
}
