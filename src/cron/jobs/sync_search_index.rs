use std::sync::Arc;

use crate::{context::AppContext, errors::AppResult};

/// Sync users index into Typesense.
pub struct SyncSearchIndexJob;

impl SyncSearchIndexJob {
    /// Job name.
    pub fn name(&self) -> &'static str {
        "sync_search_index"
    }

    /// Job schedule.
    pub fn schedule(&self) -> &'static str {
        "0 0 2 * * *"
    }

    /// Execute sync routine.
    pub async fn run(&self, _ctx: Arc<AppContext>) -> AppResult<()> {
        Ok(())
    }
}
