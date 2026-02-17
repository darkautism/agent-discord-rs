use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};
use uuid::Uuid;
use tracing::{info, error};
use std::collections::HashMap;
use tokio::sync::Mutex;
use std::path::PathBuf;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CronJobInfo {
    pub id: Uuid,
    pub channel_id: u64,
    pub cron_expr: String,
    pub prompt: String,
    pub creator_id: u64,
    pub description: String,
}

pub struct CronManager {
    scheduler: JobScheduler,
    jobs: Arc<Mutex<HashMap<Uuid, CronJobInfo>>>,
    config_dir: PathBuf,
}

impl CronManager {
    pub async fn new() -> anyhow::Result<Self> {
        let scheduler = JobScheduler::new().await?;
        scheduler.start().await?;
        
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("agent-discord-rs");
        std::fs::create_dir_all(&config_dir)?;

        Ok(Self {
            scheduler,
            jobs: Arc::new(Mutex::new(HashMap::new())),
            config_dir,
        })
    }

    pub async fn add_job(&self, info: CronJobInfo) -> anyhow::Result<Uuid> {
        let id = info.id;
        let cron_expr = info.cron_expr.clone();
        
        // Save to map
        {
            let mut jobs = self.jobs.lock().await;
            jobs.insert(id, info.clone());
        }

        // Logic to trigger the job will be added in Task 4
        // For now, just register with a placeholder
        let job = Job::new_async(cron_expr.as_str(), move |_uuid, _l| {
            Box::pin(async move {
                info!("Cron job {} triggered!", id);
            })
        })?;

        self.scheduler.add(job).await?;
        self.save_to_disk().await?;
        
        Ok(id)
    }

    async fn save_to_disk(&self) -> anyhow::Result<()> {
        let jobs = self.jobs.lock().await;
        let data = serde_json::to_string_pretty(&*jobs)?;
        let path = self.config_dir.join("cron_jobs.json");
        tokio::fs::write(path, data).await?;
        Ok(())
    }

    pub async fn load_from_disk(&self) -> anyhow::Result<()> {
        let path = self.config_dir.join("cron_jobs.json");
        if !path.exists() {
            return Ok(());
        }

        let data = tokio::fs::read_to_string(path).await?;
        let loaded_jobs: HashMap<Uuid, CronJobInfo> = serde_json::from_str(&data)?;
        
        for (_, info) in loaded_jobs {
            let _ = self.add_job(info).await;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_cron_persistence() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let manager = CronManager {
            scheduler: JobScheduler::new().await?,
            jobs: Arc::new(Mutex::new(HashMap::new())),
            config_dir: dir.path().to_path_buf(),
        };
        manager.scheduler.start().await?;

        let job_id = Uuid::new_v4();
        let info = CronJobInfo {
            id: job_id,
            channel_id: 12345,
            cron_expr: "0 0 * * * *".to_string(), // Every hour
            prompt: "Test Prompt".to_string(),
            creator_id: 67890,
            description: "Test Description".to_string(),
        };

        // Add job
        manager.add_job(info).await?;

        // Check if file exists
        let path = dir.path().join("cron_jobs.json");
        assert!(path.exists());

        // Create a new manager instance to load
        let manager2 = CronManager {
            scheduler: JobScheduler::new().await?,
            jobs: Arc::new(Mutex::new(HashMap::new())),
            config_dir: dir.path().to_path_buf(),
        };
        manager2.load_from_disk().await?;

        let jobs = manager2.jobs.lock().await;
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs.get(&job_id).unwrap().prompt, "Test Prompt");

        Ok(())
    }
}
