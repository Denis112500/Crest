//! The message a source sends to the core: priority, still going or not, when to grab
//! attention, plus the source's own payload that only its frontend view understands.

use serde::Serialize;

/// What an activity source reports to the core. The core only reads the first three
/// fields; the payload is passed to the frontend untouched, so a new source never needs
/// a change in the core.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityUpdate {
    /// When several sources have something to show, the highest priority wins.
    pub display_priority: u8,
    /// Still happening (music playing) as opposed to lingering (music paused).
    pub is_ongoing: bool,
    /// Changes whenever the pill should grab attention (for music: a new track).
    pub attention_key: String,
    /// The source's own data for its frontend views, as JSON.
    pub activity_payload: serde_json::Value,
}
