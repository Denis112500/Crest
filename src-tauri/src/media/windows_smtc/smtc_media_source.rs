//! The Windows `MediaSource`: starts the SMTC worker thread and passes button presses and
//! filter changes to it as messages.

use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::media::media_player_filter::MediaPlayerFilter;
use crate::media::media_player_filter_control::MediaPlayerFilterControl;
use crate::media::media_source::{MediaSnapshotListener, MediaSource, MediaTransportCommand};
use crate::media::windows_smtc::smtc_worker_message::SmtcWorkerMessage;
use crate::media::windows_smtc::smtc_worker_thread::run_smtc_worker_thread;

const SMTC_WORKER_THREAD_NAME: &str = "crest-smtc-media";

/// Windows implementation of `MediaSource`, reading the System Media Transport Controls:
/// the same data as the media flyout next to the volume slider.
pub struct SmtcMediaSource {
    media_player_filter: MediaPlayerFilter,
    /// Created up front (not when watching starts), so the settings window's filter control
    /// can be handed out before the source is started; messages simply wait until then.
    worker_message_sender: Sender<SmtcWorkerMessage>,
    /// Taken by the worker thread when watching starts.
    worker_message_receiver: Option<Receiver<SmtcWorkerMessage>>,
    open_player_app_identifiers: Arc<Mutex<Vec<String>>>,
}

impl SmtcMediaSource {
    pub fn new(media_player_filter: MediaPlayerFilter) -> Self {
        let (worker_message_sender, worker_message_receiver) = mpsc::channel();
        Self {
            media_player_filter,
            worker_message_sender,
            worker_message_receiver: Some(worker_message_receiver),
            open_player_app_identifiers: Arc::default(),
        }
    }
}

impl MediaSource for SmtcMediaSource {
    fn start_watching_media_sessions(
        &mut self,
        media_snapshot_listener: MediaSnapshotListener,
    ) -> Result<(), String> {
        let worker_message_receiver =
            self.worker_message_receiver.take().ok_or("media watching has already started")?;
        let media_player_filter = self.media_player_filter.clone();
        let sender_for_event_handlers = self.worker_message_sender.clone();
        let open_player_app_identifiers = Arc::clone(&self.open_player_app_identifiers);
        thread::Builder::new()
            .name(SMTC_WORKER_THREAD_NAME.to_string())
            .spawn(move || {
                run_smtc_worker_thread(
                    media_player_filter,
                    open_player_app_identifiers,
                    media_snapshot_listener,
                    sender_for_event_handlers,
                    worker_message_receiver,
                )
            })
            .map_err(|spawn_error| spawn_error.to_string())?;
        Ok(())
    }

    fn send_media_transport_command(&self, media_transport_command: MediaTransportCommand) -> Result<(), String> {
        self.worker_message_sender
            .send(SmtcWorkerMessage::TransportCommandRequested(media_transport_command))
            .map_err(|_| "the media worker thread has stopped".to_string())
    }

    fn create_player_filter_control(&self) -> MediaPlayerFilterControl {
        let filter_message_sender = self.worker_message_sender.clone();
        MediaPlayerFilterControl::new(Arc::clone(&self.open_player_app_identifiers), move |media_player_filter| {
            filter_message_sender
                .send(SmtcWorkerMessage::MediaPlayerFilterReplaced(media_player_filter))
                .map_err(|_| "the media worker thread has stopped".to_string())
        })
    }
}
