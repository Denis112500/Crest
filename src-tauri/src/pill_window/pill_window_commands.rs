use tauri::WebviewWindow;

use crate::pill_window::pill_window_placement::apply_top_center_placement_to_pill_window;
use crate::pill_window::pill_window_platform::PillWindowPlatform;
use crate::pill_window::CurrentPlatformPillWindow;

/// The frontend owns the pill's dimensions, so it tells Rust how big the window must be.
#[tauri::command]
pub fn place_pill_window_at_top_center(
    pill_window: WebviewWindow,
    logical_width: f64,
    logical_height: f64,
) -> Result<(), String> {
    apply_top_center_placement_to_pill_window(&pill_window, logical_width, logical_height)
}

/// Called by the frontend once its first frame is painted, so the window never
/// appears empty or at the wrong size.
#[tauri::command]
pub fn reveal_pill_window(pill_window: WebviewWindow) -> Result<(), String> {
    CurrentPlatformPillWindow::show_pill_window_without_activating(&pill_window)
}
