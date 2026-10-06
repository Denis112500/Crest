//! The trait each OS implements: "tell me when a fullscreen app comes and goes", plus a way
//! to ask for a fresh look when something changed that the OS doesn't announce.

use tauri::WebviewWindow;

/// Called with `true` when a fullscreen app (a game, a fullscreen video) comes to the front
/// on the pill's monitor, and with `false` once it's gone.
pub type FullscreenAppChangeListener = Box<dyn Fn(bool) + Send>;

/// Asks the running watcher to check the front window again. Needed when the pill moves to
/// another monitor: the OS only reports fullscreen apps coming and going, not the pill moving,
/// so the old answer ("a game covers the pill's monitor") would stick until the next report.
pub struct FullscreenAppRecheckTrigger {
    request_recheck: Box<dyn Fn() + Send + Sync>,
}

impl FullscreenAppRecheckTrigger {
    pub fn new(request_recheck: Box<dyn Fn() + Send + Sync>) -> Self {
        Self { request_recheck }
    }

    /// For when the watcher couldn't start: there is nothing to ask.
    pub fn without_watcher() -> Self {
        Self { request_recheck: Box::new(|| {}) }
    }

    pub fn request_fullscreen_recheck(&self) {
        (self.request_recheck)();
    }
}

/// The OS-specific part of noticing fullscreen apps, so the pill can step aside for them.
/// Each operating system gets its own implementation in its own folder.
pub trait FullscreenAppWatcher {
    /// Starts watching on a background thread that sleeps until the OS reports a change
    /// (no polling). Reports the current state once at the start, then every change.
    fn start_watching_fullscreen_apps(
        pill_window: &WebviewWindow,
        fullscreen_app_change_listener: FullscreenAppChangeListener,
    ) -> Result<FullscreenAppRecheckTrigger, String>;
}
