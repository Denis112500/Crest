use std::sync::{Arc, Mutex, PoisonError};
use std::thread;
use std::time::Duration;

/// Browsers drop their media session for a fraction of a second on every track change
/// (about 0.4 s with YouTube Music, measured). A loss is only acted on if no session
/// shows up again within the grace period, so the pill doesn't flicker between songs.
pub struct SessionLossGracePeriod {
    grace_duration: Duration,
    /// Bumped on every report; a pending loss only fires if nothing was reported since.
    /// Kept locked while acting, so a report can't slip in between "check" and "withdraw".
    report_generation: Arc<Mutex<u64>>,
}

impl SessionLossGracePeriod {
    pub fn new(grace_duration: Duration) -> Self {
        Self { grace_duration, report_generation: Arc::new(Mutex::new(0)) }
    }

    /// A session exists: act now, and cancel any loss still waiting out its grace period.
    pub fn report_session_present(&self, act_on_present_session: impl FnOnce()) {
        let mut report_generation = self.report_generation.lock().unwrap_or_else(PoisonError::into_inner);
        *report_generation += 1;
        act_on_present_session();
    }

    /// The session is gone: act only if it's still gone after the grace period. The wait
    /// happens on a short-lived sleeping thread, which costs no CPU.
    pub fn report_session_lost(&self, act_on_lost_session: impl FnOnce() + Send + 'static) {
        let generation_at_loss = {
            let mut report_generation = self.report_generation.lock().unwrap_or_else(PoisonError::into_inner);
            *report_generation += 1;
            *report_generation
        };
        let report_generation = Arc::clone(&self.report_generation);
        let grace_duration = self.grace_duration;
        thread::spawn(move || {
            thread::sleep(grace_duration);
            let current_generation = report_generation.lock().unwrap_or_else(PoisonError::into_inner);
            if *current_generation == generation_at_loss {
                act_on_lost_session();
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicBool, Ordering};

    use super::*;

    const TEST_GRACE_DURATION: Duration = Duration::from_millis(30);
    const LONGER_THAN_TEST_GRACE: Duration = Duration::from_millis(150);

    #[test]
    fn acts_on_a_loss_that_lasts_longer_than_the_grace_period() {
        let grace_period = SessionLossGracePeriod::new(TEST_GRACE_DURATION);
        let loss_was_acted_on = Arc::new(AtomicBool::new(false));
        let loss_flag_for_callback = Arc::clone(&loss_was_acted_on);
        grace_period.report_session_lost(move || loss_flag_for_callback.store(true, Ordering::SeqCst));
        thread::sleep(LONGER_THAN_TEST_GRACE);
        assert!(loss_was_acted_on.load(Ordering::SeqCst));
    }

    #[test]
    fn ignores_a_loss_when_the_session_returns_in_time() {
        let grace_period = SessionLossGracePeriod::new(TEST_GRACE_DURATION);
        let loss_was_acted_on = Arc::new(AtomicBool::new(false));
        let loss_flag_for_callback = Arc::clone(&loss_was_acted_on);
        grace_period.report_session_lost(move || loss_flag_for_callback.store(true, Ordering::SeqCst));
        grace_period.report_session_present(|| {});
        thread::sleep(LONGER_THAN_TEST_GRACE);
        assert!(!loss_was_acted_on.load(Ordering::SeqCst));
    }
}
