use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::env;

/// Application configuration structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Settings {
    pub app: AppConfig,
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub jwt: JwtConfig,
    pub cors: CorsConfig,
    pub security: SecurityConfig,
    pub email: EmailConfig,
    pub storage: StorageConfig,
    pub rate_limiting: RateLimitingConfig,
    pub logging: LoggingConfig,
    pub pagination: PaginationConfig,
    pub i18n: I18nConfig,
    pub feature_flags: FeatureFlagsConfig,
    pub cron: CronConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    pub name: String,
    pub version: String,
    pub environment: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
    pub shutdown_timeout: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout: u64,
    pub idle_timeout: u64,
    pub max_lifetime: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RedisConfig {
    pub max_connections: u32,
    pub connect_timeout: u64,
    pub idle_timeout: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JwtConfig {
    pub access_expiration: i64,
    pub refresh_expiration: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CorsConfig {
    pub max_age: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SecurityConfig {
    pub bcrypt_cost: u32,
    pub max_login_attempts: i32,
    pub lockout_duration: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EmailConfig {
    pub timeout: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StorageConfig {
    pub upload_max_size: usize,
    pub allowed_file_types: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RateLimitingConfig {
    pub enabled: bool,
    pub requests_per_minute: u32,
    pub burst_size: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PaginationConfig {
    pub default_page_size: u32,
    pub max_page_size: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct I18nConfig {
    pub default_locale: String,
    pub supported_locales: Vec<String>,
    pub fallback_locale: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FeatureFlagsConfig {
    pub registration_enabled: bool,
    pub email_verification: bool,
    pub password_reset: bool,
    pub swagger_ui: bool,
    pub metrics: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CronConfig {
    pub enabled: bool,
    pub timezone: String,
}

impl Settings {
    /// Load configuration from files and environment variables
    pub fn new() -> Result<Self, ConfigError> {
        let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());

        let mut builder = Config::builder()
            // Load default configuration
            .add_source(File::with_name("config/default").required(false))
            // Load environment-specific configuration
            .add_source(File::with_name(&format!("config/{}", environment)).required(false))
            // Load local configuration (for overrides)
            .add_source(File::with_name("config/local").required(false))
            // Load environment variables with APP_ prefix
            .add_source(Environment::with_prefix("APP").separator("__"));

        // Add environment-specific settings from env vars
        builder = builder
            .set_override_option("database.url", env::var("DATABASE_URL").ok())?
            .set_override_option("redis.url", env::var("REDIS_URL").ok())?
            .set_override_option("jwt.secret", env::var("JWT_SECRET").ok())?
            .set_override_option("server.host", env::var("HOST").ok())?
            .set_override_option("server.port", env::var("PORT").ok())?;

        builder.build()?.try_deserialize()
    }

    /// Create test configuration
    #[cfg(test)]
    pub fn new_for_tests() -> Self {
        Settings {
            app: AppConfig {
                name: "Test App".to_string(),
                version: "0.1.0-test".to_string(),
                environment: "test".to_string(),
            },
            server: ServerConfig {
                host: "localhost".to_string(),
                port: 8001,
                workers: 1,
                shutdown_timeout: 30,
            },
            database: DatabaseConfig {
                max_connections: 5,
                min_connections: 1,
                connect_timeout: 10,
                idle_timeout: 300,
                max_lifetime: 1800,
            },
            redis: RedisConfig {
                max_connections: 5,
                connect_timeout: 5,
                idle_timeout: 300,
            },
            jwt: JwtConfig {
                access_expiration: 300,
                refresh_expiration: 3600,
            },
            cors: CorsConfig { max_age: 3600 },
            security: SecurityConfig {
                bcrypt_cost: 8,
                max_login_attempts: 5,
                lockout_duration: 900,
            },
            email: EmailConfig { timeout: 30 },
            storage: StorageConfig {
                upload_max_size: 10485760,
                allowed_file_types: vec!["jpg".to_string(), "png".to_string()],
            },
            rate_limiting: RateLimitingConfig {
                enabled: false,
                requests_per_minute: 1000,
                burst_size: 1000,
            },
            logging: LoggingConfig {
                level: "debug".to_string(),
                format: "pretty".to_string(),
            },
            pagination: PaginationConfig {
                default_page_size: 10,
                max_page_size: 50,
            },
            i18n: I18nConfig {
                default_locale: "en".to_string(),
                supported_locales: vec!["en".to_string()],
                fallback_locale: "en".to_string(),
            },
            feature_flags: FeatureFlagsConfig {
                registration_enabled: true,
                email_verification: false,
                password_reset: true,
                swagger_ui: true,
                metrics: false,
            },
            cron: CronConfig {
                enabled: false,
                timezone: "UTC".to_string(),
            },
        }
    }
}
