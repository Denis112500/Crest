//! Switches starting at login on or off through tauri-plugin-autostart, and always reports
//! back what Windows really has.

use tauri::{Manager, Runtime};
use tauri_plugin_autostart::ManagerExt;

/// Whether Crest is registered to start when the user logs in. Asked from the OS every time
/// (the `Run` registry key on Windows), so a change made elsewhere, like switching Crest off
/// in Task Manager's Startup apps, shows up too.
pub fn is_launch_at_login_enabled<R: Runtime>(crest_app: &impl Manager<R>) -> bool {
    crest_app.autolaunch().is_enabled().unwrap_or_else(|check_error| {
        eprintln!("Crest: could not check whether it starts at login: {check_error}");
        false
    })
}

/// Switches starting at login on or off and returns what the OS reports afterwards, so the
/// settings switch shows the truth even if the change failed.
pub fn set_launch_at_login<R: Runtime>(crest_app: &impl Manager<R>, should_launch_at_login: bool) -> bool {
    let autostart_registration = crest_app.autolaunch();
    // Registers the exe that is running now: from `tauri dev` that's the debug build, which
    // needs the Vite server, so only test it there briefly and switch it off again.
    let change_result = if should_launch_at_login {
        autostart_registration.enable()
    } else {
        autostart_registration.disable()
    };
    if let Err(change_error) = change_result {
        eprintln!("Crest: could not change whether it starts at login: {change_error}");
    }
    is_launch_at_login_enabled(crest_app)
}
