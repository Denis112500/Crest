use crate::activity_core::activity_publisher::ActivityPublisher;

/// A plugin that has something to show in the pill: music now; timers or Claude Code
/// events later. Adding one means implementing this trait and starting it in `lib.rs`.
pub trait ActivitySource {
    /// Unique name; the frontend uses it to pick the views that draw this activity.
    fn activity_kind(&self) -> &'static str;

    /// Starts reporting in the background through `activity_publisher` and returns immediately.
    fn start_publishing(&mut self, activity_publisher: ActivityPublisher) -> Result<(), String>;
}
