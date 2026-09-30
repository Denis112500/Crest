use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use serde::Serialize;

use crate::activity_core::activity_update::ActivityUpdate;

/// What the pill shows: the winning activity and which source it came from.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PillPresentation {
    pub activity_kind: &'static str,
    #[serde(flatten)]
    pub activity_update: ActivityUpdate,
}

/// Called whenever what the pill should show changes; `None` means nothing to show.
pub type PillPresentationListener = Box<dyn Fn(Option<&PillPresentation>) + Send>;

/// Sources report from their own threads, so the arbiter is shared behind a mutex.
pub type SharedActivityArbiter = Arc<Mutex<ActivityArbiter>>;

struct RecordedActivity {
    activity_update: ActivityUpdate,
    arrival_order: u64,
}

/// The core of Crest: keeps the latest update from every source and decides which one
/// the pill shows. It never looks inside an activity's payload.
pub struct ActivityArbiter {
    recorded_activity_by_kind: HashMap<&'static str, RecordedActivity>,
    next_arrival_order: u64,
    current_presentation: Option<PillPresentation>,
    pill_presentation_listener: PillPresentationListener,
}

impl ActivityArbiter {
    pub fn new(pill_presentation_listener: PillPresentationListener) -> Self {
        Self {
            recorded_activity_by_kind: HashMap::new(),
            next_arrival_order: 0,
            current_presentation: None,
            pill_presentation_listener,
        }
    }

    pub fn record_activity_update(&mut self, activity_kind: &'static str, activity_update: ActivityUpdate) {
        let arrival_order = self.next_arrival_order;
        self.next_arrival_order += 1;
        self.recorded_activity_by_kind
            .insert(activity_kind, RecordedActivity { activity_update, arrival_order });
        self.present_winning_activity();
    }

    pub fn withdraw_activity(&mut self, activity_kind: &'static str) {
        self.recorded_activity_by_kind.remove(activity_kind);
        self.present_winning_activity();
    }

    pub fn current_presentation(&self) -> Option<PillPresentation> {
        self.current_presentation.clone()
    }

    /// Highest priority wins; between equal priorities, the most recent update wins.
    fn present_winning_activity(&mut self) {
        let winning_presentation = self
            .recorded_activity_by_kind
            .iter()
            .max_by_key(|(_, recorded)| (recorded.activity_update.display_priority, recorded.arrival_order))
            .map(|(activity_kind, recorded)| PillPresentation {
                activity_kind,
                activity_update: recorded.activity_update.clone(),
            });
        if winning_presentation != self.current_presentation {
            (self.pill_presentation_listener)(winning_presentation.as_ref());
            self.current_presentation = winning_presentation;
        }
    }
}

/// A panic on another thread while holding the lock must not take the pill down with it;
/// the arbiter's data stays consistent because every method finishes its update first.
pub fn lock_activity_arbiter(shared_activity_arbiter: &SharedActivityArbiter) -> MutexGuard<'_, ActivityArbiter> {
    shared_activity_arbiter.lock().unwrap_or_else(PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn activity_update_with_priority(display_priority: u8, attention_key: &str) -> ActivityUpdate {
        ActivityUpdate {
            display_priority,
            is_ongoing: true,
            attention_key: attention_key.to_string(),
            activity_payload: serde_json::Value::Null,
        }
    }

    fn arbiter_recording_presented_kinds() -> (ActivityArbiter, Arc<Mutex<Vec<Option<&'static str>>>>) {
        let presented_kinds = Arc::new(Mutex::new(Vec::new()));
        let presented_kinds_for_listener = Arc::clone(&presented_kinds);
        let activity_arbiter = ActivityArbiter::new(Box::new(move |pill_presentation| {
            presented_kinds_for_listener
                .lock()
                .unwrap()
                .push(pill_presentation.map(|presentation| presentation.activity_kind));
        }));
        (activity_arbiter, presented_kinds)
    }

    #[test]
    fn shows_highest_priority_and_falls_back_when_it_is_withdrawn() {
        let (mut activity_arbiter, presented_kinds) = arbiter_recording_presented_kinds();
        activity_arbiter.record_activity_update("music", activity_update_with_priority(50, "song"));
        activity_arbiter.record_activity_update("timer", activity_update_with_priority(80, "tick"));
        activity_arbiter.record_activity_update("music", activity_update_with_priority(50, "next song"));
        activity_arbiter.withdraw_activity("timer");
        activity_arbiter.withdraw_activity("music");
        assert_eq!(
            *presented_kinds.lock().unwrap(),
            vec![Some("music"), Some("timer"), Some("music"), None]
        );
    }

    #[test]
    fn does_not_notify_when_nothing_changed() {
        let (mut activity_arbiter, presented_kinds) = arbiter_recording_presented_kinds();
        activity_arbiter.record_activity_update("music", activity_update_with_priority(50, "song"));
        activity_arbiter.record_activity_update("music", activity_update_with_priority(50, "song"));
        activity_arbiter.withdraw_activity("timer");
        assert_eq!(presented_kinds.lock().unwrap().len(), 1);
    }

    #[test]
    fn most_recent_wins_between_equal_priorities() {
        let (mut activity_arbiter, _) = arbiter_recording_presented_kinds();
        activity_arbiter.record_activity_update("music", activity_update_with_priority(50, "song"));
        activity_arbiter.record_activity_update("podcast", activity_update_with_priority(50, "episode"));
        assert_eq!(activity_arbiter.current_presentation().map(|presentation| presentation.activity_kind), Some("podcast"));
    }
}
