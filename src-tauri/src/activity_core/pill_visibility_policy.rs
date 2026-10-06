//! The rules for when the pill is on screen (something happening, paused, nothing), as one
//! pure function with no timers or threads, so every rule is unit-tested. The controller runs
//! the countdowns.

use std::time::Duration;

use crate::activity_core::activity_update::ActivityPresence;
use crate::activity_core::pill_activity_arrangement::{PillActivity, PillArrangement};
use crate::backend_constants::{PILL_HIDE_DELAY_AFTER_ACTIVITY_ENDS, PILL_HIDE_DELAY_AFTER_ACTIVITY_PAUSES};

#[derive(Debug, PartialEq, Eq)]
pub enum PendingHideChange {
    /// Stay on screen for as long as this lasts.
    CancelPendingHide,
    /// (Re)start the countdown to hiding.
    HideAfter(Duration),
    /// Leave any running countdown alone.
    KeepPendingHide,
}

#[derive(Debug, PartialEq, Eq)]
pub struct PillVisibilityDecision {
    pub should_show_now: bool,
    pub pending_hide_change: PendingHideChange,
}

/// The pill's visibility rules, as a pure function of the layout before and now. Only the
/// leading activity counts (the alert, else the main one): the arrangement already puts
/// anything ongoing ahead of anything lingering.
/// - an alert, or something ongoing (music playing): show it, never hide while it lasts;
/// - something lingering (music paused): hide after a while; a new track shows it first;
/// - nothing at all: hide shortly.
pub fn decide_pill_visibility(
    previous_arrangement: &PillArrangement,
    current_arrangement: &PillArrangement,
) -> PillVisibilityDecision {
    let previous_leading_activity = previous_arrangement.leading_activity();
    let Some(current_leading_activity) = current_arrangement.leading_activity() else {
        return PillVisibilityDecision {
            should_show_now: false,
            pending_hide_change: PendingHideChange::HideAfter(PILL_HIDE_DELAY_AFTER_ACTIVITY_ENDS),
        };
    };
    if is_holding_the_pill_open(current_leading_activity) {
        return PillVisibilityDecision { should_show_now: true, pending_hide_change: PendingHideChange::CancelPendingHide };
    }
    let wants_attention = previous_leading_activity.is_none_or(|previous_leading_activity| {
        previous_leading_activity.activity_kind != current_leading_activity.activity_kind
            || previous_leading_activity.activity_update.attention_key
                != current_leading_activity.activity_update.attention_key
    });
    let has_just_stopped = previous_leading_activity.is_some_and(is_holding_the_pill_open);
    if wants_attention || has_just_stopped {
        return PillVisibilityDecision {
            should_show_now: wants_attention,
            pending_hide_change: PendingHideChange::HideAfter(PILL_HIDE_DELAY_AFTER_ACTIVITY_PAUSES),
        };
    }
    // Same paused track, something small changed (e.g. a seek): keep counting down.
    PillVisibilityDecision { should_show_now: false, pending_hide_change: PendingHideChange::KeepPendingHide }
}

fn is_holding_the_pill_open(pill_activity: &PillActivity) -> bool {
    pill_activity.activity_update.activity_presence != ActivityPresence::Lingering
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::activity_core::activity_update::ActivityUpdate;

    fn arrangement_with(
        activity_kind: &'static str,
        activity_presence: ActivityPresence,
        attention_key: &str,
    ) -> PillArrangement {
        let pill_activity = PillActivity {
            activity_kind,
            activity_update: ActivityUpdate {
                display_priority: 50,
                activity_presence,
                attention_key: attention_key.to_string(),
                activity_payload: serde_json::Value::Null,
            },
        };
        match activity_presence {
            ActivityPresence::Alert => PillArrangement { alert_activity: Some(pill_activity), ..Default::default() },
            _ => PillArrangement { main_activity: Some(pill_activity), ..Default::default() },
        }
    }

    fn music_arrangement(attention_key: &str, is_playing: bool) -> PillArrangement {
        let activity_presence = if is_playing { ActivityPresence::Ongoing } else { ActivityPresence::Lingering };
        arrangement_with("music", activity_presence, attention_key)
    }

    #[test]
    fn playing_shows_the_pill_and_cancels_any_hide() {
        let decision = decide_pill_visibility(&PillArrangement::default(), &music_arrangement("song", true));
        assert_eq!(decision, PillVisibilityDecision { should_show_now: true, pending_hide_change: PendingHideChange::CancelPendingHide });
    }

    #[test]
    fn pausing_starts_the_long_countdown_without_showing() {
        let decision = decide_pill_visibility(&music_arrangement("song", true), &music_arrangement("song", false));
        assert_eq!(
            decision,
            PillVisibilityDecision {
                should_show_now: false,
                pending_hide_change: PendingHideChange::HideAfter(PILL_HIDE_DELAY_AFTER_ACTIVITY_PAUSES)
            }
        );
    }

    #[test]
    fn a_new_track_while_paused_shows_the_pill_again() {
        let decision = decide_pill_visibility(&music_arrangement("song", false), &music_arrangement("next song", false));
        assert!(decision.should_show_now);
    }

    #[test]
    fn a_seek_while_paused_keeps_the_countdown_running() {
        let paused = music_arrangement("song", false);
        let decision = decide_pill_visibility(&paused, &paused);
        assert_eq!(decision.pending_hide_change, PendingHideChange::KeepPendingHide);
    }

    #[test]
    fn nothing_to_show_hides_shortly() {
        let decision = decide_pill_visibility(&music_arrangement("song", false), &PillArrangement::default());
        assert_eq!(decision.pending_hide_change, PendingHideChange::HideAfter(PILL_HIDE_DELAY_AFTER_ACTIVITY_ENDS));
        assert!(!decision.should_show_now);
    }

    #[test]
    fn an_alert_shows_the_pill_and_keeps_it_until_answered() {
        let alert = arrangement_with("claude-code", ActivityPresence::Alert, "permission");
        let decision = decide_pill_visibility(&music_arrangement("song", false), &alert);
        assert_eq!(decision, PillVisibilityDecision { should_show_now: true, pending_hide_change: PendingHideChange::CancelPendingHide });
        let answered = decide_pill_visibility(&alert, &music_arrangement("song", false));
        assert_eq!(answered.pending_hide_change, PendingHideChange::HideAfter(PILL_HIDE_DELAY_AFTER_ACTIVITY_PAUSES));
    }
}
