use actix_web::web;
use super::controller;

/// Register auth routes.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/login", web::post().to(controller::login))
        .route("/logout", web::post().to(controller::logout))
        .route("/register", web::post().to(controller::register))
        .route("/refresh", web::post().to(controller::refresh_token));
}
