use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};
use std::rc::Rc;
use uuid::Uuid;

/// Request ID middleware to add unique request IDs to all requests
pub struct RequestIdMiddleware;

impl<S, B> Transform<S, ServiceRequest> for RequestIdMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RequestIdMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestIdMiddlewareService {
            service: Rc::new(service),
        }))
    }
}

pub struct RequestIdMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for RequestIdMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);

        Box::pin(async move {
            // Check if request already has a request ID
            let request_id = req
                .headers()
                .get("X-Request-ID")
                .and_then(|h| h.to_str().ok())
                .map(|id| {
                    // Validate existing request ID (should be a valid UUID)
                    if Uuid::parse_str(id).is_ok() {
                        id.to_string()
                    } else {
                        // Generate new ID if existing one is invalid
                        Uuid::new_v4().to_string()
                    }
                })
                .unwrap_or_else(|| {
                    // Generate new request ID if not present
                    Uuid::new_v4().to_string()
                });

            // Add request ID to request headers
            let request_header_value = actix_web::http::header::HeaderValue::from_str(&request_id)
                .unwrap_or(actix_web::http::header::HeaderValue::from_static(
                    "invalid-request-id",
                ));
            req.headers_mut().insert(
                actix_web::http::header::HeaderName::from_static("x-request-id"),
                request_header_value,
            );

            // Store request ID in request extensions for easy access
            req.extensions_mut().insert(RequestId(request_id.clone()));

            // Call the next service
            let mut res = service.call(req).await?;

            // Add request ID to response headers
            let response_header_value = actix_web::http::header::HeaderValue::from_str(&request_id)
                .unwrap_or(actix_web::http::header::HeaderValue::from_static(
                    "invalid-request-id",
                ));
            res.headers_mut().insert(
                actix_web::http::header::HeaderName::from_static("x-request-id"),
                response_header_value,
            );

            Ok(res)
        })
    }
}

/// Wrapper for request ID stored in request extensions
#[derive(Debug, Clone)]
pub struct RequestId(pub String);

impl RequestId {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

/// Helper function to extract request ID from ServiceRequest
pub fn get_request_id(req: &ServiceRequest) -> Option<String> {
    req.extensions()
        .get::<RequestId>()
        .map(|id| id.0.clone())
        .or_else(|| {
            // Fallback: try to get from headers
            req.headers()
                .get("X-Request-ID")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string())
        })
}

/// Helper function to get request ID from ServiceRequest, generating one if not present
pub fn require_request_id(req: &ServiceRequest) -> String {
    get_request_id(req).unwrap_or_else(|| Uuid::new_v4().to_string())
}

/// Helper function to extract request ID from HttpRequest
pub fn get_request_id_from_http_request(req: &actix_web::HttpRequest) -> Option<String> {
    req.extensions()
        .get::<RequestId>()
        .map(|id| id.0.clone())
        .or_else(|| {
            req.headers()
                .get("X-Request-ID")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string())
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App, HttpResponse};

    async fn test_handler() -> HttpResponse {
        HttpResponse::Ok().body("test")
    }

    #[actix_web::test]
    async fn test_request_id_middleware() {
        let app = test::init_service(
            App::new()
                .wrap(RequestIdMiddleware)
                .route("/", web::get().to(test_handler)),
        )
        .await;

        let req = test::TestRequest::get().uri("/").to_request();
        let resp = test::call_service(&app, req).await;

        // Check that response has X-Request-ID header
        assert!(resp.headers().contains_key("x-request-id"));

        let request_id = resp.headers().get("x-request-id").unwrap();
        let request_id_str = request_id.to_str().unwrap();

        // Verify it's a valid UUID
        assert!(Uuid::parse_str(request_id_str).is_ok());
    }

    #[actix_web::test]
    async fn test_existing_request_id() {
        let app = test::init_service(
            App::new()
                .wrap(RequestIdMiddleware)
                .route("/", web::get().to(test_handler)),
        )
        .await;

        let existing_id = Uuid::new_v4().to_string();
        let req = test::TestRequest::get()
            .uri("/")
            .insert_header(("X-Request-ID", existing_id.clone()))
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Check that response preserves the existing request ID
        let response_id = resp
            .headers()
            .get("x-request-id")
            .unwrap()
            .to_str()
            .unwrap();

        assert_eq!(response_id, existing_id);
    }

    #[actix_web::test]
    async fn test_invalid_request_id_replaced() {
        let app = test::init_service(
            App::new()
                .wrap(RequestIdMiddleware)
                .route("/", web::get().to(test_handler)),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/")
            .insert_header(("X-Request-ID", "invalid-uuid"))
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Check that response has a valid UUID (replaced the invalid one)
        let response_id = resp
            .headers()
            .get("x-request-id")
            .unwrap()
            .to_str()
            .unwrap();

        assert!(Uuid::parse_str(response_id).is_ok());
        assert_ne!(response_id, "invalid-uuid");
    }

    #[actix_web::test]
    async fn test_request_id_wrapper() {
        let id = Uuid::new_v4().to_string();
        let request_id = RequestId(id.clone());

        assert_eq!(request_id.as_str(), id);
        assert_eq!(request_id.clone().into_string(), id);
    }
}
