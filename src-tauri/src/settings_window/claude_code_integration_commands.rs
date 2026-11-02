//! The settings window's commands for the Claude Code integration: show its state and exactly
//! what it adds to Claude Code's settings, preview what a switch would change, switch it on or
//! off, and switch answering permission requests in the pill on or off.

use std::sync::PoisonError;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::State;

use crate::activity_core::{SharedActivityArbiter, SharedActivitySourceRegistry};
use crate::activity_sources::claude_code::{
    build_crest_hook_command, build_crest_hooks, change_claude_code_permission_answering, claude_code_settings_file_path,
    preview_claude_code_settings_change, turn_claude_code_integration_off, turn_claude_code_integration_on,
    ClaudeCodePermissionAnsweringSwitch, ClaudeCodeSettingsChange, ClaudeCodeSettingsChangePreview,
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

/// The two switches of the card, flipped one way or the other.
#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ClaudeCodeSwitchChange {
    IntegrationOn,
    IntegrationOff,
    AnsweringOn,
    AnsweringOff,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeCodeSwitchChangePreview {
    #[serde(flatten)]
    settings_change: ClaudeCodeSettingsChangePreview,
    /// Requests waiting in the pill that the change hands back to Claude Code's own question.
    handed_back_request_count: usize,
}

/// Writes nothing: shows what flipping a switch would change, before the user confirms.
#[tauri::command]
pub fn preview_claude_code_switch_change(
    settings_store: State<'_, CrestUserSettingsStore>,
    permission_answering_switch: State<'_, ClaudeCodePermissionAnsweringSwitch>,
    switch_change: ClaudeCodeSwitchChange,
) -> Result<ClaudeCodeSwitchChangePreview, String> {
    let current_settings = settings_store.read_current_settings();
    let crest_executable_path = std::env::current_exe().map_err(|error| error.to_string())?;
    let add_crest_hooks =
        |should_add_permission_hook| ClaudeCodeSettingsChange::AddCrestHooks { crest_executable_path, should_add_permission_hook };
    let settings_change = match switch_change {
        ClaudeCodeSwitchChange::IntegrationOn => Some(add_crest_hooks(current_settings.is_claude_code_permission_answering_on)),
        ClaudeCodeSwitchChange::IntegrationOff => Some(ClaudeCodeSettingsChange::RemoveCrestHooks),
        // While the integration is off, the answering switch only changes Crest's own setting.
        ClaudeCodeSwitchChange::AnsweringOn | ClaudeCodeSwitchChange::AnsweringOff
            if !current_settings.is_claude_code_integration_on =>
        {
            None
        }
        ClaudeCodeSwitchChange::AnsweringOn => Some(add_crest_hooks(true)),
        ClaudeCodeSwitchChange::AnsweringOff => Some(add_crest_hooks(false)),
    };
    let handed_back_request_count = match switch_change {
        ClaudeCodeSwitchChange::IntegrationOff | ClaudeCodeSwitchChange::AnsweringOff => {
            permission_answering_switch.waiting_request_count()
        }
        ClaudeCodeSwitchChange::IntegrationOn | ClaudeCodeSwitchChange::AnsweringOn => 0,
    };
    Ok(ClaudeCodeSwitchChangePreview {
        settings_change: preview_claude_code_settings_change(settings_change.as_ref())?,
        handed_back_request_count,
    })
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
