//! Lets the pill page ask what's showing right now when it starts, because events sent
//! before the page listened are lost.

use tauri::State;

use crate::activity_core::activity_arbiter::{lock_activity_arbiter, PillPresentation, SharedActivityArbiter};

/// The page may load after Rust already published something, and events sent before
/// the page listened are lost, so the frontend asks for the current state once at startup.
#[tauri::command]
pub fn get_current_pill_presentation(
    shared_activity_arbiter: State<'_, SharedActivityArbiter>,
) -> Option<PillPresentation> {
    lock_activity_arbiter(&shared_activity_arbiter).current_presentation()
}
