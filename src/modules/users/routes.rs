use actix_web::web;

use super::controller;

/// Register users module routes.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(controller::list_users))
        .route("/{id}", web::get().to(controller::get_user))
        .route("/{id}", web::delete().to(controller::delete_user));
}
