//! Lets the pill page end the opening from the top edge once the pill closed again (P.3).

use tauri::State;

use crate::activity_core::pill_visibility_controller::PillVisibilityController;

/// The pill the pointer brought down from the top edge closed (the pointer left it); with
/// nothing on, Rust then hides it again.
#[tauri::command]
pub fn release_pill_top_edge_opening(pill_visibility_controller: State<'_, PillVisibilityController>) {
    pill_visibility_controller.release_top_edge_opening();
}
