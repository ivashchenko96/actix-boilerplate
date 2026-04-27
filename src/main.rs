use dotenvy::dotenv;
use std::env;
use tracing::{error, info};

mod app;
mod config;
mod context;
mod db;
mod errors;
mod middleware;
mod modules;
mod services;
mod events;
mod workers;
mod cron;
mod i18n;
mod openapi;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    let _ = dotenv();

    // Initialize logging
    init_logging();

    // Load configuration
    let settings = match config::Settings::new() {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };

    info!("Starting {} v{}", settings.app.name, settings.app.version);
    info!("Environment: {}", settings.app.environment);

    // Create application context
    let context = match context::AppContext::new(settings).await {
        Ok(ctx) => ctx,
        Err(e) => {
            error!("Failed to initialize application context: {}", e);
            std::process::exit(1);
        }
    };

    info!("Application context initialized successfully");

    // Start the application
    match app::create_app(context).await {
        Ok(_) => {
            info!("Application started successfully");
            Ok(())
        }
        Err(e) => {
            error!("Failed to start application: {}", e);
            Err(e)
        }
    }
}

fn init_logging() {
    let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
    let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());

    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(log_level)),
        );

    if environment == "production" {
        subscriber.json().init();
    } else {
        subscriber.pretty().init();
    }

    info!("Logging initialized for {} environment", environment);
}