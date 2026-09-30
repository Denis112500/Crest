// Names shared with Rust. Commands must match a `#[tauri::command]` function name;
// events and kinds must match `src-tauri/src/ipc_channel_names.rs`.
export const PLACE_PILL_WINDOW_COMMAND = "place_pill_window_at_top_center";
export const REVEAL_PILL_WINDOW_COMMAND = "reveal_pill_window";
export const SET_PILL_INTERACTIVE_AREA_COMMAND = "set_pill_interactive_area";
export const GET_CURRENT_PILL_PRESENTATION_COMMAND = "get_current_pill_presentation";

export const PILL_PRESENTATION_CHANGED_EVENT = "pill-presentation-changed";

export const MUSIC_ACTIVITY_KIND = "music";
