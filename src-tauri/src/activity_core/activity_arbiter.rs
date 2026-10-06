//! Keeps the latest update of every source, lets `arrange_pill_activities` lay them out and
//! tells the frontend only when the layout changes. The heart of the core.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use crate::activity_core::activity_update::{ActivityPresence, ActivityUpdate};
use crate::activity_core::pill_activity_arrangement::{arrange_pill_activities, PillArrangement, RecordedActivity};

/// Called whenever the pill's layout changes; an empty arrangement means nothing to show.
pub type PillArrangementListener = Box<dyn Fn(&PillArrangement) + Send>;

/// Sources report from their own threads, so the arbiter is shared behind a mutex.
pub type SharedActivityArbiter = Arc<Mutex<ActivityArbiter>>;

/// The core of Crest: keeps the latest update from every source and decides how the pill
/// shows them. It never looks inside an activity's payload.
pub struct ActivityArbiter {
    recorded_activity_by_kind: HashMap<&'static str, RecordedActivity>,
    next_arrival_order: u64,
    /// The activity the user clicked to make it the main one; remembered until it ends.
    focused_activity_kind: Option<&'static str>,
    current_arrangement: PillArrangement,
    pill_arrangement_listener: PillArrangementListener,
}

impl ActivityArbiter {
    pub fn new(pill_arrangement_listener: PillArrangementListener) -> Self {
        Self {
            recorded_activity_by_kind: HashMap::new(),
            next_arrival_order: 0,
            focused_activity_kind: None,
            current_arrangement: PillArrangement::default(),
            pill_arrangement_listener,
        }
    }

    pub fn record_activity_update(&mut self, activity_kind: &'static str, activity_update: ActivityUpdate) {
        // A repeated identical update isn't news: counting it as "most recent" would let a source
        // that re-sends the same state push a newer lingering activity out of the main place.
        let is_repeat = self
            .recorded_activity_by_kind
            .get(activity_kind)
            .is_some_and(|recorded| recorded.activity_update == activity_update);
        if is_repeat {
            return;
        }
        let arrival_order = self.next_arrival_order;
        self.next_arrival_order += 1;
        // Focus lasts while the activity is happening; once it pauses, the usual order returns.
        if self.focused_activity_kind == Some(activity_kind)
            && activity_update.activity_presence != ActivityPresence::Ongoing
        {
            self.focused_activity_kind = None;
        }
        self.recorded_activity_by_kind
            .insert(activity_kind, RecordedActivity { activity_kind, activity_update, arrival_order });
        self.present_current_arrangement();
    }

    pub fn withdraw_activity(&mut self, activity_kind: &'static str) {
        self.recorded_activity_by_kind.remove(activity_kind);
        if self.focused_activity_kind == Some(activity_kind) {
            self.focused_activity_kind = None;
        }
        self.present_current_arrangement();
    }

    /// The user clicked the companion segment: that activity becomes the main one.
    pub fn focus_activity(&mut self, activity_kind: &str) -> Result<(), String> {
        let (&known_activity_kind, _) = self
            .recorded_activity_by_kind
            .get_key_value(activity_kind)
            .ok_or_else(|| format!("no activity of kind \"{activity_kind}\" is showing"))?;
        self.focused_activity_kind = Some(known_activity_kind);
        self.present_current_arrangement();
        Ok(())
    }

    pub fn current_arrangement(&self) -> PillArrangement {
        self.current_arrangement.clone()
    }

    fn present_current_arrangement(&mut self) {
        let recorded_activities: Vec<&RecordedActivity> = self.recorded_activity_by_kind.values().collect();
        let new_arrangement = arrange_pill_activities(&recorded_activities, self.focused_activity_kind);
        if new_arrangement != self.current_arrangement {
            (self.pill_arrangement_listener)(&new_arrangement);
            self.current_arrangement = new_arrangement;
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

    fn activity_update(activity_presence: ActivityPresence, display_priority: u8, attention_key: &str) -> ActivityUpdate {
        ActivityUpdate {
            display_priority,
            activity_presence,
            attention_key: attention_key.to_string(),
            activity_payload: serde_json::Value::Null,
        }
    }

    type RecordedMainKinds = Arc<Mutex<Vec<Option<&'static str>>>>;

    fn arbiter_recording_main_kinds() -> (ActivityArbiter, RecordedMainKinds) {
        let main_kinds = Arc::new(Mutex::new(Vec::new()));
        let main_kinds_for_listener = Arc::clone(&main_kinds);
        let activity_arbiter = ActivityArbiter::new(Box::new(move |pill_arrangement| {
            main_kinds_for_listener
                .lock()
                .unwrap()
                .push(pill_arrangement.main_activity.as_ref().map(|main_activity| main_activity.activity_kind));
        }));
        (activity_arbiter, main_kinds)
    }

    fn main_kind(activity_arbiter: &ActivityArbiter) -> Option<&'static str> {
        activity_arbiter.current_arrangement().main_activity.map(|main_activity| main_activity.activity_kind)
    }

    #[test]
    fn shows_highest_priority_and_falls_back_when_it_is_withdrawn() {
        let (mut activity_arbiter, main_kinds) = arbiter_recording_main_kinds();
        activity_arbiter.record_activity_update("music", activity_update(ActivityPresence::Ongoing, 50, "song"));
        activity_arbiter.record_activity_update("timer", activity_update(ActivityPresence::Ongoing, 80, "tick"));
        activity_arbiter.record_activity_update("music", activity_update(ActivityPresence::Ongoing, 50, "next song"));
        activity_arbiter.withdraw_activity("timer");
        activity_arbiter.withdraw_activity("music");
        // The third update changes the companion's track, which is a new arrangement too.
        assert_eq!(
            *main_kinds.lock().unwrap(),
            vec![Some("music"), Some("timer"), Some("timer"), Some("music"), None]
        );
    }

    #[test]
    fn does_not_notify_when_nothing_changed() {
        let (mut activity_arbiter, main_kinds) = arbiter_recording_main_kinds();
        activity_arbiter.record_activity_update("music", activity_update(ActivityPresence::Ongoing, 50, "song"));
        activity_arbiter.record_activity_update("music", activity_update(ActivityPresence::Ongoing, 50, "song"));
        activity_arbiter.withdraw_activity("timer");
        assert_eq!(main_kinds.lock().unwrap().len(), 1);
    }

    #[test]
    fn focus_makes_the_companion_main_until_it_stops() {
        let (mut activity_arbiter, _) = arbiter_recording_main_kinds();
        activity_arbiter.record_activity_update("music", activity_update(ActivityPresence::Ongoing, 50, "song"));
        activity_arbiter.record_activity_update("timer", activity_update(ActivityPresence::Ongoing, 80, "tick"));
        activity_arbiter.focus_activity("music").unwrap();
        assert_eq!(main_kind(&activity_arbiter), Some("music"));
        activity_arbiter.record_activity_update("music", activity_update(ActivityPresence::Lingering, 50, "song"));
        activity_arbiter.record_activity_update("music", activity_update(ActivityPresence::Ongoing, 50, "song"));
        assert_eq!(main_kind(&activity_arbiter), Some("timer"));
    }

    #[test]
    fn a_repeated_identical_update_does_not_make_an_activity_newer() {
        let (mut activity_arbiter, _) = arbiter_recording_main_kinds();
        activity_arbiter.record_activity_update("music", activity_update(ActivityPresence::Lingering, 50, "song"));
        activity_arbiter.record_activity_update("claude-code", activity_update(ActivityPresence::Lingering, 40, "done"));
        activity_arbiter.record_activity_update("music", activity_update(ActivityPresence::Lingering, 50, "song"));
        assert_eq!(main_kind(&activity_arbiter), Some("claude-code"));
    }

    #[test]
    fn focusing_an_unknown_activity_is_refused() {
        let (mut activity_arbiter, _) = arbiter_recording_main_kinds();
        assert!(activity_arbiter.focus_activity("music").is_err());
    }
}
