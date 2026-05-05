use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::modules::health::routes::health_check,
        crate::modules::health::routes::database_check,
        crate::modules::health::routes::redis_check,
    ),
    info(
        title = "Actix Boilerplate API",
        version = "0.1.0",
        description = "Production-grade enterprise Actix-Web boilerplate API",
    ),
    servers(
        (url = "http://localhost:8000", description = "Development server"),
        (url = "https://api.yourapp.com", description = "Production server")
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "auth", description = "Authentication endpoints"),
        (name = "users", description = "User management endpoints")
    )
)]
pub struct ApiDoc;
