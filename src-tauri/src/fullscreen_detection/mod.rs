mod fullscreen_app_watcher;

#[cfg(target_os = "windows")]
mod windows_shell_appbar;

pub use fullscreen_app_watcher::FullscreenAppWatcher;

#[cfg(target_os = "windows")]
pub use windows_shell_appbar::AppbarFullscreenAppWatcher as CurrentPlatformFullscreenAppWatcher;
