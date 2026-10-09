// Names shared with Rust. Commands must match a `#[tauri::command]` function name;
// events, kinds and actions must match `src-tauri/src/ipc_channel_names.rs`.
export const PLACE_PILL_WINDOW_COMMAND = "place_pill_window_at_top_center";
export const REVEAL_PILL_WINDOW_COMMAND = "reveal_pill_window";
export const CONCEAL_PILL_WINDOW_COMMAND = "conceal_pill_window";
export const SET_PILL_INTERACTIVE_AREA_COMMAND = "set_pill_interactive_area";
export const GET_CURRENT_PILL_ARRANGEMENT_COMMAND = "get_current_pill_arrangement";
export const GET_CURRENT_PILL_VISIBILITY_COMMAND = "get_current_pill_visibility";
export const PERFORM_ACTIVITY_ACTION_COMMAND = "perform_activity_action";
export const FOCUS_ACTIVITY_COMMAND = "focus_activity";
// Settings window only (see src-tauri/capabilities/settings_window.json).
export const READ_CREST_BUILD_DESCRIPTION_COMMAND = "read_crest_build_description";
export const READ_LAUNCH_AT_LOGIN_SETTING_COMMAND = "read_launch_at_login_setting";
export const CHANGE_LAUNCH_AT_LOGIN_SETTING_COMMAND = "change_launch_at_login_setting";
export const LIST_PILL_DISPLAY_OPTIONS_COMMAND = "list_pill_display_options";
export const CHOOSE_PILL_DISPLAY_COMMAND = "choose_pill_display";
export const LIST_ALLOWED_PLAYER_OPTIONS_COMMAND = "list_allowed_player_options";
export const CHANGE_ALLOWED_PLAYERS_COMMAND = "change_allowed_players";
export const CHANGE_SHOW_EVERY_PLAYER_COMMAND = "change_show_every_player";
export const READ_CLAUDE_CODE_INTEGRATION_SETTING_COMMAND = "read_claude_code_integration_setting";
export const CHANGE_CLAUDE_CODE_INTEGRATION_SETTING_COMMAND = "change_claude_code_integration_setting";

export const PILL_ARRANGEMENT_CHANGED_EVENT = "pill-arrangement-changed";
export const PILL_VISIBILITY_CHANGED_EVENT = "pill-visibility-changed";
export const PILL_DISPLAY_CHANGED_EVENT = "pill-display-changed";

export const MUSIC_ACTIVITY_KIND = "music";
export const MUSIC_TOGGLE_PLAY_PAUSE_ACTION = "toggle-play-pause";
export const MUSIC_NEXT_TRACK_ACTION = "next-track";
export const MUSIC_PREVIOUS_TRACK_ACTION = "previous-track";

export const CLAUDE_CODE_ACTIVITY_KIND = "claude-code";
