use std::time::UNIX_EPOCH;

use serde::Serialize;
use tauri::{AppHandle, Runtime};

/// Which Crest is running, for the settings window's About line. Several builds can share a
/// version number while it's being tested (dev, the project's release build, the installed
/// copy), so the build time and the file's location tell them apart.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CrestBuildDescription {
    pub version: String,
    pub is_development_build: bool,
    /// Seconds since 1970 (the page formats it in local time). The exe's own file date: Cargo
    /// writes it when building and the installer keeps it when copying. `None` if unreadable.
    pub built_at_unix_seconds: Option<u64>,
    pub executable_path: Option<String>,
}

pub fn describe_running_crest_build<R: Runtime>(crest_app: &AppHandle<R>) -> CrestBuildDescription {
    let running_executable_path = std::env::current_exe().ok();
    let built_at_unix_seconds = running_executable_path
        .as_ref()
        .and_then(|executable_path| executable_path.metadata().ok())
        .and_then(|executable_metadata| executable_metadata.modified().ok())
        .and_then(|modified_time| modified_time.duration_since(UNIX_EPOCH).ok())
        .map(|time_since_epoch| time_since_epoch.as_secs());
    CrestBuildDescription {
        version: crest_app.package_info().version.to_string(),
        is_development_build: cfg!(debug_assertions),
        built_at_unix_seconds,
        executable_path: running_executable_path.map(|executable_path| executable_path.display().to_string()),
    }
}
