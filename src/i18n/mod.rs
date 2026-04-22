pub mod loader;

use fluent::{FluentBundle, FluentResource};
use unic_langid::LanguageIdentifier;
use std::collections::HashMap;
use anyhow::Result;

use crate::config::Settings;

/// i18n service for managing translations
pub struct I18nService {
    bundles: HashMap<String, FluentBundle<FluentResource>>,
    default_locale: String,
}

impl I18nService {
    pub fn new(settings: &Settings) -> Result<Self> {
        let mut service = Self {
            bundles: HashMap::new(),
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
        let lang_id: LanguageIdentifier = locale.parse()?;
        let mut bundle = FluentBundle::new(vec![lang_id]);

        // Load common translations
        let ftl_path = format!("locales/{}/common.ftl", locale);
        if let Ok(ftl_content) = std::fs::read_to_string(&ftl_path) {
            let resource = FluentResource::try_new(ftl_content)
                .map_err(|_| anyhow::anyhow!("Failed to parse FTL file for locale: {}", locale))?;
            bundle.add_resource(resource)
                .map_err(|_| anyhow::anyhow!("Failed to add resource for locale: {}", locale))?;
        }

        self.bundles.insert(locale.to_string(), bundle);
        Ok(())
    }

    /// Get a translated message
    pub fn get_message(&self, locale: &str, key: &str, args: Option<&std::collections::HashMap<String, fluent::FluentValue>>) -> String {
        let bundle = self.bundles.get(locale)
            .or_else(|| self.bundles.get(&self.default_locale))
            .unwrap(); // Should always have default locale

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
        self.bundles.keys().cloned().collect()
    }
}