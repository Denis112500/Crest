// Names shared with the frontend. Each must match `src/ipc/ipcChannelNames.ts` exactly.

/// Event carrying what the pill should show now (`null` when there is nothing).
pub const PILL_PRESENTATION_CHANGED_EVENT: &str = "pill-presentation-changed";

/// Activity kind of the music source; the frontend picks its views by this name.
pub const MUSIC_ACTIVITY_KIND: &str = "music";
