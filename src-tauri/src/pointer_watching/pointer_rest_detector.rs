//! Decides when the pointer is resting on the closed pill: inside its zone, still (a few pixels
//! of sensor jitter allowed) for a moment, and no button pressed since it arrived. A click there
//! (a browser tab under the pill) or a drag never opens the pill, and one visit opens it once.

use crate::backend_constants::{POINTER_REST_MOVEMENT_TOLERANCE_PHYSICAL_PIXELS, POINTER_REST_TO_OPEN_PILL_DELAY};
use crate::pointer_watching::pointer_rest_zone::PointerRestZone;
use crate::pointer_watching::pointer_watch::{PointerSample, PointerSampleHandler, RestTimerChange};

pub type RestZoneReader = Box<dyn Fn() -> Option<PointerRestZone> + Send>;
/// Called with the screen point the pointer rested on.
pub type PointerRestReporter = Box<dyn FnMut(i32, i32) + Send>;

pub struct PointerRestDetector {
    read_rest_zone: RestZoneReader,
    report_pointer_rest: PointerRestReporter,
    /// Where the pointer stopped, while the timer runs.
    rest_anchor: Option<(i32, i32)>,
    /// After a click or an opening: nothing more until the pointer has left the zone.
    is_waiting_for_pointer_to_leave: bool,
}

impl PointerRestDetector {
    pub fn new(read_rest_zone: RestZoneReader, report_pointer_rest: PointerRestReporter) -> Self {
        Self { read_rest_zone, report_pointer_rest, rest_anchor: None, is_waiting_for_pointer_to_leave: false }
    }

    fn is_in_rest_zone(&self, pointer_sample: PointerSample) -> bool {
        (self.read_rest_zone)()
            .is_some_and(|rest_zone| rest_zone.contains_screen_point(pointer_sample.screen_x, pointer_sample.screen_y))
    }
}

fn is_near_anchor(rest_anchor: (i32, i32), pointer_sample: PointerSample) -> bool {
    (pointer_sample.screen_x - rest_anchor.0).abs() <= POINTER_REST_MOVEMENT_TOLERANCE_PHYSICAL_PIXELS
        && (pointer_sample.screen_y - rest_anchor.1).abs() <= POINTER_REST_MOVEMENT_TOLERANCE_PHYSICAL_PIXELS
}

impl PointerSampleHandler for PointerRestDetector {
    fn handle_pointer_sample(&mut self, pointer_sample: PointerSample) -> RestTimerChange {
        if !self.is_in_rest_zone(pointer_sample) {
            self.is_waiting_for_pointer_to_leave = false;
            // Most reports are far from the pill: only a running timer needs a call to Windows.
            return if self.rest_anchor.take().is_some() { RestTimerChange::Stop } else { RestTimerChange::LeaveAsIs };
        }
        if pointer_sample.has_button_gone_down {
            self.is_waiting_for_pointer_to_leave = true;
            self.rest_anchor = None;
            return RestTimerChange::Stop;
        }
        if self.is_waiting_for_pointer_to_leave {
            return RestTimerChange::LeaveAsIs;
        }
        match self.rest_anchor {
            Some(rest_anchor) if is_near_anchor(rest_anchor, pointer_sample) => RestTimerChange::LeaveAsIs,
            _ => {
                self.rest_anchor = Some((pointer_sample.screen_x, pointer_sample.screen_y));
                RestTimerChange::Restart(POINTER_REST_TO_OPEN_PILL_DELAY)
            }
        }
    }

    fn handle_rest_timer_elapsed(&mut self, pointer_sample: PointerSample) {
        let Some(rest_anchor) = self.rest_anchor.take() else {
            return;
        };
        if self.is_waiting_for_pointer_to_leave
            || !is_near_anchor(rest_anchor, pointer_sample)
            || !self.is_in_rest_zone(pointer_sample)
        {
            return;
        }
        self.is_waiting_for_pointer_to_leave = true;
        (self.report_pointer_rest)(pointer_sample.screen_x, pointer_sample.screen_y);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;

    const CLOSED_PILL_ZONE: PointerRestZone = PointerRestZone { left_x: 100, top_y: 0, right_x: 400, bottom_y: 50 };

    fn sample_at(screen_x: i32, screen_y: i32) -> PointerSample {
        PointerSample { screen_x, screen_y, has_button_gone_down: false }
    }

    fn click_at(screen_x: i32, screen_y: i32) -> PointerSample {
        PointerSample { screen_x, screen_y, has_button_gone_down: true }
    }

    type ReportedRestPoints = Arc<Mutex<Vec<(i32, i32)>>>;

    /// A detector over the closed pill's zone, and the rest points it reported.
    fn create_test_detector() -> (PointerRestDetector, ReportedRestPoints) {
        let reported_rests = Arc::new(Mutex::new(Vec::new()));
        let reported_rests_for_detector = Arc::clone(&reported_rests);
        let rest_detector = PointerRestDetector::new(
            Box::new(|| Some(CLOSED_PILL_ZONE)),
            Box::new(move |screen_x, screen_y| reported_rests_for_detector.lock().unwrap().push((screen_x, screen_y))),
        );
        (rest_detector, reported_rests)
    }

    #[test]
    fn resting_on_the_pill_opens_it_once_per_visit() {
        let (mut rest_detector, reported_rests) = create_test_detector();
        assert_eq!(rest_detector.handle_pointer_sample(sample_at(200, 20)), RestTimerChange::Restart(POINTER_REST_TO_OPEN_PILL_DELAY));
        rest_detector.handle_rest_timer_elapsed(sample_at(201, 20));
        assert_eq!(*reported_rests.lock().unwrap(), [(201, 20)]);
        assert_eq!(rest_detector.handle_pointer_sample(sample_at(250, 20)), RestTimerChange::LeaveAsIs);
        rest_detector.handle_rest_timer_elapsed(sample_at(250, 20));
        assert_eq!(reported_rests.lock().unwrap().len(), 1);
    }

    #[test]
    fn jitter_keeps_the_timer_and_a_real_move_starts_it_over() {
        let (mut rest_detector, _) = create_test_detector();
        rest_detector.handle_pointer_sample(sample_at(200, 20));
        let jitter = POINTER_REST_MOVEMENT_TOLERANCE_PHYSICAL_PIXELS;
        assert_eq!(rest_detector.handle_pointer_sample(sample_at(200 + jitter, 20 - jitter)), RestTimerChange::LeaveAsIs);
        assert_eq!(
            rest_detector.handle_pointer_sample(sample_at(200 + jitter + 1, 20)),
            RestTimerChange::Restart(POINTER_REST_TO_OPEN_PILL_DELAY)
        );
    }

    #[test]
    fn a_click_on_the_pill_never_opens_it_until_the_pointer_leaves_and_comes_back() {
        let (mut rest_detector, reported_rests) = create_test_detector();
        rest_detector.handle_pointer_sample(sample_at(200, 20));
        assert_eq!(rest_detector.handle_pointer_sample(click_at(200, 20)), RestTimerChange::Stop);
        assert_eq!(rest_detector.handle_pointer_sample(sample_at(260, 20)), RestTimerChange::LeaveAsIs);
        rest_detector.handle_rest_timer_elapsed(sample_at(260, 20));
        assert!(reported_rests.lock().unwrap().is_empty());
        rest_detector.handle_pointer_sample(sample_at(200, 80));
        assert_eq!(rest_detector.handle_pointer_sample(sample_at(200, 20)), RestTimerChange::Restart(POINTER_REST_TO_OPEN_PILL_DELAY));
    }

    #[test]
    fn leaving_stops_the_timer_and_far_away_reports_cost_nothing() {
        let (mut rest_detector, reported_rests) = create_test_detector();
        rest_detector.handle_pointer_sample(sample_at(200, 20));
        assert_eq!(rest_detector.handle_pointer_sample(sample_at(200, 60)), RestTimerChange::Stop);
        assert_eq!(rest_detector.handle_pointer_sample(sample_at(900, 600)), RestTimerChange::LeaveAsIs);
        rest_detector.handle_rest_timer_elapsed(sample_at(200, 20));
        assert!(reported_rests.lock().unwrap().is_empty());
    }

    #[test]
    fn a_pill_that_opened_or_hid_meanwhile_is_not_opened() {
        let rest_zone_now = Arc::new(Mutex::new(Some(CLOSED_PILL_ZONE)));
        let rest_zone_for_detector = Arc::clone(&rest_zone_now);
        let mut rest_detector = PointerRestDetector::new(
            Box::new(move || *rest_zone_for_detector.lock().unwrap()),
            Box::new(|_, _| panic!("the pill must not be opened")),
        );
        rest_detector.handle_pointer_sample(sample_at(200, 20));
        *rest_zone_now.lock().unwrap() = None;
        rest_detector.handle_rest_timer_elapsed(sample_at(200, 20));
    }
}
