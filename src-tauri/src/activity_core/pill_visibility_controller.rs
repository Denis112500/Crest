//! Carries out the visibility rules: runs the hide countdowns and adds the "a fullscreen app
//! is in front" rule, then reports to the frontend. Kept apart from the pure rules on purpose.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread;

use serde::Serialize;

use crate::activity_core::pill_activity_arrangement::PillArrangement;
use crate::activity_core::pill_visibility_policy::{decide_pill_visibility, PendingHideChange};

/// Whether the pill should be on screen, and whether a fullscreen app is in front. The
/// second part travels along because the frontend hides the pill instantly for a game
/// instead of fading it out over it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PillVisibility {
    pub is_pill_visible: bool,
    pub is_fullscreen_app_in_front: bool,
}

/// Called with every change of the pill's visibility.
pub type PillVisibilityListener = Box<dyn Fn(PillVisibility) + Send>;

struct PillVisibilityState {
    /// What the activity rules want (on while music plays, off a while after it pauses).
    is_wanted_by_activity: bool,
    is_fullscreen_app_in_front: bool,
    last_reported_visibility: PillVisibility,
    previous_arrangement: PillArrangement,
    /// Bumped whenever the pending hide changes; a sleeping hide timer only acts if the
    /// generation it started with is still the current one.
    hide_generation: u64,
    pill_visibility_listener: PillVisibilityListener,
}

impl PillVisibilityState {
    fn current_visibility(&self) -> PillVisibility {
        PillVisibility {
            is_pill_visible: self.is_wanted_by_activity && !self.is_fullscreen_app_in_front,
            is_fullscreen_app_in_front: self.is_fullscreen_app_in_front,
        }
    }

    fn report_visibility_if_changed(&mut self) {
        let current_visibility = self.current_visibility();
        if current_visibility != self.last_reported_visibility {
            self.last_reported_visibility = current_visibility;
            (self.pill_visibility_listener)(current_visibility);
        }
    }

    fn set_wanted_by_activity(&mut self, is_wanted_by_activity: bool) {
        self.is_wanted_by_activity = is_wanted_by_activity;
        self.report_visibility_if_changed();
    }
}

/// Carries out the visibility rules from `pill_visibility_policy.rs` (runs the hide
/// countdowns) and keeps the pill off screen while a fullscreen app is in front; the
/// countdowns keep running meanwhile. Cheap to clone; all clones share one state.
#[derive(Clone)]
pub struct PillVisibilityController {
    shared_visibility_state: Arc<Mutex<PillVisibilityState>>,
}

impl PillVisibilityController {
    pub fn new(pill_visibility_listener: PillVisibilityListener) -> Self {
        let hidden_without_fullscreen_app = PillVisibility { is_pill_visible: false, is_fullscreen_app_in_front: false };
        Self {
            shared_visibility_state: Arc::new(Mutex::new(PillVisibilityState {
                is_wanted_by_activity: false,
                is_fullscreen_app_in_front: false,
                last_reported_visibility: hidden_without_fullscreen_app,
                previous_arrangement: PillArrangement::default(),
                hide_generation: 0,
                pill_visibility_listener,
            })),
        }
    }

    pub fn handle_arrangement_change(&self, current_arrangement: &PillArrangement) {
        let mut visibility_state = lock_visibility_state(&self.shared_visibility_state);
        let visibility_decision = decide_pill_visibility(&visibility_state.previous_arrangement, current_arrangement);
        visibility_state.previous_arrangement = current_arrangement.clone();
        if visibility_decision.should_show_now {
            visibility_state.set_wanted_by_activity(true);
        }
        match visibility_decision.pending_hide_change {
            PendingHideChange::KeepPendingHide => {}
            PendingHideChange::CancelPendingHide => visibility_state.hide_generation += 1,
            PendingHideChange::HideAfter(hide_delay) => {
                visibility_state.hide_generation += 1;
                let generation_of_this_hide = visibility_state.hide_generation;
                let shared_visibility_state = Arc::clone(&self.shared_visibility_state);
                // A sleeping thread costs no CPU; hides are rare (pause, player closed).
                thread::spawn(move || {
                    thread::sleep(hide_delay);
                    let mut visibility_state = lock_visibility_state(&shared_visibility_state);
                    if visibility_state.hide_generation == generation_of_this_hide {
                        visibility_state.set_wanted_by_activity(false);
                    }
                });
            }
        }
    }

    pub fn handle_fullscreen_app_change(&self, is_fullscreen_app_in_front: bool) {
        let mut visibility_state = lock_visibility_state(&self.shared_visibility_state);
        visibility_state.is_fullscreen_app_in_front = is_fullscreen_app_in_front;
        visibility_state.report_visibility_if_changed();
    }

    pub fn current_pill_visibility(&self) -> PillVisibility {
        lock_visibility_state(&self.shared_visibility_state).current_visibility()
    }
}

fn lock_visibility_state(shared_visibility_state: &Mutex<PillVisibilityState>) -> MutexGuard<'_, PillVisibilityState> {
    shared_visibility_state.lock().unwrap_or_else(PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::activity_core::activity_update::{ActivityPresence, ActivityUpdate};
    use crate::activity_core::pill_activity_arrangement::PillActivity;

    fn controller_recording_visibility_reports() -> (PillVisibilityController, Arc<Mutex<Vec<PillVisibility>>>) {
        let reported_visibilities = Arc::new(Mutex::new(Vec::new()));
        let recorded_visibilities = Arc::clone(&reported_visibilities);
        let pill_visibility_controller = PillVisibilityController::new(Box::new(move |pill_visibility| {
            recorded_visibilities.lock().unwrap().push(pill_visibility);
        }));
        (pill_visibility_controller, reported_visibilities)
    }

    fn playing_music_arrangement() -> PillArrangement {
        PillArrangement {
            main_activity: Some(PillActivity {
                activity_kind: "music",
                activity_update: ActivityUpdate {
                    display_priority: 50,
                    activity_presence: ActivityPresence::Ongoing,
                    attention_key: "song".to_string(),
                    activity_payload: serde_json::Value::Null,
                },
            }),
            ..Default::default()
        }
    }

    #[test]
    fn a_fullscreen_app_hides_a_playing_pill_and_says_why() {
        let (pill_visibility_controller, reported_visibilities) = controller_recording_visibility_reports();
        pill_visibility_controller.handle_arrangement_change(&playing_music_arrangement());
        pill_visibility_controller.handle_fullscreen_app_change(true);
        assert_eq!(
            reported_visibilities.lock().unwrap().last(),
            Some(&PillVisibility { is_pill_visible: false, is_fullscreen_app_in_front: true })
        );
    }

    #[test]
    fn a_playing_pill_comes_back_when_the_fullscreen_app_leaves() {
        let (pill_visibility_controller, _reported_visibilities) = controller_recording_visibility_reports();
        pill_visibility_controller.handle_arrangement_change(&playing_music_arrangement());
        pill_visibility_controller.handle_fullscreen_app_change(true);
        pill_visibility_controller.handle_fullscreen_app_change(false);
        assert!(pill_visibility_controller.current_pill_visibility().is_pill_visible);
    }

    #[test]
    fn a_pill_with_nothing_to_show_stays_hidden_after_the_fullscreen_app_leaves() {
        let (pill_visibility_controller, _reported_visibilities) = controller_recording_visibility_reports();
        pill_visibility_controller.handle_fullscreen_app_change(true);
        pill_visibility_controller.handle_fullscreen_app_change(false);
        assert!(!pill_visibility_controller.current_pill_visibility().is_pill_visible);
    }
}
