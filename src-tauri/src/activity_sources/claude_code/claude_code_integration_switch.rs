//! Switching the Claude Code integration on and off: Claude Code's hooks and Crest's listening
//! source always change together. Used by the settings window and at startup.

use crate::activity_core::{ActivitySourceRegistry, SharedActivityArbiter};
use crate::activity_sources::claude_code::claude_code_activity_source::ClaudeCodeActivitySource;
use crate::activity_sources::claude_code::claude_code_settings_file::{
    add_crest_hooks_to_claude_code_settings, remove_crest_hooks_from_claude_code_settings,
};
use crate::ipc_channel_names::CLAUDE_CODE_ACTIVITY_KIND;

/// Also used at every start while the integration is on: the hooks then point at the Crest
/// that is running now (another install folder, or a dev build).
pub fn turn_claude_code_integration_on(
    activity_source_registry: &mut ActivitySourceRegistry,
    shared_activity_arbiter: &SharedActivityArbiter,
) -> Result<(), String> {
    let crest_executable_path = std::env::current_exe().map_err(|error| error.to_string())?;
    if !activity_source_registry.is_registered(CLAUDE_CODE_ACTIVITY_KIND) {
        activity_source_registry.start_and_register(Box::<ClaudeCodeActivitySource>::default(), shared_activity_arbiter)?;
    }
    add_crest_hooks_to_claude_code_settings(&crest_executable_path).inspect_err(|_| {
        activity_source_registry.stop_and_unregister(CLAUDE_CODE_ACTIVITY_KIND);
    })
}

/// Stops listening even if Claude Code's settings can't be changed; the error then says why
/// its hooks are still there (they'd only find no Crest listening, which costs nothing).
pub fn turn_claude_code_integration_off(activity_source_registry: &mut ActivitySourceRegistry) -> Result<(), String> {
    activity_source_registry.stop_and_unregister(CLAUDE_CODE_ACTIVITY_KIND);
    remove_crest_hooks_from_claude_code_settings()
}
