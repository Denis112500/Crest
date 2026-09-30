use std::sync::mpsc::{self, Sender};
use std::thread;

use crate::media::media_source::{MediaSnapshotListener, MediaSource, MediaTransportCommand};
use crate::media::windows_smtc::smtc_worker_message::SmtcWorkerMessage;
use crate::media::windows_smtc::smtc_worker_thread::run_smtc_worker_thread;

const SMTC_WORKER_THREAD_NAME: &str = "crest-smtc-media";

/// Windows implementation of `MediaSource`, reading the System Media Transport Controls:
/// the same data as the media flyout next to the volume slider.
pub struct SmtcMediaSource {
    allowed_app_identifier_fragments: Vec<String>,
    /// Set once watching has started; button presses travel through it to the worker.
    worker_message_sender: Option<Sender<SmtcWorkerMessage>>,
}

impl SmtcMediaSource {
    pub fn new(allowed_app_identifier_fragments: Vec<String>) -> Self {
        Self { allowed_app_identifier_fragments, worker_message_sender: None }
    }
}

impl MediaSource for SmtcMediaSource {
    fn start_watching_media_sessions(
        &mut self,
        media_snapshot_listener: MediaSnapshotListener,
    ) -> Result<(), String> {
        let (worker_message_sender, worker_message_receiver) = mpsc::channel();
        let allowed_app_identifier_fragments = self.allowed_app_identifier_fragments.clone();
        let sender_for_event_handlers = worker_message_sender.clone();
        thread::Builder::new()
            .name(SMTC_WORKER_THREAD_NAME.to_string())
            .spawn(move || {
                run_smtc_worker_thread(
                    allowed_app_identifier_fragments,
                    media_snapshot_listener,
                    sender_for_event_handlers,
                    worker_message_receiver,
                )
            })
            .map_err(|spawn_error| spawn_error.to_string())?;
        self.worker_message_sender = Some(worker_message_sender);
        Ok(())
    }

    fn send_media_transport_command(&self, media_transport_command: MediaTransportCommand) -> Result<(), String> {
        self.worker_message_sender
            .as_ref()
            .ok_or("media watching has not started")?
            .send(SmtcWorkerMessage::TransportCommandRequested(media_transport_command))
            .map_err(|_| "the media worker thread has stopped".to_string())
    }
}
