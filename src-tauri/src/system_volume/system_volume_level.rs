//! What a valid system-volume level is: a share of full volume from 0 to 1, in whole percent,
//! as Windows' own slider shows it. Levels from both directions go through here, so a level
//! Crest sets and the same level reported back by Windows compare equal.

use serde::Serialize;

const PERCENT_STEPS_PER_FULL_VOLUME: f32 = 100.0;

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(transparent)]
pub struct SystemVolumeLevel(f32);

impl SystemVolumeLevel {
    /// From the OS, which reports 0..1 but as a float that may read 0.6399999 for 64 %.
    pub fn from_reported_scalar(reported_scalar: f32) -> Self {
        Self(round_to_whole_percent(clamp_to_full_range(reported_scalar)))
    }

    /// From the page, where anything may arrive: a non-number is refused, the rest is clamped.
    pub fn from_requested_scalar(requested_scalar: f64) -> Result<Self, String> {
        if !requested_scalar.is_finite() {
            return Err(format!("{requested_scalar} is not a volume level"));
        }
        Ok(Self(round_to_whole_percent(clamp_to_full_range(requested_scalar as f32))))
    }

    pub fn as_scalar(self) -> f32 {
        self.0
    }
}

fn clamp_to_full_range(scalar: f32) -> f32 {
    if scalar.is_nan() {
        return 0.0;
    }
    scalar.clamp(0.0, 1.0)
}

fn round_to_whole_percent(scalar: f32) -> f32 {
    (scalar * PERCENT_STEPS_PER_FULL_VOLUME).round() / PERCENT_STEPS_PER_FULL_VOLUME
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_reported_float_just_below_a_percent_reads_as_that_percent() {
        assert_eq!(SystemVolumeLevel::from_reported_scalar(0.639_999_9).as_scalar(), 0.64);
    }

    #[test]
    fn a_requested_level_equals_the_same_level_reported_back() {
        let requested_level = SystemVolumeLevel::from_requested_scalar(0.64).unwrap();
        let reported_level = SystemVolumeLevel::from_reported_scalar(0.640_000_1);
        assert_eq!(requested_level, reported_level);
    }

    #[test]
    fn requested_levels_outside_zero_to_one_are_clamped() {
        assert_eq!(SystemVolumeLevel::from_requested_scalar(1.7).unwrap().as_scalar(), 1.0);
        assert_eq!(SystemVolumeLevel::from_requested_scalar(-0.2).unwrap().as_scalar(), 0.0);
    }

    #[test]
    fn requested_non_numbers_are_refused() {
        assert!(SystemVolumeLevel::from_requested_scalar(f64::NAN).is_err());
        assert!(SystemVolumeLevel::from_requested_scalar(f64::INFINITY).is_err());
    }

    #[test]
    fn requested_levels_are_rounded_to_whole_percent() {
        assert_eq!(SystemVolumeLevel::from_requested_scalar(0.456).unwrap().as_scalar(), 0.46);
    }

    #[test]
    fn a_level_is_sent_to_the_page_as_a_short_number() {
        let level_json = serde_json::to_string(&SystemVolumeLevel::from_reported_scalar(0.64)).unwrap();
        assert_eq!(level_json, "0.64");
    }
}
