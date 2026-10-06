//! Lets the pill page ask whether it should be on screen when it starts (same reason as the
//! presentation command: early events would be lost).

use tauri::State;

use crate::activity_core::pill_visibility_controller::{PillVisibility, PillVisibilityController};

/// Like the presentation, the visibility may be decided before the page listens, so the
/// frontend asks once at startup.
#[tauri::command]
pub fn get_current_pill_visibility(pill_visibility_controller: State<'_, PillVisibilityController>) -> PillVisibility {
    pill_visibility_controller.current_pill_visibility()
}
