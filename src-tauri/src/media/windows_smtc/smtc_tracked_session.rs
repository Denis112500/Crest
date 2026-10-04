use std::sync::mpsc::Sender;

use windows::Media::Control::GlobalSystemMediaTransportControlsSession as SmtcSession;
use windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager as SmtcSessionManager;

use crate::media::windows_smtc::smtc_event_subscriptions::SmtcSessionEventSubscription;
use crate::media::windows_smtc::smtc_worker_message::SmtcWorkerMessage;

/// One media session the worker listens to, with the app it belongs to.
pub struct TrackedSmtcSession {
    pub source_app_identifier: String,
    pub event_subscription: SmtcSessionEventSubscription,
}

/// Every session Windows reports now, each subscribed to its change events. `None` if the
/// list couldn't be read, so the caller keeps what it had instead of dropping everything.
pub fn subscribe_to_current_sessions(
    session_manager: &SmtcSessionManager,
    worker_message_sender: &Sender<SmtcWorkerMessage>,
) -> Option<Vec<TrackedSmtcSession>> {
    let current_sessions = session_manager
        .GetSessions()
        .inspect_err(|read_error| eprintln!("Crest media: could not list media sessions: {read_error}"))
        .ok()?;
    Some(
        current_sessions
            .into_iter()
            .filter_map(|session| {
                let source_app_identifier = session.SourceAppUserModelId().ok()?.to_string_lossy();
                let event_subscription = SmtcSessionEventSubscription::subscribe(session, worker_message_sender).ok()?;
                Some(TrackedSmtcSession { source_app_identifier, event_subscription })
            })
            .collect(),
    )
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
