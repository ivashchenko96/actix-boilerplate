use actix_web::{web, HttpResponse, Result};
use std::sync::Arc;

use crate::context::AppContext;
use super::controller;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(health_check))
        .route("/db", web::get().to(database_check))
        .route("/redis", web::get().to(redis_check));
}

/// Health check endpoint
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Health check passed"),
        (status = 500, description = "Health check failed")
    )
)]
pub async fn health_check(
    ctx: web::Data<Arc<AppContext>>,
) -> Result<HttpResponse> {
    controller::health_check(ctx).await
}

/// Database health check
#[utoipa::path(
    get,
    path = "/health/db",
    responses(
        (status = 200, description = "Database is healthy"),
        (status = 500, description = "Database is unhealthy")
    )
)]
pub async fn database_check(
    ctx: web::Data<Arc<AppContext>>,
) -> Result<HttpResponse> {
    controller::database_check(ctx).await
}

/// Redis health check
#[utoipa::path(
    get,
    path = "/health/redis",
    responses(
        (status = 200, description = "Redis is healthy"),
        (status = 500, description = "Redis is unhealthy")
    )
)]
pub async fn redis_check(
    ctx: web::Data<Arc<AppContext>>,
) -> Result<HttpResponse> {
    controller::redis_check(ctx).await
}