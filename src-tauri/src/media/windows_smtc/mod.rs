//! Windows' media source: System Media Transport Controls (SMTC), the API behind the volume
//! flyout's media controls (GlobalSystemMediaTransportControlsSessionManager).

mod smtc_event_subscriptions;
mod smtc_media_source;
mod smtc_session_tracker;
mod smtc_snapshot_reader;
mod smtc_thumbnail_reader;
mod smtc_tracked_session;
mod smtc_transport_commands;
mod smtc_worker_message;
mod smtc_worker_thread;

pub use smtc_media_source::SmtcMediaSource;
