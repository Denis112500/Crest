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
/// tray checkmark shows the truth even if the switch failed.
pub fn toggle_launch_at_login<R: Runtime>(crest_app: &impl Manager<R>) -> bool {
    let autostart_registration = crest_app.autolaunch();
    // Registers the exe that is running now: from `tauri dev` that's the debug build, which
    // needs the Vite server, so only test it there briefly and switch it off again.
    let toggle_result = if is_launch_at_login_enabled(crest_app) {
        autostart_registration.disable()
    } else {
        autostart_registration.enable()
    };
    if let Err(toggle_error) = toggle_result {
        eprintln!("Crest: could not change whether it starts at login: {toggle_error}");
    }
    is_launch_at_login_enabled(crest_app)
}
