//! What a switch in the settings window changes in Claude Code's settings file: previewed
//! without writing, then written only if the file is still the one the preview was made from.

use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::Value;

use crate::activity_sources::claude_code::claude_code_hook_registration::{
    add_crest_hooks, build_crest_hook_command, remove_crest_hooks,
};
use crate::activity_sources::claude_code::claude_code_hooks_change_preview::{
    preview_claude_code_hooks_change, ClaudeCodeHooksChangePreview,
};
use crate::activity_sources::claude_code::claude_code_settings_file::{
    claude_code_settings_file_path, read_claude_code_settings, replace_claude_code_settings,
};

pub enum ClaudeCodeSettingsChange {
    AddCrestHooks { crest_executable_path: PathBuf, should_add_permission_hook: bool },
    RemoveCrestHooks,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeCodeSettingsChangePreview {
    #[serde(flatten)]
    pub hooks_change: ClaudeCodeHooksChangePreview,
    /// Sent back with the user's confirmation, so the write happens only on the file they saw.
    pub settings_file_fingerprint: String,
}

#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaudeCodeSettingsChangeOutcome {
    /// Written, or nothing needed writing.
    Applied,
    /// Nothing was written: the file isn't the one the user's preview showed.
    FileChangedSincePreview,
}

/// Reads the file without writing anything. `None` previews leaving the file as it is (the
/// answering switch while the integration is off).
pub fn preview_claude_code_settings_change(
    settings_change: Option<&ClaudeCodeSettingsChange>,
) -> Result<ClaudeCodeSettingsChangePreview, String> {
    preview_settings_file_change(&claude_code_settings_file_path()?, settings_change)
}

/// `expected_settings_file_fingerprint` comes from the preview the user confirmed; `None` where
/// no preview was shown (Crest's start re-pointing the hooks). Nothing is written if nothing
/// changes.
pub fn change_claude_code_settings(
    settings_change: &ClaudeCodeSettingsChange,
    expected_settings_file_fingerprint: Option<&str>,
) -> Result<ClaudeCodeSettingsChangeOutcome, String> {
    change_settings_file(&claude_code_settings_file_path()?, settings_change, expected_settings_file_fingerprint)
}

/// The uninstaller's way: no preview was shown, so the file is changed whatever it holds now.
pub fn remove_crest_hooks_from_claude_code_settings() -> Result<(), String> {
    change_claude_code_settings(&ClaudeCodeSettingsChange::RemoveCrestHooks, None).map(|_| ())
}

fn preview_settings_file_change(
    settings_file_path: &Path,
    settings_change: Option<&ClaudeCodeSettingsChange>,
) -> Result<ClaudeCodeSettingsChangePreview, String> {
    let settings_snapshot = read_claude_code_settings(settings_file_path)?;
    let mut settings_after_change = settings_snapshot.claude_code_settings.clone();
    if let Some(settings_change) = settings_change {
        apply_claude_code_settings_change(settings_change, &mut settings_after_change)?;
    }
    Ok(ClaudeCodeSettingsChangePreview {
        hooks_change: preview_claude_code_hooks_change(&settings_snapshot.claude_code_settings, &settings_after_change),
        settings_file_fingerprint: settings_snapshot.fingerprint(),
    })
}

/// The check and the write use the same read, so nothing Claude Code writes in between the two
/// can be overwritten unseen.
fn change_settings_file(
    settings_file_path: &Path,
    settings_change: &ClaudeCodeSettingsChange,
    expected_settings_file_fingerprint: Option<&str>,
) -> Result<ClaudeCodeSettingsChangeOutcome, String> {
    let settings_snapshot = read_claude_code_settings(settings_file_path)?;
    if expected_settings_file_fingerprint.is_some_and(|expected_fingerprint| expected_fingerprint != settings_snapshot.fingerprint()) {
        return Ok(ClaudeCodeSettingsChangeOutcome::FileChangedSincePreview);
    }
    let mut settings_after_change = settings_snapshot.claude_code_settings.clone();
    apply_claude_code_settings_change(settings_change, &mut settings_after_change)?;
    if settings_after_change != settings_snapshot.claude_code_settings {
        replace_claude_code_settings(
            settings_file_path,
            settings_snapshot.original_settings_text.as_deref(),
            &settings_after_change,
        )?;
    }
    Ok(ClaudeCodeSettingsChangeOutcome::Applied)
}

fn apply_claude_code_settings_change(
    settings_change: &ClaudeCodeSettingsChange,
    claude_code_settings: &mut Value,
) -> Result<(), String> {
    match settings_change {
        ClaudeCodeSettingsChange::AddCrestHooks { crest_executable_path, should_add_permission_hook } => {
            let crest_hook_command = build_crest_hook_command(crest_executable_path);
            add_crest_hooks(claude_code_settings, &crest_hook_command, *should_add_permission_hook)
        }
        ClaudeCodeSettingsChange::RemoveCrestHooks => {
            remove_crest_hooks(claude_code_settings);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    const TEST_CREST_EXECUTABLE_PATH: &str = r"C:\Program Files\Crest\crest.exe";

    fn create_test_settings_file_path(test_name: &str) -> PathBuf {
        let test_directory =
            std::env::temp_dir().join(format!("crest-settings-change-test-{test_name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&test_directory);
        fs::create_dir_all(&test_directory).unwrap();
        test_directory.join("settings.json")
    }

    fn add_crest_hooks_change() -> ClaudeCodeSettingsChange {
        ClaudeCodeSettingsChange::AddCrestHooks {
            crest_executable_path: PathBuf::from(TEST_CREST_EXECUTABLE_PATH),
            should_add_permission_hook: true,
        }
    }

    #[test]
    fn a_change_confirmed_on_the_file_the_preview_showed_is_written() {
        let settings_file_path = create_test_settings_file_path("confirmed");
        fs::write(&settings_file_path, r#"{"model":"opus"}"#).unwrap();
        let shown_preview = preview_settings_file_change(&settings_file_path, Some(&add_crest_hooks_change())).unwrap();
        let change_outcome = change_settings_file(
            &settings_file_path,
            &add_crest_hooks_change(),
            Some(&shown_preview.settings_file_fingerprint),
        );
        assert_eq!(change_outcome, Ok(ClaudeCodeSettingsChangeOutcome::Applied));
        let written_text = fs::read_to_string(&settings_file_path).unwrap();
        assert!(written_text.contains("hooks") && written_text.contains("opus"));
        let _ = fs::remove_dir_all(settings_file_path.parent().unwrap());
    }

    #[test]
    fn a_file_rewritten_after_the_preview_is_left_alone_until_its_new_preview_is_confirmed() {
        let settings_file_path = create_test_settings_file_path("rewritten");
        fs::write(&settings_file_path, r#"{"model":"opus"}"#).unwrap();
        let shown_preview = preview_settings_file_change(&settings_file_path, Some(&add_crest_hooks_change())).unwrap();
        fs::write(&settings_file_path, r#"{"model":"sonnet"}"#).unwrap();
        let change_outcome = change_settings_file(
            &settings_file_path,
            &add_crest_hooks_change(),
            Some(&shown_preview.settings_file_fingerprint),
        );
        assert_eq!(change_outcome, Ok(ClaudeCodeSettingsChangeOutcome::FileChangedSincePreview));
        assert_eq!(fs::read_to_string(&settings_file_path).unwrap(), r#"{"model":"sonnet"}"#);

        let new_preview = preview_settings_file_change(&settings_file_path, Some(&add_crest_hooks_change())).unwrap();
        assert_ne!(new_preview.settings_file_fingerprint, shown_preview.settings_file_fingerprint);
        let change_outcome =
            change_settings_file(&settings_file_path, &add_crest_hooks_change(), Some(&new_preview.settings_file_fingerprint));
        assert_eq!(change_outcome, Ok(ClaudeCodeSettingsChangeOutcome::Applied));
        assert!(fs::read_to_string(&settings_file_path).unwrap().contains("sonnet"));
        let _ = fs::remove_dir_all(settings_file_path.parent().unwrap());
    }

    #[test]
    fn a_missing_file_has_its_own_fingerprint_and_is_created_on_confirmation() {
        let settings_file_path = create_test_settings_file_path("missing");
        let shown_preview = preview_settings_file_change(&settings_file_path, Some(&add_crest_hooks_change())).unwrap();
        let change_outcome = change_settings_file(
            &settings_file_path,
            &add_crest_hooks_change(),
            Some(&shown_preview.settings_file_fingerprint),
        );
        assert_eq!(change_outcome, Ok(ClaudeCodeSettingsChangeOutcome::Applied));
        assert!(fs::read_to_string(&settings_file_path).unwrap().contains("hooks"));
        let _ = fs::remove_dir_all(settings_file_path.parent().unwrap());
    }

    #[test]
    fn without_a_preview_the_change_is_written_whatever_the_file_holds() {
        let settings_file_path = create_test_settings_file_path("unpreviewed");
        fs::write(&settings_file_path, r#"{"model":"opus"}"#).unwrap();
        let change_outcome = change_settings_file(&settings_file_path, &add_crest_hooks_change(), None);
        assert_eq!(change_outcome, Ok(ClaudeCodeSettingsChangeOutcome::Applied));
        assert!(fs::read_to_string(&settings_file_path).unwrap().contains("hooks"));
        let _ = fs::remove_dir_all(settings_file_path.parent().unwrap());
    }
}
