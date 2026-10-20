//! The commands the pill page calls for its window: place it, show it, hide it, set the
//! clickable area and whether it takes clicks at all.

use tauri::{State, WebviewWindow};

use crate::fullscreen_detection::FullscreenAppRecheckTrigger;
use crate::pill_window::pill_interactive_area::convert_logical_area_to_physical;
use crate::pill_window::pill_window_placement::apply_top_center_placement_to_pill_window;
use crate::pill_window::pill_window_platform::PillWindowPlatform;
use crate::pill_window::CurrentPlatformPillWindow;
use crate::pointer_watching::{PointerRestZone, SharedPillPointerZones};
use crate::user_settings_store::CrestUserSettingsStore;

/// The frontend owns the pill's dimensions, so it tells Rust how big the window must be;
/// Rust knows which monitor the user chose. Called at startup and after the choice changes.
/// Afterwards the fullscreen watcher looks again: a game may cover the new monitor, or no
/// longer cover the pill's monitor now that it moved away from it. The pointer watcher learns
/// where the window now is.
#[tauri::command]
pub fn place_pill_window_at_top_center(
    pill_window: WebviewWindow,
    settings_store: State<'_, CrestUserSettingsStore>,
    fullscreen_app_recheck_trigger: State<'_, FullscreenAppRecheckTrigger>,
    shared_pill_pointer_zones: State<'_, SharedPillPointerZones>,
    logical_width: f64,
    logical_height: f64,
) -> Result<(), String> {
    let chosen_display_name = settings_store.read_current_settings().pill_display_name;
    let pill_bounds =
        apply_top_center_placement_to_pill_window(&pill_window, logical_width, logical_height, chosen_display_name.as_deref())?;
    let window_scale_factor = pill_window.scale_factor().map_err(|error| error.to_string())?;
    shared_pill_pointer_zones.record_window_placement(
        pill_bounds.top_left_position.x,
        pill_bounds.top_left_position.y,
        window_scale_factor,
    );
    fullscreen_app_recheck_trigger.request_fullscreen_recheck();
    Ok(())
}

/// Called by the frontend once its first frame is painted, so the window never
/// appears empty or at the wrong size.
#[tauri::command]
pub fn reveal_pill_window(
    pill_window: WebviewWindow,
    shared_pill_pointer_zones: State<'_, SharedPillPointerZones>,
) -> Result<(), String> {
    CurrentPlatformPillWindow::show_pill_window_without_activating(&pill_window)?;
    shared_pill_pointer_zones.record_window_shown(true);
    Ok(())
}

/// Called by the frontend after its hide animation has finished.
#[tauri::command]
pub fn conceal_pill_window(
    pill_window: WebviewWindow,
    shared_pill_pointer_zones: State<'_, SharedPillPointerZones>,
) -> Result<(), String> {
    shared_pill_pointer_zones.record_window_shown(false);
    CurrentPlatformPillWindow::hide_pill_window(&pill_window)
}

/// The frontend tells Rust where the pill currently is inside the window (CSS pixels);
/// only that part is drawn and, unless `is_click_through` (the closed pill), takes the mouse.
#[tauri::command]
pub fn set_pill_interactive_area(
    pill_window: WebviewWindow,
    shared_pill_pointer_zones: State<'_, SharedPillPointerZones>,
    logical_left: f64,
    logical_top: f64,
    logical_width: f64,
    logical_height: f64,
    is_click_through: bool,
) -> Result<(), String> {
    let window_scale_factor = pill_window.scale_factor().map_err(|error| error.to_string())?;
    let (area_top_left, area_size) = convert_logical_area_to_physical(
        logical_left,
        logical_top,
        logical_width,
        logical_height,
        window_scale_factor,
    );
    CurrentPlatformPillWindow::set_pill_window_interactive_area(&pill_window, area_top_left, area_size, is_click_through)?;
    // The closed pill takes no mouse, so the pointer watcher stands in for its hover.
    shared_pill_pointer_zones.record_closed_pill_area(is_click_through.then(|| PointerRestZone {
        left_x: area_top_left.x,
        top_y: area_top_left.y,
        right_x: area_top_left.x + area_size.width as i32,
        bottom_y: area_top_left.y + area_size.height as i32,
    }));
    Ok(())
}
