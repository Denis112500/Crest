use std::thread;

use crate::media::media_source::{MediaSnapshotListener, MediaSource};
use crate::media::windows_smtc::smtc_worker_thread::run_smtc_worker_thread;

const SMTC_WORKER_THREAD_NAME: &str = "crest-smtc-media";

/// Windows implementation of `MediaSource`, reading the System Media Transport Controls:
/// the same data as the media flyout next to the volume slider.
pub struct SmtcMediaSource {
    allowed_app_identifier_fragments: Vec<String>,
}

impl SmtcMediaSource {
    pub fn new(allowed_app_identifier_fragments: Vec<String>) -> Self {
        Self { allowed_app_identifier_fragments }
    }
}

impl MediaSource for SmtcMediaSource {
    fn start_watching_media_sessions(
        &mut self,
        media_snapshot_listener: MediaSnapshotListener,
    ) -> Result<(), String> {
        let allowed_app_identifier_fragments = self.allowed_app_identifier_fragments.clone();
        thread::Builder::new()
            .name(SMTC_WORKER_THREAD_NAME.to_string())
            .spawn(move || run_smtc_worker_thread(allowed_app_identifier_fragments, media_snapshot_listener))
            .map(|_worker_thread| ())
            .map_err(|spawn_error| spawn_error.to_string())
    }
}
