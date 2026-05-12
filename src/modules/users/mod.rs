pub mod controller;
pub mod dto;
pub mod errors;
pub mod models;
pub mod repository;
pub mod routes;
pub mod service;

use crate::cron::CronRegistry;
use crate::modules::registry::{AppModule, OpenApiRegistry, PermissionRegistry};
use actix_web::web;

pub struct UsersModule;

impl AppModule for UsersModule {
    fn name(&self) -> &'static str {
        "users"
    }

    fn register_routes(&self, cfg: &mut web::ServiceConfig) {
        cfg.service(web::scope("/users").configure(routes::configure));
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
