use tauri::WebviewWindow;

use crate::pill_window::pill_interactive_area::convert_logical_area_to_physical;
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

/// Called by the frontend after its hide animation has finished.
#[tauri::command]
pub fn conceal_pill_window(pill_window: WebviewWindow) -> Result<(), String> {
    CurrentPlatformPillWindow::hide_pill_window(&pill_window)
}

/// The frontend tells Rust where the pill currently is inside the window (CSS pixels);
/// only that part takes the mouse.
#[tauri::command]
pub fn set_pill_interactive_area(
    pill_window: WebviewWindow,
    logical_left: f64,
    logical_top: f64,
    logical_width: f64,
    logical_height: f64,
) -> Result<(), String> {
    let window_scale_factor = pill_window.scale_factor().map_err(|error| error.to_string())?;
    let (area_top_left, area_size) = convert_logical_area_to_physical(
        logical_left,
        logical_top,
        logical_width,
        logical_height,
        window_scale_factor,
    );
    CurrentPlatformPillWindow::set_pill_window_interactive_area(&pill_window, area_top_left, area_size)
}
