use actix_web::{middleware as actix_middleware, middleware::Condition, web, App, HttpServer};
use actix_web_prom::PrometheusMetricsBuilder;
use std::sync::Arc;
use tracing::{info, warn};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    context::AppContext,
    cron::CronRegistry,
    middleware::{
        cors::cors_middleware, locale::LocaleMiddleware, logger::logger_middleware,
        request_id::RequestIdMiddleware, security_headers::SecurityHeadersMiddleware,
    },
    modules::{
        auth::AuthModule, health::HealthModule, registry::ModuleRegistry, users::UsersModule,
    },
    openapi::ApiDoc,
};

/// Create and configure the Actix-Web application
pub async fn create_app(context: AppContext) -> std::io::Result<()> {
    let host = context.settings.server.host.clone();
    let port = context.settings.server.port;
    let workers = context.settings.server.workers;

    // Create metrics middleware if enabled
    let prometheus = PrometheusMetricsBuilder::new("actix_boilerplate")
        .endpoint("/metrics")
        .build()
        .map_err(|e| {
            std::io::Error::other(format!("Failed to create Prometheus metrics: {}", e))
        })?;

    // Create module registry and register modules
    let mut module_registry = ModuleRegistry::new();
    module_registry.register(Box::new(HealthModule));
    module_registry.register(Box::new(AuthModule));
    module_registry.register(Box::new(UsersModule));

    // Create and start cron scheduler if enabled
    let cron_registry = if context.settings.cron.enabled {
        let mut cron_registry = CronRegistry::new()
            .await
            .map_err(|e| std::io::Error::other(format!("Cron init failed: {}", e)))?;

        // Register cron jobs from modules
        for module in &module_registry.modules {
            module.register_jobs(&mut cron_registry);
        }

        if let Err(e) = cron_registry.start().await {
            warn!("Failed to start cron scheduler: {}", e);
        } else {
            info!(
                "Cron scheduler started with {} jobs",
                cron_registry.job_count()
            );
        }

        Some(cron_registry)
    } else {
        None
    };

    let context = Arc::new(context);
    let module_registry = Arc::new(module_registry);
    let _cron_registry = Arc::new(cron_registry);

    info!("Starting HTTP server on {}:{}", host, port);

    HttpServer::new(move || {
        let mut app = App::new()
            .app_data(web::Data::new(Arc::clone(&context)))
            .app_data(web::Data::new(Arc::clone(&module_registry)))
            .wrap(actix_middleware::NormalizePath::trim())
            .wrap(SecurityHeadersMiddleware::new(&context.settings))
            .wrap(cors_middleware(&context.settings))
            .wrap(RequestIdMiddleware)
            .wrap(LocaleMiddleware::new(Arc::clone(&context)))
            .wrap(logger_middleware())
            .wrap(Condition::new(
                context.settings.feature_flags.metrics,
                prometheus.clone(),
            ));

        // Register module routes
        for module in &module_registry.modules {
            app = app.configure(|cfg| module.register_routes(cfg));
        }

        app = app.service(
            SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-doc/openapi.json", ApiDoc::openapi()),
        );

        app
    })
    .workers(workers)
    .bind((host, port))?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Settings;

    #[actix_rt::test]
    async fn test_app_creation() {
        if std::env::var("RUN_INTEGRATION_TESTS").is_err() {
            return;
        }

        let settings = Settings::new_for_tests();
        let context = AppContext::new(settings)
            .await
            .expect("Failed to create context");

        // This would create the app but not run it
        // In a real test, you'd use actix_web::test for integration testing
        assert!(!context.settings.app.name.is_empty());
    }
}
