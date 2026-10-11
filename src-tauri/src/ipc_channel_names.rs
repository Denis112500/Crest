// Names shared with the frontend. Each must match `src/ipc/ipcChannelNames.ts` exactly.

/// Event carrying the pill's layout now: alert, main and companion activity (each may be `null`).
pub const PILL_ARRANGEMENT_CHANGED_EVENT: &str = "pill-arrangement-changed";

/// Event carrying whether the pill should be on screen and whether a fullscreen app is in front.
pub const PILL_VISIBILITY_CHANGED_EVENT: &str = "pill-visibility-changed";

/// Event telling the pill window the user chose another monitor, so it places itself again.
pub const PILL_DISPLAY_CHANGED_EVENT: &str = "pill-display-changed";

/// Activity kind of the music source; the frontend picks its views by this name.
pub const MUSIC_ACTIVITY_KIND: &str = "music";

/// Actions the music activity understands (sent by the expanded view's buttons).
pub const MUSIC_TOGGLE_PLAY_PAUSE_ACTION: &str = "toggle-play-pause";
pub const MUSIC_NEXT_TRACK_ACTION: &str = "next-track";
pub const MUSIC_PREVIOUS_TRACK_ACTION: &str = "previous-track";

/// Activity kind of the Claude Code source.
pub const CLAUDE_CODE_ACTIVITY_KIND: &str = "claude-code";

/// Actions the Claude Code alert sends, followed by the request's id ("allow-permission:7"), so a
/// click can only answer the request it was shown for.
pub const CLAUDE_CODE_ALLOW_PERMISSION_ACTION_PREFIX: &str = "allow-permission:";
pub const CLAUDE_CODE_DENY_PERMISSION_ACTION_PREFIX: &str = "deny-permission:";
