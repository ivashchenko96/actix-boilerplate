use std::path::Path;

use anyhow::{anyhow, Result};

/// Load FTL source from locales directory.
pub fn load_locale_file(locale: &str) -> Result<String> {
    let path = format!("locales/{}/common.ftl", locale);
    let locale_path = Path::new(&path);
    if !locale_path.exists() {
        return Err(anyhow!("locale file not found: {}", path));
    }
    std::fs::read_to_string(locale_path).map_err(Into::into)
}
