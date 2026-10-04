use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::backend_constants::{DEFAULT_ALLOWED_MEDIA_APP_IDENTIFIER_FRAGMENT, USER_SETTINGS_FILE_NAME};

/// Crest's settings, kept in `settings.json` in its config folder (%APPDATA%\dev.crest.pill).
/// The folder is outside the install folder, so updates don't touch it. Every field is
/// optional in the file; missing fields keep their default.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CrestUserSettings {
    /// A media session is shown only if its app ID contains one of these (ignoring case).
    /// An empty list shows every app.
    pub allowed_media_app_identifier_fragments: Vec<String>,
    /// Windows' name of the monitor the pill sits on (e.g. `\\.\DISPLAY2`). `None`, or a
    /// monitor that isn't connected, means the main display.
    pub pill_display_name: Option<String>,
}

impl Default for CrestUserSettings {
    fn default() -> Self {
        Self {
            allowed_media_app_identifier_fragments: vec![DEFAULT_ALLOWED_MEDIA_APP_IDENTIFIER_FRAGMENT.to_string()],
            pill_display_name: None,
        }
    }
}

/// The settings as they are now, shared by Tauri commands, plus where to save them.
pub struct CrestUserSettingsStore {
    settings_file_path: PathBuf,
    current_settings: Mutex<CrestUserSettings>,
}

impl CrestUserSettingsStore {
    pub fn load_from_config_directory(crest_config_directory: &Path) -> Self {
        let settings_file_path = crest_config_directory.join(USER_SETTINGS_FILE_NAME);
        let current_settings = read_settings_file(&settings_file_path);
        Self { settings_file_path, current_settings: Mutex::new(current_settings) }
    }

    pub fn read_current_settings(&self) -> CrestUserSettings {
        self.lock_current_settings().clone()
    }

    /// Applies a change and writes the file right away, so it survives a crash or restart.
    /// The lock is held while writing, so two quick changes can't save out of order.
    pub fn change_and_save(&self, apply_change: impl FnOnce(&mut CrestUserSettings)) -> Result<(), String> {
        let mut current_settings = self.lock_current_settings();
        apply_change(&mut current_settings);
        write_settings_file_safely(&self.settings_file_path, &current_settings)
    }

    // A panic while the lock was held can't leave the settings half-changed (changes are
    // plain field writes), so a poisoned lock is still safe to use.
    fn lock_current_settings(&self) -> std::sync::MutexGuard<'_, CrestUserSettings> {
        self.current_settings.lock().unwrap_or_else(|poisoned_lock| poisoned_lock.into_inner())
    }
}

fn read_settings_file(settings_file_path: &Path) -> CrestUserSettings {
    // No settings file is the normal case, not an error.
    let Ok(settings_file_text) = fs::read_to_string(settings_file_path) else {
        return CrestUserSettings::default();
    };
    serde_json::from_str(&settings_file_text).unwrap_or_else(|parse_error| {
        eprintln!("Crest: ignoring {} because it is not valid: {parse_error}", settings_file_path.display());
        CrestUserSettings::default()
    })
}

/// Writes a temporary file next to the real one, then renames it over the real one. A rename
/// replaces the file in one step, so a crash mid-write leaves the old settings intact instead
/// of a half-written file.
fn write_settings_file_safely(settings_file_path: &Path, settings: &CrestUserSettings) -> Result<(), String> {
    let settings_file_text = serde_json::to_string_pretty(settings).map_err(|error| error.to_string())?;
    if let Some(crest_config_directory) = settings_file_path.parent() {
        fs::create_dir_all(crest_config_directory).map_err(|error| error.to_string())?;
    }
    let temporary_settings_file_path = settings_file_path.with_extension("json.tmp");
    fs::write(&temporary_settings_file_path, settings_file_text).map_err(|error| error.to_string())?;
    fs::rename(&temporary_settings_file_path, settings_file_path).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_empty_test_directory(test_name: &str) -> PathBuf {
        let test_directory = std::env::temp_dir().join(format!("crest-settings-test-{test_name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&test_directory);
        test_directory
    }

    #[test]
    fn a_saved_change_is_there_after_loading_again() {
        let test_directory = create_empty_test_directory("round-trip");
        let settings_store = CrestUserSettingsStore::load_from_config_directory(&test_directory);
        settings_store
            .change_and_save(|settings| settings.pill_display_name = Some(r"\\.\DISPLAY2".to_string()))
            .unwrap();

        let reloaded_settings = CrestUserSettingsStore::load_from_config_directory(&test_directory).read_current_settings();
        assert_eq!(reloaded_settings.pill_display_name.as_deref(), Some(r"\\.\DISPLAY2"));
        assert_eq!(reloaded_settings.allowed_media_app_identifier_fragments, CrestUserSettings::default().allowed_media_app_identifier_fragments);
        fs::remove_dir_all(&test_directory).unwrap();
    }

    #[test]
    fn a_file_from_an_older_version_keeps_its_values_and_gets_defaults_for_new_fields() {
        let test_directory = create_empty_test_directory("older-file");
        fs::create_dir_all(&test_directory).unwrap();
        fs::write(test_directory.join(USER_SETTINGS_FILE_NAME), r#"{ "allowedMediaAppIdentifierFragments": [] }"#).unwrap();

        let loaded_settings = CrestUserSettingsStore::load_from_config_directory(&test_directory).read_current_settings();
        assert!(loaded_settings.allowed_media_app_identifier_fragments.is_empty());
        assert_eq!(loaded_settings.pill_display_name, None);
        fs::remove_dir_all(&test_directory).unwrap();
    }
}
