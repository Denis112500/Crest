use tauri::{PhysicalPosition, PhysicalSize};

/// Converts a rectangle from the frontend's CSS pixels to the window's physical pixels.
/// The edges are rounded outward so a fractional pixel of the pill is never cut off.
pub fn convert_logical_area_to_physical(
    logical_left: f64,
    logical_top: f64,
    logical_width: f64,
    logical_height: f64,
    window_scale_factor: f64,
) -> (PhysicalPosition<i32>, PhysicalSize<u32>) {
    let physical_left = (logical_left * window_scale_factor).floor();
    let physical_top = (logical_top * window_scale_factor).floor();
    let physical_right = ((logical_left + logical_width) * window_scale_factor).ceil();
    let physical_bottom = ((logical_top + logical_height) * window_scale_factor).ceil();
    (
        PhysicalPosition::new(physical_left as i32, physical_top as i32),
        PhysicalSize::new((physical_right - physical_left) as u32, (physical_bottom - physical_top) as u32),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scales_the_area_and_never_cuts_a_partial_pixel() {
        let (area_top_left, area_size) = convert_logical_area_to_physical(88.5, 0.0, 220.0, 36.0, 1.5);
        assert_eq!(area_top_left, PhysicalPosition::new(132, 0));
        assert_eq!(area_size, PhysicalSize::new(331, 54));
    }
}
