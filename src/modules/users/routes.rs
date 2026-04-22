use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(|| async { "Users module - implement endpoints" }));
}
