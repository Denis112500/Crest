//! The trait each OS implements: "tell me when a file in this folder is written".

use std::path::Path;

/// A running watch; dropping it stops the watch and its thread.
pub trait FolderChangeWatch: Sized + Send {
    /// Calls `on_folder_change` (on the watch's own thread, which sleeps until the OS reports a
    /// change: no polling) whenever a file in `watched_folder` grows or is written.
    fn start_watching(watched_folder: &Path, on_folder_change: Box<dyn Fn() + Send>) -> Result<Self, String>;
}
