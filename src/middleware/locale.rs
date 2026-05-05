use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};
use std::rc::Rc;
use std::sync::Arc;
use unic_langid::LanguageIdentifier;

use crate::{context::AppContext, middleware::auth::get_auth_user};

/// Locale information extracted from request
#[derive(Debug, Clone)]
pub struct Locale {
    pub language: LanguageIdentifier,
    pub code: String,
}

impl Locale {
    pub fn new(code: &str) -> Result<Self, unic_langid::LanguageIdentifierError> {
        let language = code.parse::<LanguageIdentifier>()?;
        Ok(Self {
            language,
            code: code.to_string(),
        })
    }

    pub fn code(&self) -> &str {
        &self.code
    }
}

/// Locale detection middleware
pub struct LocaleMiddleware {
    context: Arc<AppContext>,
}

impl LocaleMiddleware {
    pub fn new(context: Arc<AppContext>) -> Self {
        Self { context }
    }
}

impl<S, B> Transform<S, ServiceRequest> for LocaleMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = LocaleMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(LocaleMiddlewareService {
            service: Rc::new(service),
            context: Arc::clone(&self.context),
        }))
    }
}

pub struct LocaleMiddlewareService<S> {
    service: Rc<S>,
    context: Arc<AppContext>,
}

impl<S, B> Service<ServiceRequest> for LocaleMiddlewareService<S>
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
        let context = Arc::clone(&self.context);

        Box::pin(async move {
            let locale = detect_locale(&req, &context);

            // Store locale in request extensions
            req.extensions_mut().insert(locale);

            service.call(req).await
        })
    }
}

/// Detect locale from request using the following priority:
/// 1. Query parameter `lang`
/// 2. Accept-Language header
/// 3. JWT token locale claim (if authenticated)
/// 4. Default locale from configuration
fn detect_locale(req: &ServiceRequest, context: &AppContext) -> Locale {
    let supported_locales = &context.settings.i18n.supported_locales;
    let default_locale = &context.settings.i18n.default_locale;

    // 1. Check query parameter
    if let Some(lang) = req.query_string().split('&').find_map(|param| {
        let mut parts = param.split('=');
        if parts.next() == Some("lang") {
            parts.next()
        } else {
            None
        }
    }) {
        if is_supported_locale(lang, supported_locales) {
            if let Ok(locale) = Locale::new(lang) {
                return locale;
            }
        }
    }

    // 2. Check Accept-Language header
    if let Some(accept_language) = req.headers().get("Accept-Language") {
        if let Ok(accept_lang_str) = accept_language.to_str() {
            // Parse Accept-Language header (e.g., "en-US,en;q=0.9,de;q=0.8")
            let mut languages: Vec<(String, f32)> = accept_lang_str
                .split(',')
                .filter_map(|lang| {
                    let parts: Vec<&str> = lang.trim().split(';').collect();
                    let language = parts[0].trim();
                    let quality = if parts.len() > 1 {
                        parts[1].trim().strip_prefix("q=")?.parse().unwrap_or(1.0)
                    } else {
                        1.0
                    };
                    Some((language.to_string(), quality))
                })
                .collect();

            // Sort by quality (preference)
            languages.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

            // Find first supported language
            for (lang, _) in languages {
                // Try exact match first
                if is_supported_locale(&lang, supported_locales) {
                    if let Ok(locale) = Locale::new(&lang) {
                        return locale;
                    }
                }

                // Try language without region (e.g., "en" from "en-US")
                if let Some(base_lang) = lang.split('-').next() {
                    if is_supported_locale(base_lang, supported_locales) {
                        if let Ok(locale) = Locale::new(base_lang) {
                            return locale;
                        }
                    }
                }
            }
        }
    }

    // 3. Check JWT token locale claim (if user is authenticated)
    if let Some(auth_user) = get_auth_user(req) {
        if is_supported_locale(&auth_user.locale, supported_locales) {
            if let Ok(locale) = Locale::new(&auth_user.locale) {
                return locale;
            }
        }
    }

    // 4. Fall back to default locale
    Locale::new(default_locale).unwrap_or_else(|_| Locale {
        language: LanguageIdentifier::default(),
        code: "en".to_string(),
    })
}

/// Check if a locale is in the supported locales list
fn is_supported_locale(locale: &str, supported_locales: &[String]) -> bool {
    supported_locales
        .iter()
        .any(|supported| supported == locale)
}

/// Helper function to get locale from request extensions
pub fn get_locale(req: &ServiceRequest) -> Option<Locale> {
    req.extensions().get::<Locale>().cloned()
}

/// Helper function to require locale from request extensions
pub fn require_locale(req: &ServiceRequest) -> Locale {
    get_locale(req).unwrap_or_else(|| Locale {
        language: LanguageIdentifier::default(),
        code: "en".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locale_creation() {
        let locale = Locale::new("en").unwrap();
        assert_eq!(locale.code(), "en");
        assert_eq!(locale.language.to_string(), "en");

        let locale_with_region = Locale::new("en-US").unwrap();
        assert_eq!(locale_with_region.code(), "en-US");
    }

    #[test]
    fn test_is_supported_locale() {
        let supported = vec!["en".to_string(), "de".to_string(), "ar".to_string()];

        assert!(is_supported_locale("en", &supported));
        assert!(is_supported_locale("de", &supported));
        assert!(is_supported_locale("ar", &supported));
        assert!(!is_supported_locale("fr", &supported));
    }

    #[test]
    fn test_invalid_locale() {
        assert!(Locale::new("invalid-locale-code-that-is-too-long").is_err());
    }
}
