//! Program entry point. Kept tiny: everything lives in lib.rs (Tauri's usual layout).

// Without this, release builds on Windows open an extra console window next to the app.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Claude Code starts `crest.exe --claude-code-hook` for every hook event. That copy must end
    // before Tauri starts: as a second Crest it would otherwise open the settings window.
    if std::env::args().nth(1).as_deref() == Some(crest_lib::CLAUDE_CODE_HOOK_ARGUMENT) {
        crest_lib::run_as_claude_code_hook();
        return;
    }
    crest_lib::run_crest_app()
}
