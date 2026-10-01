use windows::Media::Control::GlobalSystemMediaTransportControlsSession as SmtcSession;

use crate::media::windows_smtc::smtc_event_subscriptions::SmtcSessionEventSubscription;

/// One media session the worker listens to, with the app it belongs to.
pub struct TrackedSmtcSession {
    pub source_app_identifier: String,
    pub event_subscription: SmtcSessionEventSubscription,
}

/// A free function rather than a tracker method, so it borrows only the session list and the
/// caller can still change the tracker's other fields (the held presses) while using the result.
pub fn find_session_of_app<'tracked>(
    tracked_sessions: &'tracked [TrackedSmtcSession],
    source_app_identifier: Option<&str>,
) -> Option<&'tracked SmtcSession> {
    let source_app_identifier = source_app_identifier?;
    tracked_sessions
        .iter()
        .find(|tracked_session| tracked_session.source_app_identifier == source_app_identifier)
        .map(|tracked_session| tracked_session.event_subscription.session())
}
