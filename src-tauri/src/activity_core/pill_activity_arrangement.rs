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
    /// The arrival order of the update that last changed the attention key: when this activity
    /// last had news. `None` until its key first changes, and again once the user has seen it.
    pub latest_attention_order: Option<u64>,
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
///   as companion) first, then priority, then the most recent; among lingering ones only the
///   most recent counts, whatever its priority: "Claude Code is done" just now says more than
///   a song paused minutes ago;
/// - companion: the next ongoing activity; failing that, next to an ongoing main one, a
///   lingering activity with news newer than the main one's ("Claude Code is done" next to the
///   music), until the main one has news of its own or the user has seen it. Otherwise lingering
///   ones never sit next to the main one: a paused player would stay on screen forever.
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
        let ranking_priority = if is_ongoing { recorded.activity_update.display_priority } else { 0 };
        Reverse((is_ongoing, is_focused, ranking_priority, recorded.arrival_order))
    });
    let mut ordered_candidates = main_candidates.into_iter();
    let main_record = ordered_candidates.next();
    let (remaining_ongoing, remaining_lingering): (Vec<&RecordedActivity>, Vec<&RecordedActivity>) =
        ordered_candidates.partition(|recorded| recorded.activity_update.activity_presence == ActivityPresence::Ongoing);
    let companion_activity = remaining_ongoing
        .first()
        .copied()
        .or_else(|| find_notice_companion(main_record, &remaining_lingering))
        .map(place_in_pill);
    let main_activity = main_record.map(place_in_pill);
    let other_ongoing_count = remaining_ongoing.len().saturating_sub(1);

    PillArrangement { alert_activity, main_activity, companion_activity, other_ongoing_count }
}

/// The lingering activity with the newest news, if that is newer than the main one's and the
/// main one is ongoing.
fn find_notice_companion<'a>(
    main_record: Option<&RecordedActivity>,
    remaining_lingering: &[&'a RecordedActivity],
) -> Option<&'a RecordedActivity> {
    let ongoing_main_record =
        main_record.filter(|recorded| recorded.activity_update.activity_presence == ActivityPresence::Ongoing)?;
    remaining_lingering
        .iter()
        .copied()
        .filter(|recorded| recorded.latest_attention_order > ongoing_main_record.latest_attention_order)
        .max_by_key(|recorded| recorded.latest_attention_order)
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
            latest_attention_order: None,
        }
    }

    fn with_news(mut recorded_activity: RecordedActivity, latest_attention_order: u64) -> RecordedActivity {
        recorded_activity.latest_attention_order = Some(latest_attention_order);
        recorded_activity
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
    fn among_lingering_activities_the_most_recent_is_main_whatever_its_priority() {
        let paused_music = recorded("music", ActivityPresence::Lingering, 50, 0);
        let finished_session = recorded("claude-code", ActivityPresence::Lingering, 40, 1);
        let arrangement = arrange_pill_activities(&[&paused_music, &finished_session], None);
        assert_eq!(kind_of(&arrangement.main_activity), Some("claude-code"));
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
    fn a_lingering_activity_without_news_never_becomes_the_companion() {
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

    #[test]
    fn news_on_a_lingering_activity_sits_next_to_the_playing_main_one() {
        let music = with_news(recorded("music", ActivityPresence::Ongoing, 50, 3), 0);
        let finished_session = with_news(recorded("claude-code", ActivityPresence::Lingering, 40, 4), 4);
        let arrangement = arrange_pill_activities(&[&music, &finished_session], None);
        assert_eq!(kind_of(&arrangement.main_activity), Some("music"));
        assert_eq!(kind_of(&arrangement.companion_activity), Some("claude-code"));
        assert_eq!(arrangement.other_ongoing_count, 0);
    }

    #[test]
    fn news_never_sits_next_to_a_paused_main_one() {
        let paused_music = recorded("music", ActivityPresence::Lingering, 50, 5);
        let finished_session = with_news(recorded("claude-code", ActivityPresence::Lingering, 40, 4), 4);
        let arrangement = arrange_pill_activities(&[&paused_music, &finished_session], None);
        assert_eq!(kind_of(&arrangement.main_activity), Some("music"));
        assert_eq!(arrangement.companion_activity, None);
    }

    #[test]
    fn news_leaves_when_the_main_one_has_newer_news_or_it_was_seen() {
        let music_with_new_track = with_news(recorded("music", ActivityPresence::Ongoing, 50, 6), 6);
        let finished_session = with_news(recorded("claude-code", ActivityPresence::Lingering, 40, 4), 4);
        assert_eq!(arrange_pill_activities(&[&music_with_new_track, &finished_session], None).companion_activity, None);

        let music = recorded("music", ActivityPresence::Ongoing, 50, 3);
        let seen_session = recorded("claude-code", ActivityPresence::Lingering, 40, 4);
        assert_eq!(arrange_pill_activities(&[&music, &seen_session], None).companion_activity, None);
    }

    #[test]
    fn an_ongoing_companion_goes_before_news() {
        let music = recorded("music", ActivityPresence::Ongoing, 50, 0);
        let timer = recorded("timer", ActivityPresence::Ongoing, 40, 1);
        let finished_session = with_news(recorded("claude-code", ActivityPresence::Lingering, 40, 2), 2);
        let arrangement = arrange_pill_activities(&[&music, &timer, &finished_session], None);
        assert_eq!(kind_of(&arrangement.companion_activity), Some("timer"));
    }
}
