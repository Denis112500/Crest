#[cfg(target_os = "windows")]
use std::time::Duration;

/// Must match the window `label` in `tauri.conf.json`.
pub const PILL_WINDOW_LABEL: &str = "pill";

/// Gap between the top edge of the screen and the pill, like the island floating below the notch.
pub const PILL_WINDOW_TOP_MARGIN_LOGICAL_PIXELS: f64 = 8.0;

/// Optional settings file inside Crest's config folder (%APPDATA%\dev.crest.pill on Windows).
pub const USER_SETTINGS_FILE_NAME: &str = "settings.json";

/// The YouTube Music web app ID. Chromium browsers report the installed app as
/// "<Browser>._crx_cinhimbnkkghhklpknlkffjgod" (verified with Brave).
pub const DEFAULT_ALLOWED_MEDIA_APP_IDENTIFIER_FRAGMENT: &str = "_crx_cinhimbnkkghhklpknlkffjgod";

/// SMTC events arrive in bursts (dragging the seek bar fires about 10 per second), so
/// everything that arrives within this window after the first event is handled once.
#[cfg(target_os = "windows")]
pub const SMTC_EVENT_COALESCING_WINDOW: Duration = Duration::from_millis(50);

/// WinRT `TimeSpan` and `DateTime` count 100-nanosecond ticks.
#[cfg(target_os = "windows")]
pub const WINRT_TICKS_PER_MILLISECOND: i64 = 10_000;

/// WinRT `DateTime` counts from 1601-01-01, Unix time from 1970-01-01: the gap in ticks.
#[cfg(target_os = "windows")]
pub const WINRT_TICKS_FROM_1601_TO_UNIX_EPOCH: i64 = 116_444_736_000_000_000;
