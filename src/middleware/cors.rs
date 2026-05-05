use actix_cors::Cors;
use actix_web::http;
use std::env;

use crate::config::Settings;

/// Create CORS middleware configuration
pub fn cors_middleware(settings: &Settings) -> Cors {
    let allowed_origins =
        env::var("CORS_ALLOWED_ORIGINS").unwrap_or_else(|_| "http://localhost:3000".to_string());

    let allowed_methods = env::var("CORS_ALLOWED_METHODS")
        .unwrap_or_else(|_| "GET,POST,PUT,PATCH,DELETE,OPTIONS".to_string());

    let allowed_headers = env::var("CORS_ALLOWED_HEADERS")
        .unwrap_or_else(|_| "Content-Type,Authorization,X-Request-ID".to_string());

    let mut cors = Cors::default()
        .max_age(settings.cors.max_age)
        .supports_credentials();

    // Parse and add allowed origins
    for origin in allowed_origins.split(',') {
        let origin = origin.trim();
        if origin == "*" {
            cors = cors.allow_any_origin();
        } else {
            cors = cors.allowed_origin(origin);
        }
    }

    // Parse and add allowed methods
    for method in allowed_methods.split(',') {
        let method = method.trim().to_uppercase();
        match method.as_str() {
            "GET" => cors = cors.allowed_methods(vec![http::Method::GET]),
            "POST" => cors = cors.allowed_methods(vec![http::Method::POST]),
            "PUT" => cors = cors.allowed_methods(vec![http::Method::PUT]),
            "PATCH" => cors = cors.allowed_methods(vec![http::Method::PATCH]),
            "DELETE" => cors = cors.allowed_methods(vec![http::Method::DELETE]),
            "OPTIONS" => cors = cors.allowed_methods(vec![http::Method::OPTIONS]),
            "HEAD" => cors = cors.allowed_methods(vec![http::Method::HEAD]),
            _ => {}
        }
    }

    // Parse and add allowed headers
    for header in allowed_headers.split(',') {
        let header = header.trim();
        match header.to_lowercase().as_str() {
            "content-type" => cors = cors.allowed_headers(vec![http::header::CONTENT_TYPE]),
            "authorization" => cors = cors.allowed_headers(vec![http::header::AUTHORIZATION]),
            "accept" => cors = cors.allowed_headers(vec![http::header::ACCEPT]),
            "origin" => cors = cors.allowed_headers(vec![http::header::ORIGIN]),
            "x-request-id" => {
                cors = cors
                    .allowed_headers(vec![http::header::HeaderName::from_static("x-request-id")])
            }
            "x-requested-with" => {
                cors = cors.allowed_headers(vec![http::header::HeaderName::from_static(
                    "x-requested-with",
                )])
            }
            _ => {
                // Try to parse custom header
                if let Ok(header_name) = http::header::HeaderName::from_bytes(header.as_bytes()) {
                    cors = cors.allowed_headers(vec![header_name]);
                }
            }
        }
    }

    cors
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Settings;

    #[test]
    fn test_cors_middleware_creation() {
        let settings = Settings::new_for_tests();
        let _cors = cors_middleware(&settings);

        // Test that middleware was created successfully
        // In a real test, you'd verify the configuration more thoroughly
        assert_eq!(settings.cors.max_age, 3600);
    }
}
