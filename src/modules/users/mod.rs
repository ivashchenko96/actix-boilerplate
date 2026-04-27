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

pub struct UsersModule;

impl AppModule for UsersModule {
    fn name(&self) -> &'static str {
        "users"
    }

    fn register_routes(&self, cfg: &mut web::ServiceConfig) {
        cfg.service(
            web::scope("/users")
                .configure(routes::configure)
        );
    }

    fn register_jobs(&self, _registry: &mut CronRegistry) {
        // User-related cron jobs
    }

    fn register_permissions(&self, _registry: &mut PermissionRegistry) {
        // User permissions
    }

    fn register_openapi(&self, _registry: &mut OpenApiRegistry) {
        // OpenAPI specs for user endpoints
    }
}