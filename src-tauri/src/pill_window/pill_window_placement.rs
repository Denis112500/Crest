use tauri::{PhysicalPosition, PhysicalSize, WebviewWindow};

use crate::pill_window::connected_display_reader::read_connected_displays;
use crate::pill_window::pill_display_choice::{find_display_for_pill, ConnectedDisplay};

#[derive(Debug, PartialEq)]
struct PillWindowPhysicalBounds {
    top_left_position: PhysicalPosition<i32>,
    size: PhysicalSize<u32>,
}

/// Sizes the pill window and centers it on the top edge of the chosen monitor (the main one
/// if none is chosen or it's unplugged), with no gap: the pill is drawn as a notch hanging
/// from the edge. The frontend passes logical pixels (CSS pixels); Windows positions windows
/// in physical pixels, so everything is converted with that monitor's scale factor.
pub fn apply_top_center_placement_to_pill_window(
    pill_window: &WebviewWindow,
    pill_logical_width: f64,
    pill_logical_height: f64,
    chosen_display_name: Option<&str>,
) -> Result<(), String> {
    let connected_displays = read_connected_displays(pill_window)?;
    let pill_display = find_display_for_pill(&connected_displays, chosen_display_name)
        .ok_or("Windows reported no monitor to put the pill on")?;
    let pill_bounds = calculate_top_center_bounds(pill_display, pill_logical_width, pill_logical_height);
    pill_window
        .set_size(pill_bounds.size)
        .and_then(|()| pill_window.set_position(pill_bounds.top_left_position))
        // Moving onto a monitor with a different scale factor makes Windows rescale the
        // window, so the size is applied once more at its final position.
        .and_then(|()| pill_window.set_size(pill_bounds.size))
        .map_err(|error| error.to_string())
}

fn calculate_top_center_bounds(
    pill_display: &ConnectedDisplay,
    pill_logical_width: f64,
    pill_logical_height: f64,
) -> PillWindowPhysicalBounds {
    let pill_physical_width = (pill_logical_width * pill_display.scale_factor).round() as u32;
    let pill_physical_height = (pill_logical_height * pill_display.scale_factor).round() as u32;
    let horizontal_offset_inside_display = (pill_display.physical_width as i32 - pill_physical_width as i32) / 2;
    PillWindowPhysicalBounds {
        top_left_position: PhysicalPosition::new(
            pill_display.physical_left + horizontal_offset_inside_display,
            pill_display.physical_top,
        ),
        size: PhysicalSize::new(pill_physical_width, pill_physical_height),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_display(physical_left: i32, physical_width: u32, physical_height: u32, scale_factor: f64) -> ConnectedDisplay {
        ConnectedDisplay {
            display_name: r"\\.\DISPLAY1".to_string(),
            physical_left,
            physical_top: 0,
            physical_width,
            physical_height,
            scale_factor,
            is_main_display: true,
        }
    }

    #[test]
    fn centers_pill_and_scales_it_on_a_150_percent_monitor() {
        let pill_bounds = calculate_top_center_bounds(&create_test_display(0, 2560, 1440, 1.5), 220.0, 36.0);
        assert_eq!(pill_bounds.size, PhysicalSize::new(330, 54));
        assert_eq!(pill_bounds.top_left_position, PhysicalPosition::new(1115, 0));
    }

    #[test]
    fn centers_pill_on_a_monitor_that_does_not_start_at_zero() {
        let pill_bounds = calculate_top_center_bounds(&create_test_display(-1920, 1920, 1080, 1.0), 220.0, 36.0);
        assert_eq!(pill_bounds.top_left_position, PhysicalPosition::new(-1070, 0));
    }

    #[test]
    fn centers_pill_on_the_second_monitor_to_the_right() {
        let pill_bounds = calculate_top_center_bounds(&create_test_display(2560, 1920, 1080, 1.0), 412.0, 184.0);
        assert_eq!(pill_bounds.top_left_position, PhysicalPosition::new(2560 + 754, 0));
    }
}
