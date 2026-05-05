use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};
use std::rc::Rc;

use crate::config::Settings;

/// Security headers middleware
pub struct SecurityHeadersMiddleware {
    csp_policy: String,
    hsts_max_age: String,
}

impl SecurityHeadersMiddleware {
    pub fn new(_settings: &Settings) -> Self {
        let csp_policy = std::env::var("CSP_POLICY")
            .unwrap_or_else(|_| "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'".to_string());

        let hsts_max_age = std::env::var("HSTS_MAX_AGE").unwrap_or_else(|_| "31536000".to_string());

        Self {
            csp_policy,
            hsts_max_age,
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for SecurityHeadersMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = SecurityHeadersMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SecurityHeadersMiddlewareService {
            service: Rc::new(service),
            csp_policy: self.csp_policy.clone(),
            hsts_max_age: self.hsts_max_age.clone(),
        }))
    }
}

pub struct SecurityHeadersMiddlewareService<S> {
    service: Rc<S>,
    csp_policy: String,
    hsts_max_age: String,
}

impl<S, B> Service<ServiceRequest> for SecurityHeadersMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);
        let csp_policy = self.csp_policy.clone();
        let hsts_max_age = self.hsts_max_age.clone();

        Box::pin(async move {
            let mut res = service.call(req).await?;

            // Add security headers
            let headers = res.headers_mut();

            // Content Security Policy
            if let Ok(header_value) = actix_web::http::header::HeaderValue::from_str(&csp_policy) {
                headers.insert(
                    actix_web::http::header::HeaderName::from_static("content-security-policy"),
                    header_value,
                );
            }

            // HTTP Strict Transport Security (HSTS)
            if let Ok(header_value) =
                actix_web::http::header::HeaderValue::from_str(&format!("max-age={}", hsts_max_age))
            {
                headers.insert(
                    actix_web::http::header::HeaderName::from_static("strict-transport-security"),
                    header_value,
                );
            }

            // X-Content-Type-Options
            headers.insert(
                actix_web::http::header::HeaderName::from_static("x-content-type-options"),
                actix_web::http::header::HeaderValue::from_static("nosniff"),
            );

            // X-Frame-Options
            headers.insert(
                actix_web::http::header::HeaderName::from_static("x-frame-options"),
                actix_web::http::header::HeaderValue::from_static("DENY"),
            );

            // X-XSS-Protection
            headers.insert(
                actix_web::http::header::HeaderName::from_static("x-xss-protection"),
                actix_web::http::header::HeaderValue::from_static("1; mode=block"),
            );

            // Referrer Policy
            headers.insert(
                actix_web::http::header::HeaderName::from_static("referrer-policy"),
                actix_web::http::header::HeaderValue::from_static(
                    "strict-origin-when-cross-origin",
                ),
            );

            // Permissions Policy (Feature Policy)
            headers.insert(
                actix_web::http::header::HeaderName::from_static("permissions-policy"),
                actix_web::http::header::HeaderValue::from_static(
                    "accelerometer=(), camera=(), geolocation=(), gyroscope=(), magnetometer=(), microphone=(), payment=(), usb=()"
                ),
            );

            // Cross-Origin Opener Policy
            headers.insert(
                actix_web::http::header::HeaderName::from_static("cross-origin-opener-policy"),
                actix_web::http::header::HeaderValue::from_static("same-origin"),
            );

            // Cross-Origin Embedder Policy
            headers.insert(
                actix_web::http::header::HeaderName::from_static("cross-origin-embedder-policy"),
                actix_web::http::header::HeaderValue::from_static("require-corp"),
            );

            // Cross-Origin Resource Policy
            headers.insert(
                actix_web::http::header::HeaderName::from_static("cross-origin-resource-policy"),
                actix_web::http::header::HeaderValue::from_static("cross-origin"),
            );

            // Remove potentially sensitive headers
            headers.remove("server");
            headers.remove("x-powered-by");

            Ok(res)
        })
    }
}

/// Additional security utilities
pub struct SecurityUtils;

impl SecurityUtils {
    /// Generate a secure nonce for CSP
    pub fn generate_nonce() -> String {
        use base64::Engine;
        use rand::RngCore;

        let mut nonce_bytes = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        base64::engine::general_purpose::STANDARD.encode(nonce_bytes)
    }

    /// Build CSP with nonce
    pub fn build_csp_with_nonce(nonce: &str) -> String {
        format!(
            "default-src 'self'; script-src 'self' 'nonce-{}'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self' https:; connect-src 'self'; frame-ancestors 'none';",
            nonce
        )
    }

    /// Validate Content-Security-Policy header value
    pub fn is_valid_csp(csp: &str) -> bool {
        // Basic validation - check for common CSP directives
        let valid_directives = [
            "default-src",
            "script-src",
            "style-src",
            "img-src",
            "font-src",
            "connect-src",
            "frame-src",
            "frame-ancestors",
            "object-src",
            "media-src",
            "worker-src",
            "manifest-src",
            "base-uri",
            "form-action",
        ];

        // Check if CSP contains at least one valid directive
        valid_directives
            .iter()
            .any(|directive| csp.contains(directive))
    }

    /// Get recommended security headers for different environments
    pub fn get_production_headers() -> Vec<(&'static str, &'static str)> {
        vec![
            ("Strict-Transport-Security", "max-age=31536000; includeSubDomains; preload"),
            ("X-Content-Type-Options", "nosniff"),
            ("X-Frame-Options", "DENY"),
            ("X-XSS-Protection", "1; mode=block"),
            ("Referrer-Policy", "strict-origin-when-cross-origin"),
            ("Content-Security-Policy", "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self' https:; connect-src 'self'; frame-ancestors 'none'; object-src 'none';"),
        ]
    }

    pub fn get_development_headers() -> Vec<(&'static str, &'static str)> {
        vec![
            ("X-Content-Type-Options", "nosniff"),
            ("X-Frame-Options", "SAMEORIGIN"),
            ("Referrer-Policy", "strict-origin-when-cross-origin"),
            ("Content-Security-Policy", "default-src 'self' 'unsafe-inline' 'unsafe-eval'; img-src 'self' data: https:; font-src 'self' https:"),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App, HttpResponse};

    async fn test_handler() -> HttpResponse {
        HttpResponse::Ok().body("test")
    }

    #[actix_web::test]
    async fn test_security_headers_middleware() {
        let settings = crate::config::Settings::new_for_tests();
        let middleware = SecurityHeadersMiddleware::new(&settings);

        let app = test::init_service(
            App::new()
                .wrap(middleware)
                .route("/", web::get().to(test_handler)),
        )
        .await;

        let req = test::TestRequest::get().uri("/").to_request();
        let resp = test::call_service(&app, req).await;

        // Check that security headers are present
        assert!(resp.headers().contains_key("content-security-policy"));
        assert!(resp.headers().contains_key("x-content-type-options"));
        assert!(resp.headers().contains_key("x-frame-options"));
        assert!(resp.headers().contains_key("referrer-policy"));

        // Check that sensitive headers are removed
        assert!(!resp.headers().contains_key("server"));
        assert!(!resp.headers().contains_key("x-powered-by"));
    }

    #[actix_web::test]
    async fn test_security_utils() {
        // Test nonce generation
        let nonce = SecurityUtils::generate_nonce();
        assert!(!nonce.is_empty());
        assert_eq!(nonce.len(), 24); // Base64 encoded 16 bytes = 24 characters

        // Test CSP with nonce
        let csp = SecurityUtils::build_csp_with_nonce(&nonce);
        assert!(csp.contains(&format!("'nonce-{}'", nonce)));

        // Test CSP validation
        assert!(SecurityUtils::is_valid_csp("default-src 'self'"));
        assert!(SecurityUtils::is_valid_csp(
            "script-src 'self' 'unsafe-inline'"
        ));
        assert!(!SecurityUtils::is_valid_csp("invalid csp policy"));
    }

    #[actix_web::test]
    async fn test_recommended_headers() {
        let prod_headers = SecurityUtils::get_production_headers();
        assert!(!prod_headers.is_empty());
        assert!(prod_headers
            .iter()
            .any(|(name, _)| *name == "Strict-Transport-Security"));

        let dev_headers = SecurityUtils::get_development_headers();
        assert!(!dev_headers.is_empty());
        assert!(dev_headers
            .iter()
            .any(|(name, _)| *name == "Content-Security-Policy"));
    }
}
