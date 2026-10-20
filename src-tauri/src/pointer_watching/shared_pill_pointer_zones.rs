//! Where the pill is on screen, as the pointer watcher's thread needs it: written by the pill
//! window's commands (placed, shown or hidden, closed area), read on every mouse report.

use std::sync::{Arc, Mutex};

use serde::Serialize;

use crate::pointer_watching::pointer_rest_zone::PointerRestZone;

/// The pointer's resting place, relative to the pill window in CSS pixels, so the page can
/// tell what is under it (the companion's mark or the main activity).
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PillPointerRestPoint {
    pub logical_x: f64,
    pub logical_y: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct PillPointerZones {
    window_left_x: i32,
    window_top_y: i32,
    window_scale_factor: f64,
    is_window_shown: bool,
    /// Relative to the window's top-left corner; only while the closed pill lets clicks through.
    closed_pill_area: Option<PointerRestZone>,
}

/// Empty until the pill page has placed the window for the first time.
#[derive(Clone, Default)]
pub struct SharedPillPointerZones {
    current_zones: Arc<Mutex<Option<PillPointerZones>>>,
}

impl SharedPillPointerZones {
    pub fn record_window_placement(&self, window_left_x: i32, window_top_y: i32, window_scale_factor: f64) {
        self.change_zones(|current_zones| {
            let previous_zones = current_zones.take();
            *current_zones = Some(PillPointerZones {
                window_left_x,
                window_top_y,
                window_scale_factor,
                is_window_shown: previous_zones.is_some_and(|zones| zones.is_window_shown),
                closed_pill_area: previous_zones.and_then(|zones| zones.closed_pill_area),
            });
        });
    }

    pub fn record_window_shown(&self, is_window_shown: bool) {
        self.change_zones(|current_zones| {
            if let Some(zones) = current_zones.as_mut() {
                zones.is_window_shown = is_window_shown;
            }
        });
    }

    /// `None` while the pill is open: then it takes the mouse itself and the page sees the hover.
    pub fn record_closed_pill_area(&self, closed_pill_area: Option<PointerRestZone>) {
        self.change_zones(|current_zones| {
            if let Some(zones) = current_zones.as_mut() {
                zones.closed_pill_area = closed_pill_area;
            }
        });
    }

    /// The closed pill's area on screen, while the window is shown; otherwise nothing.
    pub fn read_rest_zone(&self) -> Option<PointerRestZone> {
        let zones = self.read_zones()?;
        let closed_pill_area = zones.closed_pill_area.filter(|_| zones.is_window_shown)?;
        Some(PointerRestZone {
            left_x: zones.window_left_x + closed_pill_area.left_x,
            top_y: zones.window_top_y + closed_pill_area.top_y,
            right_x: zones.window_left_x + closed_pill_area.right_x,
            bottom_y: zones.window_top_y + closed_pill_area.bottom_y,
        })
    }

    pub fn convert_to_window_logical_point(&self, screen_x: i32, screen_y: i32) -> Option<PillPointerRestPoint> {
        let zones = self.read_zones()?;
        Some(PillPointerRestPoint {
            logical_x: f64::from(screen_x - zones.window_left_x) / zones.window_scale_factor,
            logical_y: f64::from(screen_y - zones.window_top_y) / zones.window_scale_factor,
        })
    }

    fn read_zones(&self) -> Option<PillPointerZones> {
        self.current_zones.lock().ok().and_then(|current_zones| *current_zones)
    }

    fn change_zones(&self, zones_change: impl FnOnce(&mut Option<PillPointerZones>)) {
        if let Ok(mut current_zones) = self.current_zones.lock() {
            zones_change(&mut current_zones);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLOSED_PILL_AREA: PointerRestZone = PointerRestZone { left_x: 600, top_y: 0, right_x: 1014, bottom_y: 72 };

    fn placed_and_shown_zones() -> SharedPillPointerZones {
        let shared_zones = SharedPillPointerZones::default();
        shared_zones.record_window_placement(1000, 0, 1.5);
        shared_zones.record_window_shown(true);
        shared_zones.record_closed_pill_area(Some(CLOSED_PILL_AREA));
        shared_zones
    }

    #[test]
    fn the_rest_zone_is_the_closed_pill_on_screen_while_the_window_is_shown() {
        let shared_zones = placed_and_shown_zones();
        assert_eq!(
            shared_zones.read_rest_zone(),
            Some(PointerRestZone { left_x: 1600, top_y: 0, right_x: 2014, bottom_y: 72 })
        );
        shared_zones.record_window_shown(false);
        assert_eq!(shared_zones.read_rest_zone(), None);
    }

    #[test]
    fn an_open_pill_or_an_unplaced_window_has_no_rest_zone() {
        let shared_zones = placed_and_shown_zones();
        shared_zones.record_closed_pill_area(None);
        assert_eq!(shared_zones.read_rest_zone(), None);
        assert_eq!(SharedPillPointerZones::default().read_rest_zone(), None);
    }

    #[test]
    fn placing_the_window_again_keeps_the_closed_area_and_moves_the_zone() {
        let shared_zones = placed_and_shown_zones();
        shared_zones.record_window_placement(-2000, 0, 1.0);
        assert_eq!(shared_zones.read_rest_zone().map(|rest_zone| rest_zone.left_x), Some(-1400));
    }

    #[test]
    fn a_screen_point_becomes_css_pixels_inside_the_window() {
        let shared_zones = placed_and_shown_zones();
        assert_eq!(
            shared_zones.convert_to_window_logical_point(1660, 30),
            Some(PillPointerRestPoint { logical_x: 440.0, logical_y: 20.0 })
        );
    }
}
