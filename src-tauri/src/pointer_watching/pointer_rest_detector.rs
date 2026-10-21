//! Decides when the pointer is resting where the pill opens: inside the rest zone, still (a few
//! pixels of sensor jitter allowed) for a moment, and no button pressed since it arrived. A click
//! there (a browser tab under the pill) or a drag never opens the pill, and one visit opens it once.

use std::time::Duration;

use crate::backend_constants::{
    POINTER_REST_AT_TOP_EDGE_TO_OPEN_PILL_DELAY, POINTER_REST_MOVEMENT_TOLERANCE_PHYSICAL_PIXELS,
    POINTER_REST_TO_OPEN_PILL_DELAY,
};
use crate::pointer_watching::pointer_rest_zone::{PointerRestPurpose, PointerRestTarget};
use crate::pointer_watching::pointer_watch::{PointerSample, PointerSampleHandler, RestTimerChange};

pub type RestTargetReader = Box<dyn Fn() -> Option<PointerRestTarget> + Send>;
/// Called with what the rest means and the screen point the pointer rested on.
pub type PointerRestReporter = Box<dyn FnMut(PointerRestPurpose, i32, i32) + Send>;

pub struct PointerRestDetector {
    read_rest_target: RestTargetReader,
    report_pointer_rest: PointerRestReporter,
    /// Where the pointer stopped and what for, while the timer runs.
    rest_anchor: Option<(i32, i32, PointerRestPurpose)>,
    /// After a click or an opening: nothing more until the pointer has left the zone.
    is_waiting_for_pointer_to_leave: bool,
}

impl PointerRestDetector {
    pub fn new(read_rest_target: RestTargetReader, report_pointer_rest: PointerRestReporter) -> Self {
        Self { read_rest_target, report_pointer_rest, rest_anchor: None, is_waiting_for_pointer_to_leave: false }
    }

    /// What resting here would do, or `None` outside every rest zone.
    fn find_rest_purpose_at(&self, pointer_sample: PointerSample) -> Option<PointerRestPurpose> {
        (self.read_rest_target)()
            .filter(|rest_target| rest_target.rest_zone.contains_screen_point(pointer_sample.screen_x, pointer_sample.screen_y))
            .map(|rest_target| rest_target.rest_purpose)
    }
}

/// The top edge waits longer: a pointer thrown to the top of the screen is usually on its way to
/// a browser tab or a title bar there.
fn rest_delay_for(rest_purpose: PointerRestPurpose) -> Duration {
    match rest_purpose {
        PointerRestPurpose::OpenClosedPill => POINTER_REST_TO_OPEN_PILL_DELAY,
        PointerRestPurpose::OpenHiddenPill => POINTER_REST_AT_TOP_EDGE_TO_OPEN_PILL_DELAY,
    }
}

fn is_near_anchor(anchor_x: i32, anchor_y: i32, pointer_sample: PointerSample) -> bool {
    (pointer_sample.screen_x - anchor_x).abs() <= POINTER_REST_MOVEMENT_TOLERANCE_PHYSICAL_PIXELS
        && (pointer_sample.screen_y - anchor_y).abs() <= POINTER_REST_MOVEMENT_TOLERANCE_PHYSICAL_PIXELS
}

impl PointerSampleHandler for PointerRestDetector {
    fn handle_pointer_sample(&mut self, pointer_sample: PointerSample) -> RestTimerChange {
        let Some(rest_purpose) = self.find_rest_purpose_at(pointer_sample) else {
            self.is_waiting_for_pointer_to_leave = false;
            // Most reports are far from the pill: only a running timer needs a call to Windows.
            return if self.rest_anchor.take().is_some() { RestTimerChange::Stop } else { RestTimerChange::LeaveAsIs };
        };
        if pointer_sample.has_button_gone_down {
            self.is_waiting_for_pointer_to_leave = true;
            self.rest_anchor = None;
            return RestTimerChange::Stop;
        }
        if self.is_waiting_for_pointer_to_leave {
            return RestTimerChange::LeaveAsIs;
        }
        match self.rest_anchor {
            Some((anchor_x, anchor_y, anchor_purpose))
                if anchor_purpose == rest_purpose && is_near_anchor(anchor_x, anchor_y, pointer_sample) =>
            {
                RestTimerChange::LeaveAsIs
            }
            _ => {
                self.rest_anchor = Some((pointer_sample.screen_x, pointer_sample.screen_y, rest_purpose));
                RestTimerChange::Restart(rest_delay_for(rest_purpose))
            }
        }
    }

    fn handle_rest_timer_elapsed(&mut self, pointer_sample: PointerSample) {
        let Some((anchor_x, anchor_y, anchor_purpose)) = self.rest_anchor.take() else {
            return;
        };
        if self.is_waiting_for_pointer_to_leave
            || !is_near_anchor(anchor_x, anchor_y, pointer_sample)
            || self.find_rest_purpose_at(pointer_sample) != Some(anchor_purpose)
        {
            return;
        }
        self.is_waiting_for_pointer_to_leave = true;
        (self.report_pointer_rest)(anchor_purpose, pointer_sample.screen_x, pointer_sample.screen_y);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;
    use crate::pointer_watching::pointer_rest_zone::PointerRestZone;

    const CLOSED_PILL_TARGET: PointerRestTarget = PointerRestTarget {
        rest_zone: PointerRestZone { left_x: 100, top_y: 0, right_x: 400, bottom_y: 50 },
        rest_purpose: PointerRestPurpose::OpenClosedPill,
    };
    const TOP_EDGE_TARGET: PointerRestTarget = PointerRestTarget {
        rest_zone: PointerRestZone { left_x: 100, top_y: 0, right_x: 400, bottom_y: 1 },
        rest_purpose: PointerRestPurpose::OpenHiddenPill,
    };

    type ReportedRests = Arc<Mutex<Vec<(PointerRestPurpose, i32, i32)>>>;
    type SharedRestTarget = Arc<Mutex<Option<PointerRestTarget>>>;

    fn sample_at(screen_x: i32, screen_y: i32) -> PointerSample {
        PointerSample { screen_x, screen_y, has_button_gone_down: false }
    }

    fn click_at(screen_x: i32, screen_y: i32) -> PointerSample {
        PointerSample { screen_x, screen_y, has_button_gone_down: true }
    }

    /// A detector over a rest target the test can change, and the rests it reported.
    fn create_test_detector(rest_target: PointerRestTarget) -> (PointerRestDetector, SharedRestTarget, ReportedRests) {
        let rest_target_now = Arc::new(Mutex::new(Some(rest_target)));
        let rest_target_for_detector = Arc::clone(&rest_target_now);
        let reported_rests = Arc::new(Mutex::new(Vec::new()));
        let reported_rests_for_detector = Arc::clone(&reported_rests);
        let rest_detector = PointerRestDetector::new(
            Box::new(move || *rest_target_for_detector.lock().unwrap()),
            Box::new(move |rest_purpose, screen_x, screen_y| {
                reported_rests_for_detector.lock().unwrap().push((rest_purpose, screen_x, screen_y))
            }),
        );
        (rest_detector, rest_target_now, reported_rests)
    }

    #[test]
    fn resting_on_the_closed_pill_opens_it_once_per_visit() {
        let (mut rest_detector, _, reported_rests) = create_test_detector(CLOSED_PILL_TARGET);
        assert_eq!(rest_detector.handle_pointer_sample(sample_at(200, 20)), RestTimerChange::Restart(POINTER_REST_TO_OPEN_PILL_DELAY));
        rest_detector.handle_rest_timer_elapsed(sample_at(201, 20));
        assert_eq!(*reported_rests.lock().unwrap(), [(PointerRestPurpose::OpenClosedPill, 201, 20)]);
        assert_eq!(rest_detector.handle_pointer_sample(sample_at(250, 20)), RestTimerChange::LeaveAsIs);
        rest_detector.handle_rest_timer_elapsed(sample_at(250, 20));
        assert_eq!(reported_rests.lock().unwrap().len(), 1);
    }

    #[test]
    fn resting_against_the_top_edge_opens_the_hidden_pill_after_its_longer_delay() {
        let (mut rest_detector, _, reported_rests) = create_test_detector(TOP_EDGE_TARGET);
        assert_eq!(
            rest_detector.handle_pointer_sample(sample_at(200, 0)),
            RestTimerChange::Restart(POINTER_REST_AT_TOP_EDGE_TO_OPEN_PILL_DELAY)
        );
        assert_eq!(rest_detector.handle_pointer_sample(sample_at(200, 1)), RestTimerChange::Stop);
        rest_detector.handle_pointer_sample(sample_at(220, 0));
        rest_detector.handle_rest_timer_elapsed(sample_at(220, 0));
        assert_eq!(*reported_rests.lock().unwrap(), [(PointerRestPurpose::OpenHiddenPill, 220, 0)]);
    }

    #[test]
    fn jitter_keeps_the_timer_and_a_real_move_starts_it_over() {
        let (mut rest_detector, _, _) = create_test_detector(CLOSED_PILL_TARGET);
        rest_detector.handle_pointer_sample(sample_at(200, 20));
        let jitter = POINTER_REST_MOVEMENT_TOLERANCE_PHYSICAL_PIXELS;
        assert_eq!(rest_detector.handle_pointer_sample(sample_at(200 + jitter, 20 - jitter)), RestTimerChange::LeaveAsIs);
        assert_eq!(
            rest_detector.handle_pointer_sample(sample_at(200 + jitter + 1, 20)),
            RestTimerChange::Restart(POINTER_REST_TO_OPEN_PILL_DELAY)
        );
    }

    #[test]
    fn a_click_never_opens_the_pill_until_the_pointer_leaves_and_comes_back() {
        let (mut rest_detector, _, reported_rests) = create_test_detector(TOP_EDGE_TARGET);
        rest_detector.handle_pointer_sample(sample_at(200, 0));
        assert_eq!(rest_detector.handle_pointer_sample(click_at(200, 0)), RestTimerChange::Stop);
        assert_eq!(rest_detector.handle_pointer_sample(sample_at(260, 0)), RestTimerChange::LeaveAsIs);
        rest_detector.handle_rest_timer_elapsed(sample_at(260, 0));
        assert!(reported_rests.lock().unwrap().is_empty());
        rest_detector.handle_pointer_sample(sample_at(200, 80));
        assert_eq!(
            rest_detector.handle_pointer_sample(sample_at(200, 0)),
            RestTimerChange::Restart(POINTER_REST_AT_TOP_EDGE_TO_OPEN_PILL_DELAY)
        );
    }

    #[test]
    fn leaving_stops_the_timer_and_far_away_reports_cost_nothing() {
        let (mut rest_detector, _, reported_rests) = create_test_detector(CLOSED_PILL_TARGET);
        rest_detector.handle_pointer_sample(sample_at(200, 20));
        assert_eq!(rest_detector.handle_pointer_sample(sample_at(200, 60)), RestTimerChange::Stop);
        assert_eq!(rest_detector.handle_pointer_sample(sample_at(900, 600)), RestTimerChange::LeaveAsIs);
        rest_detector.handle_rest_timer_elapsed(sample_at(200, 20));
        assert!(reported_rests.lock().unwrap().is_empty());
    }

    #[test]
    fn a_pill_that_opened_hid_or_appeared_meanwhile_is_not_opened() {
        let (mut rest_detector, rest_target_now, reported_rests) = create_test_detector(CLOSED_PILL_TARGET);
        rest_detector.handle_pointer_sample(sample_at(200, 20));
        *rest_target_now.lock().unwrap() = None;
        rest_detector.handle_rest_timer_elapsed(sample_at(200, 20));
        rest_detector.handle_pointer_sample(sample_at(900, 600));
        *rest_target_now.lock().unwrap() = Some(TOP_EDGE_TARGET);
        rest_detector.handle_pointer_sample(sample_at(200, 0));
        *rest_target_now.lock().unwrap() = Some(CLOSED_PILL_TARGET);
        rest_detector.handle_rest_timer_elapsed(sample_at(200, 0));
        assert!(reported_rests.lock().unwrap().is_empty());
    }
}
