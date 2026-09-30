use tauri::State;

use crate::activity_core::pill_visibility_controller::PillVisibilityController;

/// Like the presentation, the visibility may be decided before the page listens, so the
/// frontend asks once at startup.
#[tauri::command]
pub fn get_current_pill_visibility(pill_visibility_controller: State<'_, PillVisibilityController>) -> bool {
    pill_visibility_controller.is_pill_visible()
}
