use crate::activity_core::activity_publisher::ActivityPublisher;

/// A plugin that has something to show in the pill: music now; timers or Claude Code
/// events later. Adding one means implementing this trait and registering it in `lib.rs`.
/// `Send` because the app keeps sources where commands from any thread can reach them.
pub trait ActivitySource: Send {
    /// Unique name; the frontend uses it to pick the views that draw this activity.
    fn activity_kind(&self) -> &'static str;

    /// Starts reporting in the background through `activity_publisher` and returns immediately.
    fn start_publishing(&mut self, activity_publisher: ActivityPublisher) -> Result<(), String>;

    /// Handles a user action from this activity's views (e.g. a button). The action names
    /// are this source's own vocabulary; the core only routes them.
    fn perform_activity_action(&self, activity_action: &str) -> Result<(), String>;
}
