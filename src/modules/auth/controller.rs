use std::sync::Arc;

use actix_web::{web, HttpResponse, Result};
use validator::Validate;

use crate::{
    context::AppContext,
    errors::ApiResponse,
    modules::auth::{
        dto::{LoginRequest, RefreshRequest, RegisterRequest},
        repository::AuthRepository,
        service::AuthService,
    },
};

/// Login endpoint handler.
pub async fn login(
    ctx: web::Data<Arc<AppContext>>,
    payload: web::Json<LoginRequest>,
) -> Result<HttpResponse> {
    payload.validate().map_err(actix_web::error::ErrorBadRequest)?;
    let service = AuthService::new(AuthRepository::new(ctx.db.clone()));
    let data = service
        .login(payload.into_inner())
        .await
        .map_err(actix_web::error::ErrorUnauthorized)?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(
        data,
        "Login successful".to_string(),
        "en".to_string(),
        "auth-login".to_string(),
    )))
}

/// Logout endpoint handler.
pub async fn logout() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(ApiResponse::<()>::success(
        (),
        "Logout successful".to_string(),
        "en".to_string(),
        "auth-logout".to_string(),
    )))
}

/// Register endpoint handler.
pub async fn register(
    ctx: web::Data<Arc<AppContext>>,
    payload: web::Json<RegisterRequest>,
) -> Result<HttpResponse> {
    payload.validate().map_err(actix_web::error::ErrorBadRequest)?;
    let service = AuthService::new(AuthRepository::new(ctx.db.clone()));
    let data = service
        .register(payload.into_inner())
        .await
        .map_err(actix_web::error::ErrorBadRequest)?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(
        data,
        "Registration successful".to_string(),
        "en".to_string(),
        "auth-register".to_string(),
    )))
}

/// Refresh token endpoint handler.
pub async fn refresh_token(payload: web::Json<RefreshRequest>) -> Result<HttpResponse> {
    payload.validate().map_err(actix_web::error::ErrorBadRequest)?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(
        serde_json::json!({ "refresh_token": payload.refresh_token }),
        "Refresh accepted".to_string(),
        "en".to_string(),
        "auth-refresh".to_string(),
    )))
}
