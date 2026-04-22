use actix_web::{web, App, HttpServer, middleware as actix_middleware};
use actix_web_prom::PrometheusMetricsBuilder;
use std::sync::Arc;
use tracing::{info, warn};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    context::AppContext,
    middleware::{
        cors::cors_middleware,
        logger::logger_middleware,
        request_id::RequestIdMiddleware,
        security_headers::SecurityHeadersMiddleware,
        locale::LocaleMiddleware,
    },
    modules::{registry::ModuleRegistry, health::HealthModule, auth::AuthModule, users::UsersModule},
    cron::CronRegistry,
    openapi::ApiDoc,
};

/// Create and configure the Actix-Web application
pub async fn create_app(context: AppContext) -> std::io::Result<()> {
    let host = context.settings.server.host.clone();
    let port = context.settings.server.port;
    let workers = context.settings.server.workers;

    // Create metrics middleware if enabled
    let prometheus = if context.settings.feature_flags.metrics {
        Some(
            PrometheusMetricsBuilder::new("actix_boilerplate")
                .endpoint("/metrics")
                .build()
                .map_err(|e| {
                    std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to create Prometheus metrics: {}", e),
                    )
                })?,
        )
    } else {
        None
    };

    // Create module registry and register modules
    let mut module_registry = ModuleRegistry::new();
    module_registry.register(Box::new(HealthModule));
    module_registry.register(Box::new(AuthModule));
    module_registry.register(Box::new(UsersModule));

    // Create and start cron scheduler if enabled
    let cron_registry = if context.settings.cron.enabled {
        let mut cron_registry = CronRegistry::new();
        
        // Register cron jobs from modules
        for module in &module_registry.modules {
            module.register_jobs(&mut cron_registry);
        }

        if let Err(e) = cron_registry.start().await {
            warn!("Failed to start cron scheduler: {}", e);
        } else {
            info!("Cron scheduler started with {} jobs", cron_registry.job_count());
        }

        Some(cron_registry)
    } else {
        None
    };

    let context = Arc::new(context);
    let module_registry = Arc::new(module_registry);
    let cron_registry = Arc::new(cron_registry);

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
            .wrap(logger_middleware());

        // Add Prometheus metrics if enabled
        if let Some(prometheus) = prometheus {
            app = app.wrap(prometheus);
        }

        // Register module routes
        for module in &module_registry.modules {
            app = app.configure(|cfg| module.register_routes(cfg));
        }

        // Add Swagger UI if enabled
        if context.settings.feature_flags.swagger_ui {
            app = app.service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-doc/openapi.json", ApiDoc::openapi()),
            );
        }

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
        let settings = Settings::new_for_tests();
        let context = AppContext::new(settings).await.expect("Failed to create context");
        
        // This would create the app but not run it
        // In a real test, you'd use actix_web::test for integration testing
        assert!(!context.settings.app.name.is_empty());
    }
}