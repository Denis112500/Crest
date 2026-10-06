//! The settings window: opening it, and the commands only its page may call.

// Public (not re-exported) because `tauri::generate_handler!` needs the path where each
// `#[tauri::command]` is defined.
pub mod claude_code_integration_commands;
pub mod settings_window_commands;

mod allowed_player_options;
mod crest_build_description;
mod settings_window_opener;

pub use settings_window_opener::open_or_focus_settings_window;
