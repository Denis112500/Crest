use std::time::Duration;

/// Must match the window `label` in `tauri.conf.json`.
pub const PILL_WINDOW_LABEL: &str = "pill";

/// The settings window is created when it's opened and destroyed when it's closed, so it
/// costs no memory the rest of the time.
pub const SETTINGS_WINDOW_LABEL: &str = "settings";

/// Built by Vite next to the pill's index.html (see `build.rollupOptions.input` in vite.config.ts).
pub const SETTINGS_WINDOW_PAGE_PATH: &str = "settings.html";

pub const SETTINGS_WINDOW_LOGICAL_WIDTH: f64 = 480.0;
pub const SETTINGS_WINDOW_LOGICAL_HEIGHT: f64 = 560.0;

/// Optional settings file inside Crest's config folder (%APPDATA%\dev.crest.pill on Windows).
pub const USER_SETTINGS_FILE_NAME: &str = "settings.json";

/// The YouTube Music web app ID. Chromium browsers report the installed app as
/// "<Browser>._crx_cinhimbnkkghhklpknlkffjgod" (verified with Brave).
pub const DEFAULT_ALLOWED_MEDIA_APP_IDENTIFIER_FRAGMENT: &str = "_crx_cinhimbnkkghhklpknlkffjgod";

/// Music sits in the middle, so future sources can rank above it (a finished timer)
/// or below it (a background status).
pub const MUSIC_ACTIVITY_DISPLAY_PRIORITY: u8 = 50;

/// A paused activity (music on pause) keeps the pill on screen this long, then it hides.
pub const PILL_HIDE_DELAY_AFTER_ACTIVITY_PAUSES: Duration = Duration::from_secs(30);

/// When no activity is left (the player closed), the pill hides after this long.
pub const PILL_HIDE_DELAY_AFTER_ACTIVITY_ENDS: Duration = Duration::from_secs(3);

/// How long a vanished media session may stay away before the music activity is withdrawn.
/// Browsers drop the session for about 0.4 s on every track change (measured).
pub const MUSIC_SESSION_LOSS_GRACE_PERIOD: Duration = Duration::from_millis(1500);

/// How long a button press is kept while the player's session is gone, waiting for it to
/// come back. The session vanishes for about 0.5 s on every track change (measured); an
/// older press would surprise the user, so it's dropped.
pub const PENDING_MEDIA_TRANSPORT_COMMAND_LIFETIME: Duration = Duration::from_secs(2);

/// How long a new track's album art is held back (the pill shows "loading" instead). Brave
/// first sends its own logo as the thumbnail and the real cover 60–130 ms later (measured
/// on 6 skips), so this is about twice the slowest case.
pub const ALBUM_ART_SETTLE_WINDOW: Duration = Duration::from_millis(300);

/// After Windows reports that a fullscreen app has gone, the pill waits this long before it
/// comes back: switching a fullscreen app's display mode sends "closed" and "opened" 20 ms apart
/// (measured), and the pill shouldn't flash in between.
#[cfg(target_os = "windows")]
pub const FULLSCREEN_APP_LEAVE_SETTLE_DELAY: Duration = Duration::from_millis(250);

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
