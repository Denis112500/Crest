//! Where each activity goes in the pill (alert, main, companion), as one pure function with
//! no state, so every layout rule is unit-tested. The arbiter feeds it; the visibility rules
//! and the frontend read its result.

use std::cmp::Reverse;

use serde::Serialize;

use crate::activity_core::activity_update::{ActivityPresence, ActivityUpdate};

/// One activity as the arbiter keeps it: the latest update of a source and when it came.
pub struct RecordedActivity {
    pub activity_kind: &'static str,
    pub activity_update: ActivityUpdate,
    /// Grows with every update; tells which of two activities reported last.
    pub arrival_order: u64,
}

/// An activity placed in the pill: which source it came from and its latest update.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PillActivity {
    pub activity_kind: &'static str,
    #[serde(flatten)]
    pub activity_update: ActivityUpdate,
}

/// The layout of the pill (Phase 2, item 8 design): an alert takes over the whole pill;
/// otherwise `[ main | companion ]`, and the companion stands for `other_ongoing_count`
/// more ongoing activities ("+N").
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PillArrangement {
    pub alert_activity: Option<PillActivity>,
    pub main_activity: Option<PillActivity>,
    pub companion_activity: Option<PillActivity>,
    pub other_ongoing_count: usize,
}

impl PillArrangement {
    /// The activity filling the pill right now: the alert while there is one, else the main one.
    pub fn leading_activity(&self) -> Option<&PillActivity> {
        self.alert_activity.as_ref().or(self.main_activity.as_ref())
    }
}

/// The layout rules:
/// - alerts queue: the highest priority goes first, then the one that came first;
/// - main: ongoing before lingering; among ongoing ones, the one the user focused (clicked
///   as companion) first, then priority, then the most recent;
/// - companion: the next ongoing activity; lingering ones never sit next to the main one,
///   because a paused player would otherwise stay on screen forever.
pub fn arrange_pill_activities(
    recorded_activities: &[&RecordedActivity],
    focused_activity_kind: Option<&str>,
) -> PillArrangement {
    let alert_activity = recorded_activities
        .iter()
        .filter(|recorded| recorded.activity_update.activity_presence == ActivityPresence::Alert)
        .min_by_key(|recorded| (Reverse(recorded.activity_update.display_priority), recorded.arrival_order))
        .map(|recorded| place_in_pill(recorded));

    let mut main_candidates: Vec<&RecordedActivity> = recorded_activities
        .iter()
        .copied()
        .filter(|recorded| recorded.activity_update.activity_presence != ActivityPresence::Alert)
        .collect();
    main_candidates.sort_by_key(|recorded| {
        let is_ongoing = recorded.activity_update.activity_presence == ActivityPresence::Ongoing;
        let is_focused = is_ongoing && focused_activity_kind == Some(recorded.activity_kind);
        Reverse((is_ongoing, is_focused, recorded.activity_update.display_priority, recorded.arrival_order))
    });
    let mut ordered_candidates = main_candidates.into_iter();
    let main_activity = ordered_candidates.next().map(place_in_pill);
    let remaining_ongoing: Vec<&RecordedActivity> = ordered_candidates
        .filter(|recorded| recorded.activity_update.activity_presence == ActivityPresence::Ongoing)
        .collect();
    let companion_activity = remaining_ongoing.first().map(|recorded| place_in_pill(recorded));
    let other_ongoing_count = remaining_ongoing.len().saturating_sub(1);

    PillArrangement { alert_activity, main_activity, companion_activity, other_ongoing_count }
}

fn place_in_pill(recorded_activity: &RecordedActivity) -> PillActivity {
    PillActivity {
        activity_kind: recorded_activity.activity_kind,
        activity_update: recorded_activity.activity_update.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn recorded(
        activity_kind: &'static str,
        activity_presence: ActivityPresence,
        display_priority: u8,
        arrival_order: u64,
    ) -> RecordedActivity {
        RecordedActivity {
            activity_kind,
            activity_update: ActivityUpdate {
                display_priority,
                activity_presence,
                attention_key: String::new(),
                activity_payload: serde_json::Value::Null,
            },
            arrival_order,
        }
    }

    fn kind_of(pill_activity: &Option<PillActivity>) -> Option<&'static str> {
        pill_activity.as_ref().map(|placed| placed.activity_kind)
    }

    #[test]
    fn nothing_recorded_gives_an_empty_pill() {
        assert_eq!(arrange_pill_activities(&[], None), PillArrangement::default());
    }

    #[test]
    fn music_alone_is_the_main_activity_playing_or_paused() {
        let playing = recorded("music", ActivityPresence::Ongoing, 50, 0);
        let paused = recorded("music", ActivityPresence::Lingering, 50, 1);
        assert_eq!(kind_of(&arrange_pill_activities(&[&playing], None).main_activity), Some("music"));
        let paused_arrangement = arrange_pill_activities(&[&paused], None);
        assert_eq!(kind_of(&paused_arrangement.main_activity), Some("music"));
        assert_eq!(paused_arrangement.companion_activity, None);
    }

    #[test]
    fn two_ongoing_activities_share_the_pill_by_priority() {
        let music = recorded("music", ActivityPresence::Ongoing, 50, 0);
        let claude_code = recorded("claude-code", ActivityPresence::Ongoing, 60, 1);
        let arrangement = arrange_pill_activities(&[&music, &claude_code], None);
        assert_eq!(kind_of(&arrangement.main_activity), Some("claude-code"));
        assert_eq!(kind_of(&arrangement.companion_activity), Some("music"));
        assert_eq!(arrangement.other_ongoing_count, 0);
    }

    #[test]
    fn the_most_recent_goes_first_between_equal_priorities() {
        let music = recorded("music", ActivityPresence::Ongoing, 50, 0);
        let podcast = recorded("podcast", ActivityPresence::Ongoing, 50, 1);
        assert_eq!(kind_of(&arrange_pill_activities(&[&music, &podcast], None).main_activity), Some("podcast"));
    }

    #[test]
    fn the_focused_activity_becomes_main_while_it_is_ongoing() {
        let music = recorded("music", ActivityPresence::Ongoing, 50, 0);
        let claude_code = recorded("claude-code", ActivityPresence::Ongoing, 60, 1);
        let arrangement = arrange_pill_activities(&[&music, &claude_code], Some("music"));
        assert_eq!(kind_of(&arrangement.main_activity), Some("music"));
        assert_eq!(kind_of(&arrangement.companion_activity), Some("claude-code"));

        let paused_music = recorded("music", ActivityPresence::Lingering, 50, 2);
        let arrangement = arrange_pill_activities(&[&paused_music, &claude_code], Some("music"));
        assert_eq!(kind_of(&arrangement.main_activity), Some("claude-code"));
    }

    #[test]
    fn a_lingering_activity_never_becomes_the_companion() {
        let paused_music = recorded("music", ActivityPresence::Lingering, 50, 0);
        let claude_code = recorded("claude-code", ActivityPresence::Ongoing, 40, 1);
        let arrangement = arrange_pill_activities(&[&paused_music, &claude_code], None);
        assert_eq!(kind_of(&arrangement.main_activity), Some("claude-code"));
        assert_eq!(arrangement.companion_activity, None);
    }

    #[test]
    fn more_than_two_ongoing_activities_are_counted_behind_the_companion() {
        let music = recorded("music", ActivityPresence::Ongoing, 50, 0);
        let first_session = recorded("claude-code", ActivityPresence::Ongoing, 60, 1);
        let timer = recorded("timer", ActivityPresence::Ongoing, 40, 2);
        let arrangement = arrange_pill_activities(&[&music, &first_session, &timer], None);
        assert_eq!(kind_of(&arrangement.companion_activity), Some("music"));
        assert_eq!(arrangement.other_ongoing_count, 1);
    }

    #[test]
    fn alerts_take_over_and_queue_by_priority_then_arrival() {
        let music = recorded("music", ActivityPresence::Ongoing, 50, 0);
        let earlier_alert = recorded("claude-code", ActivityPresence::Alert, 60, 1);
        let later_alert = recorded("calendar", ActivityPresence::Alert, 60, 2);
        let arrangement = arrange_pill_activities(&[&music, &later_alert, &earlier_alert], None);
        assert_eq!(kind_of(&arrangement.alert_activity), Some("claude-code"));
        assert_eq!(kind_of(&arrangement.main_activity), Some("music"));
        assert_eq!(arrangement.leading_activity().map(|leading| leading.activity_kind), Some("claude-code"));

        let urgent_alert = recorded("timer", ActivityPresence::Alert, 90, 3);
        let arrangement = arrange_pill_activities(&[&earlier_alert, &urgent_alert], None);
        assert_eq!(kind_of(&arrangement.alert_activity), Some("timer"));
    }
}
