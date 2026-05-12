use std::sync::Arc;

use actix_web::{web, HttpResponse, Result};
use uuid::Uuid;
use validator::Validate;

use crate::{
    context::AppContext,
    errors::ApiResponse,
    modules::users::{dto::UsersQuery, repository::UsersRepository, service::UsersService},
};

/// List users endpoint handler.
pub async fn list_users(
    ctx: web::Data<Arc<AppContext>>,
    query: web::Query<UsersQuery>,
) -> Result<HttpResponse> {
    query
        .validate()
        .map_err(actix_web::error::ErrorBadRequest)?;
    let service = UsersService::new(UsersRepository::new(ctx.db.clone()));
    let data = service
        .list(query.into_inner())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(
        data,
        "Users fetched".to_string(),
        "en".to_string(),
        "users-list".to_string(),
    )))
}

/// Get user by id endpoint handler.
pub async fn get_user(
    ctx: web::Data<Arc<AppContext>>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let service = UsersService::new(UsersRepository::new(ctx.db.clone()));
    let data = service
        .get(id.into_inner())
        .await
        .map_err(actix_web::error::ErrorNotFound)?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(
        data,
        "User fetched".to_string(),
        "en".to_string(),
        "users-get".to_string(),
    )))
}

/// Soft delete user endpoint handler.
pub async fn delete_user(
    ctx: web::Data<Arc<AppContext>>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let service = UsersService::new(UsersRepository::new(ctx.db.clone()));
    service
        .delete(id.into_inner())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(ApiResponse::<()>::success(
        (),
        "User deleted".to_string(),
        "en".to_string(),
        "users-delete".to_string(),
    )))
}
