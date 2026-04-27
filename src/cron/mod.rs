pub mod jobs;

use tokio_cron_scheduler::{JobScheduler, Job};
use anyhow::Result;

/// Cron registry for managing scheduled jobs
pub struct CronRegistry {
    scheduler: JobScheduler,
    job_count: usize,
}

impl CronRegistry {
    pub async fn new() -> Result<Self> {
        let scheduler = JobScheduler::new().await?;
        Ok(Self {
            scheduler,
            job_count: 0,
        })
    }

    /// Start the cron scheduler
    pub async fn start(&mut self) -> Result<()> {
        self.scheduler.start().await?;
        Ok(())
    }

    /// Add a job to the scheduler
    pub async fn add_job(&mut self, job: Job) -> Result<uuid::Uuid> {
        let job_id = self.scheduler.add(job).await?;
        self.job_count += 1;
        Ok(job_id)
    }

    /// Remove a job from the scheduler
    pub async fn remove_job(&mut self, job_id: &uuid::Uuid) -> Result<()> {
        self.scheduler.remove(job_id).await?;
        self.job_count = self.job_count.saturating_sub(1);
        Ok(())
    }

    /// Get the number of registered jobs
    pub fn job_count(&self) -> usize {
        self.job_count
    }

    /// Shutdown the scheduler
    pub async fn shutdown(&mut self) -> Result<()> {
        self.scheduler.shutdown().await?;
        Ok(())
    }
}
