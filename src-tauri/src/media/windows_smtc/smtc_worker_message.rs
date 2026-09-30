use crate::media::media_source::MediaTransportCommand;

/// Everything that wakes the SMTC worker thread. SMTC event handlers (on Windows thread-pool
/// threads) and button presses (on Tauri's thread) only send one of these; all WinRT reading
/// and controlling happens on the worker thread.
pub enum SmtcWorkerMessage {
    SessionListChanged,
    SessionActivity { source_app_identifier: String },
    TransportCommandRequested(MediaTransportCommand),
}
