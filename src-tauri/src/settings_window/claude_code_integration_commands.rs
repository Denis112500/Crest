//! The settings window's commands for the Claude Code integration: show its state and exactly
//! what it adds to Claude Code's settings, switch it on or off, and switch answering permission
//! requests in the pill on or off.

use std::sync::PoisonError;

use serde::Serialize;
use serde_json::{json, Value};
use tauri::State;

use crate::activity_core::{SharedActivityArbiter, SharedActivitySourceRegistry};
use crate::activity_sources::claude_code::{
    build_crest_hook_command, build_crest_hooks, change_claude_code_permission_answering, claude_code_settings_file_path,
    turn_claude_code_integration_off, turn_claude_code_integration_on, ClaudeCodePermissionAnsweringSwitch,
};
use crate::user_settings_store::{CrestUserSettings, CrestUserSettingsStore};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeCodeIntegrationSetting {
    is_integration_on: bool,
    is_permission_answering_on: bool,
    /// Where Claude Code's settings file is, shown so the user knows which file changes.
    claude_code_settings_file_path: String,
    /// The `hooks` block Crest adds, pretty-printed, shown before switching on.
    added_hooks_preview: String,
}

#[tauri::command]
pub fn read_claude_code_integration_setting(
    settings_store: State<'_, CrestUserSettingsStore>,
) -> Result<ClaudeCodeIntegrationSetting, String> {
    describe_claude_code_integration(&settings_store.read_current_settings())
}

/// Saves the choice only once Claude Code's settings were changed, so the switch never shows
/// "on" while the hooks are missing. Returns the state afterwards, which the switch shows.
#[tauri::command]
pub fn change_claude_code_integration_setting(
    settings_store: State<'_, CrestUserSettingsStore>,
    shared_activity_source_registry: State<'_, SharedActivitySourceRegistry>,
    shared_activity_arbiter: State<'_, SharedActivityArbiter>,
    permission_answering_switch: State<'_, ClaudeCodePermissionAnsweringSwitch>,
    should_integration_be_on: bool,
) -> Result<ClaudeCodeIntegrationSetting, String> {
    let mut activity_source_registry = shared_activity_source_registry.lock().unwrap_or_else(PoisonError::into_inner);
    if should_integration_be_on {
        turn_claude_code_integration_on(&mut activity_source_registry, &shared_activity_arbiter, &permission_answering_switch)?;
    } else {
        // Saved as off even if Claude Code's file couldn't be cleaned up: Crest stopped listening.
        let switch_off_result = turn_claude_code_integration_off(&mut activity_source_registry);
        settings_store.change_and_save(|settings| settings.is_claude_code_integration_on = false)?;
        switch_off_result?;
        return describe_claude_code_integration(&settings_store.read_current_settings());
    }
    settings_store.change_and_save(|settings| settings.is_claude_code_integration_on = true)?;
    describe_claude_code_integration(&settings_store.read_current_settings())
}

/// Like the integration switch: saved only once Claude Code's settings were changed.
#[tauri::command]
pub fn change_claude_code_permission_answering_setting(
    settings_store: State<'_, CrestUserSettingsStore>,
    permission_answering_switch: State<'_, ClaudeCodePermissionAnsweringSwitch>,
    should_permission_answering_be_on: bool,
) -> Result<ClaudeCodeIntegrationSetting, String> {
    let is_integration_on = settings_store.read_current_settings().is_claude_code_integration_on;
    change_claude_code_permission_answering(&permission_answering_switch, should_permission_answering_be_on, is_integration_on)?;
    settings_store.change_and_save(|settings| settings.is_claude_code_permission_answering_on = should_permission_answering_be_on)?;
    describe_claude_code_integration(&settings_store.read_current_settings())
}

fn describe_claude_code_integration(current_settings: &CrestUserSettings) -> Result<ClaudeCodeIntegrationSetting, String> {
    let crest_executable_path = std::env::current_exe().map_err(|error| error.to_string())?;
    let crest_hooks = build_crest_hooks(
        &build_crest_hook_command(&crest_executable_path),
        current_settings.is_claude_code_permission_answering_on,
    );
    Ok(ClaudeCodeIntegrationSetting {
        is_integration_on: current_settings.is_claude_code_integration_on,
        is_permission_answering_on: current_settings.is_claude_code_permission_answering_on,
        claude_code_settings_file_path: claude_code_settings_file_path()?.display().to_string(),
        added_hooks_preview: serde_json::to_string_pretty(&json!({ "hooks": Value::Object(crest_hooks) }))
            .map_err(|error| error.to_string())?,
    })
}
