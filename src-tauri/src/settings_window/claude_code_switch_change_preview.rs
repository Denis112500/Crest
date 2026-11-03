//! What flipping one of the Claude Code card's switches would change, shown in the card before
//! the user confirms: the lines of Claude Code's settings file, and the requests handed back.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::activity_sources::claude_code::{
    preview_claude_code_settings_change, ClaudeCodePermissionAnsweringSwitch, ClaudeCodeSettingsChange,
    ClaudeCodeSettingsChangePreview,
};
use crate::user_settings_store::{CrestUserSettings, CrestUserSettingsStore};

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
    describe_claude_code_switch_change(&settings_store.read_current_settings(), &permission_answering_switch, switch_change)
}

/// Also answers a confirmation that came too late, when the file changed since its preview.
pub fn describe_claude_code_switch_change(
    current_settings: &CrestUserSettings,
    permission_answering_switch: &ClaudeCodePermissionAnsweringSwitch,
    switch_change: ClaudeCodeSwitchChange,
) -> Result<ClaudeCodeSwitchChangePreview, String> {
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
