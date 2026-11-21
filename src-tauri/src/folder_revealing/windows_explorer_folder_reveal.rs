//! Windows: File Explorer opened on the folder.

use std::path::Path;
use std::process::Command;

use crate::folder_revealing::folder_reveal::FolderReveal;

pub struct ExplorerFolderReveal;

impl FolderReveal for ExplorerFolderReveal {
    /// Not waited for: `explorer.exe` hands the window to the running Explorer and exits with 1
    /// even when it worked, so its exit code says nothing.
    fn reveal_folder(revealed_folder: &Path) -> Result<(), String> {
        Command::new("explorer.exe")
            .arg(revealed_folder)
            .spawn()
            .map(|_| ())
            .map_err(|error| format!("could not open File Explorer: {error}"))
    }
}
