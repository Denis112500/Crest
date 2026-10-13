//! Program entry point. Kept tiny: everything lives in lib.rs (Tauri's usual layout).

// Without this, release builds on Windows open an extra console window next to the app.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Claude Code starts `crest.exe --claude-code-hook` for every hook event. That copy must end
    // before Tauri starts: as a second Crest it would otherwise open the settings window.
    let first_argument = std::env::args().nth(1);
    if first_argument.as_deref() == Some(crest_lib::CLAUDE_CODE_HOOK_ARGUMENT) {
        crest_lib::run_as_claude_code_hook();
        return;
    }
    // Run by the uninstaller (crest_installer_hooks.nsh); the exit code says whether it worked.
    if first_argument.as_deref() == Some(crest_lib::CLAUDE_CODE_HOOK_REMOVAL_ARGUMENT) {
        if let Err(removal_error) = crest_lib::remove_crest_hooks_from_claude_code_settings() {
            eprintln!("Crest could not remove its hooks from Claude Code's settings: {removal_error}");
            std::process::exit(1);
        }
        return;
    }
    crest_lib::run_crest_app()
}
