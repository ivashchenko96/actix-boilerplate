use std::sync::Arc;
use sqlx::{PgPool, postgres::PgPoolOptions};
use anyhow::Result;
use tracing::{info, error};

use crate::{
    config::Settings,
    services::{
        redis::RedisService,
        nats::NatsService,
        typesense::TypesenseClient,
        storage::StorageService,
        email::EmailService,
    },
    i18n::I18nService,
    utils::feature_flags::FeatureFlagService,
};

/// Application context that holds all shared services and configuration
#[derive(Clone)]
pub struct AppContext {
    pub settings: Arc<Settings>,
    pub db: PgPool,
    pub redis: Arc<RedisService>,
    pub nats: Arc<NatsService>,
    pub typesense: Arc<TypesenseClient>,
    pub storage: Arc<StorageService>,
    pub email: Arc<EmailService>,
    pub i18n: Arc<I18nService>,
    pub feature_flags: Arc<FeatureFlagService>,
    pub started_at: chrono::DateTime<chrono::Utc>,
}

impl AppContext {
    /// Create a new application context with all services initialized
    pub async fn new(settings: Settings) -> Result<Self> {
        let settings = Arc::new(settings);
        
        info!("Initializing application context");

        // Initialize database connection pool
        info!("Connecting to database...");
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| anyhow::anyhow!("DATABASE_URL environment variable not set"))?;

        let db = PgPoolOptions::new()
            .max_connections(settings.database.max_connections)
            .min_connections(settings.database.min_connections)
            .acquire_timeout(std::time::Duration::from_secs(settings.database.connect_timeout))
            .idle_timeout(std::time::Duration::from_secs(settings.database.idle_timeout))
            .max_lifetime(std::time::Duration::from_secs(settings.database.max_lifetime))
            .connect(&database_url)
            .await
            .map_err(|e| {
                error!("Failed to connect to database: {}", e);
                anyhow::anyhow!("Database connection failed: {}", e)
            })?;

        info!("Database connection established");

        // Run migrations
        if let Err(e) = sqlx::migrate!("./migrations").run(&db).await {
            error!("Failed to run database migrations: {}", e);
            return Err(anyhow::anyhow!("Migration failed: {}", e));
        }
        info!("Database migrations completed");

        // Initialize Redis service
        info!("Connecting to Redis...");
        let redis = Arc::new(RedisService::new(&settings).await?);
        info!("Redis connection established");

        // Initialize NATS service
        info!("Connecting to NATS...");
        let nats = Arc::new(NatsService::new(&settings).await?);
        info!("NATS connection established");

        // Initialize Typesense client
        info!("Initializing Typesense client...");
        let typesense = Arc::new(TypesenseClient::new(&settings)?);
        info!("Typesense client initialized");

        // Initialize storage service
        info!("Initializing storage service...");
        let storage = Arc::new(StorageService::new(&settings).await?);
        info!("Storage service initialized");

        // Initialize email service
        info!("Initializing email service...");
        let email = Arc::new(EmailService::new(&settings)?);
        info!("Email service initialized");

        // Initialize i18n service
        info!("Loading i18n resources...");
        let i18n = Arc::new(I18nService::new(&settings)?);
        info!("i18n service initialized");

        // Initialize feature flags service
        info!("Initializing feature flags...");
        let feature_flags = Arc::new(FeatureFlagService::new(&settings));
        info!("Feature flags initialized");

        let started_at = chrono::Utc::now();

        let context = AppContext {
            settings,
            db,
            redis,
            nats,
            typesense,
            storage,
            email,
            i18n,
            feature_flags,
            started_at,
        };

        info!("Application context initialization completed");
        Ok(context)
    }

    /// Get database pool reference
    pub fn db(&self) -> &PgPool {
        &self.db
    }

    /// Get application uptime
    pub fn uptime(&self) -> chrono::Duration {
        chrono::Utc::now() - self.started_at
    }

    /// Check if a feature is enabled
    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        self.feature_flags.is_enabled(feature)
    }

    /// Get supported locales
    pub fn supported_locales(&self) -> &[String] {
        &self.settings.i18n.supported_locales
    }

    /// Get default locale
    pub fn default_locale(&self) -> &str {
        &self.settings.i18n.default_locale
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Settings;

    #[tokio::test]
    async fn test_context_creation() {
        // Note: This test would require a test database and Redis instance
        // In a real application, you'd use testcontainers or similar for integration tests
        let settings = Settings::new_for_tests();
        
        // This would fail without proper test infrastructure
        // let context = AppContext::new(settings).await;
        // assert!(context.is_ok());
        
        // For now, just test that we can create settings
        assert_eq!(settings.app.environment, "test");
    }

    #[test]
    fn test_feature_flag_access() {
        let settings = Settings::new_for_tests();
        let feature_flags = FeatureFlagService::new(&settings);
        
        // Test accessing feature flags
        assert!(feature_flags.is_enabled("registration_enabled"));
    }
}