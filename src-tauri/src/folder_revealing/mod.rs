//! Showing a folder in the system's file manager. The OS-specific part sits behind a trait, like
//! the other platform code. Used by the settings window's "Show the copies".

mod folder_reveal;

#[cfg(target_os = "windows")]
mod windows_explorer_folder_reveal;

pub use folder_reveal::FolderReveal;

#[cfg(target_os = "windows")]
pub use windows_explorer_folder_reveal::ExplorerFolderReveal as CurrentPlatformFolderReveal;
