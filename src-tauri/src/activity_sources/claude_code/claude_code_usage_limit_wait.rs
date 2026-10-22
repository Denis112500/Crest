//! A session waiting for a reached usage limit to reset: when it resets as Claude Code wrote it
//! (for the pill's "resets 1:50") and as a moment on Crest's own clock (for how long the session
//! stays listed). Claude Code itself waits and continues at the reset, often hours away, so the
//! usual silence timeout would drop the session long before.

use std::time::{Duration, Instant};

use crate::backend_constants::CLAUDE_CODE_USAGE_LIMIT_MAX_LISTED_WAIT;

#[derive(Clone, Copy)]
pub struct ClaudeCodeUsageLimitWait {
    pub resets_at_unix_seconds: u64,
    reset_time: Instant,
}

impl ClaudeCodeUsageLimitWait {
    /// Counted to the reset, but never further than a day ahead.
    pub fn new(resets_at_unix_seconds: u64, time_until_reset: Duration, current_time: Instant) -> Self {
        let reset_time = current_time + time_until_reset.min(CLAUDE_CODE_USAGE_LIMIT_MAX_LISTED_WAIT);
        Self { resets_at_unix_seconds, reset_time }
    }

    /// The silence timeout counts from the reset, or from the session's last event if later.
    pub fn silence_deadline(&self, after_last_event: Instant, silence_timeout: Duration) -> Instant {
        after_last_event.max(self.reset_time + silence_timeout)
    }
}
