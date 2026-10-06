//! The command the pill page calls when the companion segment is clicked: that activity
//! becomes the main one until it stops.

use tauri::State;

use crate::activity_core::activity_arbiter::{lock_activity_arbiter, SharedActivityArbiter};

#[tauri::command]
pub fn focus_activity(shared_activity_arbiter: State<'_, SharedActivityArbiter>, activity_kind: String) -> Result<(), String> {
    lock_activity_arbiter(&shared_activity_arbiter).focus_activity(&activity_kind)
}
