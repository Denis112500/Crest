use tauri::WebviewWindow;

/// The OS-specific part of making the pill behave like an overlay rather than an app window.
/// Each operating system gets its own implementation in its own folder.
pub trait PillWindowPlatform {
    /// Called once at startup, while the window is still hidden: keep it out of the
    /// taskbar and Alt+Tab, and make sure clicking it never takes keyboard focus.
    fn prepare_pill_window_as_overlay(pill_window: &WebviewWindow) -> Result<(), String>;

    /// Shows the window without taking focus away from the app the user is typing in.
    fn show_pill_window_without_activating(pill_window: &WebviewWindow) -> Result<(), String>;
}
