// Names shared with the frontend. Each must match `src/ipc/ipcChannelNames.ts` exactly.

/// Event carrying what the pill should show now (`null` when there is nothing).
pub const PILL_PRESENTATION_CHANGED_EVENT: &str = "pill-presentation-changed";

/// Activity kind of the music source; the frontend picks its views by this name.
pub const MUSIC_ACTIVITY_KIND: &str = "music";

/// Actions the music activity understands (sent by the expanded view's buttons).
pub const MUSIC_TOGGLE_PLAY_PAUSE_ACTION: &str = "toggle-play-pause";
pub const MUSIC_NEXT_TRACK_ACTION: &str = "next-track";
pub const MUSIC_PREVIOUS_TRACK_ACTION: &str = "previous-track";
