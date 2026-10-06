//! The message a source sends to the core: priority, what kind of presence it has, when to
//! grab attention, plus the source's own payload that only its frontend view understands.

use serde::Serialize;

/// What an activity is, as opposed to where it goes in the pill (that's the arrangement's
/// job). Sources describe themselves; one pure function decides the layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ActivityPresence {
    /// Still there but not happening (music paused): shown only while nothing else is
    /// ongoing, and the pill hides after a while.
    Lingering,
    /// Happening now (music playing): keeps the pill on screen and can share it as the
    /// companion segment.
    Ongoing,
    /// Needs the user now (a permission request): takes over the whole pill until it ends.
    Alert,
}

/// What an activity source reports to the core. The core only reads the first three
/// fields; the payload is passed to the frontend untouched, so a new source never needs
/// a change in the core.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityUpdate {
    /// Between activities with the same presence, the highest priority goes first.
    pub display_priority: u8,
    pub activity_presence: ActivityPresence,
    /// Changes whenever the pill should grab attention (for music: a new track).
    pub attention_key: String,
    /// The source's own data for its frontend views, as JSON.
    pub activity_payload: serde_json::Value,
}
