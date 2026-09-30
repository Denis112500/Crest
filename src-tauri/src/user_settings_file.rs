use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::backend_constants::{DEFAULT_ALLOWED_MEDIA_APP_IDENTIFIER_FRAGMENT, USER_SETTINGS_FILE_NAME};

/// Settings a user can change by writing `settings.json` in Crest's config folder.
/// Every field is optional; missing fields keep their default.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CrestUserSettings {
    /// A media session is shown only if its app ID contains one of these (ignoring case).
    /// An empty list shows every app.
    pub allowed_media_app_identifier_fragments: Vec<String>,
}

impl Default for CrestUserSettings {
    fn default() -> Self {
        Self {
            allowed_media_app_identifier_fragments: vec![DEFAULT_ALLOWED_MEDIA_APP_IDENTIFIER_FRAGMENT.to_string()],
        }
    }
}

pub fn load_crest_user_settings(crest_config_directory: &Path) -> CrestUserSettings {
    let settings_file_path = crest_config_directory.join(USER_SETTINGS_FILE_NAME);
    // No settings file is the normal case, not an error.
    let Ok(settings_file_text) = fs::read_to_string(&settings_file_path) else {
        return CrestUserSettings::default();
    };
    serde_json::from_str(&settings_file_text).unwrap_or_else(|parse_error| {
        eprintln!("Crest: ignoring {} because it is not valid: {parse_error}", settings_file_path.display());
        CrestUserSettings::default()
    })
}
