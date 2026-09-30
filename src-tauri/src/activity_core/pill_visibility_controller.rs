use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread;

use crate::activity_core::activity_arbiter::PillPresentation;
use crate::activity_core::pill_visibility_policy::{decide_pill_visibility, PendingHideChange};

/// Called with `true` when the pill should appear and `false` when it should hide.
pub type PillVisibilityListener = Box<dyn Fn(bool) + Send>;

struct PillVisibilityState {
    is_pill_visible: bool,
    previous_presentation: Option<PillPresentation>,
    /// Bumped whenever the pending hide changes; a sleeping hide timer only acts if the
    /// generation it started with is still the current one.
    hide_generation: u64,
    pill_visibility_listener: PillVisibilityListener,
}

impl PillVisibilityState {
    fn change_visibility(&mut self, is_pill_visible: bool) {
        if self.is_pill_visible != is_pill_visible {
            self.is_pill_visible = is_pill_visible;
            (self.pill_visibility_listener)(is_pill_visible);
        }
    }
}

/// Carries out the visibility rules from `pill_visibility_policy.rs`: runs the hide
/// countdowns and reports every change. Cheap to clone; all clones share one state.
#[derive(Clone)]
pub struct PillVisibilityController {
    shared_visibility_state: Arc<Mutex<PillVisibilityState>>,
}

impl PillVisibilityController {
    pub fn new(pill_visibility_listener: PillVisibilityListener) -> Self {
        Self {
            shared_visibility_state: Arc::new(Mutex::new(PillVisibilityState {
                is_pill_visible: false,
                previous_presentation: None,
                hide_generation: 0,
                pill_visibility_listener,
            })),
        }
    }

    pub fn handle_presentation_change(&self, current_presentation: Option<&PillPresentation>) {
        let mut visibility_state = lock_visibility_state(&self.shared_visibility_state);
        let visibility_decision =
            decide_pill_visibility(visibility_state.previous_presentation.as_ref(), current_presentation);
        visibility_state.previous_presentation = current_presentation.cloned();
        if visibility_decision.should_show_now {
            visibility_state.change_visibility(true);
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
                        visibility_state.change_visibility(false);
                    }
                });
            }
        }
    }

    pub fn is_pill_visible(&self) -> bool {
        lock_visibility_state(&self.shared_visibility_state).is_pill_visible
    }
}

fn lock_visibility_state(shared_visibility_state: &Mutex<PillVisibilityState>) -> MutexGuard<'_, PillVisibilityState> {
    shared_visibility_state.lock().unwrap_or_else(PoisonError::into_inner)
}
