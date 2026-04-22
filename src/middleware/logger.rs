use tracing_actix_web::{TracingLogger, RootSpanBuilder};
use actix_web::dev::ServiceRequest;
use tracing::Span;
use uuid::Uuid;

/// Create logger middleware with custom configuration
pub fn logger_middleware() -> TracingLogger<CustomRootSpanBuilder> {
    TracingLogger::<CustomRootSpanBuilder>::new()
}

/// Custom root span builder for structured logging
pub struct CustomRootSpanBuilder;

impl RootSpanBuilder for CustomRootSpanBuilder {
    fn on_request_start(request: &ServiceRequest) -> Span {
        let request_id = request
            .headers()
            .get("X-Request-ID")
            .and_then(|h| h.to_str().ok())
            .unwrap_or_else(|| {
                // Generate a new request ID if not present
                &Uuid::new_v4().to_string()
            });

        let user_agent = request
            .headers()
            .get("User-Agent")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("unknown");

        let forwarded_for = request
            .headers()
            .get("X-Forwarded-For")
            .and_then(|h| h.to_str().ok())
            .or_else(|| {
                request
                    .headers()
                    .get("X-Real-IP")
                    .and_then(|h| h.to_str().ok())
            });

        let remote_addr = request
            .connection_info()
            .realip_remote_addr()
            .unwrap_or("unknown");

        tracing::info_span!(
            "HTTP request",
            method = %request.method(),
            uri = %request.uri(),
            version = ?request.version(),
            request_id = %request_id,
            user_agent = %user_agent,
            remote_addr = %remote_addr,
            forwarded_for = forwarded_for,
            status_code = tracing::field::Empty,
            response_time_ms = tracing::field::Empty,
        )
    }

    fn on_request_end<B>(span: Span, outcome: &Result<actix_web::dev::ServiceResponse<B>, actix_web::Error>) {
        match outcome {
            Ok(response) => {
                let status_code = response.status().as_u16();
                span.record("status_code", status_code);
                
                if status_code >= 400 {
                    if status_code >= 500 {
                        tracing::error!("HTTP request completed with server error");
                    } else {
                        tracing::warn!("HTTP request completed with client error");
                    }
                } else {
                    tracing::info!("HTTP request completed successfully");
                }
            }
            Err(error) => {
                span.record("status_code", 500);
                tracing::error!(error = %error, "HTTP request failed");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_root_span_builder() {
        // Test that the CustomRootSpanBuilder can be created
        let _builder = CustomRootSpanBuilder;
        
        // In a real test, you would create a mock ServiceRequest 
        // and test the span creation, but that requires more setup
    }
}