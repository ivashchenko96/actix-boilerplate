pub mod routes;
pub mod controller;

use actix_web::web;

use crate::{
    modules::registry::{AppModule, PermissionRegistry, OpenApiRegistry},
    cron::CronRegistry,
};

pub struct HealthModule;

impl AppModule for HealthModule {
    fn name(&self) -> &'static str {
        "health"
    }

    fn register_routes(&self, cfg: &mut web::ServiceConfig) {
        cfg.service(
            web::scope("/health")
                .configure(routes::configure)
        );
    }

    fn register_jobs(&self, _registry: &mut CronRegistry) {
        // Health module doesn't need cron jobs
    }

    fn register_permissions(&self, _registry: &mut PermissionRegistry) {
        // Health endpoints are public
    }

    fn register_openapi(&self, _registry: &mut OpenApiRegistry) {
        // OpenAPI specs would be registered here
    }
}