//! The settings window's commands for the Claude Code integration: show its state and exactly
//! what it adds to Claude Code's settings, and switch it on or off.

use std::sync::PoisonError;

use serde::Serialize;
use serde_json::{json, Value};
use tauri::State;

use crate::activity_core::{SharedActivityArbiter, SharedActivitySourceRegistry};
use crate::activity_sources::claude_code::{
    build_crest_hook_command, build_crest_hooks, claude_code_settings_file_path, turn_claude_code_integration_off,
    turn_claude_code_integration_on,
};
use crate::user_settings_store::CrestUserSettingsStore;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeCodeIntegrationSetting {
    is_integration_on: bool,
    /// Where Claude Code's settings file is, shown so the user knows which file changes.
    claude_code_settings_file_path: String,
    /// The `hooks` block Crest adds, pretty-printed, shown before switching on.
    added_hooks_preview: String,
}

#[tauri::command]
pub fn read_claude_code_integration_setting(
    settings_store: State<'_, CrestUserSettingsStore>,
) -> Result<ClaudeCodeIntegrationSetting, String> {
    describe_claude_code_integration(settings_store.read_current_settings().is_claude_code_integration_on)
}

/// Saves the choice only once Claude Code's settings were changed, so the switch never shows
/// "on" while the hooks are missing. Returns the state afterwards, which the switch shows.
#[tauri::command]
pub fn change_claude_code_integration_setting(
    settings_store: State<'_, CrestUserSettingsStore>,
    shared_activity_source_registry: State<'_, SharedActivitySourceRegistry>,
    shared_activity_arbiter: State<'_, SharedActivityArbiter>,
    should_integration_be_on: bool,
) -> Result<ClaudeCodeIntegrationSetting, String> {
    let mut activity_source_registry = shared_activity_source_registry.lock().unwrap_or_else(PoisonError::into_inner);
    if should_integration_be_on {
        turn_claude_code_integration_on(&mut activity_source_registry, &shared_activity_arbiter)?;
    } else {
        // Saved as off even if Claude Code's file couldn't be cleaned up: Crest stopped listening.
        let switch_off_result = turn_claude_code_integration_off(&mut activity_source_registry);
        settings_store.change_and_save(|settings| settings.is_claude_code_integration_on = false)?;
        switch_off_result?;
        return describe_claude_code_integration(false);
    }
    settings_store.change_and_save(|settings| settings.is_claude_code_integration_on = true)?;
    describe_claude_code_integration(true)
}

fn describe_claude_code_integration(is_integration_on: bool) -> Result<ClaudeCodeIntegrationSetting, String> {
    let crest_executable_path = std::env::current_exe().map_err(|error| error.to_string())?;
    let added_hooks = json!({ "hooks": Value::Object(build_crest_hooks(&build_crest_hook_command(&crest_executable_path))) });
    Ok(ClaudeCodeIntegrationSetting {
        is_integration_on,
        claude_code_settings_file_path: claude_code_settings_file_path()?.display().to_string(),
        added_hooks_preview: serde_json::to_string_pretty(&added_hooks).map_err(|error| error.to_string())?,
    })
}
