//! The screen rectangle where resting the pointer opens the pill, in physical screen pixels.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PointerRestZone {
    pub left_x: i32,
    pub top_y: i32,
    /// The first column to the right that no longer belongs to the zone.
    pub right_x: i32,
    /// The first row below that no longer belongs to the zone.
    pub bottom_y: i32,
}

impl PointerRestZone {
    pub fn contains_screen_point(&self, screen_x: i32, screen_y: i32) -> bool {
        (self.left_x..self.right_x).contains(&screen_x) && (self.top_y..self.bottom_y).contains(&screen_y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_right_and_bottom_edges_are_outside() {
        let rest_zone = PointerRestZone { left_x: 100, top_y: 0, right_x: 200, bottom_y: 50 };
        assert!(rest_zone.contains_screen_point(100, 0));
        assert!(rest_zone.contains_screen_point(199, 49));
        assert!(!rest_zone.contains_screen_point(200, 10));
        assert!(!rest_zone.contains_screen_point(150, 50));
        assert!(!rest_zone.contains_screen_point(99, 10));
    }
}
