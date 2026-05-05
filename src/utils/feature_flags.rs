use crate::config::Settings;

/// Feature flag service for managing feature toggles
pub struct FeatureFlagService {
    flags: std::collections::HashMap<String, bool>,
}

impl FeatureFlagService {
    pub fn new(settings: &Settings) -> Self {
        let mut flags = std::collections::HashMap::new();

        // Load flags from settings
        flags.insert(
            "registration_enabled".to_string(),
            settings.feature_flags.registration_enabled,
        );
        flags.insert(
            "email_verification".to_string(),
            settings.feature_flags.email_verification,
        );
        flags.insert(
            "password_reset".to_string(),
            settings.feature_flags.password_reset,
        );
        flags.insert("swagger_ui".to_string(), settings.feature_flags.swagger_ui);
        flags.insert("metrics".to_string(), settings.feature_flags.metrics);

        Self { flags }
    }

    pub fn is_enabled(&self, feature: &str) -> bool {
        self.flags.get(feature).copied().unwrap_or(false)
    }

    pub fn enable_feature(&mut self, feature: &str) {
        self.flags.insert(feature.to_string(), true);
    }

    pub fn disable_feature(&mut self, feature: &str) {
        self.flags.insert(feature.to_string(), false);
    }

    pub fn get_all_flags(&self) -> &std::collections::HashMap<String, bool> {
        &self.flags
    }
}
