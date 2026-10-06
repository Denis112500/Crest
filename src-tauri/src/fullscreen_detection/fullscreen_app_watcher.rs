//! The trait each OS implements: "tell me when a fullscreen app comes and goes".

use tauri::WebviewWindow;

/// Called with `true` when a fullscreen app (a game, a fullscreen video) comes to the front
/// on the pill's monitor, and with `false` once it's gone.
pub type FullscreenAppChangeListener = Box<dyn Fn(bool) + Send>;

/// The OS-specific part of noticing fullscreen apps, so the pill can step aside for them.
/// Each operating system gets its own implementation in its own folder.
pub trait FullscreenAppWatcher {
    /// Starts watching on a background thread that sleeps until the OS reports a change
    /// (no polling). Reports the current state once at the start, then every change.
    fn start_watching_fullscreen_apps(
        pill_window: &WebviewWindow,
        fullscreen_app_change_listener: FullscreenAppChangeListener,
    ) -> Result<(), String>;
}
