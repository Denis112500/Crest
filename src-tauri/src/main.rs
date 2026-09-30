// Without this, release builds on Windows open an extra console window next to the app.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    crest_lib::run_crest_app()
}
