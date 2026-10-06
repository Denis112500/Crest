//! Windows: uses the shell's own fullscreen signal (the one that hides the taskbar) by
//! registering an invisible "appbar".

mod appbar_fullscreen_app_watcher;
mod appbar_watcher_thread_state;
mod appbar_watcher_window_procedure;
mod front_window_change_hook;
mod front_window_fullscreen_check;

pub use appbar_fullscreen_app_watcher::AppbarFullscreenAppWatcher;
