use actix_web::{web, HttpResponse, Result};
use serde::Serialize;
use std::sync::Arc;

use crate::{context::AppContext, errors::ApiResponse};

#[derive(Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub uptime: i64,
    pub version: String,
    pub services: ServiceStatuses,
}

#[derive(Serialize)]
pub struct ServiceStatuses {
    pub database: String,
    pub redis: String,
    pub nats: String,
}

pub async fn health_check(ctx: web::Data<Arc<AppContext>>) -> Result<HttpResponse> {
    let request_id = "health-check".to_string(); // Simple for health checks
    let locale = "en".to_string();

    // Check all services
    let db_status = check_database_health(&ctx).await;
    let redis_status = check_redis_health(&ctx).await;
    let nats_status = check_nats_health(&ctx).await;

    let overall_status =
        if db_status == "healthy" && redis_status == "healthy" && nats_status == "healthy" {
            "healthy"
        } else {
            "unhealthy"
        };

    let health_status = HealthStatus {
        status: overall_status.to_string(),
        timestamp: chrono::Utc::now(),
        uptime: ctx.uptime().num_seconds(),
        version: ctx.settings.app.version.clone(),
        services: ServiceStatuses {
            database: db_status,
            redis: redis_status,
            nats: nats_status,
        },
    };

    let response = ApiResponse::success(
        health_status,
        "Health check completed".to_string(),
        locale,
        request_id,
    );

    if overall_status == "healthy" {
        Ok(HttpResponse::Ok().json(response))
    } else {
        Ok(HttpResponse::ServiceUnavailable().json(response))
    }
}

pub async fn database_check(ctx: web::Data<Arc<AppContext>>) -> Result<HttpResponse> {
    let request_id = "db-health-check".to_string();
    let locale = "en".to_string();

    let status = check_database_health(&ctx).await;

    let response = ApiResponse::success(
        serde_json::json!({ "status": status }),
        "Database health check completed".to_string(),
        locale,
        request_id,
    );

    if status == "healthy" {
        Ok(HttpResponse::Ok().json(response))
    } else {
        Ok(HttpResponse::ServiceUnavailable().json(response))
    }
}

pub async fn redis_check(ctx: web::Data<Arc<AppContext>>) -> Result<HttpResponse> {
    let request_id = "redis-health-check".to_string();
    let locale = "en".to_string();

    let status = check_redis_health(&ctx).await;

    let response = ApiResponse::success(
        serde_json::json!({ "status": status }),
        "Redis health check completed".to_string(),
        locale,
        request_id,
    );

    if status == "healthy" {
        Ok(HttpResponse::Ok().json(response))
    } else {
        Ok(HttpResponse::ServiceUnavailable().json(response))
    }
}

async fn check_database_health(ctx: &AppContext) -> String {
    match sqlx::query("SELECT 1").fetch_one(ctx.db()).await {
        Ok(_) => "healthy".to_string(),
        Err(_) => "unhealthy".to_string(),
    }
}

async fn check_redis_health(ctx: &AppContext) -> String {
    match ctx.redis.ping().await {
        Ok(_) => "healthy".to_string(),
        Err(_) => "unhealthy".to_string(),
    }
}

async fn check_nats_health(ctx: &AppContext) -> String {
    if ctx.nats.client().connection_state() == async_nats::connection::State::Connected {
        "healthy".to_string()
    } else {
        "unhealthy".to_string()
    }
}
