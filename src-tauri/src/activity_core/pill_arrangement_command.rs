//! Lets the pill page ask how the pill is laid out right now when it starts, because events
//! sent before the page listened are lost.

use tauri::State;

use crate::activity_core::activity_arbiter::{lock_activity_arbiter, SharedActivityArbiter};
use crate::activity_core::pill_activity_arrangement::PillArrangement;

/// The page may load after Rust already published something, and events sent before
/// the page listened are lost, so the frontend asks for the current layout once at startup.
#[tauri::command]
pub fn get_current_pill_arrangement(shared_activity_arbiter: State<'_, SharedActivityArbiter>) -> PillArrangement {
    lock_activity_arbiter(&shared_activity_arbiter).current_arrangement()
}
