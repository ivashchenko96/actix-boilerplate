pub mod loader;

use fluent::{FluentBundle, FluentResource};
use fluent_bundle::FluentArgs;
use unic_langid::LanguageIdentifier;
use std::collections::HashMap;
use anyhow::Result;

use crate::config::Settings;

/// i18n service for managing translations
pub struct I18nService {
    resources: HashMap<String, String>,
    default_locale: String,
}

impl I18nService {
    pub fn new(settings: &Settings) -> Result<Self> {
        let mut service = Self {
            resources: HashMap::new(),
            default_locale: settings.i18n.default_locale.clone(),
        };

        // Load translations for each supported locale
        for locale in &settings.i18n.supported_locales {
            service.load_locale(locale)?;
        }

        Ok(service)
    }

    /// Load translations for a specific locale
    pub fn load_locale(&mut self, locale: &str) -> Result<()> {
        let ftl_path = format!("locales/{}/common.ftl", locale);
        if let Ok(ftl_content) = std::fs::read_to_string(&ftl_path) {
            self.resources.insert(locale.to_string(), ftl_content);
        }
        Ok(())
    }

    /// Get a translated message
    pub fn get_message(&self, locale: &str, key: &str, args: Option<&FluentArgs<'_>>) -> String {
        let selected_locale = if self.resources.contains_key(locale) {
            locale
        } else {
            &self.default_locale
        };
        let Some(ftl_content) = self
            .resources
            .get(selected_locale)
        else {
            return key.to_string();
        };

        let lang_id = selected_locale
            .parse::<LanguageIdentifier>()
            .unwrap_or_else(|_| LanguageIdentifier::default());
        let mut bundle = FluentBundle::new(vec![lang_id]);
        let resource = match FluentResource::try_new(ftl_content.clone()) {
            Ok(resource) => resource,
            Err(_) => return key.to_string(),
        };
        if bundle.add_resource(resource).is_err() {
            return key.to_string();
        }

        if let Some(message) = bundle.get_message(key) {
            if let Some(pattern) = message.value() {
                let mut errors = vec![];
                return bundle.format_pattern(pattern, args, &mut errors).to_string();
            }
        }

        // Fallback to key if message not found
        key.to_string()
    }

    /// Get available locales
    pub fn get_locales(&self) -> Vec<String> {
        self.resources.keys().cloned().collect()
    }
}
