use crate::activity_core::activity_arbiter::{lock_activity_arbiter, SharedActivityArbiter};
use crate::activity_core::activity_update::ActivityUpdate;

/// The handle a source uses to talk to the core. It already knows the source's kind, so
/// a source can only ever change its own activity. Cheap to clone into other threads.
#[derive(Clone)]
pub struct ActivityPublisher {
    activity_kind: &'static str,
    shared_activity_arbiter: SharedActivityArbiter,
}

impl ActivityPublisher {
    pub fn new(activity_kind: &'static str, shared_activity_arbiter: SharedActivityArbiter) -> Self {
        Self { activity_kind, shared_activity_arbiter }
    }

    pub fn publish_activity_update(&self, activity_update: ActivityUpdate) {
        lock_activity_arbiter(&self.shared_activity_arbiter).record_activity_update(self.activity_kind, activity_update);
    }

    /// The source has nothing to show any more.
    pub fn withdraw_activity(&self) {
        lock_activity_arbiter(&self.shared_activity_arbiter).withdraw_activity(self.activity_kind);
    }
}
