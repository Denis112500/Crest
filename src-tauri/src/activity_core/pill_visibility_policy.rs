//! The rules for when the pill is on screen (playing, paused, nothing), as one pure function
//! with no timers or threads, so every rule is unit-tested. The controller runs the countdowns.

use std::time::Duration;

use crate::activity_core::activity_arbiter::PillPresentation;
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

/// The pill's visibility rules, as a pure function of what was shown before and what
/// should be shown now (no timers, no threads, so the rules are easy to test):
/// - something ongoing (music playing): show it, and never hide while it lasts;
/// - something lingering (music paused): hide after a while; a new track shows it first;
/// - nothing at all: hide shortly.
pub fn decide_pill_visibility(
    previous_presentation: Option<&PillPresentation>,
    current_presentation: Option<&PillPresentation>,
) -> PillVisibilityDecision {
    let Some(current_presentation) = current_presentation else {
        return PillVisibilityDecision {
            should_show_now: false,
            pending_hide_change: PendingHideChange::HideAfter(PILL_HIDE_DELAY_AFTER_ACTIVITY_ENDS),
        };
    };
    if current_presentation.activity_update.is_ongoing {
        return PillVisibilityDecision { should_show_now: true, pending_hide_change: PendingHideChange::CancelPendingHide };
    }
    let wants_attention = previous_presentation.is_none_or(|previous_presentation| {
        previous_presentation.activity_kind != current_presentation.activity_kind
            || previous_presentation.activity_update.attention_key != current_presentation.activity_update.attention_key
    });
    let has_just_stopped =
        previous_presentation.is_some_and(|previous_presentation| previous_presentation.activity_update.is_ongoing);
    if wants_attention || has_just_stopped {
        return PillVisibilityDecision {
            should_show_now: wants_attention,
            pending_hide_change: PendingHideChange::HideAfter(PILL_HIDE_DELAY_AFTER_ACTIVITY_PAUSES),
        };
    }
    // Same paused track, something small changed (e.g. a seek): keep counting down.
    PillVisibilityDecision { should_show_now: false, pending_hide_change: PendingHideChange::KeepPendingHide }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::activity_core::activity_update::ActivityUpdate;

    fn music_presentation(attention_key: &str, is_ongoing: bool) -> PillPresentation {
        PillPresentation {
            activity_kind: "music",
            activity_update: ActivityUpdate {
                display_priority: 50,
                is_ongoing,
                attention_key: attention_key.to_string(),
                activity_payload: serde_json::Value::Null,
            },
        }
    }

    #[test]
    fn playing_shows_the_pill_and_cancels_any_hide() {
        let playing = music_presentation("song", true);
        let decision = decide_pill_visibility(None, Some(&playing));
        assert_eq!(decision, PillVisibilityDecision { should_show_now: true, pending_hide_change: PendingHideChange::CancelPendingHide });
    }

    #[test]
    fn pausing_starts_the_long_countdown_without_showing() {
        let (playing, paused) = (music_presentation("song", true), music_presentation("song", false));
        let decision = decide_pill_visibility(Some(&playing), Some(&paused));
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
        let (paused_song, paused_next_song) = (music_presentation("song", false), music_presentation("next song", false));
        assert!(decide_pill_visibility(Some(&paused_song), Some(&paused_next_song)).should_show_now);
    }

    #[test]
    fn a_seek_while_paused_keeps_the_countdown_running() {
        let paused = music_presentation("song", false);
        let decision = decide_pill_visibility(Some(&paused), Some(&paused));
        assert_eq!(decision.pending_hide_change, PendingHideChange::KeepPendingHide);
    }

    #[test]
    fn nothing_to_show_hides_shortly() {
        let paused = music_presentation("song", false);
        let decision = decide_pill_visibility(Some(&paused), None);
        assert_eq!(decision.pending_hide_change, PendingHideChange::HideAfter(PILL_HIDE_DELAY_AFTER_ACTIVITY_ENDS));
        assert!(!decision.should_show_now);
    }
}
