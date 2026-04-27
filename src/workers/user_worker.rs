use anyhow::Result;
use tracing::info;

use crate::events::user_events::UserCreatedEvent;

/// Background worker for user events.
pub struct UserWorker;

impl UserWorker {
    /// Process user created events.
    pub async fn handle_user_created(event: UserCreatedEvent) -> Result<()> {
        info!(user_id = %event.user_id, "processed user created event");
        Ok(())
    }
}
