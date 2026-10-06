//! Hides the pill while a fullscreen app (a game, a video) covers its monitor. The OS-specific
//! part sits behind a trait, like the other platform code.

mod fullscreen_app_watcher;

#[cfg(target_os = "windows")]
mod windows_shell_appbar;

pub use fullscreen_app_watcher::{FullscreenAppRecheckTrigger, FullscreenAppWatcher};

#[cfg(target_os = "windows")]
pub use windows_shell_appbar::AppbarFullscreenAppWatcher as CurrentPlatformFullscreenAppWatcher;
