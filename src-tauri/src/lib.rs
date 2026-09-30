pub fn run_crest_app() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("Crest failed to start the Tauri application");
}
