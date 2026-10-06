// Listing Crest's own commands here makes Tauri generate an `allow-<command>` permission for
// each, and then a window may only call the commands its capability file grants. Without the
// list, every page could call every command (e.g. the pill page could switch autostart).
// Every `#[tauri::command]` registered in lib.rs must be listed, or no window can call it.
const CREST_COMMAND_NAMES: &[&str] = &[
    "place_pill_window_at_top_center",
    "reveal_pill_window",
    "conceal_pill_window",
    "set_pill_interactive_area",
    "get_current_pill_arrangement",
    "focus_activity",
    "get_current_pill_visibility",
    "perform_activity_action",
    "read_crest_build_description",
    "read_launch_at_login_setting",
    "change_launch_at_login_setting",
    "list_pill_display_options",
    "choose_pill_display",
    "list_allowed_player_options",
    "change_allowed_players",
    "change_show_every_player",
];

fn main() {
    let crest_build_attributes =
        tauri_build::Attributes::new().app_manifest(tauri_build::AppManifest::new().commands(CREST_COMMAND_NAMES));
    tauri_build::try_build(crest_build_attributes).expect("tauri-build failed");
}
