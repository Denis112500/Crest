use std::sync::mpsc::Sender;

use windows::core::{Ref, RuntimeType};
use windows::Foundation::TypedEventHandler;
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession as SmtcSession,
    GlobalSystemMediaTransportControlsSessionManager as SmtcSessionManager,
};

use crate::media::windows_smtc::smtc_worker_message::SmtcWorkerMessage;

/// The manager lives as long as the app, so this subscription is never removed.
pub fn subscribe_to_session_list_changes(
    session_manager: &SmtcSessionManager,
    worker_message_sender: Sender<SmtcWorkerMessage>,
) -> windows::core::Result<()> {
    session_manager.SessionsChanged(&TypedEventHandler::new(move |_manager, _change_details| {
        // Sending only fails once the worker has stopped, which means the app is exiting.
        let _ = worker_message_sender.send(SmtcWorkerMessage::SessionListChanged);
        Ok(())
    }))?;
    Ok(())
}

/// Listens to one session's track, playback and timeline changes. Dropping it removes the
/// handlers, so a closed session stops calling us.
pub struct SmtcSessionEventSubscription {
    session: SmtcSession,
    media_properties_changed_token: i64,
    playback_info_changed_token: i64,
    timeline_properties_changed_token: i64,
}

impl SmtcSessionEventSubscription {
    pub fn subscribe(
        session: SmtcSession,
        worker_message_sender: &Sender<SmtcWorkerMessage>,
    ) -> windows::core::Result<Self> {
        let media_properties_changed_token = session
            .MediaPropertiesChanged(&create_session_activity_handler(worker_message_sender.clone()))?;
        let playback_info_changed_token = session
            .PlaybackInfoChanged(&create_session_activity_handler(worker_message_sender.clone()))?;
        let timeline_properties_changed_token = session
            .TimelinePropertiesChanged(&create_session_activity_handler(worker_message_sender.clone()))?;
        Ok(Self {
            session,
            media_properties_changed_token,
            playback_info_changed_token,
            timeline_properties_changed_token,
        })
    }

    pub fn session(&self) -> &SmtcSession {
        &self.session
    }
}

impl Drop for SmtcSessionEventSubscription {
    fn drop(&mut self) {
        // The session may already be gone; failing to unsubscribe from it is harmless.
        let _ = self.session.RemoveMediaPropertiesChanged(self.media_properties_changed_token);
        let _ = self.session.RemovePlaybackInfoChanged(self.playback_info_changed_token);
        let _ = self.session.RemoveTimelinePropertiesChanged(self.timeline_properties_changed_token);
    }
}

fn create_session_activity_handler<EventDetails: RuntimeType + 'static>(
    worker_message_sender: Sender<SmtcWorkerMessage>,
) -> TypedEventHandler<SmtcSession, EventDetails> {
    TypedEventHandler::new(move |changed_session: Ref<SmtcSession>, _event_details: Ref<EventDetails>| {
        let source_app_identifier = changed_session.ok()?.SourceAppUserModelId()?.to_string_lossy();
        let _ = worker_message_sender.send(SmtcWorkerMessage::SessionActivity { source_app_identifier });
        Ok(())
    })
}
