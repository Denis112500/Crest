//! The trait each OS implements: "show this folder in the system's file manager".

use std::path::Path;

pub trait FolderReveal {
    /// Returns once the file manager was started; it opens on its own, Crest doesn't wait for it.
    fn reveal_folder(revealed_folder: &Path) -> Result<(), String>;
}
