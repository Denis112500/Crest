//! Watching a folder for written files. The OS-specific part sits behind a trait, like the other
//! platform code. Used to notice Claude Code's interrupt line in a session transcript.

mod folder_change_watch;

#[cfg(target_os = "windows")]
mod windows_change_notification;

pub use folder_change_watch::FolderChangeWatch;

#[cfg(target_os = "windows")]
pub use windows_change_notification::ChangeNotificationFolderWatch as CurrentPlatformFolderChangeWatch;
