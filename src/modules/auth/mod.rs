pub mod routes;
pub mod controller;
pub mod service;
pub mod repository;
pub mod models;
pub mod dto;
pub mod errors;

use actix_web::web;
use crate::modules::registry::{AppModule, PermissionRegistry, OpenApiRegistry};
use crate::cron::CronRegistry;

pub struct AuthModule;

impl AppModule for AuthModule {
    fn name(&self) -> &'static str {
        "auth"
    }

    fn register_routes(&self, cfg: &mut web::ServiceConfig) {
        cfg.service(
            web::scope("/auth")
                .configure(routes::configure)
        );
    }

    fn register_jobs(&self, _registry: &mut CronRegistry) {
        // Auth cleanup jobs would be registered here
    }

    fn register_permissions(&self, _registry: &mut PermissionRegistry) {
        // Auth permissions are handled separately
    }

    fn register_openapi(&self, _registry: &mut OpenApiRegistry) {
        // OpenAPI specs for auth endpoints
    }
}