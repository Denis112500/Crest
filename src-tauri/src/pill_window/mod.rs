// Public (not re-exported) because `tauri::generate_handler!` needs the path where each
// `#[tauri::command]` is defined: the macro generates hidden helpers next to it.
pub mod pill_window_commands;

mod pill_interactive_area;
mod pill_window_placement;
mod pill_window_platform;

#[cfg(target_os = "windows")]
mod windows_native;

pub use pill_window_platform::PillWindowPlatform;

#[cfg(target_os = "windows")]
pub use windows_native::WindowsPillWindowPlatform as CurrentPlatformPillWindow;
