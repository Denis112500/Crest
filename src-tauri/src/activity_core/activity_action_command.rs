//! The command the pill page calls when one of an activity's buttons is pressed; it only
//! routes the action name, the source decides what it means.

use std::sync::PoisonError;

use tauri::State;

use crate::activity_core::activity_source_registry::SharedActivitySourceRegistry;

/// A button in some activity's view was pressed. The core doesn't interpret the action;
/// it hands it to the source that owns `activity_kind`.
#[tauri::command]
pub fn perform_activity_action(
    shared_activity_source_registry: State<'_, SharedActivitySourceRegistry>,
    activity_kind: String,
    activity_action: String,
) -> Result<(), String> {
    shared_activity_source_registry
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .perform_activity_action(&activity_kind, &activity_action)
}
