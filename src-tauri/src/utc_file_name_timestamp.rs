//! A point in time as `2026-11-03_09-15-07-UTC`, for file names: fixed width, so sorting the names
//! sorts the times. UTC because the standard library has no time zones, and because local names
//! would sort wrong across a daylight-saving change.

use std::time::{SystemTime, UNIX_EPOCH};

const SECONDS_PER_DAY: u64 = 86_400;

pub fn format_utc_file_name_timestamp(point_in_time: SystemTime) -> String {
    let seconds_since_epoch = point_in_time.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let (year, month, day) = civil_date_from_days_since_epoch(seconds_since_epoch / SECONDS_PER_DAY);
    let seconds_into_day = seconds_since_epoch % SECONDS_PER_DAY;
    format!(
        "{year:04}-{month:02}-{day:02}_{:02}-{:02}-{:02}-UTC",
        seconds_into_day / 3600,
        seconds_into_day / 60 % 60,
        seconds_into_day % 60
    )
}

/// Howard Hinnant's `civil_from_days` (public domain, http://howardhinnant.github.io/date_algorithms.html),
/// for days on or after 1970-01-01. The calendar is counted in 400-year eras starting on March 1st,
/// so the leap day falls at the end of each counted year.
fn civil_date_from_days_since_epoch(days_since_epoch: u64) -> (u64, u64, u64) {
    let days_since_era_start = days_since_epoch + 719_468;
    let era = days_since_era_start / 146_097;
    let day_of_era = days_since_era_start % 146_097;
    let year_of_era = (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_from_march = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_from_march + 2) / 5 + 1;
    let month = if month_from_march < 10 { month_from_march + 3 } else { month_from_march - 9 };
    let year = year_of_era + era * 400 + u64::from(month <= 2);
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    fn timestamp_at(seconds_since_epoch: u64) -> String {
        format_utc_file_name_timestamp(UNIX_EPOCH + Duration::from_secs(seconds_since_epoch))
    }

    #[test]
    fn known_moments_are_written_in_utc() {
        assert_eq!(timestamp_at(0), "1970-01-01_00-00-00-UTC");
        // 2000-02-29 (a leap day in a century year) 23:59:59.
        assert_eq!(timestamp_at(951_868_799), "2000-02-29_23-59-59-UTC");
        // 2024-03-01 00:00:00, the day after a leap day.
        assert_eq!(timestamp_at(1_709_251_200), "2024-03-01_00-00-00-UTC");
        // 2026-11-03 09:15:07.
        assert_eq!(timestamp_at(1_793_697_307), "2026-11-03_09-15-07-UTC");
    }

    #[test]
    fn names_sort_like_the_times_they_hold() {
        let earlier = timestamp_at(1_793_697_307);
        let later = timestamp_at(1_793_697_307 + SECONDS_PER_DAY * 40);
        assert!(earlier < later);
    }
}
