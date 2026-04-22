use actix_web::{web, HttpResponse, Result};
use super::controller;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/login", web::post().to(login))
       .route("/logout", web::post().to(logout))
       .route("/register", web::post().to(register))
       .route("/refresh", web::post().to(refresh_token));
}

async fn login() -> Result<HttpResponse> {
    controller::login().await
}

async fn logout() -> Result<HttpResponse> {
    controller::logout().await
}

async fn register() -> Result<HttpResponse> {
    controller::register().await
}

async fn refresh_token() -> Result<HttpResponse> {
    controller::refresh_token().await
}