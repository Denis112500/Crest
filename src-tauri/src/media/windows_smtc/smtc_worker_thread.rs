use std::sync::mpsc::{Receiver, Sender};
use std::time::Instant;

use windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager as SmtcSessionManager;
use windows::Win32::System::WinRT::{RoInitialize, RO_INIT_MULTITHREADED};

use crate::backend_constants::SMTC_EVENT_COALESCING_WINDOW;
use crate::media::media_source::MediaSnapshotListener;
use crate::media::windows_smtc::smtc_event_subscriptions::subscribe_to_session_list_changes;
use crate::media::windows_smtc::smtc_session_tracker::SmtcSessionTracker;
use crate::media::windows_smtc::smtc_worker_message::SmtcWorkerMessage;

/// Runs for the whole life of the app on its own thread. It sleeps in `recv()` until an
/// SMTC event or a button press arrives, so it costs no CPU while nothing changes.
pub fn run_smtc_worker_thread(
    allowed_app_identifier_fragments: Vec<String>,
    media_snapshot_listener: MediaSnapshotListener,
    worker_message_sender: Sender<SmtcWorkerMessage>,
    worker_message_receiver: Receiver<SmtcWorkerMessage>,
) {
    // The multithreaded apartment lets SMTC deliver updates without a window message loop;
    // in the default single-threaded apartment the session values went stale (tested).
    // SAFETY: called once, first thing on this new thread, before any other WinRT call on it.
    if let Err(initialization_error) = unsafe { RoInitialize(RO_INIT_MULTITHREADED) } {
        eprintln!("Crest media: could not initialize WinRT: {initialization_error}");
        return;
    }
    let session_manager = match SmtcSessionManager::RequestAsync().and_then(|request| request.join()) {
        Ok(session_manager) => session_manager,
        Err(request_error) => {
            eprintln!("Crest media: Windows media controls are unavailable: {request_error}");
            return;
        }
    };
    if let Err(subscribe_error) = subscribe_to_session_list_changes(&session_manager, worker_message_sender.clone()) {
        eprintln!("Crest media: could not watch for new media sessions: {subscribe_error}");
        return;
    }
    let mut session_tracker = SmtcSessionTracker::new(
        session_manager,
        worker_message_sender,
        allowed_app_identifier_fragments,
        media_snapshot_listener,
    );
    session_tracker.refresh_session_list();
    session_tracker.publish_preferred_session_if_changed();

    while let Ok(first_message) = worker_message_receiver.recv() {
        let mut received_messages = vec![first_message];
        collect_messages_within_coalescing_window(&worker_message_receiver, &mut received_messages);
        for received_message in received_messages {
            match received_message {
                SmtcWorkerMessage::SessionListChanged => session_tracker.refresh_session_list(),
                SmtcWorkerMessage::SessionActivity { source_app_identifier } => {
                    session_tracker.record_session_activity(source_app_identifier)
                }
                SmtcWorkerMessage::TransportCommandRequested(media_transport_command) => {
                    session_tracker.send_transport_command_to_shown_session(media_transport_command)
                }
            }
        }
        session_tracker.publish_preferred_session_if_changed();
    }
}

fn collect_messages_within_coalescing_window(
    worker_message_receiver: &Receiver<SmtcWorkerMessage>,
    received_messages: &mut Vec<SmtcWorkerMessage>,
) {
    let coalescing_deadline = Instant::now() + SMTC_EVENT_COALESCING_WINDOW;
    while let Some(time_left) = coalescing_deadline.checked_duration_since(Instant::now()) {
        match worker_message_receiver.recv_timeout(time_left) {
            Ok(next_message) => received_messages.push(next_message),
            Err(_) => break,
        }
    }
}
