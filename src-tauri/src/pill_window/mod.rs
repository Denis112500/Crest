//! The pill's native window: where it sits, which part takes the mouse, and the OS-specific
//! overlay behavior behind a trait.

// Public (not re-exported) because `tauri::generate_handler!` needs the path where each
// `#[tauri::command]` is defined: the macro generates hidden helpers next to it.
pub mod pill_window_commands;

mod connected_display_reader;
mod pill_display_choice;
mod pill_interactive_area;
mod pill_window_placement;
mod pill_window_platform;

#[cfg(target_os = "windows")]
mod windows_native;

pub use connected_display_reader::read_connected_displays;
pub use pill_display_choice::{describe_pill_display_options, PillDisplayOption};
pub use pill_window_platform::PillWindowPlatform;

#[cfg(target_os = "windows")]
pub use windows_native::WindowsPillWindowPlatform as CurrentPlatformPillWindow;
