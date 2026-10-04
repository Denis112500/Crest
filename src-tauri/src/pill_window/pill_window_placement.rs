use tauri::{PhysicalPosition, PhysicalSize, WebviewWindow};

#[derive(Debug, PartialEq)]
struct PillWindowPhysicalBounds {
    top_left_position: PhysicalPosition<i32>,
    size: PhysicalSize<u32>,
}

/// Sizes the pill window and centers it on the top edge of the primary monitor, with no
/// gap: the pill is drawn as a notch hanging from the edge.
/// The frontend passes logical pixels (CSS pixels); Windows positions windows in
/// physical pixels, so everything is converted with the monitor's scale factor.
pub fn apply_top_center_placement_to_pill_window(
    pill_window: &WebviewWindow,
    pill_logical_width: f64,
    pill_logical_height: f64,
) -> Result<(), String> {
    let primary_monitor = pill_window
        .primary_monitor()
        .map_err(|error| error.to_string())?
        .ok_or("Windows reported no primary monitor")?;
    let pill_bounds = calculate_top_center_bounds(
        *primary_monitor.position(),
        *primary_monitor.size(),
        primary_monitor.scale_factor(),
        pill_logical_width,
        pill_logical_height,
    );
    pill_window
        .set_size(pill_bounds.size)
        .and_then(|()| pill_window.set_position(pill_bounds.top_left_position))
        // Moving onto a monitor with a different scale factor makes Windows rescale the
        // window, so the size is applied once more at its final position.
        .and_then(|()| pill_window.set_size(pill_bounds.size))
        .map_err(|error| error.to_string())
}

fn calculate_top_center_bounds(
    monitor_top_left_position: PhysicalPosition<i32>,
    monitor_size: PhysicalSize<u32>,
    monitor_scale_factor: f64,
    pill_logical_width: f64,
    pill_logical_height: f64,
) -> PillWindowPhysicalBounds {
    let pill_physical_width = (pill_logical_width * monitor_scale_factor).round() as u32;
    let pill_physical_height = (pill_logical_height * monitor_scale_factor).round() as u32;
    let horizontal_offset_inside_monitor =
        (monitor_size.width as i32 - pill_physical_width as i32) / 2;
    PillWindowPhysicalBounds {
        top_left_position: PhysicalPosition::new(
            monitor_top_left_position.x + horizontal_offset_inside_monitor,
            monitor_top_left_position.y,
        ),
        size: PhysicalSize::new(pill_physical_width, pill_physical_height),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centers_pill_and_scales_it_on_a_150_percent_monitor() {
        let pill_bounds = calculate_top_center_bounds(
            PhysicalPosition::new(0, 0),
            PhysicalSize::new(2560, 1440),
            1.5,
            220.0,
            36.0,
        );
        assert_eq!(pill_bounds.size, PhysicalSize::new(330, 54));
        assert_eq!(pill_bounds.top_left_position, PhysicalPosition::new(1115, 0));
    }

    #[test]
    fn respects_a_primary_monitor_that_does_not_start_at_zero() {
        let pill_bounds = calculate_top_center_bounds(
            PhysicalPosition::new(-1920, 0),
            PhysicalSize::new(1920, 1080),
            1.0,
            220.0,
            36.0,
        );
        assert_eq!(pill_bounds.top_left_position, PhysicalPosition::new(-1070, 0));
    }
}
