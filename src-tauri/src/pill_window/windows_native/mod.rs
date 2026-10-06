//! Windows' implementation of the overlay window (Win32 calls).

mod classic_frame_painting_blocker;
mod webview_memory_usage_target;
mod windows_pill_window_platform;

pub use windows_pill_window_platform::WindowsPillWindowPlatform;
