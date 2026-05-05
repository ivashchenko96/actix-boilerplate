use actix_web::{
    body::EitherBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use governor::{
    clock::{QuantaClock, QuantaInstant},
    middleware::NoOpMiddleware,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};
use std::{
    collections::HashMap,
    future::{ready, Ready},
    net::IpAddr,
    num::NonZeroU32,
    rc::Rc,
    sync::Arc,
};

use crate::{
    config::Settings,
    errors::{ApiError, ApiResponse},
};

type DefaultRateLimiter =
    RateLimiter<NotKeyed, InMemoryState, QuantaClock, NoOpMiddleware<QuantaInstant>>;
type IpRateLimiterStore = Arc<tokio::sync::RwLock<HashMap<IpAddr, DefaultRateLimiter>>>;

/// Rate limiting middleware
pub struct RateLimitMiddleware {
    settings: Settings,
    limiter: Arc<DefaultRateLimiter>,
}

impl RateLimitMiddleware {
    fn non_zero_or(value: u32, fallback: u32) -> NonZeroU32 {
        NonZeroU32::new(value)
            .or_else(|| NonZeroU32::new(fallback))
            .unwrap_or(NonZeroU32::MIN)
    }

    pub fn new(settings: &Settings) -> Self {
        let quota = Quota::per_minute(Self::non_zero_or(
            settings.rate_limiting.requests_per_minute,
            60,
        ))
        .allow_burst(Self::non_zero_or(settings.rate_limiting.burst_size, 100));

        let limiter = Arc::new(RateLimiter::direct(quota));

        Self {
            settings: settings.clone(),
            limiter,
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimitMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = RateLimitMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimitMiddlewareService {
            service: Rc::new(service),
            limiter: Arc::clone(&self.limiter),
            enabled: self.settings.rate_limiting.enabled,
        }))
    }
}

pub struct RateLimitMiddlewareService<S> {
    service: Rc<S>,
    limiter: Arc<DefaultRateLimiter>,
    enabled: bool,
}

impl<S, B> Service<ServiceRequest> for RateLimitMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);
        let limiter = Arc::clone(&self.limiter);
        let enabled = self.enabled;

        Box::pin(async move {
            if !enabled {
                return service
                    .call(req)
                    .await
                    .map(ServiceResponse::map_into_left_body);
            }

            // Check rate limit
            match limiter.check() {
                Ok(_) => {
                    // Rate limit not exceeded, continue with request
                    service
                        .call(req)
                        .await
                        .map(ServiceResponse::map_into_left_body)
                }
                Err(_) => {
                    // Rate limit exceeded, return error
                    let request_id = req
                        .headers()
                        .get("X-Request-ID")
                        .and_then(|h| h.to_str().ok())
                        .unwrap_or("unknown")
                        .to_string();

                    let error = ApiError {
                        code: "RATE_LIMIT_EXCEEDED".to_string(),
                        message: "Too many requests".to_string(),
                        field: None,
                    };

                    let response = ApiResponse::<()>::error(
                        vec![error],
                        "Rate limit exceeded".to_string(),
                        "en".to_string(), // Should use locale from request
                        request_id,
                    );

                    let http_response = HttpResponse::TooManyRequests()
                        .insert_header(("Retry-After", "60")) // Suggest retry after 60 seconds
                        .json(response);

                    Ok(req.into_response(http_response.map_into_right_body()))
                }
            }
        })
    }
}

/// IP-based rate limiter for more granular control
pub struct IpRateLimiter {
    limiters: IpRateLimiterStore,
    quota: Quota,
}

impl IpRateLimiter {
    fn non_zero_or(value: u32, fallback: u32) -> NonZeroU32 {
        NonZeroU32::new(value)
            .or_else(|| NonZeroU32::new(fallback))
            .unwrap_or(NonZeroU32::MIN)
    }

    pub fn new(requests_per_minute: u32, burst_size: u32) -> Self {
        let quota = Quota::per_minute(Self::non_zero_or(requests_per_minute, 60))
            .allow_burst(Self::non_zero_or(burst_size, 100));

        Self {
            limiters: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            quota,
        }
    }

    pub async fn check_rate_limit(
        &self,
        ip: IpAddr,
    ) -> Result<(), governor::NotUntil<QuantaInstant>> {
        // Try to get existing limiter
        {
            let limiters = self.limiters.read().await;
            if let Some(limiter) = limiters.get(&ip) {
                return limiter.check();
            }
        }

        // Create new limiter for this IP
        {
            let mut limiters = self.limiters.write().await;
            let limiter = limiters
                .entry(ip)
                .or_insert_with(|| RateLimiter::direct(self.quota));
            limiter.check()
        }
    }

    /// Clean up old limiters (should be called periodically)
    pub async fn cleanup(&self) {
        let mut limiters = self.limiters.write().await;
        limiters.retain(|_, _| true);

        // Limit the number of stored limiters to prevent memory exhaustion
        if limiters.len() > 10000 {
            let excess = limiters.len() - 10000;
            let keys_to_remove: Vec<IpAddr> = limiters.keys().take(excess).cloned().collect();
            for key in keys_to_remove {
                limiters.remove(&key);
            }
        }
    }
}

/// Get client IP address from request
pub fn get_client_ip(req: &ServiceRequest) -> Option<IpAddr> {
    // Check X-Forwarded-For header first
    if let Some(forwarded) = req.headers().get("X-Forwarded-For") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            if let Some(first_ip) = forwarded_str.split(',').next() {
                if let Ok(ip) = first_ip.trim().parse::<IpAddr>() {
                    return Some(ip);
                }
            }
        }
    }

    // Check X-Real-IP header
    if let Some(real_ip) = req.headers().get("X-Real-IP") {
        if let Ok(ip_str) = real_ip.to_str() {
            if let Ok(ip) = ip_str.parse::<IpAddr>() {
                return Some(ip);
            }
        }
    }

    // Fall back to connection info
    req.connection_info().realip_remote_addr().and_then(|addr| {
        // Remove port if present
        if let Some(ip_part) = addr.split(':').next() {
            ip_part.parse::<IpAddr>().ok()
        } else {
            addr.parse::<IpAddr>().ok()
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[tokio::test]
    async fn test_ip_rate_limiter() {
        let limiter = IpRateLimiter::new(1, 1);
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        // First request should succeed
        assert!(limiter.check_rate_limit(ip).await.is_ok());

        // Should be rate limited now
        assert!(limiter.check_rate_limit(ip).await.is_err());
    }

    #[test]
    fn test_rate_limit_middleware_creation() {
        let settings = crate::config::Settings::new_for_tests();
        let middleware = RateLimitMiddleware::new(&settings);

        assert!(!middleware.settings.rate_limiting.enabled); // Should be disabled in test config
    }
}
